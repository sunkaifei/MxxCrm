//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::prelude::{Date, DateTime, Decimal};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};

use crate::modules::inventory::entity::stock_snapshot;

/// 快照列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StockSnapshotListQuery {
    #[serde(rename = "page")]
    pub page_num: u64,
    pub page_size: u64,
    pub snapshot_date: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub warehouse_id: Option<i64>,
    pub product_id: Option<i64>,
}

/// 快照列表 VO
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StockSnapshotListVO {
    pub list: Vec<StockSnapshotListItem>,
    pub total: u64,
}

/// 快照列表项
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StockSnapshotListItem {
    pub id: i64,
    pub snapshot_date: Option<Date>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub quantity: Option<Decimal>,
    pub available_quantity: Option<Decimal>,
    pub frozen_quantity: Option<Decimal>,
    pub in_transit_quantity: Option<Decimal>,
    pub avg_cost: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub create_time: Option<DateTime>,
}

impl From<stock_snapshot::Model> for StockSnapshotListItem {
    fn from(m: stock_snapshot::Model) -> Self {
        Self {
            id: m.id,
            snapshot_date: m.snapshot_date,
            warehouse_id: m.warehouse_id,
            warehouse_name: None,
            product_id: m.product_id,
            product_name: m.product_name,
            product_sku: m.product_sku,
            quantity: m.quantity,
            available_quantity: m.available_quantity,
            frozen_quantity: m.frozen_quantity,
            in_transit_quantity: m.in_transit_quantity,
            avg_cost: m.avg_cost,
            total_cost: m.total_cost,
            create_time: m.create_time,
        }
    }
}

// ==================== DB 辅助方法 ====================

pub async fn select_page<C: ConnectionTrait>(
    db: &C,
    query: &StockSnapshotListQuery,
) -> Result<(Vec<stock_snapshot::Model>, u64), DbErr> {
    let mut q = stock_snapshot::Entity::find().filter(stock_snapshot::Column::Deleted.eq(0));

    if let Some(ref d) = query.snapshot_date {
        if let Ok(date) = d.parse::<Date>() {
            q = q.filter(stock_snapshot::Column::SnapshotDate.eq(date));
        }
    }
    if let Some(ref s) = query.start_date {
        if let Ok(date) = s.parse::<Date>() {
            q = q.filter(stock_snapshot::Column::SnapshotDate.gte(date));
        }
    }
    if let Some(ref e) = query.end_date {
        if let Ok(date) = e.parse::<Date>() {
            q = q.filter(stock_snapshot::Column::SnapshotDate.lte(date));
        }
    }
    if let Some(wid) = query.warehouse_id {
        q = q.filter(stock_snapshot::Column::WarehouseId.eq(wid));
    }
    if let Some(pid) = query.product_id {
        q = q.filter(stock_snapshot::Column::ProductId.eq(pid));
    }

    let total = q.clone().count(db).await?;
    let rows = q
        .order_by_desc(stock_snapshot::Column::SnapshotDate)
        .order_by_desc(stock_snapshot::Column::CreateTime)
        .offset((query.page_num - 1) * query.page_size)
        .limit(query.page_size)
        .all(db)
        .await?;

    Ok((rows, total))
}

/// 批量插入快照记录（事务内调用）
pub async fn batch_insert<C: ConnectionTrait>(
    db: &C,
    records: Vec<stock_snapshot::ActiveModel>,
) -> Result<i64, DbErr> {
    if records.is_empty() {
        return Ok(0);
    }
    let result = stock_snapshot::Entity::insert_many(records).exec(db).await?;
    Ok(result.last_insert_id.unwrap_or(0))
}

/// 删除指定日期的快照（用于重新生成）
pub async fn delete_by_date<C: ConnectionTrait>(
    db: &C,
    snapshot_date: Date,
) -> Result<i64, DbErr> {
    let result = stock_snapshot::Entity::update_many()
        .col_expr(stock_snapshot::Column::Deleted, Expr::value(1))
        .filter(stock_snapshot::Column::SnapshotDate.eq(snapshot_date))
        .filter(stock_snapshot::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}
