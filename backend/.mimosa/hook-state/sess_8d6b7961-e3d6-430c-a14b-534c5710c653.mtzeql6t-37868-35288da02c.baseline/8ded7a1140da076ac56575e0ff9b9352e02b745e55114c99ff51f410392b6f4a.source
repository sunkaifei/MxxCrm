//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 换货业务逻辑层
//!

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::entity::exchange::{self, Entity, Column};
use crate::modules::sale::entity::exchange_item;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbConn, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeItemInput {
    pub original_order_item_id: Option<i64>,
    pub original_product_id: Option<i64>,
    pub original_product_name: Option<String>,
    pub original_qty: Option<Decimal>,
    pub new_product_id: Option<i64>,
    pub new_product_name: Option<String>,
    pub new_qty: Option<Decimal>,
    pub new_unit_price: Option<Decimal>,
    pub price_diff: Option<Decimal>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeListQuery {
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub refund_id: Option<i64>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub status: Option<i32>,
    pub keywords: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExchangeDetailVO {
    #[serde(flatten)]
    pub exchange: exchange::Model,
    pub items: Vec<exchange_item::Model>,
}

/// 创建换货单 EX+yyyyMMdd+4位，写入 exchange + exchange_item
pub async fn create_exchange(
    db: &DbConn,
    refund_id: Option<i64>,
    order_id: i64,
    items: Vec<ExchangeItemInput>,
    user_id: i64,
) -> Result<i64> {
    if items.is_empty() {
        return Err(Error::from("换货明细不能为空"));
    }

    let date_prefix = format!("EX{}", chrono::Local::now().format("%Y%m%d"));
    let today_records = Entity::find()
        .filter(Column::ExchangeNo.starts_with(&date_prefix))
        .filter(Column::Deleted.eq(0))
        .all(db)
        .await?;
    let max_seq = today_records
        .iter()
        .filter_map(|e| e.exchange_no.as_ref())
        .filter_map(|no| no.get(date_prefix.len()..).and_then(|s| s.parse::<u32>().ok()))
        .max()
        .unwrap_or(0);
    let exchange_no = format!("{}{:04}", date_prefix, max_seq + 1);

    let now = chrono::Local::now().naive_local();

    let txn = db.begin().await?;

    let model = exchange::ActiveModel {
        exchange_no: Set(Some(exchange_no)),
        refund_id: Set(refund_id),
        order_id: Set(Some(order_id)),
        status: Set(Some(1)),
        approval_status: Set(Some(0)),
        owner_user_id: Set(Some(user_id)),
        create_by: Set(Some(user_id)),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    let result = model.insert(&txn).await?;
    let exchange_id = result.id;

    for item in &items {
        let item_model = exchange_item::ActiveModel {
            exchange_id: Set(Some(exchange_id)),
            original_order_item_id: Set(item.original_order_item_id),
            original_product_id: Set(item.original_product_id),
            original_product_name: Set(item.original_product_name.clone()),
            original_qty: Set(item.original_qty),
            new_product_id: Set(item.new_product_id),
            new_product_name: Set(item.new_product_name.clone()),
            new_qty: Set(item.new_qty),
            new_unit_price: Set(item.new_unit_price),
            price_diff: Set(item.price_diff),
            remark: Set(item.remark.clone()),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        item_model.insert(&txn).await?;
    }

    txn.commit().await?;

    Ok(exchange_id)
}

/// 提交审批
pub async fn submit_exchange(db: &DbConn, id: i64, operator_id: i64, operator_name: &str) -> Result<i64> {
    let existing = Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("换货单不存在"))?;

    let approval_status = existing.approval_status.unwrap_or(0);
    if approval_status != 0 && approval_status != 4 {
        return Err(Error::from("仅草稿或已驳回状态可提交审批"));
    }

    let now = chrono::Local::now().naive_local();
    let txn = db.begin().await?;
    let mut active: exchange::ActiveModel = existing.into();
    active.approval_status = Set(Some(1));
    active.status = Set(Some(2));
    active.update_time = Set(Some(now));
    active.update(&txn).await?;
    txn.commit().await?;

    let _ = (operator_id, operator_name);
    Ok(id)
}

/// 审批通过后自动生成换出发货单（占位实现：更新状态，记录换出发货单号占位）
pub async fn approve_exchange(db: &DbConn, id: i64) -> Result<i64> {
    let existing = Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("换货单不存在"))?;

    let approval_status = existing.approval_status.unwrap_or(0);
    if approval_status != 1 && approval_status != 2 {
        return Err(Error::from("仅待审批或审批中状态可进行审批操作"));
    }

    let now = chrono::Local::now().naive_local();
    let txn = db.begin().await?;

    // 审批通过：更新审批状态和业务状态
    let mut active: exchange::ActiveModel = existing.into();
    active.approval_status = Set(Some(3));
    active.status = Set(Some(3));
    active.update_time = Set(Some(now));
    active.update(&txn).await?;

    txn.commit().await?;

    // 审批通过后自动生成换出发货单（best-effort）
    if let Err(e) = auto_create_outbound_shipment(db, id).await {
        log::warn!("[approve_exchange] 换出发货单自动创建失败 exchange_id={}: {}", id, e);
    }

    Ok(id)
}

/// 审批通过后自动生成换出发货单（占位：标记换发出货单标识）
async fn auto_create_outbound_shipment(db: &DbConn, exchange_id: i64) -> Result<()> {
    let now = chrono::Local::now().naive_local();
    let txn = db.begin().await?;
    exchange::Entity::update_many()
        .col_expr(exchange::Column::OutboundShipmentId, sea_orm::sea_query::Expr::value(exchange_id))
        .col_expr(exchange::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
        .filter(Column::Id.eq(exchange_id))
        .filter(Column::Deleted.eq(0))
        .exec(&txn)
        .await
        .map_err(|e| Error::from(format!("更新换发出货单失败: {}", e)))?;
    txn.commit().await?;
    Ok(())
}

/// 详情
pub async fn get_info(db: &DbConn, id: i64) -> Result<ExchangeDetailVO> {
    let exchange = Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("换货单不存在"))?;

    let items = exchange_item::Entity::find()
        .filter(exchange_item::Column::ExchangeId.eq(id))
        .all(db)
        .await?;

    Ok(ExchangeDetailVO {
        exchange,
        items,
    })
}

/// 分页列表
pub async fn get_list(db: &DbConn, query: &ExchangeListQuery) -> Result<ResultPage<Vec<exchange::Model>>> {
    let page = query.page_num.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1);

    let mut cond = Condition::all().add(Column::Deleted.eq(0));
    if let Some(refund_id) = query.refund_id {
        if refund_id > 0 {
            cond = cond.add(Column::RefundId.eq(refund_id));
        }
    }
    if let Some(order_id) = query.order_id {
        if order_id > 0 {
            cond = cond.add(Column::OrderId.eq(order_id));
        }
    }
    if let Some(customer_id) = query.customer_id {
        if customer_id > 0 {
            cond = cond.add(Column::CustomerId.eq(customer_id));
        }
    }
    if let Some(status) = query.status {
        if status > 0 {
            cond = cond.add(Column::Status.eq(status));
        }
    }
    if let Some(keywords) = &query.keywords {
        if !keywords.is_empty() {
            cond = cond.add(
                Condition::any()
                    .add(Column::ExchangeNo.contains(keywords))
                    .add(Column::Title.contains(keywords)),
            );
        }
    }

    let total = Entity::find()
        .filter(cond.clone())
        .count(db)
        .await? as i64;

    let items = Entity::find()
        .filter(cond)
        .order_by_desc(Column::Id)
        .offset(((page - 1) * page_size) as u64)
        .limit(page_size as u64)
        .all(db)
        .await?;

    Ok(ResultPage::new(items, total, page, page_size))
}
