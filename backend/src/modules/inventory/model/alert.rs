//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use sea_orm::sea_query::Expr;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use crate::modules::inventory::entity::alert_rule;

// 预警规则列表查询参数
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AlertRuleListQuery {
    #[serde(alias = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub product_id: Option<i64>,
    pub warehouse_id: Option<i64>,
    pub product_name: Option<String>,
    pub warehouse_name: Option<String>,
}

// 预警规则保存请求
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AlertRuleSaveRequest {
    pub product_id: Option<i64>,
    pub warehouse_id: Option<i64>,
    pub min_quantity: Option<Decimal>,
    pub max_quantity: Option<Decimal>,
    pub stale_days: Option<i32>,
    pub enable_low_alert: Option<bool>,
    pub enable_high_alert: Option<bool>,
    pub enable_stale_alert: Option<bool>,
    pub notify_users: Option<String>,
}

// 预警规则列表VO
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AlertRuleListVO {
    pub total: i64,
    pub items: Vec<AlertRuleListItem>,
}

// 预警规则列表项
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AlertRuleListItem {
    pub id: i64,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    pub min_quantity: Option<Decimal>,
    pub max_quantity: Option<Decimal>,
    pub stale_days: Option<i32>,
    pub enable_low_alert: Option<bool>,
    pub enable_high_alert: Option<bool>,
    pub enable_stale_alert: Option<bool>,
    pub notify_users: Option<String>,
    pub created_by: Option<i64>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

/// 分页查询预警规则
///
/// `product_ids` 与 `warehouse_ids` 为名称模糊匹配后解析得到的 ID 列表，
/// 当传入空 `Vec` 时表示名称匹配不到任何记录，应直接返回空结果；
/// 传入 `None` 时表示不按名称过滤。
pub async fn select_page<C: ConnectionTrait>(
    db: &C,
    page: u64,
    page_size: u64,
    product_id: Option<i64>,
    warehouse_id: Option<i64>,
    product_ids: Option<&[i64]>,
    warehouse_ids: Option<&[i64]>,
) -> Result<(Vec<alert_rule::Model>, u64), DbErr> {
    let mut query = alert_rule::Entity::find()
        .filter(alert_rule::Column::Deleted.eq(0));

    if let Some(pid) = product_id {
        query = query.filter(alert_rule::Column::ProductId.eq(pid));
    }
    if let Some(wid) = warehouse_id {
        query = query.filter(alert_rule::Column::WarehouseId.eq(wid));
    }
    // 名称匹配后的 ID 列表过滤：空切片表示匹配不到，结果集应为空
    if let Some(ids) = product_ids {
        if ids.is_empty() {
            query = query.filter(alert_rule::Column::Id.eq(0));
        } else {
            query = query.filter(alert_rule::Column::ProductId.is_in(ids.to_vec()));
        }
    }
    if let Some(ids) = warehouse_ids {
        if ids.is_empty() {
            query = query.filter(alert_rule::Column::Id.eq(0));
        } else {
            query = query.filter(alert_rule::Column::WarehouseId.is_in(ids.to_vec()));
        }
    }

    let total = query.clone().count(db).await?;
    let rows = query
        .order_by_desc(alert_rule::Column::UpdateTime)
        .offset((page - 1) * page_size)
        .limit(page_size)
        .all(db)
        .await?;

    Ok((rows, total))
}

/// 按ID查询预警规则
pub async fn find_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
) -> Result<Option<alert_rule::Model>, DbErr> {
    alert_rule::Entity::find_by_id(id)
        .filter(alert_rule::Column::Deleted.eq(0))
        .one(db)
        .await
}

/// 插入预警规则
pub async fn insert<C: ConnectionTrait>(
    db: &C,
    req: &AlertRuleSaveRequest,
    created_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let active = alert_rule::ActiveModel {
        product_id: Set(req.product_id),
        warehouse_id: Set(req.warehouse_id),
        min_quantity: Set(req.min_quantity),
        max_quantity: Set(req.max_quantity),
        stale_days: Set(req.stale_days),
        enable_low_alert: Set(req.enable_low_alert),
        enable_high_alert: Set(req.enable_high_alert),
        enable_stale_alert: Set(req.enable_stale_alert),
        notify_users: Set(req.notify_users.clone()),
        deleted: Set(Some(0)),
        created_by: Set(Some(created_by)),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let result = active.insert(db).await?;
    Ok(result.id)
}

/// 更新预警规则
pub async fn update<C: ConnectionTrait>(
    db: &C,
    id: i64,
    req: &AlertRuleSaveRequest,
    _updated_by: i64,
) -> Result<(), DbErr> {
    let now = chrono::Local::now().naive_local();
    alert_rule::Entity::update_many()
        .col_expr(alert_rule::Column::ProductId, Expr::value(req.product_id))
        .col_expr(alert_rule::Column::WarehouseId, Expr::value(req.warehouse_id))
        .col_expr(alert_rule::Column::MinQuantity, Expr::value(req.min_quantity))
        .col_expr(alert_rule::Column::MaxQuantity, Expr::value(req.max_quantity))
        .col_expr(alert_rule::Column::StaleDays, Expr::value(req.stale_days))
        .col_expr(alert_rule::Column::EnableLowAlert, Expr::value(req.enable_low_alert))
        .col_expr(alert_rule::Column::EnableHighAlert, Expr::value(req.enable_high_alert))
        .col_expr(alert_rule::Column::EnableStaleAlert, Expr::value(req.enable_stale_alert))
        .col_expr(alert_rule::Column::NotifyUsers, Expr::value(req.notify_users.clone()))
        // alert_rule entity does not have updated_by field, skip
        .col_expr(alert_rule::Column::UpdateTime, Expr::value(now))
        .filter(alert_rule::Column::Id.eq(id))
        .filter(alert_rule::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(())
}

/// 批量删除预警规则
pub async fn batch_delete<C: ConnectionTrait>(
    db: &C,
    ids: &[i64],
) -> Result<(), DbErr> {
    alert_rule::Entity::update_many()
        .col_expr(alert_rule::Column::Deleted, Expr::value(1))
        .filter(alert_rule::Column::Id.is_in(ids.iter().map(|&id| id).collect::<Vec<_>>()))
        .exec(db)
        .await?;
    Ok(())
}