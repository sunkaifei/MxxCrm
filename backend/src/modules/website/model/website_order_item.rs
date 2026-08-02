//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::website::entity::{website_order_item, website_order_item::Entity as WebsiteOrderItem};
use crate::modules::website::model::website_order::OrderItemDTO;

/// 订单项 Model
pub struct WebsiteOrderItemModel;

impl WebsiteOrderItemModel {
    /// 批量插入订单项
    pub async fn batch_insert<C: ConnectionTrait>(
        db: &C,
        order_id: i64,
        items: &[OrderItemDTO],
    ) -> Result<i64, DbErr> {
        if items.is_empty() {
            return Ok(0);
        }
        let now = chrono::Local::now().naive_local().to_owned();
        let mut total_count = 0i64;
        for item in items {
            let total = item.price * Decimal::from(item.quantity);
            let payload = website_order_item::ActiveModel {
                order_id: Set(order_id),
                product_id: Set(item.product_id),
                sku_id: Set(item.sku_id),
                product_name: Set(item.product_name.clone()),
                product_image: Set(item.product_image.clone()),
                sku_code: Set(item.sku_code.clone()),
                sku_specs: Set(item.sku_specs.clone()),
                price: Set(item.price),
                quantity: Set(item.quantity),
                total_amount: Set(total),
                refund_status: Set(Some(0)),
                create_time: Set(Some(now.clone())),
                ..Default::default()
            };
            WebsiteOrderItem::insert(payload).exec(db).await?;
            total_count += 1;
        }
        Ok(total_count)
    }

    /// 按订单ID查询订单项
    pub async fn find_by_order_id<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<Vec<website_order_item::Model>, DbErr> {
        WebsiteOrderItem::find()
            .filter(website_order_item::Column::OrderId.eq(order_id))
            .all(db)
            .await
    }

    /// 按订单ID批量查询订单项
    pub async fn find_by_order_ids<C: ConnectionTrait>(db: &C, order_ids: Vec<i64>) -> Result<Vec<website_order_item::Model>, DbErr> {
        if order_ids.is_empty() {
            return Ok(vec![]);
        }
        WebsiteOrderItem::find()
            .filter(website_order_item::Column::OrderId.is_in(order_ids))
            .all(db)
            .await
    }

    /// 根据ID查询
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<website_order_item::Model>, DbErr> {
        WebsiteOrderItem::find_by_id(id).one(db).await
    }

    /// 更新退款状态
    pub async fn update_refund_status<C: ConnectionTrait>(
        db: &C,
        id: i64,
        status: i32,
    ) -> Result<i64, DbErr> {
        let result: UpdateResult = WebsiteOrderItem::update_many()
            .col_expr(website_order_item::Column::RefundStatus, sea_orm::sea_query::Expr::value(status))
            .filter(website_order_item::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}

// ==================== VO ====================

/// 订单项简略 VO（退款申请时使用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OrderItemSimpleVO {
    pub id: Option<i64>,
    pub order_id: Option<i64>,
    pub product_name: Option<String>,
    pub sku_code: Option<String>,
    pub price: Option<Decimal>,
    pub quantity: Option<i32>,
    pub total_amount: Option<Decimal>,
    pub refund_status: Option<i32>,
    pub create_time: Option<DateTime>,
}

impl From<website_order_item::Model> for OrderItemSimpleVO {
    fn from(item: website_order_item::Model) -> Self {
        OrderItemSimpleVO {
            id: Option::from(item.id),
            order_id: Some(item.order_id),
            product_name: item.product_name,
            sku_code: item.sku_code,
            price: Some(item.price),
            quantity: Some(item.quantity),
            total_amount: Some(item.total_amount),
            refund_status: item.refund_status,
            create_time: item.create_time,
        }
    }
}
