//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};

use crate::modules::inventory::entity::batch;

/// 批次列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchListQuery {
    #[serde(rename = "page")]
    pub page_num: u64,
    pub page_size: u64,
    pub batch_no: Option<String>,
    pub product_id: Option<i64>,
    pub warehouse_id: Option<i64>,
    pub status: Option<i32>,
}

/// 批次列表 VO
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchListVO {
    pub list: Vec<BatchListItem>,
    pub total: u64,
}

/// 批次列表项
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchListItem {
    pub id: i64,
    pub batch_no: Option<String>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    pub production_date: Option<chrono::NaiveDateTime>,
    pub expiry_date: Option<chrono::NaiveDateTime>,
    pub initial_quantity: Option<Decimal>,
    pub current_quantity: Option<Decimal>,
    pub status: Option<i32>,
    pub supplier_id: Option<i64>,
    pub inbound_id: Option<i64>,
    pub remark: Option<String>,
    pub create_time: Option<chrono::NaiveDateTime>,
}

impl From<batch::Model> for BatchListItem {
    fn from(m: batch::Model) -> Self {
        Self {
            id: m.id,
            batch_no: m.batch_no,
            product_id: m.product_id,
            product_name: m.product_name,
            product_sku: m.product_sku,
            warehouse_id: m.warehouse_id,
            warehouse_name: None,
            production_date: m.production_date,
            expiry_date: m.expiry_date,
            initial_quantity: m.initial_quantity,
            current_quantity: m.current_quantity,
            status: m.status,
            supplier_id: m.supplier_id,
            inbound_id: m.inbound_id,
            remark: m.remark,
            create_time: m.create_time,
        }
    }
}

use sea_orm::prelude::Decimal;

/// 创建批次请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateRequest {
    pub batch_no: String,
    pub product_id: i64,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub warehouse_id: i64,
    pub production_date: Option<chrono::NaiveDateTime>,
    pub expiry_date: Option<chrono::NaiveDateTime>,
    pub initial_quantity: Decimal,
    pub supplier_id: Option<i64>,
    pub inbound_id: Option<i64>,
    pub remark: Option<String>,
}

// ==================== DB 辅助方法 ====================

/// 分页查询批次列表
pub async fn select_page<C: ConnectionTrait>(
    db: &C,
    query: &BatchListQuery,
) -> Result<(Vec<batch::Model>, u64), DbErr> {
    let mut q = batch::Entity::find().filter(batch::Column::Deleted.eq(0));

    if let Some(ref no) = query.batch_no {
        q = q.filter(batch::Column::BatchNo.contains(no));
    }
    if let Some(pid) = query.product_id {
        q = q.filter(batch::Column::ProductId.eq(pid));
    }
    if let Some(wid) = query.warehouse_id {
        q = q.filter(batch::Column::WarehouseId.eq(wid));
    }
    if let Some(s) = query.status {
        q = q.filter(batch::Column::Status.eq(s));
    }

    let total = q.clone().count(db).await?;
    let rows = q
        .order_by_desc(batch::Column::CreateTime)
        .offset((query.page_num - 1) * query.page_size)
        .limit(query.page_size)
        .all(db)
        .await?;

    Ok((rows, total))
}

/// 根据 ID 查询批次
pub async fn find_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
) -> Result<Option<batch::Model>, DbErr> {
    batch::Entity::find_by_id(id)
        .filter(batch::Column::Deleted.eq(0))
        .one(db)
        .await
}

/// 根据产品ID查询有效批次（状态=0 正常）
pub async fn find_active_by_product<C: ConnectionTrait>(
    db: &C,
    product_id: i64,
) -> Result<Vec<batch::Model>, DbErr> {
    batch::Entity::find()
        .filter(batch::Column::ProductId.eq(product_id))
        .filter(batch::Column::Status.eq(0))
        .filter(batch::Column::Deleted.eq(0))
        .filter(batch::Column::CurrentQuantity.gt(Decimal::ZERO))
        .order_by_desc(batch::Column::CreateTime)
        .all(db)
        .await
}

/// 插入批次记录
pub async fn insert<C: ConnectionTrait>(
    db: &C,
    req: &BatchCreateRequest,
    created_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let active = batch::ActiveModel {
        batch_no: Set(Some(req.batch_no.clone())),
        product_id: Set(Some(req.product_id)),
        product_name: Set(req.product_name.clone()),
        product_sku: Set(req.product_sku.clone()),
        warehouse_id: Set(Some(req.warehouse_id)),
        production_date: Set(req.production_date),
        expiry_date: Set(req.expiry_date),
        initial_quantity: Set(Some(req.initial_quantity)),
        current_quantity: Set(Some(req.initial_quantity)),
        status: Set(Some(0)),
        supplier_id: Set(req.supplier_id),
        inbound_id: Set(req.inbound_id),
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

/// 扣减批次当前数量（出库审核时调用）
pub async fn decrease_quantity<C: ConnectionTrait>(
    db: &C,
    batch_id: i64,
    quantity: Decimal,
) -> Result<(), DbErr> {
    let now = chrono::Local::now().naive_local();
    let existing = batch::Entity::find_by_id(batch_id)
        .filter(batch::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| DbErr::Custom("批次不存在".into()))?;

    let current = existing.current_quantity.unwrap_or_default();
    if current < quantity {
        return Err(DbErr::Custom(format!(
            "批次当前数量不足：当前 {}，需要 {}",
            current, quantity
        )));
    }

    let new_qty = current - quantity;
    let new_status = if new_qty <= Decimal::ZERO { 1 } else { 0 };

    batch::Entity::update_many()
        .col_expr(batch::Column::CurrentQuantity, Expr::value(new_qty))
        .col_expr(batch::Column::Status, Expr::value(new_status))
        .col_expr(batch::Column::UpdateTime, Expr::value(now))
        .filter(batch::Column::Id.eq(batch_id))
        .filter(batch::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(())
}

/// 根据批次号查询是否存在
pub async fn find_by_batch_no<C: ConnectionTrait>(
    db: &C,
    batch_no: &str,
    product_id: i64,
    warehouse_id: i64,
) -> Result<Option<batch::Model>, DbErr> {
    batch::Entity::find()
        .filter(batch::Column::BatchNo.eq(batch_no))
        .filter(batch::Column::ProductId.eq(product_id))
        .filter(batch::Column::WarehouseId.eq(warehouse_id))
        .filter(batch::Column::Deleted.eq(0))
        .one(db)
        .await
}
