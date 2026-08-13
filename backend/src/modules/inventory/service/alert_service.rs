//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::collections::{HashMap, HashSet};
use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::model::alert::{self as alert_model, AlertRuleListQuery, AlertRuleListVO, AlertRuleListItem, AlertRuleSaveRequest};
use crate::modules::product::entity::product as product_entity;
use crate::modules::inventory::entity::warehouse as warehouse_entity;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

/// 获取预警规则列表
pub async fn get_list(db: &DatabaseConnection, query: &AlertRuleListQuery) -> Result<AlertRuleListVO> {
    let page = query.page_num.unwrap_or(1) as u64;
    let page_size = query.page_size.unwrap_or(10) as u64;

    // 名称模糊匹配解析为 ID 列表（空 Vec 表示名称匹配不到任何记录）
    let product_ids: Option<Vec<i64>> = match query.product_name.as_ref().map(|s| s.trim()) {
        Some(name) if !name.is_empty() => {
            let rows = product_entity::Entity::find()
                .filter(product_entity::Column::Deleted.eq(0))
                .filter(product_entity::Column::Name.like(format!("%{}%", name)))
                .all(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?;
            Some(rows.into_iter().map(|p| p.id).collect())
        }
        _ => None,
    };

    let warehouse_ids: Option<Vec<i64>> = match query.warehouse_name.as_ref().map(|s| s.trim()) {
        Some(name) if !name.is_empty() => {
            let rows = warehouse_entity::Entity::find()
                .filter(warehouse_entity::Column::Deleted.eq(0))
                .filter(warehouse_entity::Column::Name.like(format!("%{}%", name)))
                .all(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?;
            Some(rows.into_iter().map(|w| w.id).collect())
        }
        _ => None,
    };

    let (models, total) = alert_model::select_page(
        db,
        page,
        page_size,
        query.product_id,
        query.warehouse_id,
        product_ids.as_deref(),
        warehouse_ids.as_deref(),
    )
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    // 批量收集 product_id 和 warehouse_id，避免循环内逐条查询（N+1）
    let product_id_set: HashSet<i64> = models.iter().filter_map(|m| m.product_id).collect();
    let warehouse_id_set: HashSet<i64> = models.iter().filter_map(|m| m.warehouse_id).collect();

    let product_map: HashMap<i64, String> = if product_id_set.is_empty() {
        HashMap::new()
    } else {
        product_entity::Entity::find()
            .filter(product_entity::Column::Id.is_in(product_id_set.into_iter().collect::<Vec<_>>()))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
            .into_iter()
            .filter_map(|p| p.name.map(|n| (p.id, n)))
            .collect()
    };

    let warehouse_map: HashMap<i64, String> = if warehouse_id_set.is_empty() {
        HashMap::new()
    } else {
        warehouse_entity::Entity::find()
            .filter(warehouse_entity::Column::Id.is_in(warehouse_id_set.into_iter().collect::<Vec<_>>()))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
            .into_iter()
            .filter_map(|w| w.name.map(|n| (w.id, n)))
            .collect()
    };

    let mut items = Vec::new();
    for m in models {
        let product_name = if let Some(pid) = m.product_id {
            product_map.get(&pid).cloned()
        } else {
            Some("全部产品".to_string())
        };

        let warehouse_name = if let Some(wid) = m.warehouse_id {
            warehouse_map.get(&wid).cloned()
        } else {
            Some("全部仓库".to_string())
        };

        items.push(AlertRuleListItem {
            id: m.id,
            product_id: m.product_id,
            product_name,
            warehouse_id: m.warehouse_id,
            warehouse_name,
            min_quantity: m.min_quantity,
            max_quantity: m.max_quantity,
            stale_days: m.stale_days,
            enable_low_alert: m.enable_low_alert,
            enable_high_alert: m.enable_high_alert,
            enable_stale_alert: m.enable_stale_alert,
            notify_users: m.notify_users,
            created_by: m.created_by,
            create_time: m.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: m.update_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        });
    }

    Ok(AlertRuleListVO { total: total as i64, items })
}

/// 获取预警规则详情
pub async fn get_detail(db: &DatabaseConnection, id: i64) -> Result<AlertRuleListItem> {
    let m = alert_model::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("预警规则不存在"))?;

    let product_name = if let Some(pid) = m.product_id {
        product_entity::Entity::find_by_id(pid)
            .one(db)
            .await
            .ok()
            .flatten()
            .map(|p| p.name)
            .unwrap_or_default()
    } else {
        Some("全部产品".to_string())
    };

    let warehouse_name = if let Some(wid) = m.warehouse_id {
        warehouse_entity::Entity::find_by_id(wid)
            .one(db)
            .await
            .ok()
            .flatten()
            .map(|w| w.name)
            .unwrap_or_default()
    } else {
        Some("全部仓库".to_string())
    };

    Ok(AlertRuleListItem {
        id: m.id,
        product_id: m.product_id,
        product_name,
        warehouse_id: m.warehouse_id,
        warehouse_name,
        min_quantity: m.min_quantity,
        max_quantity: m.max_quantity,
        stale_days: m.stale_days,
        enable_low_alert: m.enable_low_alert,
        enable_high_alert: m.enable_high_alert,
        enable_stale_alert: m.enable_stale_alert,
        notify_users: m.notify_users,
        created_by: m.created_by,
        create_time: m.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        update_time: m.update_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
    })
}

/// 创建预警规则
pub async fn create(db: &DatabaseConnection, req: &AlertRuleSaveRequest, created_by: i64) -> Result<i64> {
    let id = alert_model::insert(db, req, created_by)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 更新预警规则
pub async fn update(db: &DatabaseConnection, id: i64, req: &AlertRuleSaveRequest, updated_by: i64) -> Result<i64> {
    alert_model::update(db, id, req, updated_by)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 删除预警规则
pub async fn batch_delete(db: &DatabaseConnection, ids: &[i64]) -> Result<i64> {
    alert_model::batch_delete(db, ids)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(ids.len() as i64)
}