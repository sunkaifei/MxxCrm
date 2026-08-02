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
use crate::modules::website::entity::{website_order, website_order::Entity as WebsiteOrder};

// ==================== 常量 ====================

/// 订单状态：0待付款 1待发货 2待收货 3已完成 4已取消 5已关闭
pub const STATUS_PENDING_PAY: i32 = 0;
pub const STATUS_PENDING_SHIP: i32 = 1;
pub const STATUS_PENDING_RECEIVE: i32 = 2;
pub const STATUS_COMPLETED: i32 = 3;
pub const STATUS_CANCELLED: i32 = 4;
pub const STATUS_CLOSED: i32 = 5;

/// 支付状态：0未支付 1已支付 2已退款 3部分退款
pub const PAY_STATUS_UNPAID: i32 = 0;
pub const PAY_STATUS_PAID: i32 = 1;
pub const PAY_STATUS_REFUNDED: i32 = 2;
pub const PAY_STATUS_PARTIAL_REFUND: i32 = 3;

/// 发货状态：0未发货 1部分发货 2已发货 3已签收
pub const SHIP_STATUS_UNSHIPPED: i32 = 0;
pub const SHIP_STATUS_PARTIAL: i32 = 1;
pub const SHIP_STATUS_SHIPPED: i32 = 2;
pub const SHIP_STATUS_RECEIVED: i32 = 3;

// ==================== DTO ====================

/// 创建订单请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderCreateRequest {
    /// 购物车ID列表（从购物车下单时使用）
    pub cart_ids: Option<Vec<i64>>,
    /// 直接下单的商品项（立即购买时使用）
    pub items: Option<Vec<OrderItemDTO>>,
    /// 收货信息
    pub consignee_name: String,
    pub consignee_phone: String,
    pub consignee_address: String,
    pub consignee_province: Option<String>,
    pub consignee_city: Option<String>,
    pub consignee_district: Option<String>,
    pub consignee_zipcode: Option<String>,
    /// 买家备注
    pub buyer_remark: Option<String>,
    /// 站点ID
    pub website_id: Option<i64>,
    /// 支付方式：1微信 2支付宝 3余额
    pub pay_type: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderItemDTO {
    pub product_id: i64,
    pub sku_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_image: Option<String>,
    pub sku_code: Option<String>,
    pub sku_specs: Option<String>,
    pub price: Decimal,
    pub quantity: i32,
}

/// 后台订单列表查询
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub order_no: Option<String>,
    pub status: Option<i32>,
    pub pay_status: Option<i32>,
    pub ship_status: Option<i32>,
    pub user_id: Option<i64>,
}

/// 后台更新订单（卖家备注、状态等）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderUpdateRequest {
    pub seller_remark: Option<String>,
    pub status: Option<i32>,
}

/// 后台发货请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ShipRequest {
    pub delivery_no: String,
    pub delivery_company: String,
    pub delivery_type: Option<i32>,
    pub remark: Option<String>,
}

// ==================== VO ====================

/// 订单项 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OrderItemVO {
    pub id: Option<i64>,
    pub order_id: Option<i64>,
    pub product_id: Option<i64>,
    pub sku_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_image: Option<String>,
    pub sku_code: Option<String>,
    pub sku_specs: Option<String>,
    pub price: Option<Decimal>,
    pub quantity: Option<i32>,
    pub total_amount: Option<Decimal>,
    pub refund_status: Option<i32>,
    pub create_time: Option<DateTime>,
}

impl From<crate::modules::website::entity::website_order_item::Model> for OrderItemVO {
    fn from(item: crate::modules::website::entity::website_order_item::Model) -> Self {
        OrderItemVO {
            id: Option::from(item.id),
            order_id: Some(item.order_id),
            product_id: Some(item.product_id),
            sku_id: item.sku_id,
            product_name: item.product_name,
            product_image: item.product_image,
            sku_code: item.sku_code,
            sku_specs: item.sku_specs,
            price: Some(item.price),
            quantity: Some(item.quantity),
            total_amount: Some(item.total_amount),
            refund_status: item.refund_status,
            create_time: item.create_time,
        }
    }
}

/// 订单 VO（含订单项）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OrderVO {
    pub id: Option<i64>,
    pub order_no: Option<String>,
    pub user_id: Option<i64>,
    pub website_id: Option<i64>,
    pub total_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub shipping_fee: Option<Decimal>,
    pub pay_amount: Option<Decimal>,
    pub status: Option<i32>,
    pub pay_status: Option<i32>,
    pub ship_status: Option<i32>,
    pub pay_type: Option<i32>,
    pub pay_time: Option<DateTime>,
    pub ship_time: Option<DateTime>,
    pub finish_time: Option<DateTime>,
    pub cancel_time: Option<DateTime>,
    pub cancel_reason: Option<String>,
    pub consignee_name: Option<String>,
    pub consignee_phone: Option<String>,
    pub consignee_address: Option<String>,
    pub consignee_province: Option<String>,
    pub consignee_city: Option<String>,
    pub consignee_district: Option<String>,
    pub consignee_zipcode: Option<String>,
    pub buyer_remark: Option<String>,
    pub seller_remark: Option<String>,
    pub transaction_id: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    /// 订单项列表
    pub items: Vec<OrderItemVO>,
}

impl From<website_order::Model> for OrderVO {
    fn from(item: website_order::Model) -> Self {
        OrderVO {
            id: Option::from(item.id),
            order_no: Some(item.order_no),
            user_id: Some(item.user_id),
            website_id: item.website_id,
            total_amount: Some(item.total_amount),
            discount_amount: item.discount_amount,
            shipping_fee: item.shipping_fee,
            pay_amount: Some(item.pay_amount),
            status: item.status,
            pay_status: item.pay_status,
            ship_status: item.ship_status,
            pay_type: item.pay_type,
            pay_time: item.pay_time,
            ship_time: item.ship_time,
            finish_time: item.finish_time,
            cancel_time: item.cancel_time,
            cancel_reason: item.cancel_reason,
            consignee_name: item.consignee_name,
            consignee_phone: item.consignee_phone,
            consignee_address: item.consignee_address,
            consignee_province: item.consignee_province,
            consignee_city: item.consignee_city,
            consignee_district: item.consignee_district,
            consignee_zipcode: item.consignee_zipcode,
            buyer_remark: item.buyer_remark,
            seller_remark: item.seller_remark,
            transaction_id: item.transaction_id,
            create_time: item.create_time,
            update_time: item.update_time,
            items: vec![],
        }
    }
}

// ==================== Model ====================

pub struct WebsiteOrderModel;

impl WebsiteOrderModel {
    /// 新增订单
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        order_no: String,
        user_id: i64,
        req: &OrderCreateRequest,
        total_amount: Decimal,
        pay_amount: Decimal,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = website_order::ActiveModel {
            order_no: Set(order_no),
            user_id: Set(user_id),
            website_id: Set(req.website_id),
            total_amount: Set(total_amount),
            discount_amount: Set(Some(Decimal::from(0))),
            shipping_fee: Set(Some(Decimal::from(0))),
            pay_amount: Set(pay_amount),
            status: Set(Some(STATUS_PENDING_PAY)),
            pay_status: Set(Some(PAY_STATUS_UNPAID)),
            ship_status: Set(Some(SHIP_STATUS_UNSHIPPED)),
            pay_type: Set(req.pay_type),
            consignee_name: Set(Some(req.consignee_name.clone())),
            consignee_phone: Set(Some(req.consignee_phone.clone())),
            consignee_address: Set(Some(req.consignee_address.clone())),
            consignee_province: Set(req.consignee_province.clone()),
            consignee_city: Set(req.consignee_city.clone()),
            consignee_district: Set(req.consignee_district.clone()),
            consignee_zipcode: Set(req.consignee_zipcode.clone()),
            buyer_remark: Set(req.buyer_remark.clone()),
            create_time: Set(Some(now.clone())),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        WebsiteOrder::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据ID查询
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<website_order::Model>, DbErr> {
        WebsiteOrder::find_by_id(id)
            .filter(website_order::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 根据订单号查询
    pub async fn find_by_order_no<C: ConnectionTrait>(db: &C, order_no: &str) -> Result<Option<website_order::Model>, DbErr> {
        WebsiteOrder::find()
            .filter(website_order::Column::OrderNo.eq(order_no))
            .filter(website_order::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 用户订单列表（分页）
    pub async fn select_user_page(
        db: &DbConn,
        user_id: i64,
        page: i64,
        per_page: i64,
        status: Option<i32>,
    ) -> Result<(Vec<website_order::Model>, i64), DbErr> {
        let mut query = WebsiteOrder::find()
            .filter(website_order::Column::Deleted.eq(0))
            .filter(website_order::Column::UserId.eq(user_id));
        if let Some(s) = status { query = query.filter(website_order::Column::Status.eq(s)); }
        let paginator = query
            .order_by_desc(website_order::Column::CreateTime)
            .paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        let rows = paginator.fetch_page((page - 1) as u64).await?;
        Ok((rows, total))
    }

    /// 后台订单列表（分页）
    pub async fn select_admin_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        query: &OrderListQuery,
    ) -> Result<(Vec<website_order::Model>, i64), DbErr> {
        let mut q = WebsiteOrder::find()
            .filter(website_order::Column::Deleted.eq(0));
        if let Some(no) = &query.order_no { q = q.filter(website_order::Column::OrderNo.like(format!("%{}%", no))); }
        if let Some(s) = query.status { q = q.filter(website_order::Column::Status.eq(s)); }
        if let Some(s) = query.pay_status { q = q.filter(website_order::Column::PayStatus.eq(s)); }
        if let Some(s) = query.ship_status { q = q.filter(website_order::Column::ShipStatus.eq(s)); }
        if let Some(u) = query.user_id { q = q.filter(website_order::Column::UserId.eq(u)); }
        let paginator = q
            .order_by_desc(website_order::Column::CreateTime)
            .paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        let rows = paginator.fetch_page((page - 1) as u64).await?;
        Ok((rows, total))
    }

    /// 更新订单状态
    pub async fn update_status<C: ConnectionTrait>(
        db: &C,
        id: i64,
        status: i32,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteOrder::update_many()
            .col_expr(website_order::Column::Status, sea_orm::sea_query::Expr::value(status))
            .col_expr(website_order::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_order::Column::Id.eq(id))
            .filter(website_order::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 支付成功更新
    pub async fn update_paid<C: ConnectionTrait>(
        db: &C,
        id: i64,
        pay_type: i32,
        transaction_id: String,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteOrder::update_many()
            .col_expr(website_order::Column::Status, sea_orm::sea_query::Expr::value(STATUS_PENDING_SHIP))
            .col_expr(website_order::Column::PayStatus, sea_orm::sea_query::Expr::value(PAY_STATUS_PAID))
            .col_expr(website_order::Column::PayType, sea_orm::sea_query::Expr::value(pay_type))
            .col_expr(website_order::Column::PayTime, sea_orm::sea_query::Expr::value(now.clone()))
            .col_expr(website_order::Column::TransactionId, sea_orm::sea_query::Expr::value(transaction_id))
            .col_expr(website_order::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_order::Column::Id.eq(id))
            .filter(website_order::Column::Deleted.eq(0))
            .filter(website_order::Column::PayStatus.eq(PAY_STATUS_UNPAID))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 发货更新
    pub async fn update_shipped<C: ConnectionTrait>(
        db: &C,
        id: i64,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteOrder::update_many()
            .col_expr(website_order::Column::Status, sea_orm::sea_query::Expr::value(STATUS_PENDING_RECEIVE))
            .col_expr(website_order::Column::ShipStatus, sea_orm::sea_query::Expr::value(SHIP_STATUS_SHIPPED))
            .col_expr(website_order::Column::ShipTime, sea_orm::sea_query::Expr::value(now.clone()))
            .col_expr(website_order::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_order::Column::Id.eq(id))
            .filter(website_order::Column::Deleted.eq(0))
            .filter(website_order::Column::Status.eq(STATUS_PENDING_SHIP))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 确认收货
    pub async fn update_received<C: ConnectionTrait>(
        db: &C,
        id: i64,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteOrder::update_many()
            .col_expr(website_order::Column::Status, sea_orm::sea_query::Expr::value(STATUS_COMPLETED))
            .col_expr(website_order::Column::ShipStatus, sea_orm::sea_query::Expr::value(SHIP_STATUS_RECEIVED))
            .col_expr(website_order::Column::FinishTime, sea_orm::sea_query::Expr::value(now.clone()))
            .col_expr(website_order::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_order::Column::Id.eq(id))
            .filter(website_order::Column::Deleted.eq(0))
            .filter(website_order::Column::Status.eq(STATUS_PENDING_RECEIVE))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 取消订单
    pub async fn update_cancelled<C: ConnectionTrait>(
        db: &C,
        id: i64,
        reason: String,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteOrder::update_many()
            .col_expr(website_order::Column::Status, sea_orm::sea_query::Expr::value(STATUS_CANCELLED))
            .col_expr(website_order::Column::CancelTime, sea_orm::sea_query::Expr::value(now.clone()))
            .col_expr(website_order::Column::CancelReason, sea_orm::sea_query::Expr::value(reason))
            .col_expr(website_order::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_order::Column::Id.eq(id))
            .filter(website_order::Column::Deleted.eq(0))
            .filter(website_order::Column::Status.eq(STATUS_PENDING_PAY))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 后台卖家备注更新
    pub async fn update_seller_remark<C: ConnectionTrait>(
        db: &C,
        id: i64,
        remark: String,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteOrder::update_many()
            .col_expr(website_order::Column::SellerRemark, sea_orm::sea_query::Expr::value(remark))
            .col_expr(website_order::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_order::Column::Id.eq(id))
            .filter(website_order::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 软删除（批量）
    pub async fn batch_soft_delete<C: ConnectionTrait>(db: &C, ids: Vec<i64>) -> Result<i64, DbErr> {
        if ids.is_empty() { return Ok(0); }
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteOrder::update_many()
            .col_expr(website_order::Column::Deleted, sea_orm::sea_query::Expr::value(1))
            .col_expr(website_order::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_order::Column::Id.is_in(ids))
            .filter(website_order::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}
