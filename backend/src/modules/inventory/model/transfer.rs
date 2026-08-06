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
use crate::modules::inventory::entity::transfer;

// 调拨单保存请求
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransferSaveRequest {
    pub from_warehouse_id: i64,
    pub to_warehouse_id: i64,
    pub remark: Option<String>,
    pub items: Vec<TransferItemRequest>,
}

// 调拨单明细请求
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransferItemRequest {
    pub product_id: i64,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub quantity: Decimal,
    pub remark: Option<String>,
}

// 调拨单列表查询参数
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransferListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub transfer_no: Option<String>,
    pub from_warehouse_id: Option<i64>,
    pub to_warehouse_id: Option<i64>,
    pub status: Option<i32>,
}

// 调拨单列表VO
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransferListVO {
    pub total: i64,
    pub items: Vec<TransferListItem>,
}

// 调拨单列表项
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransferListItem {
    pub id: i64,
    pub transfer_no: Option<String>,
    pub from_warehouse_id: Option<i64>,
    pub from_warehouse_name: Option<String>,
    pub to_warehouse_id: Option<i64>,
    pub to_warehouse_name: Option<String>,
    pub status: Option<i32>,
    pub total_quantity: Option<Decimal>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub created_by_name: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

impl From<transfer::Model> for TransferListItem {
    fn from(m: transfer::Model) -> Self {
        Self {
            id: m.id,
            transfer_no: m.transfer_no,
            from_warehouse_id: m.from_warehouse_id,
            from_warehouse_name: None,
            to_warehouse_id: m.to_warehouse_id,
            to_warehouse_name: None,
            status: m.status,
            total_quantity: m.total_quantity,
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
    query: &TransferListQuery,
) -> Result<(Vec<transfer::Model>, u64), DbErr> {
    let page_num = std::cmp::Ord::max(query.page_num.unwrap_or(1), 1);
    let page_size = std::cmp::Ord::max(query.page_size.unwrap_or(10), 1);

    let mut q = transfer::Entity::find()
        .filter(transfer::Column::Deleted.eq(0));

    if let Some(ref no) = query.transfer_no {
        if !no.is_empty() {
            q = q.filter(transfer::Column::TransferNo.contains(no));
        }
    }
    if let Some(wid) = query.from_warehouse_id {
        q = q.filter(transfer::Column::FromWarehouseId.eq(wid));
    }
    if let Some(wid) = query.to_warehouse_id {
        q = q.filter(transfer::Column::ToWarehouseId.eq(wid));
    }
    if let Some(s) = query.status {
        q = q.filter(transfer::Column::Status.eq(s));
    }

    let total = q.clone().count(db).await?;
    let rows = q
        .order_by_desc(transfer::Column::CreateTime)
        .offset(((page_num - 1) as u64) * page_size as u64)
        .limit(page_size as u64)
        .all(db)
        .await?;

    Ok((rows, total))
}

pub async fn find_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
) -> Result<Option<transfer::Model>, DbErr> {
    transfer::Entity::find_by_id(id)
        .filter(transfer::Column::Deleted.eq(0))
        .one(db)
        .await
}

pub async fn insert<C: ConnectionTrait>(
    db: &C,
    transfer_no: &str,
    req: &TransferSaveRequest,
    total_quantity: Decimal,
    created_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let active = transfer::ActiveModel {
        transfer_no: Set(Some(transfer_no.to_string())),
        from_warehouse_id: Set(Some(req.from_warehouse_id)),
        to_warehouse_id: Set(Some(req.to_warehouse_id)),
        status: Set(Some(0)), // 草稿
        total_quantity: Set(Some(total_quantity)),
        remark: Set(req.remark.clone()),
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
    req: &TransferSaveRequest,
    total_quantity: Decimal,
    updated_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = transfer::Entity::update_many()
        .col_expr(transfer::Column::FromWarehouseId, Expr::value(req.from_warehouse_id))
        .col_expr(transfer::Column::ToWarehouseId, Expr::value(req.to_warehouse_id))
        .col_expr(transfer::Column::TotalQuantity, Expr::value(total_quantity))
        .col_expr(transfer::Column::Remark, Expr::value(req.remark.clone()))
        .col_expr(transfer::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(transfer::Column::UpdateTime, Expr::value(now))
        .filter(transfer::Column::Id.eq(id))
        .filter(transfer::Column::Deleted.eq(0))
        .filter(transfer::Column::Status.eq(0)) // 仅草稿可编辑
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
    let result = transfer::Entity::update_many()
        .col_expr(transfer::Column::Status, Expr::value(status))
        .col_expr(transfer::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(transfer::Column::UpdateTime, Expr::value(now))
        .filter(transfer::Column::Id.eq(id))
        .filter(transfer::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

pub async fn batch_delete<C: ConnectionTrait>(
    db: &C,
    ids: &[i64],
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = transfer::Entity::update_many()
        .col_expr(transfer::Column::Deleted, Expr::value(1))
        .col_expr(transfer::Column::UpdateTime, Expr::value(now))
        .filter(transfer::Column::Id.is_in(ids.iter().map(|&id| id).collect::<Vec<_>>()))
        .filter(transfer::Column::Deleted.eq(0))
        .filter(transfer::Column::Status.eq(0)) // 仅草稿可删除
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}
