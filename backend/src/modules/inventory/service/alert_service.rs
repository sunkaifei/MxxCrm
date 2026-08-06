//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::model::alert::{self as alert_model, AlertRuleListQuery, AlertRuleListVO, AlertRuleListItem, AlertRuleSaveRequest};
use crate::modules::product::entity::product as product_entity;
use crate::modules::inventory::entity::warehouse as warehouse_entity;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

/// 获取预警规则列表
pub async fn get_list(db: &DatabaseConnection, query: &AlertRuleListQuery) -> Result<AlertRuleListVO> {
    let page = query.page_num.unwrap_or(1) as u64;
    let page_size = query.page_size.unwrap_or(10) as u64;

    let (models, total) = alert_model::select_page(db, page, page_size, query.product_id, query.warehouse_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut items = Vec::new();
    for m in models {
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