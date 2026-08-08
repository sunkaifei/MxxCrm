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
use crate::modules::inventory::entity::stocktake;

// 盘点单保存请求
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StocktakeSaveRequest {
    pub warehouse_id: i64,
    /// 盘点类型：full/partial
    pub stocktake_type: Option<String>,
    pub remark: Option<String>,
    /// 盘点明细（创建时可只传 product_id，system_quantity 由后端自动填充）
    pub items: Vec<StocktakeItemRequest>,
}

// 盘点单明细请求
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StocktakeItemRequest {
    pub id: Option<i64>,
    pub product_id: i64,
    /// SKU ID（多规格产品按SKU盘点时使用）
    #[serde(default)]
    pub sku_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub system_quantity: Option<Decimal>,
    pub actual_quantity: Option<Decimal>,
    #[serde(default)]
    pub assignee_ids: Option<serde_json::Value>,
    #[serde(default)]
    pub diff_reason: Option<String>,
    #[serde(default)]
    pub handling: Option<String>,
    pub remark: Option<String>,
}

// 盘点录入请求（录入实盘数量）
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StocktakeInputRequest {
    pub items: Vec<StocktakeInputItem>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StocktakeInputItem {
    pub id: i64,
    pub actual_quantity: Decimal,
    pub remark: Option<String>,
    /// 盘点人ID列表（前端传数组，序列化为 JSON 数组字符串存库）
    #[serde(default)]
    pub assignee_ids: Option<serde_json::Value>,
    /// 复盘数量
    #[serde(default)]
    pub recheck_quantity: Option<Decimal>,
    /// 复盘人ID列表
    #[serde(default)]
    pub recheck_assignee_ids: Option<serde_json::Value>,
    /// 差异原因
    #[serde(default)]
    pub diff_reason: Option<String>,
    /// 处理方式
    #[serde(default)]
    pub handling: Option<String>,
}

// 盘点单列表查询参数
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StocktakeListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub stocktake_no: Option<String>,
    pub warehouse_id: Option<i64>,
    pub status: Option<i32>,
    pub stocktake_type: Option<String>,
}

// 盘点单列表VO
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StocktakeListVO {
    pub total: i64,
    pub items: Vec<StocktakeListItem>,
}

// 盘点单列表项
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StocktakeListItem {
    pub id: i64,
    pub stocktake_no: Option<String>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    pub stocktake_type: Option<String>,
    pub status: Option<i32>,
    pub total_items: Option<i32>,
    pub surplus_count: Option<i32>,
    pub shortage_count: Option<i32>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub created_by_name: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

impl From<stocktake::Model> for StocktakeListItem {
    fn from(m: stocktake::Model) -> Self {
        Self {
            id: m.id,
            stocktake_no: m.stocktake_no,
            warehouse_id: m.warehouse_id,
            warehouse_name: None,
            stocktake_type: m.stocktake_type,
            status: m.status,
            total_items: m.total_items,
            surplus_count: m.surplus_count,
            shortage_count: m.shortage_count,
            remark: m.remark,
            created_by: m.created_by,
            created_by_name: None,
            create_time: m.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: m.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

// DB helper functions
pub async fn select_page<C: ConnectionTrait>(
    db: &C,
    query: &StocktakeListQuery,
) -> Result<(Vec<stocktake::Model>, u64), DbErr> {
    let page_num = std::cmp::Ord::max(query.page_num.unwrap_or(1), 1);
    let page_size = std::cmp::Ord::max(query.page_size.unwrap_or(10), 1);

    let mut q = stocktake::Entity::find()
        .filter(stocktake::Column::Deleted.eq(0));

    if let Some(ref no) = query.stocktake_no {
        if !no.is_empty() {
            q = q.filter(stocktake::Column::StocktakeNo.contains(no));
        }
    }
    if let Some(wid) = query.warehouse_id {
        q = q.filter(stocktake::Column::WarehouseId.eq(wid));
    }
    if let Some(s) = query.status {
        q = q.filter(stocktake::Column::Status.eq(s));
    }
    if let Some(ref t) = query.stocktake_type {
        if !t.is_empty() {
            q = q.filter(stocktake::Column::StocktakeType.eq(t));
        }
    }

    let total = q.clone().count(db).await?;
    let rows = q
        .order_by_desc(stocktake::Column::CreateTime)
        .offset(((page_num - 1) as u64) * page_size as u64)
        .limit(page_size as u64)
        .all(db)
        .await?;

    Ok((rows, total))
}

pub async fn find_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
) -> Result<Option<stocktake::Model>, DbErr> {
    stocktake::Entity::find_by_id(id)
        .filter(stocktake::Column::Deleted.eq(0))
        .one(db)
        .await
}

pub async fn insert<C: ConnectionTrait>(
    db: &C,
    stocktake_no: &str,
    warehouse_id: i64,
    stocktake_type: &str,
    remark: Option<&str>,
    total_items: i32,
    created_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let active = stocktake::ActiveModel {
        stocktake_no: Set(Some(stocktake_no.to_string())),
        warehouse_id: Set(Some(warehouse_id)),
        stocktake_type: Set(Some(stocktake_type.to_string())),
        status: Set(Some(0)), // 草稿
        total_items: Set(Some(total_items)),
        surplus_count: Set(Some(0)),
        shortage_count: Set(Some(0)),
        remark: Set(remark.map(|s| s.to_string())),
        deleted: Set(Some(0)),
        created_by: Set(Some(created_by)),
        updated_by: Set(Some(created_by)),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let result = active.insert(db).await?;
    Ok(result.id)
}

pub async fn update_main<C: ConnectionTrait>(
    db: &C,
    id: i64,
    stocktake_type: &str,
    remark: Option<&str>,
    total_items: i32,
    updated_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = stocktake::Entity::update_many()
        .col_expr(stocktake::Column::StocktakeType, Expr::value(stocktake_type))
        .col_expr(stocktake::Column::Remark, Expr::value(remark.map(|s| s.to_string())))
        .col_expr(stocktake::Column::TotalItems, Expr::value(total_items))
        .col_expr(stocktake::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(stocktake::Column::UpdateTime, Expr::value(now))
        .filter(stocktake::Column::Id.eq(id))
        .filter(stocktake::Column::Deleted.eq(0))
        .filter(stocktake::Column::Status.eq(0)) // 仅草稿可编辑
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

pub async fn update_status<C: ConnectionTrait>(
    db: &C,
    id: i64,
    status: i32,
    updated_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = stocktake::Entity::update_many()
        .col_expr(stocktake::Column::Status, Expr::value(status))
        .col_expr(stocktake::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(stocktake::Column::UpdateTime, Expr::value(now))
        .filter(stocktake::Column::Id.eq(id))
        .filter(stocktake::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

pub async fn update_summary<C: ConnectionTrait>(
    db: &C,
    id: i64,
    surplus_count: i32,
    shortage_count: i32,
    updated_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = stocktake::Entity::update_many()
        .col_expr(stocktake::Column::SurplusCount, Expr::value(surplus_count))
        .col_expr(stocktake::Column::ShortageCount, Expr::value(shortage_count))
        .col_expr(stocktake::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(stocktake::Column::UpdateTime, Expr::value(now))
        .filter(stocktake::Column::Id.eq(id))
        .filter(stocktake::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

pub async fn batch_delete<C: ConnectionTrait>(
    db: &C,
    ids: &[i64],
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = stocktake::Entity::update_many()
        .col_expr(stocktake::Column::Deleted, Expr::value(1))
        .col_expr(stocktake::Column::UpdateTime, Expr::value(now))
        .filter(stocktake::Column::Id.is_in(ids.iter().map(|&id| id).collect::<Vec<_>>()))
        .filter(stocktake::Column::Deleted.eq(0))
        .filter(stocktake::Column::Status.eq(0)) // 仅草稿可删除
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}
