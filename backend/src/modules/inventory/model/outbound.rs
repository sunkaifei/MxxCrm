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
use crate::modules::inventory::entity::outbound;

// 出库单保存请求
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutboundSaveRequest {
    pub outbound_type: String,
    pub warehouse_id: i64,
    pub source_order_id: Option<i64>,
    pub source_order_no: Option<String>,
    pub total_quantity: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub remark: Option<String>,
    pub items: Vec<OutboundItemRequest>,
}

// 出库单明细请求
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutboundItemRequest {
    pub product_id: i64,
    pub product_sku: Option<String>,
    pub quantity: Decimal,
    pub batch_no: Option<String>,
    pub remark: Option<String>,
}

// 出库单列表查询参数
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutboundListQuery {
    pub page_num: u64,
    pub page_size: u64,
    pub outbound_no: Option<String>,
    pub outbound_type: Option<String>,
    pub warehouse_id: Option<i64>,
    pub status: Option<i32>,
}

// 出库单列表VO
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutboundListVO {
    pub list: Vec<OutboundListItem>,
    pub total: u64,
}

// 出库单列表项
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutboundListItem {
    pub id: i64,
    pub outbound_no: Option<String>,
    pub outbound_type: Option<String>,
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

impl From<outbound::Model> for OutboundListItem {
    fn from(m: outbound::Model) -> Self {
        Self {
            id: m.id,
            outbound_no: m.outbound_no,
            outbound_type: m.outbound_type,
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
    query: &OutboundListQuery,
) -> Result<(Vec<outbound::Model>, u64), DbErr> {
    let mut q = outbound::Entity::find()
        .filter(outbound::Column::Deleted.eq(0));

    if let Some(ref no) = query.outbound_no {
        q = q.filter(outbound::Column::OutboundNo.contains(no));
    }
    if let Some(ref t) = query.outbound_type {
        q = q.filter(outbound::Column::OutboundType.eq(t));
    }
    if let Some(wid) = query.warehouse_id {
        q = q.filter(outbound::Column::WarehouseId.eq(wid));
    }
    if let Some(s) = query.status {
        q = q.filter(outbound::Column::Status.eq(s));
    }

    let total = q.clone().count(db).await?;
    let rows = q
        .order_by_desc(outbound::Column::CreateTime)
        .offset((query.page_num - 1) * query.page_size)
        .limit(query.page_size)
        .all(db)
        .await?;

    Ok((rows, total))
}

pub async fn find_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
) -> Result<Option<outbound::Model>, DbErr> {
    outbound::Entity::find_by_id(id)
        .filter(outbound::Column::Deleted.eq(0))
        .one(db)
        .await
}

pub async fn insert<C: ConnectionTrait>(
    db: &C,
    req: &OutboundSaveRequest,
    outbound_no: &str,
    created_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let active = outbound::ActiveModel {
        outbound_no: Set(Some(outbound_no.to_string())),
        outbound_type: Set(Some(req.outbound_type.clone())),
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
    let result = outbound::Entity::update_many()
        .col_expr(outbound::Column::Status, Expr::value(status))
        .col_expr(outbound::Column::AuditBy, Expr::value(audit_by))
        .col_expr(outbound::Column::AuditTime, Expr::value(now))
        .col_expr(outbound::Column::UpdateTime, Expr::value(now))
        .filter(outbound::Column::Id.eq(id))
        .filter(outbound::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

pub async fn update_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
    req: &OutboundSaveRequest,
    updated_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = outbound::Entity::update_many()
        .col_expr(outbound::Column::OutboundType, Expr::value(req.outbound_type.clone()))
        .col_expr(outbound::Column::WarehouseId, Expr::value(req.warehouse_id))
        .col_expr(outbound::Column::SourceOrderId, Expr::value(req.source_order_id))
        .col_expr(outbound::Column::SourceOrderNo, Expr::value(req.source_order_no.clone()))
        .col_expr(outbound::Column::TotalQuantity, Expr::value(req.total_quantity))
        .col_expr(outbound::Column::TotalAmount, Expr::value(req.total_amount))
        .col_expr(outbound::Column::Remark, Expr::value(req.remark.clone()))
        .col_expr(outbound::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(outbound::Column::UpdateTime, Expr::value(now))
        .filter(outbound::Column::Id.eq(id))
        .filter(outbound::Column::Deleted.eq(0))
        .filter(outbound::Column::Status.eq(0)) // 仅草稿可编辑
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

pub async fn batch_delete<C: ConnectionTrait>(
    db: &C,
    ids: &[i64],
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = outbound::Entity::update_many()
        .col_expr(outbound::Column::Deleted, Expr::value(1))
        .col_expr(outbound::Column::UpdateTime, Expr::value(now))
        .filter(outbound::Column::Id.is_in(ids.iter().map(|&id| id).collect::<Vec<_>>()))
        .filter(outbound::Column::Deleted.eq(0))
        .filter(outbound::Column::Status.eq(0)) // 仅草稿可删除
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}