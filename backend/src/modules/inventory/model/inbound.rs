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
use crate::modules::inventory::entity::inbound;

// 入库单保存请求
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InboundSaveRequest {
    pub inbound_type: String,
    pub warehouse_id: i64,
    pub source_order_id: Option<i64>,
    pub source_order_no: Option<String>,
    pub total_quantity: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub remark: Option<String>,
    pub items: Vec<InboundItemRequest>,
}

// 入库单明细请求
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InboundItemRequest {
    pub product_id: i64,
    pub product_sku: Option<String>,
    pub quantity: Decimal,
    pub unit_price: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub batch_no: Option<String>,
    pub remark: Option<String>,
}

// 入库单列表查询参数
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InboundListQuery {
    pub page_num: u64,
    pub page_size: u64,
    pub inbound_no: Option<String>,
    pub inbound_type: Option<String>,
    pub warehouse_id: Option<i64>,
    pub status: Option<i32>,
}

// 入库单列表VO
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InboundListVO {
    pub list: Vec<InboundListItem>,
    pub total: u64,
}

// 入库单列表项
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InboundListItem {
    pub id: i64,
    pub inbound_no: Option<String>,
    pub inbound_type: Option<String>,
    pub source_order_no: Option<String>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    pub status: Option<i32>,
    pub total_quantity: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub created_by_name: Option<String>,
    pub create_time: Option<chrono::NaiveDateTime>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

impl From<inbound::Model> for InboundListItem {
    fn from(m: inbound::Model) -> Self {
        Self {
            id: m.id,
            inbound_no: m.inbound_no,
            inbound_type: m.inbound_type,
            source_order_no: m.source_order_no,
            warehouse_id: m.warehouse_id,
            warehouse_name: None,
            status: m.status,
            total_quantity: m.total_quantity,
            total_amount: m.total_amount,
            remark: m.remark,
            created_by: m.created_by,
            created_by_name: None,
            create_time: m.create_time,
            update_time: m.update_time,
        }
    }
}

// DB helper functions
pub async fn select_page<C: ConnectionTrait>(
    db: &C,
    query: &InboundListQuery,
) -> Result<(Vec<inbound::Model>, u64), DbErr> {
    let mut q = inbound::Entity::find()
        .filter(inbound::Column::Deleted.eq(0));

    if let Some(ref no) = query.inbound_no {
        q = q.filter(inbound::Column::InboundNo.contains(no));
    }
    if let Some(ref t) = query.inbound_type {
        q = q.filter(inbound::Column::InboundType.eq(t));
    }
    if let Some(wid) = query.warehouse_id {
        q = q.filter(inbound::Column::WarehouseId.eq(wid));
    }
    if let Some(s) = query.status {
        q = q.filter(inbound::Column::Status.eq(s));
    }

    let total = q.clone().count(db).await?;
    let rows = q
        .order_by_desc(inbound::Column::CreateTime)
        .offset((query.page_num - 1) * query.page_size)
        .limit(query.page_size)
        .all(db)
        .await?;

    Ok((rows, total))
}

pub async fn find_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
) -> Result<Option<inbound::Model>, DbErr> {
    inbound::Entity::find_by_id(id)
        .filter(inbound::Column::Deleted.eq(0))
        .one(db)
        .await
}

pub async fn insert<C: ConnectionTrait>(
    db: &C,
    req: &InboundSaveRequest,
    inbound_no: &str,
    created_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let active = inbound::ActiveModel {
        inbound_no: Set(Some(inbound_no.to_string())),
        inbound_type: Set(Some(req.inbound_type.clone())),
        source_order_id: Set(req.source_order_id),
        source_order_no: Set(req.source_order_no.clone()),
        warehouse_id: Set(Some(req.warehouse_id)),
        status: Set(Some(0)), // 草稿
        total_quantity: Set(req.total_quantity),
        total_amount: Set(req.total_amount),
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

pub async fn update_status<C: ConnectionTrait>(
    db: &C,
    id: i64,
    status: i32,
    audit_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = inbound::Entity::update_many()
        .col_expr(inbound::Column::Status, Expr::value(status))
        .col_expr(inbound::Column::AuditBy, Expr::value(audit_by))
        .col_expr(inbound::Column::AuditTime, Expr::value(now))
        .col_expr(inbound::Column::UpdateTime, Expr::value(now))
        .filter(inbound::Column::Id.eq(id))
        .filter(inbound::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

pub async fn update_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
    req: &InboundSaveRequest,
    updated_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = inbound::Entity::update_many()
        .col_expr(inbound::Column::InboundType, Expr::value(req.inbound_type.clone()))
        .col_expr(inbound::Column::WarehouseId, Expr::value(req.warehouse_id))
        .col_expr(inbound::Column::SourceOrderId, Expr::value(req.source_order_id))
        .col_expr(inbound::Column::SourceOrderNo, Expr::value(req.source_order_no.clone()))
        .col_expr(inbound::Column::TotalQuantity, Expr::value(req.total_quantity))
        .col_expr(inbound::Column::TotalAmount, Expr::value(req.total_amount))
        .col_expr(inbound::Column::Remark, Expr::value(req.remark.clone()))
        .col_expr(inbound::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(inbound::Column::UpdateTime, Expr::value(now))
        .filter(inbound::Column::Id.eq(id))
        .filter(inbound::Column::Deleted.eq(0))
        .filter(inbound::Column::Status.eq(0)) // 仅草稿可编辑
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

pub async fn batch_delete<C: ConnectionTrait>(
    db: &C,
    ids: &[i64],
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = inbound::Entity::update_many()
        .col_expr(inbound::Column::Deleted, Expr::value(1))
        .col_expr(inbound::Column::UpdateTime, Expr::value(now))
        .filter(inbound::Column::Id.is_in(ids.iter().map(|&id| id).collect::<Vec<_>>()))
        .filter(inbound::Column::Deleted.eq(0))
        .filter(inbound::Column::Status.eq(0)) // 仅草稿可删除
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}