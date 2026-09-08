//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 自定义字段定义服务（混合方案 P0）
//! - 管理侧：元数据 CRUD + 七步校验链 + 生命周期锁定规则（7.4）
//! - 运行侧：schema 下发（Redis 60s 缓存存全量、角色过滤在缓存外，防串角色 P0-23）
//! - 业务侧：validate_custom_fields 写入校验器（未知键 400 拒写 P1-11；editable_roles 越权写入 400 拦截 P0-24；角色配置落库前归一化，空数组→JSON null）
//! - 校验器元数据走进程内 OnceLock<RwLock> 缓存（30s 兜底 TTL，变更即失效，校验零网络开销）

use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

use regex::Regex;
use sea_orm::{ConnectionTrait, DbConn, EntityTrait, Order, Statement, TransactionTrait};
use sea_orm::sea_query::{Expr, SimpleExpr};

use crate::core::errors::error::{Error, Result};
use crate::core::kit::CONTEXT;
use crate::core::web::response::ResultPage;
use crate::modules::system::entity::{admin, field_def};
use crate::modules::system::model::field_def::{
    FieldDefListVO, FieldDefModel, FieldDefSaveDTO, FieldDefSaveRequest, FieldDefUpdateRequest,
    FieldModuleVO, ListQuery, PageWhere, SchemaItem,
};

/// 已接入自定义字段的业务模块注册表（module → 物理表名）
/// 唯一事实来源：管理页 module 下拉、schema 校验、information_schema 黑名单查询全部同源
pub const MODULE_TABLES: [(&str, &str); 7] = [
    ("crm_customer", "mxx_crm_customer"),
    ("crm_lead", "mxx_crm_lead"),
    ("crm_opportunity", "mxx_crm_opportunity"),
    ("crm_contact", "mxx_crm_contact"),
    ("crm_contract", "mxx_crm_contract"),
    ("sale_quotation", "mxx_sale_quotation"),
    ("sale_order", "mxx_sale_order"),
];

/// 运行侧 schema 的 Redis 缓存 TTL（秒），管理侧变更即失效，TTL 仅兜底
const SCHEMA_CACHE_TTL_SECS: u64 = 60;
/// 校验器元数据内存缓存 TTL（秒），多实例部署下收敛配置延迟
const VALIDATOR_TTL_SECS: u64 = 30;

struct ValidatorEntry {
    defs: Vec<field_def::Model>,
    loaded_at: Instant,
}

static VALIDATOR_CACHE: OnceLock<RwLock<HashMap<String, ValidatorEntry>>> = OnceLock::new();

fn validator_cache() -> &'static RwLock<HashMap<String, ValidatorEntry>> {
    VALIDATOR_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn schema_cache_key(module: &str) -> String {
    format!("field:schema:{}", module)
}

/// 模块白名单解析：返回物理表名，未注册返回 None
pub fn resolve_table(module: &str) -> Option<&'static str> {
    MODULE_TABLES.iter().find(|(m, _)| *m == module).map(|(_, t)| *t)
}

fn module_label(module: &str) -> &'static str {
    match module {
        "crm_customer" => "客户",
        "crm_lead" => "线索",
        "crm_opportunity" => "商机",
        "crm_contact" => "联系人",
        "crm_contract" => "合同",
        "sale_quotation" => "报价单",
        "sale_order" => "销售订单",
        _ => "",
    }
}

/// 已接入模块下拉（管理页 module 下拉唯一数据源，前后端同源）
pub fn get_module_options() -> Vec<FieldModuleVO> {
    MODULE_TABLES
        .iter()
        .map(|(m, _)| FieldModuleVO {
            module: Some(m.to_string()),
            label: Some(module_label(m).to_string()),
        })
        .collect()
}

// ============================ 管理侧 CRUD ============================

/// 新增字段定义（7.1 校验链：白名单 → key 格式 → 标准列黑名单 → label → 类型 → choices → 必填×角色 → 唯一）
pub async fn save(db: &DbConn, form: &FieldDefSaveRequest, operator: Option<String>) -> Result<i64> {
    // 1. module 白名单（P0-20：未注册模块 400 拒绝）
    let module = form.module.clone().unwrap_or_default().trim().to_string();
    let table = resolve_table(&module)
        .ok_or_else(|| Error::from(format!("业务模块「{}」未接入自定义字段", module)))?;

    // 2. field_key 格式
    let field_key = form.field_key.clone().unwrap_or_default().trim().to_string();
    check_field_key_format(&field_key)?;

    // 3. 标准列黑名单（information_schema 实时获取，非硬编码；表名来自白名单常量，无注入面）
    let columns = get_standard_columns(db, table).await?;
    if columns.contains(&field_key) {
        return Err(Error::from(format!("字段键「{}」与模块标准列冲突，请更换", field_key)));
    }

    // 4. label
    let field_label = form.field_label.clone().unwrap_or_default().trim().to_string();
    if field_label.is_empty() {
        return Err(Error::from("字段显示名不能为空"));
    }

    // 5. field_type 合法性
    let field_type = form.field_type.ok_or_else(|| Error::from("字段类型不能为空"))?;
    if !(1..=11).contains(&field_type) {
        return Err(Error::from("无效的字段类型"));
    }

    // 6. 单选/多选必须配置 choices 至少 1 项
    if field_type == 6 || field_type == 7 {
        let options = form
            .options
            .as_ref()
            .ok_or_else(|| Error::from("单选/多选字段必须配置 options.choices"))?;
        extract_choice_values(options)?;
    }

    // 6.5 数字/金额 options 精度校验（L210 options 规范：{"precision":2,"min":0,"max":9999}）
    check_number_precision(field_type, &form.options)?;

    // 7. 角色配置归一化：管理页"留空"提交 []，归一为 JSON null（未配置语义，对齐文档「NULL=跟随 visible_roles」；
    //    若原样落库，field_editable/field_visible 会把 [] 判为"仅超管"，与 UI 语义「留空=全员」相悖）
    let mut form_data = form.clone();
    form_data.visible_roles = normalize_roles(form_data.visible_roles);
    form_data.editable_roles = normalize_roles(form_data.editable_roles);

    // 8. 必填字段必须全员可见（P0-19，按归一化后的口径校验）
    let required = form.required.unwrap_or(0);
    check_required_visible(required, &form_data.visible_roles)?;

    // 9. 落库（事务内：标准列黑名单 + 同模块唯一 + insert 原子执行，防并发重复创建）
    let (new_id, module) = db
        .transaction::<_, (i64, String), Error>(|txn| {
            Box::pin(async move {
                // 标准列黑名单（information_schema 实时获取，非硬编码；表名来自白名单常量，无注入面）
                let columns = get_standard_columns(txn, table).await?;
                if columns.contains(&field_key) {
                    return Err(Error::from(format!(
                        "字段键「{}」与模块标准列冲突，请更换",
                        field_key
                    )));
                }
                // 同模块唯一（deleted=0 范围）
                if FieldDefModel::count_by_key(txn, &module, &field_key, &None)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?
                    > 0
                {
                    return Err(Error::from(format!(
                        "字段键「{}」在该模块下已存在",
                        field_key
                    )));
                }
                let mut dto: FieldDefSaveDTO = form_data.into();
                dto.module = Some(module.clone());
                dto.field_key = Some(field_key);
                dto.field_label = Some(field_label);
                dto.field_type = Some(field_type);
                dto.required = Some(required);
                dto.create_by = operator.clone();
                dto.update_by = operator;
                let new_id = FieldDefModel::insert(txn, &dto)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
                Ok((new_id, module))
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 9. 变更即失效（内存校验缓存 + Redis schema 缓存；事务提交成功后执行）
    invalidate_caches(&module).await;
    Ok(new_id)
}

/// 更新字段定义（生命周期锁定：module/field_key/field_type 不可改 P0-12/P0-17；choices value 不可改/删 P0-18）
/// 锁定判定与更新同事务执行，防并发窗口内定义被改/删后仍按旧快照放行
pub async fn update(db: &DbConn, form: &FieldDefUpdateRequest, operator: Option<String>) -> Result<i64> {
    let id = form.id.ok_or_else(|| Error::from("字段ID不能为空"))?;
    let mut form_data = form.clone();
    // 角色配置归一化：管理页"留空"提交 []，归一为 JSON null（未配置语义，对齐文档「NULL=跟随 visible_roles」）；
    // 保留 Some(JSON null) 而非归 None，使下方 update_by_id 的部分更新语义仍区分「未提交=沿用旧值」与「提交空=放开限制」
    form_data.visible_roles = normalize_roles(form_data.visible_roles);
    form_data.editable_roles = normalize_roles(form_data.editable_roles);

    let (affected, old_module) = db
        .transaction::<_, (i64, String), Error>(|txn| {
            Box::pin(async move {
                // 1. 旧值
                let old = FieldDefModel::find_by_id(txn, id)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?
                    .ok_or_else(|| Error::from("字段定义不存在"))?;
                let old_module = old.module.clone().unwrap_or_default();

                // 2. 三字段锁定（提交了才比对，未提交视为沿用旧值）
                if let Some(m) = &form_data.module {
                    if m.trim() != old_module {
                        return Err(Error::from("所属模块创建后不可修改"));
                    }
                }
                if let Some(k) = &form_data.field_key {
                    if k.trim() != old.field_key.clone().unwrap_or_default() {
                        return Err(Error::from("字段键创建后不可修改"));
                    }
                }
                if let Some(t) = form_data.field_type {
                    if t != old.field_type.unwrap_or(1) {
                        return Err(Error::from("字段类型创建后不可修改，如需更换请新建字段并做数据迁移"));
                    }
                }

                // 3. choices value 锁定（P0-18）：旧 value 必须全部保留（不可改/删），允许新增
                let effective_type = form_data.field_type.unwrap_or_else(|| old.field_type.unwrap_or(1));
                if effective_type == 6 || effective_type == 7 {
                    if let Some(new_options) = &form_data.options {
                        let new_values = extract_choice_values(new_options)?;
                        if let Some(old_options) = &old.options {
                            for v in extract_choice_values(old_options).unwrap_or_default() {
                                if !new_values.contains(&v) {
                                    return Err(Error::from(format!(
                                        "选项 value「{}」可能已存在业务数据，不可删除或修改，仅可停用（active=0）",
                                        v
                                    )));
                                }
                            }
                        }
                    }
                }

                // 3.5 数字/金额 options 精度校验（仅提交 options 时生效，L210 options 规范）
                check_number_precision(effective_type, &form_data.options)?;

                // 4. 必填 × 角色约束（P0-19）：required/visible_roles 均按"新值优先、旧值兜底"合并后校验
                let required = form_data.required.unwrap_or_else(|| old.required.unwrap_or(0));
                let visible_roles = match &form_data.visible_roles {
                    Some(v) => Some(v.clone()),
                    None => old.visible_roles.clone(),
                };
                check_required_visible(required, &visible_roles)?;

                // 5. 更新（label 可改、choices 可加可停用，其余配置项可调）
                let mut dto: FieldDefSaveDTO = form_data.into();
                dto.update_by = operator;
                let affected = FieldDefModel::update_by_id(txn, &Some(id), &dto)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
                Ok((affected, old_module))
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 6. 变更即失效（事务提交成功后执行）
    invalidate_caches(&old_module).await;
    Ok(affected)
}

/// 停用/启用字段定义（存在性校验与状态更新同事务，防并发窗口内定义被删后照常更新）
pub async fn update_status(db: &DbConn, id: i64, status: i32) -> Result<i64> {
    if status != 0 && status != 1 {
        return Err(Error::from("无效的状态值，仅支持 0=停用 1=启用"));
    }
    let (affected, module) = db
        .transaction::<_, (i64, String), Error>(|txn| {
            Box::pin(async move {
                let old = FieldDefModel::find_by_id(txn, id)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?
                    .ok_or_else(|| Error::from("字段定义不存在"))?;
                let affected = FieldDefModel::update_status(txn, id, status)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
                Ok((affected, old.module.unwrap_or_default()))
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    invalidate_caches(&module).await;
    Ok(affected)
}

/// 逻辑删除字段定义（不清理业务表存量数据，恢复零成本；module 收集与删除同事务，防并发新建漏失效缓存）
pub async fn batch_delete(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let ids_in_txn = ids.clone();
    let (affected, modules) = db
        .transaction::<_, (i64, HashSet<String>), Error>(|txn| {
            Box::pin(async move {
                // 收集涉及的 module 用于缓存失效
                let mut modules: HashSet<String> = HashSet::new();
                for &id in &ids_in_txn {
                    if let Ok(Some(model)) = FieldDefModel::find_by_id(txn, id).await {
                        if let Some(m) = model.module {
                            modules.insert(m);
                        }
                    }
                }
                let affected = FieldDefModel::soft_delete_by_ids(txn, &ids_in_txn)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
                Ok((affected, modules))
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    for m in &modules {
        invalidate_caches(m).await;
    }
    Ok(affected)
}

/// 分页查询字段定义
pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<FieldDefListVO>>> {
    let select_where = PageWhere {
        module: query.module,
        keyword: query.keyword,
        status: query.status,
    }
    .format();

    let (list, _num_pages) = FieldDefModel::select_in_page(
        db,
        query.page_num.unwrap_or(1),
        query.page_size.unwrap_or(10),
        select_where.clone(),
    )
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    let list_data: Vec<FieldDefListVO> = list.into_iter().map(FieldDefListVO::from).collect();
    let count = FieldDefModel::select_count(db, select_where).await.unwrap_or(0);
    Ok(ResultPage::new_simple(list_data, count))
}

/// 字段定义详情
pub async fn get_detail(db: &DbConn, id: i64) -> Result<FieldDefListVO> {
    let model = FieldDefModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from(format!("{}={}", "字段定义不存在，id", id)))?;
    Ok(FieldDefListVO::from(model))
}

// ============================ 运行侧 schema ============================

/// 运行侧 schema 下发：Redis 60s 缓存存未过滤全量，角色过滤在缓存外执行（P0-23 防串角色）
/// 安全边界在后端：字段 label 可能含敏感内部语义，无角色者不得通过直连接口获取
pub async fn get_schema(db: &DbConn, module: &str, is_admin: bool, role_keys: &[String]) -> Result<Vec<SchemaItem>> {
    if resolve_table(module).is_none() {
        return Err(Error::from(format!("业务模块「{}」未接入自定义字段", module)));
    }

    let cache_key = schema_cache_key(module);
    let all: Vec<SchemaItem> = match CONTEXT.cache_service.get_json::<Vec<SchemaItem>>(&cache_key).await {
        Ok(items) => items,
        Err(_) => {
            // 缓存 miss 回源：仅 status=1 且 deleted=0
            let models = FieldDefModel::find_by_module(db, module, true)
                .await
                .map_err(|e| Error::from(e.to_string()))?;
            let items: Vec<SchemaItem> = models.into_iter().map(SchemaItem::from).collect();
            // 回填全量带 TTL（设计文档 7.2：60s 兜底过期；管理侧变更即主动失效，TTL 仅兜底脏读窗口）
            if let Err(e) = CONTEXT
                .cache_service
                .set_json_ex(&cache_key, &items, Some(Duration::from_secs(SCHEMA_CACHE_TTL_SECS)))
                .await
            {
                log::warn!("[field_schema] 缓存回填失败: module={} err={}", module, e);
            }
            items
        }
    };

    let list = all
        .into_iter()
        .filter(|item| field_visible(item, is_admin, role_keys))
        .collect();
    Ok(list)
}

/// 管理侧变更后失效缓存（内存校验缓存立即移除 + Redis schema 缓存 DEL）
async fn invalidate_caches(module: &str) {
    if let Ok(mut map) = validator_cache().write() {
        map.remove(module);
    }
    if let Err(e) = CONTEXT.cache_service.del(&schema_cache_key(module)).await {
        log::warn!(
            "[field_schema] 缓存失效失败(等待 {}s TTL 兜底): module={} err={}",
            SCHEMA_CACHE_TTL_SECS,
            module,
            e
        );
    }
}

// ============================ 校验器（业务写路径） ============================

/// 业务模块保存链路的自定义字段校验器（7.3，P1 强校验）
/// - 未知键/停用键/已删除键：400 拒绝（P1-11）
/// - 类型不符：400 拒绝；可安全转型时原地归一化（数字字符串→number、金额 number→字符串）（P1-8）
/// - 必填：新建（is_create=true）缺失 400；编辑放行（7.4 规则 5 存量弱约束，P0-21）
/// - P0-24 越权拦截：editable_roles 限制且不含当前角色，且本次提交值发生变化 → 400
/// - 7.3 防丢失更新：编辑时将旧值中未被本次提交覆盖的键合并回写（应用层按 key 合并，等价 jsonb_set）
/// - 调用方拿到的 custom_fields 已是（合并+归一化）结果，须赋回 DTO 落库
/// - 在调用方事务内执行（ConnectionTrait 泛型，DatabaseTransaction 可直接传入）
/// - is_create 为调用方业务语义（新建/编辑）：必填强约束仅新建生效。不得用 old_custom_fields 是否为空代替——
///   老记录 custom_fields 列为 NULL 时编辑会传 None，误判为新建导致必填强校验错误拦截存量编辑（7.4 规则 5/P0-21 破坏）
pub async fn validate_custom_fields<C: ConnectionTrait>(
    db: &C,
    module: &str,
    custom_fields: &mut Option<serde_json::Value>,
    user_id: i64,
    is_create: bool,
    old_custom_fields: Option<&serde_json::Value>,
) -> Result<()> {
    // 未接入模块直接放行
    if resolve_table(module).is_none() {
        return Ok(());
    }
    // 提交了非对象结构（数组/字符串等）→ 400
    if let Some(v) = custom_fields.as_ref() {
        if !v.is_object() {
            return Err(Error::from("自定义字段 custom_fields 必须为 JSON 对象"));
        }
    }

    // 仅「启用中」字段参与校验（7.3 键白名单；30s 内存缓存，校验零网络开销）
    let defs = get_active_defs_cached(db, module).await?;
    let def_map: HashMap<&str, &field_def::Model> = defs
        .iter()
        .filter_map(|d| d.field_key.as_deref().map(|k| (k, d)))
        .collect();

    if let Some(serde_json::Value::Object(map)) = custom_fields.as_mut() {
        if !map.is_empty() {
            let (is_admin, role_keys) = load_user_role(db, user_id).await?;

            for (key, value) in map.iter_mut() {
                let def = match def_map.get(key.as_str()) {
                    Some(d) => *d,
                    None => {
                        // P1-11：未定义/已停用/已删除的键一律 400 拒写
                        return Err(Error::from(format!(
                            "自定义字段「{}」未定义或已停用，无法写入",
                            key
                        )));
                    }
                };
                let label = def
                    .field_label
                    .clone()
                    .unwrap_or_else(|| key.to_string());

                // P0-24：editable_roles 限制且不含当前角色，且本次提交改变了该字段值 → 拒绝
                let old_val = old_custom_fields.and_then(|o| o.get(key.as_str()));
                let field_type = def.field_type.unwrap_or(1);

                // null 视为清空该键（不做类型校验；必填字段由下方必填校验兜底）
                if value.is_null() {
                    // 清空也是值变化：受限字段同样拦截（保持原语义）
                    if old_val.is_some() && !field_editable(def, is_admin, &role_keys) {
                        return Err(Error::from(format!(
                            "自定义字段「{}」不允许当前角色编辑",
                            label
                        )));
                    }
                    continue;
                }

                // P1 强校验：类型不符 400，可安全转型的值原地归一化（7.3 数值归一化）
                let normalized = match check_and_normalize_value(field_type, value) {
                    Ok(n) => n,
                    Err(e) => {
                        return Err(Error::from(format!("自定义字段「{}」: {}", label, e)));
                    }
                };

                // P0-24 值变化判定须在归一化之后、旧值做同源归一化后比较：
                // 控件往返存在格式漂移（金额 "100.50"→InputNumber→"100.5"、附件/成员 [1,2]→["1","2"]、
                // 单选数字 value 1→"1"），受限字段若按原始值比较会把"未修改"误判为"已变化"→ 400 卡死普通用户整表单。
                // 归一化后仍可能数值等价而字面不等（金额尾零），故再过一层类型感知等价判定；
                // 旧值归一化失败（历史脏数据）时回退原始比较，不放大拦截面。
                let changed = match old_val {
                    None => true,
                    Some(o) => {
                        let o_norm =
                            check_and_normalize_value(field_type, o).unwrap_or_else(|_| o.clone());
                        o_norm != normalized && !values_equivalent(field_type, &o_norm, &normalized)
                    }
                };
                if changed && !field_editable(def, is_admin, &role_keys) {
                    return Err(Error::from(format!(
                        "自定义字段「{}」不允许当前角色编辑",
                        label
                    )));
                }
                *value = normalized;
            }
        }
    }

    // 7.4 规则 5：必填字段新建强约束、存量弱约束（编辑不阻断，P0-21）
    // 判定用显式 is_create（调用方业务语义），不能以 old_custom_fields 是否为空代替（见函数文档）
    if is_create {
        for d in &defs {
            if d.required != Some(1) {
                continue;
            }
            let key = d.field_key.as_deref().unwrap_or_default();
            if key.is_empty() {
                continue;
            }
            let provided = custom_fields
                .as_ref()
                .and_then(|v| v.get(key))
                .map(|v| !v.is_null())
                .unwrap_or(false);
            if !provided {
                let label = d
                    .field_label
                    .clone()
                    .unwrap_or_else(|| key.to_string());
                return Err(Error::from(format!("自定义字段「{}」为必填项", label)));
            }
        }
    }

    // 7.3 防丢失更新：编辑时将旧值中未被本次提交覆盖的键合并回来
    // （应用层按 key 合并，整体落库效果等价 jsonb_set(base, '{key}', value, true)；停用键存量值由此保留）
    if let (Some(serde_json::Value::Object(new_map)), Some(serde_json::Value::Object(old_map))) =
        (custom_fields.as_mut(), old_custom_fields)
    {
        for (k, v) in old_map {
            new_map.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }

    Ok(())
}

/// 校验器元数据 get_or_load（进程内 30s 兜底 TTL，变更即失效；管理侧写路径调 invalidate_caches）
pub async fn get_active_defs_cached<C: ConnectionTrait>(
    db: &C,
    module: &str,
) -> Result<Vec<field_def::Model>> {
    if let Ok(map) = validator_cache().read() {
        if let Some(entry) = map.get(module) {
            if entry.loaded_at.elapsed() < Duration::from_secs(VALIDATOR_TTL_SECS) {
                return Ok(entry.defs.clone());
            }
        }
    }
    let defs = FieldDefModel::find_by_module(db, module, true)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if let Ok(mut map) = validator_cache().write() {
        map.insert(
            module.to_string(),
            ValidatorEntry {
                defs: defs.clone(),
                loaded_at: Instant::now(),
            },
        );
    }
    Ok(defs)
}

/// 查询当前用户角色上下文：(是否超管, role_key 列表)
/// 超管口径与 permission_cache_service 一致（user_type=1）；
/// 角色查询用原生 SQL，使校验器可在调用方事务内执行（AdminRoleMergeModel 仅接受 &DbConn）
pub async fn load_user_role<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<(bool, Vec<String>)> {
    let is_admin = match admin::Entity::find_by_id(user_id).one(db).await {
        Ok(Some(a)) => a.user_type == Some(1),
        _ => false,
    };

    let sql = r#"SELECT r.role_key FROM mxx_system_admin_role_merge m
                 INNER JOIN mxx_system_role r ON r.id = m.role_id
                 WHERE m.admin_id = $1 AND COALESCE(r.deleted, 0) = 0 AND COALESCE(r.status, 1) = 1"#;
    let mut role_keys = Vec::new();
    if let Ok(rows) = db
        .query_all_raw(Statement::from_sql_and_values(
            db.get_database_backend(),
            sql,
            [user_id.into()],
        ))
        .await
    {
        for row in rows {
            if let Ok(Some(k)) = row.try_get::<Option<String>>("", "role_key") {
                if !k.is_empty() {
                    role_keys.push(k);
                }
            }
        }
    }
    Ok((is_admin, role_keys))
}

/// 字段对当前用户是否可编辑：超管恒可编辑；editable_roles 空 → 跟随 visible_roles；两者都空 → 全员可编辑
fn field_editable(def: &field_def::Model, is_admin: bool, role_keys: &[String]) -> bool {
    if is_admin {
        return true;
    }
    let effective = match &def.editable_roles {
        Some(v) if is_role_array(v) => v,
        _ => match &def.visible_roles {
            Some(v) if is_role_array(v) => v,
            _ => return true,
        },
    };
    roles_intersect(effective, role_keys)
}

/// 字段对当前用户是否可见（schema 过滤用）：NULL=全部可见；数组=交集判定
fn field_visible(item: &SchemaItem, is_admin: bool, role_keys: &[String]) -> bool {
    if is_admin {
        return true;
    }
    match &item.visible_roles {
        None | Some(serde_json::Value::Null) => true,
        Some(v) if is_role_array(v) => roles_intersect(v, role_keys),
        _ => true,
    }
}

/// 是否为有效的角色限制数组（空数组=明确限制：仅超管可见/可编辑；
/// 管理侧 save/update 落库前已把空数组归一为 JSON null，此处 [] 仅兼容历史数据/手工 DB 配置）
/// pub：供 field_perm_service 复用（P2-1 同语义角色判定）
pub fn is_role_array(v: &serde_json::Value) -> bool {
    matches!(v, serde_json::Value::Array(_))
}

/// 角色配置归一化：空数组 → JSON null（未配置语义，对齐文档「NULL=跟随 visible_roles」）
/// 防止管理页"留空"提交 [] 落库后被 field_editable/field_visible 判为"仅超管"
/// pub：供 field_perm_service 复用（P2-1 落库归一化）
pub fn normalize_roles(v: Option<serde_json::Value>) -> Option<serde_json::Value> {
    match v {
        Some(serde_json::Value::Array(ref a)) if a.is_empty() => Some(serde_json::Value::Null),
        other => other,
    }
}

pub fn roles_intersect(roles_json: &serde_json::Value, role_keys: &[String]) -> bool {
    match roles_json {
        serde_json::Value::Array(arr) => arr.iter().any(|v| {
            v.as_str()
                .map(|s| role_keys.iter().any(|k| k == s))
                .unwrap_or(false)
        }),
        _ => true,
    }
}

// ============================ 校验辅助 ============================

fn field_key_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z][a-z0-9_]{1,63}$").unwrap())
}

/// 字段键格式白名单校验（^[a-z][a-z0-9_]{1,63}$）；pub 供 field_index_service 防御性复用（索引名/表达式内联拼 SQL 前兜底）
pub fn check_field_key_format(field_key: &str) -> Result<()> {
    if !field_key_regex().is_match(field_key) {
        return Err(Error::from("字段键格式错误：必须以小写字母开头，仅含小写字母/数字/下划线，长度 2-64"));
    }
    Ok(())
}

/// 必填字段必须全员可见（P0-19）：required=1 时 visible_roles 必须为 NULL
fn check_required_visible(required: i32, visible_roles: &Option<serde_json::Value>) -> Result<()> {
    if required == 1 {
        if let Some(v) = visible_roles {
            if is_role_array(v) {
                return Err(Error::from("必填字段必须全员可见，不能限制可见角色"));
            }
        }
    }
    Ok(())
}

/// 校验 choices 结构并提取 value 列表（field_type=6/7）：至少 1 项、value/label 非空、value 唯一
fn extract_choice_values(options: &serde_json::Value) -> Result<Vec<String>> {
    let choices = options
        .get("choices")
        .ok_or_else(|| Error::from("单选/多选字段必须配置 options.choices"))?;
    let arr = choices
        .as_array()
        .ok_or_else(|| Error::from("choices 必须为数组"))?;
    if arr.is_empty() {
        return Err(Error::from("单选/多选字段至少需要 1 个选项"));
    }
    let mut values = Vec::with_capacity(arr.len());
    let mut seen: HashSet<String> = HashSet::new();
    for (i, item) in arr.iter().enumerate() {
        let value = item
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .trim()
            .to_string();
        if value.is_empty() {
            return Err(Error::from(format!("第 {} 个选项的 value 不能为空", i + 1)));
        }
        let label = item
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .trim()
            .to_string();
        if label.is_empty() {
            return Err(Error::from(format!("第 {} 个选项的 label 不能为空", i + 1)));
        }
        if !seen.insert(value.clone()) {
            return Err(Error::from(format!("选项 value「{}」重复", value)));
        }
        values.push(value);
    }
    Ok(values)
}

/// 数字/金额字段 options 精度校验（field_type=3/11，L210 options 规范：{"precision":2,"min":0,"max":9999}）
/// - precision：若配置必须为 0-10 整数（前端 InputNumber/金额格式化的精度上限）
/// - min/max：若配置必须为数字且 min ≤ max
/// - 未配置 options / precision / min/max 时放行（均非必配项）
fn check_number_precision(field_type: i32, options: &Option<serde_json::Value>) -> Result<()> {
    if field_type != 3 && field_type != 11 {
        return Ok(());
    }
    let Some(options) = options else {
        return Ok(());
    };
    if options.is_null() {
        return Ok(());
    }
    if !options.is_object() {
        return Err(Error::from("数字/金额字段 options 必须为 JSON 对象"));
    }
    match options.get("precision") {
        None | Some(serde_json::Value::Null) => {}
        Some(v) => match v.as_i64() {
            Some(p) if (0..=10).contains(&p) => {}
            _ => return Err(Error::from("options.precision 必须为 0-10 的整数")),
        },
    }
    let get_num = |k: &str| -> Result<Option<f64>> {
        match options.get(k) {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(v) => v
                .as_f64()
                .map(Some)
                .ok_or_else(|| Error::from(format!("options.{} 必须为数字", k))),
        }
    };
    let min = get_num("min")?;
    let max = get_num("max")?;
    if let (Some(min), Some(max)) = (min, max) {
        if min > max {
            return Err(Error::from("options.min 不能大于 options.max"));
        }
    }
    Ok(())
}

/// 值等价判定（P0-24 变化检测专用）：控件往返格式漂移不视为值变化
/// - 数字/金额/单选值：数值等价（"100.50"≡100.5≡"100.5"、1≡"1"），parse 失败则严格相等
/// - 数组（多选/附件/成员）：长度一致且逐元素数值等价（[1,2]≡["1","2"]）
/// - 其余类型：严格相等（serde_json::Value PartialEq）
fn values_equivalent(field_type: i32, a: &serde_json::Value, b: &serde_json::Value) -> bool {
    if a == b {
        return true;
    }
    let num_of = |v: &serde_json::Value| -> Option<f64> {
        match v {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => s.trim().parse::<f64>().ok(),
            _ => None,
        }
    };
    match field_type {
        // 3 数字 / 11 金额 / 6 单选（choices value 历史数字脏数据兜底）
        3 | 6 | 11 => match (num_of(a), num_of(b)) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        },
        // 7 多选 / 9 附件 / 10 成员：数组逐元素等价
        7 | 9 | 10 => match (a.as_array(), b.as_array()) {
            (Some(x), Some(y)) => {
                x.len() == y.len()
                    && x.iter().zip(y.iter()).all(|(u, v)| values_equivalent(3, u, v))
            }
            _ => false,
        },
        _ => false,
    }
}

/// 按 6.2 值结构约定做强校验 + 写入时归一化（P1-8/P1-9）
/// - 类型相符：原值返回
/// - 可安全转型：数字字符串→number（3）、number→金额字符串（11）
/// - 其余类型不符：400 拒绝（外层错误信息拼接字段 label，指明字段与原因）
fn check_and_normalize_value(field_type: i32, value: &serde_json::Value) -> Result<serde_json::Value> {
    let type_err = |expect: &str| {
        Error::from(format!(
            "值类型不符，需要 {}（实际为 {}）",
            expect,
            match value {
                serde_json::Value::Null => "null".to_string(),
                serde_json::Value::Bool(b) => format!("布尔值 {}", b),
                serde_json::Value::Number(n) => format!("数字 {}", n),
                serde_json::Value::String(s) => format!("字符串「{}」", s),
                serde_json::Value::Array(_) => "数组".to_string(),
                serde_json::Value::Object(_) => "对象".to_string(),
            }
        ))
    };
    match field_type {
        // 1 文本 / 2 多行文本 / 6 单选：string
        1 | 2 | 6 => {
            if value.is_string() {
                Ok(value.clone())
            } else {
                Err(type_err("字符串"))
            }
        }
        // 3 数字：number；数字字符串可转型（整数优先保精度）
        3 => {
            if value.is_number() {
                Ok(value.clone())
            } else if let Some(s) = value.as_str() {
                let t = s.trim();
                if let Ok(n) = t.parse::<i64>() {
                    Ok(serde_json::json!(n))
                } else if let Ok(f) = t.parse::<f64>() {
                    serde_json::Number::from_f64(f)
                        .map(serde_json::Value::Number)
                        .ok_or_else(|| Error::from(format!("数字值「{}」无效", s)))
                } else {
                    Err(type_err("数字"))
                }
            } else {
                Err(type_err("数字"))
            }
        }
        // 4 日期：字符串 "YYYY-MM-DD"
        4 => match value.as_str() {
            Some(s) if chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() => Ok(value.clone()),
            Some(s) => Err(Error::from(format!(
                "日期格式不符，需为 YYYY-MM-DD（实际「{}」）",
                s
            ))),
            None => Err(type_err("日期字符串")),
        },
        // 5 日期时间：字符串 "YYYY-MM-DD HH:mm:ss"
        5 => match value.as_str() {
            Some(s) if chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").is_ok() => Ok(value.clone()),
            Some(s) => Err(Error::from(format!(
                "日期时间格式不符，需为 YYYY-MM-DD HH:mm:ss（实际「{}」）",
                s
            ))),
            None => Err(type_err("日期时间字符串")),
        },
        // 7 多选 / 9 附件 / 10 成员：数组
        7 | 9 | 10 => {
            if value.is_array() {
                Ok(value.clone())
            } else {
                Err(type_err("数组"))
            }
        }
        // 8 布尔：boolean
        8 => {
            if value.is_boolean() {
                Ok(value.clone())
            } else {
                Err(type_err("布尔值"))
            }
        }
        // 11 金额：ISO 数字字符串（保留 Decimal 精度，禁用 number 防 0.1+0.2 漂移）；number 可转型为字符串
        11 => {
            if let Some(s) = value.as_str() {
                let t = s.trim();
                if t.parse::<f64>().is_ok() {
                    Ok(serde_json::Value::String(t.to_string()))
                } else {
                    Err(Error::from(format!(
                        "金额格式不符，需为数字字符串（实际「{}」）",
                        s
                    )))
                }
            } else if value.is_number() {
                // number → 金额字符串，按字面量形式落库避免精度漂移
                Ok(serde_json::Value::String(value.to_string()))
            } else {
                Err(type_err("金额数字字符串"))
            }
        }
        // 未知 field_type：不拦截
        _ => Ok(value.clone()),
    }
}

/// 查询模块标准列名集合（information_schema 实时获取，黑名单非硬编码）
async fn get_standard_columns<C: ConnectionTrait>(db: &C, table: &str) -> Result<HashSet<String>> {
    let sql = "SELECT column_name FROM information_schema.columns WHERE table_schema = current_schema() AND table_name = $1";
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(
            db.get_database_backend(),
            sql,
            [table.into()],
        ))
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let mut columns = HashSet::new();
    for row in rows {
        if let Ok(Some(name)) = row.try_get::<Option<String>>("", "column_name") {
            columns.insert(name);
        }
    }
    Ok(columns)
}

// ============================ 列表筛选/排序翻译器（P1-2/P1-3/P1-5/P1-15） ============================

/// 自定义字段筛选翻译器（7.3）：按元数据 field_type 生成与 6.3 表达式索引同源的 SimpleExpr
/// - 转型规则与索引完全一致（12 风险表）：文本/单选/日期裸 ->>，数字/金额 ::numeric，布尔 ::boolean，否则索引不命中
/// - key 先过格式白名单（^[a-z][a-z0-9_]{1,63}$），值一律参数绑定，双重防注入（P1-2）
/// - 数组型（7/9/10）走 custom_fields @> ?::jsonb 包含语义，命中模块级 GIN jsonb_path_ops 索引（P1-5/P1-15）
/// - 元数据走 30s 进程内缓存，筛选零额外网络开销；字段不存在/已停用 400（7.4 规则 6：停用后不再出现于筛选）
pub async fn build_filter_expr<C: ConnectionTrait>(
    db: &C,
    module: &str,
    key: &str,
    op: &str,
    val: &serde_json::Value,
) -> Result<SimpleExpr> {
    // 函数级导入：ExprTrait 为 blanket impl，文件级导入会遮蔽旧代码中 serde_json::Value 的固有方法
    use sea_orm::sea_query::ExprTrait;
    check_field_key_format(key)?;
    if resolve_table(module).is_none() {
        return Err(Error::from(format!("业务模块「{}」未接入自定义字段", module)));
    }
    let def = find_def_by_key(db, module, key).await?;
    let field_type = def.field_type.unwrap_or(1);
    let label = def.field_label.clone().unwrap_or_else(|| key.to_string());
    let bad_op = |allow: &str| {
        Error::from(format!(
            "自定义字段「{}」不支持筛选操作符「{}」（支持：{}）",
            label, op, allow
        ))
    };
    let bad_val = |expect: &str| {
        Error::from(format!(
            "自定义字段「{}」筛选值类型不符，需要 {}",
            label, expect
        ))
    };
    // 标量字段取值表达式：custom_fields->>'{key}'（key 已过白名单，且走参数绑定）
    let text_left = Expr::cust_with_values("custom_fields->>?", [key.to_string()]);

    match field_type {
        // 数字/金额（3/11）：(custom_fields->>?)::numeric 数值比较；金额为字符串存储，右侧同样 ::numeric 保精度
        3 | 11 => {
            let v = match val {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) if s.trim().parse::<f64>().is_ok() => s.trim().to_string(),
                _ => return Err(bad_val("数字")),
            };
            let left = Expr::cust_with_values("(custom_fields->$1)::numeric", [key.to_string()]);
            let right = Expr::cust_with_values("$1::numeric", [v]);
            match op {
                "eq" => Ok(left.eq(right)),
                "ne" => Ok(left.ne(right)),
                "gt" => Ok(left.gt(right)),
                "gte" => Ok(left.gte(right)),
                "lt" => Ok(left.lt(right)),
                "lte" => Ok(left.lte(right)),
                _ => Err(bad_op("eq/ne/gt/gte/lt/lte")),
            }
        }
        // 布尔（8）：(custom_fields->>?)::boolean
        8 => {
            let b = match val {
                serde_json::Value::Bool(b) => *b,
                _ => return Err(bad_val("布尔值")),
            };
            let left = Expr::cust_with_values("(custom_fields->>?)::boolean", [key.to_string()]);
            match op {
                "eq" => Ok(left.eq(b)),
                "ne" => Ok(left.ne(b)),
                _ => Err(bad_op("eq/ne")),
            }
        }
        // 日期/日期时间（4/5）：ISO 字典序=时间序，裸 ->> 字符串比较与索引一致，无需转型
        4 | 5 => {
            let s = val
                .as_str()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| bad_val("日期字符串"))?;
            match op {
                "eq" => Ok(text_left.eq(s.to_string())),
                "gt" => Ok(text_left.gt(s.to_string())),
                "gte" => Ok(text_left.gte(s.to_string())),
                "lt" => Ok(text_left.lt(s.to_string())),
                "lte" => Ok(text_left.lte(s.to_string())),
                _ => Err(bad_op("eq/gt/gte/lt/lte")),
            }
        }
        // 数组型（7/9/10）：包含语义，参数形如 {"key":["官网"]}，命中 GIN jsonb_path_ops
        7 | 9 | 10 => {
            if op != "contains" {
                return Err(bad_op("contains"));
            }
            let arr = match val {
                serde_json::Value::Array(a) => a.clone(),
                serde_json::Value::Null => return Err(bad_val("非空数组或单个值")),
                v => vec![v.clone()],
            };
            let json = serde_json::json!({ key: arr });
            Ok(Expr::cust_with_values("custom_fields @> ?::jsonb", [json.to_string()]))
        }
        // 文本（1/2）/单选（6）：字符串直用；数字/布尔为 query 反序列化副作用（纯数字文本经 JSON 解析变数字），字符串化容错
        _ => {
            let s = match val {
                serde_json::Value::String(v) => Some(v.clone()),
                serde_json::Value::Number(n) => Some(n.to_string()),
                serde_json::Value::Bool(b) => Some(b.to_string()),
                serde_json::Value::Null => None,
                _ => return Err(bad_val("字符串")),
            };
            match op {
                "eq" => match s {
                    // 筛「为空」：jsonb 键不存在或值为 null，->> 结果均为 NULL
                    None => Ok(text_left.is_null()),
                    Some(v) => Ok(text_left.eq(v)),
                },
                "ne" => Ok(text_left.ne(s.ok_or_else(|| bad_val("字符串"))?)),
                "like" => {
                    let v = s.ok_or_else(|| bad_val("字符串"))?;
                    // ILIKE 内联于 cust SQL，key 与模糊值均参数绑定（sea-query 1.0 的 ilike 在 postgres 扩展 trait 上）
                    Ok(Expr::cust_with_values(
                        "custom_fields->>? ILIKE ?",
                        [key.to_string(), format!("%{}%", escape_like(&v))],
                    ))
                }
                _ => Err(bad_op("eq/ne/like")),
            }
        }
    }
}

/// 自定义字段排序翻译器（P1-3）：数字/金额 ::numeric 按数值序（非字典序），布尔 ::boolean，
/// 日期 ISO 字典序=时间序；转型与 6.3 索引同源。数组型（7/9/10）无序语义拒绝（8.2 末）。
/// 排序方向由调用方以 Order::Asc/Desc 追加，建议再挂次级排序（如 create_time）保证分页稳定。
pub async fn build_order_expr<C: ConnectionTrait>(
    db: &C,
    module: &str,
    key: &str,
) -> Result<SimpleExpr> {
    check_field_key_format(key)?;
    if resolve_table(module).is_none() {
        return Err(Error::from(format!("业务模块「{}」未接入自定义字段", module)));
    }
    let def = find_def_by_key(db, module, key).await?;
    let field_type = def.field_type.unwrap_or(1);
    match field_type {
        3 | 11 => Ok(Expr::cust_with_values("(custom_fields->>?)::numeric", [key.to_string()])),
        8 => Ok(Expr::cust_with_values("(custom_fields->>?)::boolean", [key.to_string()])),
        7 | 9 | 10 => Err(Error::from(format!(
            "自定义字段「{}」为多选/成员/附件类型，不支持排序",
            def.field_label.clone().unwrap_or_else(|| key.to_string())
        ))),
        // 1/2/4/5/6：裸 ->> 字符串排序（日期 ISO 字典序=时间序）
        _ => Ok(Expr::cust_with_values("custom_fields->$1", [key.to_string()])),
    }
}

/// 按模块+key 取启用中的字段定义（复用校验器 30s 内存缓存，命中时零查询）
async fn find_def_by_key<C: ConnectionTrait>(db: &C, module: &str, key: &str) -> Result<field_def::Model> {
    get_active_defs_cached(db, module)
        .await?
        .into_iter()
        .find(|d| d.field_key.as_deref() == Some(key))
        .ok_or_else(|| Error::from(format!("自定义字段「{}」不存在或已停用，无法筛选", key)))
}

/// ILIKE 通配符转义（% _ \），防筛选值扩大匹配范围（PostgreSQL 默认转义符为反斜杠）
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}
