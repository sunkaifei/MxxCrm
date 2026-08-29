//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 标准敏感字段权限服务（FLS，P2-1，docs/权限体系优化方案.md）
//! - 管理侧：白名单字段清单查询 + 角色配置保存（仅允许更新白名单内已有行，白名单本身由 v45 SQL 登记）
//! - 业务侧（安全真源）：
//!   * `trim_for_view`  序列化出口裁剪：用户不可见的标准敏感字段从响应 JSON 中删除
//!   * `filter_editable` 更新入口过滤：不可编辑字段从提交 JSON 中剔除（防绕过）
//! - 角色语义与 field_def（A8 模式）一致：visible_roles 数组=限定角色可见，NULL=全部可见；
//!   editable_roles NULL=跟随 visible_roles；空数组落库前归一化为 NULL
//! - 元数据走进程内缓存（30s 兜底 TTL，save 成功即失效），高频 detail 接口零额外网络开销

use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::field_perm;
use crate::modules::system::model::field_perm::{FieldPermListVO, FieldPermSaveRequest};
use crate::modules::system::service::field_def_service;

/// 字段权限元数据内存缓存 TTL（秒），与 field_def 校验器缓存口径一致
const PERM_CACHE_TTL_SECS: u64 = 30;

type PermCache = RwLock<HashMap<String, (Vec<field_perm::Model>, Instant)>>;

fn perm_cache() -> &'static PermCache {
    static CACHE: OnceLock<PermCache> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// 清空指定模块（或全部）字段权限缓存：save 成功后调用，变更即时生效
fn invalidate_perm_cache(module: Option<&str>) {
    if let Ok(mut map) = perm_cache().write() {
        match module {
            Some(m) => {
                map.remove(m);
            },
            None => map.clear(),
        }
    }
}

/// 按模块加载字段权限配置（带 30s 进程内缓存）
async fn get_module_perms<C: ConnectionTrait>(db: &C, module: &str) -> Result<Vec<field_perm::Model>> {
    if let Ok(map) = perm_cache().read() {
        if let Some((perms, loaded_at)) = map.get(module) {
            if loaded_at.elapsed() < Duration::from_secs(PERM_CACHE_TTL_SECS) {
                return Ok(perms.clone());
            }
        }
    }
    let perms = field_perm::Entity::find()
        .filter(field_perm::Column::Module.eq(module))
        .filter(field_perm::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if let Ok(mut map) = perm_cache().write() {
        map.insert(module.to_string(), (perms.clone(), Instant::now()));
    }
    Ok(perms)
}

/// 管理侧清单：按模块查白名单字段及角色配置（module 空=全部模块）
pub async fn get_list<C: ConnectionTrait>(db: &C, module: Option<String>) -> Result<Vec<FieldPermListVO>> {
    let query = field_perm::Entity::find().filter(field_perm::Column::Deleted.eq(0));
    let query = match module.as_deref() {
        Some(m) if !m.is_empty() => query.filter(field_perm::Column::Module.eq(m)),
        _ => query,
    };
    let list = query
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(list.into_iter().map(FieldPermListVO::from).collect())
}

/// 管理侧保存：仅允许更新白名单内已有行（module+field_key 定位），不允许新增白名单外字段
pub async fn save<C: ConnectionTrait>(db: &C, form: &FieldPermSaveRequest, operator: Option<String>) -> Result<i64> {
    let module = form.module.as_deref().unwrap_or_default().trim().to_string();
    let field_key = form.field_key.as_deref().unwrap_or_default().trim().to_string();
    if module.is_empty() || field_key.is_empty() {
        return Err(Error::from("模块标识与字段键不能为空"));
    }

    // 角色配置归一化：空数组 → JSON null（未配置语义，对齐 field_def 落库规则）
    let visible_roles = field_def_service::normalize_roles(form.visible_roles.clone());
    let editable_roles = field_def_service::normalize_roles(form.editable_roles.clone());

    let existing = field_perm::Entity::find()
        .filter(field_perm::Column::Module.eq(&module))
        .filter(field_perm::Column::FieldKey.eq(&field_key))
        .filter(field_perm::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let model = match existing {
        Some(m) => {
            // 更新已有白名单行（update(self) 会消耗 active，主键需提前取出）
            let mut active: field_perm::ActiveModel = m.into();
            let record_id = active.id.clone().unwrap();
            active.visible_roles = Set(visible_roles);
            active.editable_roles = Set(editable_roles);
            active.update_by = Set(operator);
            active.update(db).await.map_err(|e| Error::from(e.to_string()))?;
            record_id
        },
        None => {
            return Err(Error::from(format!(
                "字段 {}.{} 不在敏感字段白名单内，无法配置（白名单由 v45 SQL 登记维护）",
                module, field_key
            )));
        },
    };

    // 配置变更即时生效（清模块缓存，多实例部署下 30s TTL 兜底收敛）
    invalidate_perm_cache(Some(&module));
    Ok(model)
}

/// 判断字段对当前用户是否可见：visible_roles 是数组 → 须有交集；NULL/非数组 → 全部可见
fn field_visible(perm: &field_perm::Model, role_keys: &[String]) -> bool {
    match &perm.visible_roles {
        Some(v) if field_def_service::is_role_array(v) => field_def_service::roles_intersect(v, role_keys),
        _ => true,
    }
}

/// 判断字段对当前用户是否可编辑：超管恒可编辑；
/// editable_roles 是数组 → 须有交集；NULL/非数组 → 跟随 visible_roles；两者皆未配置 → 全员可编辑
fn field_editable(perm: &field_perm::Model, is_admin: bool, role_keys: &[String]) -> bool {
    if is_admin {
        return true;
    }
    match &perm.editable_roles {
        Some(v) if field_def_service::is_role_array(v) => field_def_service::roles_intersect(v, role_keys),
        _ => field_visible(perm, role_keys),
    }
}

/// snake_case → camelCase（白名单键为 snake_case，DetailVO 序列化 rename_all(camelCase)，
/// 如 personal_mobile → personalMobile；无下划线键原样返回）
fn to_camel(key: &str) -> String {
    let mut out = String::with_capacity(key.len());
    let mut upper_next = false;
    for ch in key.chars() {
        if ch == '_' {
            upper_next = true;
        } else if upper_next {
            out.extend(ch.to_uppercase());
            upper_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// 序列化出口裁剪（安全真源，P2-1）
///
/// 把当前用户不可见的标准敏感字段从 JSON 对象中删除（同时兼容 snake_case 与 camelCase 键）；超管直通。
/// value 须为 JSON 对象（实体序列化结果），非对象时静默跳过。
/// 用法：`let mut v = serde_json::to_value(&model)?; trim_for_view(...).await;` 之后回传/反序列化。
pub async fn trim_for_view<C: ConnectionTrait>(db: &C, module: &str, is_admin: bool, role_keys: &[String], value: &mut serde_json::Value) {
    if is_admin {
        return;
    }
    let Some(obj) = value.as_object_mut() else { return };
    let Ok(perms) = get_module_perms(db, module).await else { return };
    for perm in &perms {
        if field_visible(perm, role_keys) {
            continue;
        }
        if let Some(key) = perm.field_key.as_deref() {
            obj.remove(key);
            let camel = to_camel(key);
            if camel != key {
                obj.remove(camel.as_str());
            }
        }
    }
}

/// 更新入口白名单过滤（防绕过，P2-1）
///
/// 把当前用户不可编辑的标准敏感字段从提交 JSON 中剔除（后端以剔除后的负载更新，越权字段不生效）；
/// 超管直通。value 须为 JSON 对象（提交负载），非对象时静默跳过。
pub async fn filter_editable<C: ConnectionTrait>(db: &C, module: &str, is_admin: bool, role_keys: &[String], value: &mut serde_json::Value) {
    if is_admin {
        return;
    }
    let Some(obj) = value.as_object_mut() else { return };
    let Ok(perms) = get_module_perms(db, module).await else { return };
    for perm in &perms {
        if field_editable(perm, is_admin, role_keys) {
            continue;
        }
        if let Some(key) = perm.field_key.as_deref() {
            obj.remove(key);
            let camel = to_camel(key);
            if camel != key {
                obj.remove(camel.as_str());
            }
        }
    }
}

/// 写路径守卫输入（强类型更新防绕过，P2-1）
///
/// 返回当前用户"不可编辑"的敏感字段键集合（snake_case，与实体/DTO 字段名一致）；
/// None 表示无任何限制（超管或全部字段未配置），调用方可零开销跳过回填逻辑。
/// 用法：调用方把锁定字段从旧实体回填到 DTO（CRM 的 update_by_id 全字段 Set，None 会清空列，
/// 单纯剔除提交 JSON 会造成数据丢失，故必须回填而非剔除）。
pub async fn editable_guard_keys<C: ConnectionTrait>(
    db: &C,
    module: &str,
    is_admin: bool,
    role_keys: &[String],
) -> Result<Option<HashSet<String>>> {
    if is_admin {
        return Ok(None);
    }
    let perms = get_module_perms(db, module).await?;
    let mut locked: HashSet<String> = HashSet::new();
    for perm in &perms {
        if !field_editable(perm, is_admin, role_keys) {
            if let Some(k) = perm.field_key.as_deref() {
                locked.insert(k.to_string());
            }
        }
    }
    if locked.is_empty() {
        Ok(None)
    } else {
        Ok(Some(locked))
    }
}

/// 批量裁剪工具：对列表场景逐条裁剪（避免 N 次重复查询，perms 只加载一次）
///
/// 用于 list 接口：rows 中每项先序列化为 Value，裁剪后由调用方回传。
pub async fn trim_for_view_batch<C: ConnectionTrait>(
    db: &C,
    module: &str,
    is_admin: bool,
    role_keys: &[String],
    rows: &mut [serde_json::Value],
) {
    if is_admin || rows.is_empty() {
        return;
    }
    let Ok(perms) = get_module_perms(db, module).await else { return };
    let mut hidden_keys: Vec<String> = Vec::new();
    for p in &perms {
        if field_visible(p, role_keys) {
            continue;
        }
        if let Some(k) = p.field_key.as_deref() {
            hidden_keys.push(k.to_string());
            let camel = to_camel(k);
            if camel != k {
                hidden_keys.push(camel);
            }
        }
    }
    if hidden_keys.is_empty() {
        return;
    }
    for row in rows.iter_mut() {
        if let Some(obj) = row.as_object_mut() {
            for key in &hidden_keys {
                obj.remove(key.as_str());
            }
        }
    }
}

/// 角色数组与 role_keys 是否有交集（HashSet 加速，供 trim_for_view_batch 等高频场景复用）
pub fn has_any_role(roles_json: &serde_json::Value, role_keys_set: &HashSet<String>) -> bool {
    match roles_json {
        serde_json::Value::Array(arr) => arr.iter().any(|v| {
            v.as_str()
                .map(|s| role_keys_set.contains(s))
                .unwrap_or(false)
        }),
        _ => false,
    }
}
