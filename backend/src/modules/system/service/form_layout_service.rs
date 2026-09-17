//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 模块表单/详情布局服务
//! 布局是独立元数据（mxx_system_form_layout），与字段定义（mxx_system_field_def）分离；
//! 无布局行或停用 = 前端回落现用默认渲染，保证"现用设计不变"。
//! D7 角色差异化：同一模块同一布局类型可存"默认布局(role_key=NULL) + 若干角色专属布局"，
//! 解析顺序 = 用户角色命中的专属布局 > 默认布局 > 无（回落默认渲染）。
//!

use std::collections::HashSet;

use sea_orm::*;
use chrono::Utc;

use crate::core::kit::CONTEXT;
use crate::modules::system::entity::{field_def, form_layout};
use crate::modules::system::model::form_layout::{FormLayoutSaveRequest, FormLayoutVO, LayoutJson};
use crate::modules::system::service::field_def_service;

/// 布局缓存 TTL（秒），缓存存"该模块该类型的全部启用行"，解析按用户角色在内存完成
const LAYOUT_CACHE_TTL_SECS: u64 = 60;

fn layout_cache_key(module: &str, layout_type: i32) -> String {
    format!("form_layout:{}:{}", module, layout_type)
}

/// 取模块布局（按用户角色解析：角色专属优先，否则默认布局；都没有返回 None）
pub async fn get_for_user(
    db: &DbConn,
    module: &str,
    layout_type: i32,
    role_keys: &[String],
) -> Result<Option<FormLayoutVO>, String> {
    if field_def_service::resolve_table(module).is_none() {
        return Err(format!("业务模块「{}」未接入自定义字段", module));
    }
    let cache_key = layout_cache_key(module, layout_type);
    // get_json 未命中返回 Err（与 field schema 缓存同语义），直接回源
    let all: Vec<FormLayoutVO> = match CONTEXT.cache_service.get_json(&cache_key).await {
        Ok(v) => v,
        Err(_) => {
            let models = form_layout::Entity::find()
                .filter(form_layout::Column::Module.eq(module))
                .filter(form_layout::Column::LayoutType.eq(layout_type))
                .filter(form_layout::Column::Deleted.eq(0))
                .filter(form_layout::Column::Status.eq(1))
                .all(db)
                .await
                .map_err(|e| e.to_string())?;
            let vos: Vec<FormLayoutVO> = models
                .into_iter()
                .map(|m| FormLayoutVO {
                    id: m.id,
                    module: m.module.unwrap_or_default(),
                    layout_type: m.layout_type.unwrap_or(1),
                    role_key: m.role_key,
                    layout_json: m.layout_json.unwrap_or(serde_json::json!({})),
                    version: m.version.unwrap_or(1),
                    status: m.status.unwrap_or(1),
                    update_time: m
                        .update_time
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                })
                .collect();
            if let Err(e) = CONTEXT
                .cache_service
                .set_json_ex(&cache_key, &vos, Some(std::time::Duration::from_secs(LAYOUT_CACHE_TTL_SECS)))
                .await
            {
                log::warn!("[form_layout] 缓存回填失败: {} err={}", cache_key, e);
            }
            vos
        }
    };

    // 解析：角色专属（多角色命中取库序第一条）> 默认布局
    let hit = all
        .iter()
        .find(|v| {
            v.role_key
                .as_deref()
                .map(|rk| !rk.is_empty() && role_keys.contains(&rk.to_string()))
                .unwrap_or(false)
        })
        .or_else(|| {
            all.iter()
                .find(|v| v.role_key.as_deref().map(|rk| rk.is_empty()).unwrap_or(true))
        });
    Ok(hit.cloned())
}

/// 保存布局（upsert 按 module+layout_type+role_key 定位；乐观锁 + JSON 校验 + 字段存在性校验）
pub async fn save(db: &DbConn, req: &FormLayoutSaveRequest, operator: Option<String>) -> Result<i64, String> {
    if field_def_service::resolve_table(&req.module).is_none() {
        return Err(format!("业务模块「{}」未接入自定义字段", req.module));
    }
    if req.layout_type != 1 && req.layout_type != 2 {
        return Err("布局类型仅支持 1=表单 2=详情页".to_string());
    }
    let role_key: Option<String> = req
        .role_key
        .as_deref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // 1. 布局 JSON 结构校验
    let layout: LayoutJson = serde_json::from_value(req.layout_json.clone())
        .map_err(|e| format!("布局 JSON 结构不合法：{}", e))?;
    validate_layout(db, &req.module, &layout).await?;

    // 2. version 乐观锁：前端持有旧版本时拒绝覆盖（仅更新场景）
    let now = Utc::now().naive_utc();
    let mut query = form_layout::Entity::find()
        .filter(form_layout::Column::Module.eq(&req.module))
        .filter(form_layout::Column::LayoutType.eq(req.layout_type))
        .filter(form_layout::Column::Deleted.eq(0));
    match &role_key {
        Some(rk) => query = query.filter(form_layout::Column::RoleKey.eq(rk.clone())),
        None => query = query.filter(form_layout::Column::RoleKey.is_null()),
    }
    let existing = query.one(db).await.map_err(|e| e.to_string())?;
    if let Some(old) = &existing {
        if let Some(v) = req.version {
            if v != old.version.unwrap_or(1) {
                return Err(format!(
                    "布局已被他人更新（当前版本 {}，你提交时基于版本 {}），请刷新后重试",
                    old.version.unwrap_or(1), v
                ));
            }
        }
    }

    let new_version = existing.as_ref().map(|o| o.version.unwrap_or(1)).unwrap_or(0) + 1;
    let layout_json = serde_json::json!({
        "version": new_version,
        "tabs": layout.tabs,
        "fields": layout.fields,
        "unassignedPolicy": layout.unassigned_policy,
    });

    let id = match existing {
        Some(old) => {
            let mut active: form_layout::ActiveModel = old.into();
            active.layout_json = Set(Some(layout_json));
            active.version = Set(Some(new_version));
            active.status = Set(Some(1));
            active.update_by = Set(operator.clone());
            active.update_time = Set(Some(now));
            active.update(db).await.map_err(|e| e.to_string())?.id
        }
        None => {
            let active = form_layout::ActiveModel {
                module: Set(Some(req.module.clone())),
                layout_type: Set(Some(req.layout_type)),
                role_key: Set(role_key),
                layout_json: Set(Some(layout_json)),
                version: Set(Some(new_version)),
                status: Set(Some(1)),
                create_by: Set(operator.clone()),
                create_time: Set(Some(now)),
                update_by: Set(operator),
                update_time: Set(Some(now)),
                deleted: Set(Some(0)),
                ..Default::default()
            };
            active.insert(db).await.map_err(|e| e.to_string())?.id
        }
    };

    invalidate_cache(&req.module, req.layout_type).await;
    Ok(id)
}

/// 恢复布局（逻辑删除布局行；role_key 缺省=恢复默认布局，指定则恢复该角色专属布局）
pub async fn reset(
    db: &DbConn,
    module: &str,
    layout_type: i32,
    role_key: Option<String>,
    operator: Option<String>,
) -> Result<i64, String> {
    let role_key: Option<String> = role_key
        .as_deref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let mut query = form_layout::Entity::find()
        .filter(form_layout::Column::Module.eq(module))
        .filter(form_layout::Column::LayoutType.eq(layout_type))
        .filter(form_layout::Column::Deleted.eq(0));
    match &role_key {
        Some(rk) => query = query.filter(form_layout::Column::RoleKey.eq(rk.clone())),
        None => query = query.filter(form_layout::Column::RoleKey.is_null()),
    }
    let existing = query.one(db).await.map_err(|e| e.to_string())?;
    if let Some(old) = existing {
        let mut active: form_layout::ActiveModel = old.into();
        active.deleted = Set(Some(1));
        active.update_by = Set(operator);
        active.update_time = Set(Some(Utc::now().naive_utc()));
        active.update(db).await.map_err(|e| e.to_string())?;
        invalidate_cache(module, layout_type).await;
    }
    // 返回 1 表示语义成功（handle_result 以受影响行数判定成败，0 会被误判为失败）
    Ok(1)
}

/// 布局内容校验：字段必须存在于该模块 field_def（系统行或自定义行）；tab 引用必须存在；span 合法
async fn validate_layout(db: &DbConn, module: &str, layout: &LayoutJson) -> Result<(), String> {
    if layout.fields.is_empty() {
        return Err("布局至少需要编排一个字段".to_string());
    }
    // tabs key 唯一
    let mut tab_keys: HashSet<&str> = HashSet::new();
    for t in &layout.tabs {
        if t.key.trim().is_empty() || t.title.trim().is_empty() {
            return Err("选项卡的 key 与标题不能为空".to_string());
        }
        if !tab_keys.insert(t.key.as_str()) {
            return Err(format!("选项卡 key「{}」重复", t.key));
        }
    }
    // 字段存在性与来源一致性
    let defs = field_def::Entity::find()
        .filter(field_def::Column::Module.eq(module))
        .filter(field_def::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    let def_map: std::collections::HashMap<String, i32> = defs
        .into_iter()
        .map(|d| (d.field_key.unwrap_or_default(), d.is_system.unwrap_or(0)))
        .collect();
    let mut seen: HashSet<String> = HashSet::new();
    for f in &layout.fields {
        if !seen.insert(f.key.clone()) {
            return Err(format!("字段「{}」在布局中重复编排", f.key));
        }
        let Some(&is_system) = def_map.get(&f.key) else {
            return Err(format!("字段「{}」不存在或已删除，请刷新字段池", f.key));
        };
        let expected_source = if is_system == 1 { "system" } else { "field_def" };
        if f.source != expected_source {
            return Err(format!(
                "字段「{}」来源标记错误：应为 {}",
                f.key, expected_source
            ));
        }
        if f.span != 1 && f.span != 2 {
            return Err(format!("字段「{}」列宽仅支持 1=半行 2=整行", f.key));
        }
        if let Some(tab) = &f.tab {
            if !tab_keys.contains(tab.as_str()) {
                return Err(format!("字段「{}」引用了不存在的选项卡「{}」", f.key, tab));
            }
        }
    }
    if layout.unassigned_policy != "append" && layout.unassigned_policy != "hidden" {
        return Err("unassignedPolicy 仅支持 append/hidden".to_string());
    }
    Ok(())
}

async fn invalidate_cache(module: &str, layout_type: i32) {
    if let Err(e) = CONTEXT
        .cache_service
        .del(&layout_cache_key(module, layout_type))
        .await
    {
        log::warn!("[form_layout] 缓存失效失败(等 {}s TTL 兜底): module={} err={}", LAYOUT_CACHE_TTL_SECS, module, e);
    }
}
