//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::shop::entity::shop_order::{self, Entity as OrderEntity};
use crate::modules::shop::entity::shop_order_item::{self, Entity as OrderItemEntity};
use crate::utils::string_utils::serialize_option_u64_to_string;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::{DateTime, Decimal};
use sea_orm::*;

/// Order Request
/// 订单创建请求结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    /// 店铺ID
    pub shop_id: Option<i64>,
    /// 订单总金额
    pub total_amount: Option<f64>,
    /// 运费
    pub freight_amount: Option<f64>,
    /// 商品数量
    pub goods_count: Option<i32>,
    /// 收货人
    pub receiver_name: Option<String>,
    /// 收货电话
    pub receiver_phone: Option<String>,
    /// 收货地址
    pub receiver_address: Option<String>,
    /// 买家备注
    pub buyer_remark: Option<String>,
    /// 订单明细列表
    pub items: Option<Vec<OrderItemRequest>>,
}

/// Order Item Request
/// 订单明细请求结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemRequest {
    /// 商品ID
    pub spu_id: Option<i64>,
    /// SKU ID
    pub sku_id: Option<i64>,
    /// 商品标题
    pub goods_title: Option<String>,
    /// 商品图片
    pub goods_image: Option<String>,
    /// 规格描述
    pub spec_desc: Option<String>,
    /// 成交单价
    pub price: Option<f64>,
    /// 数量
    pub quantity: Option<i32>,
    /// 供货底价
    pub base_price: Option<f64>,
}

/// Order DTO
/// 订单数据传输对象
pub struct OrderDTO {
    /// 买家用户ID
    pub user_id: i64,
    /// 店铺ID
    pub shop_id: i64,
    /// 订单总金额
    pub total_amount: Decimal,
    /// 运费
    pub freight_amount: Decimal,
    /// 商品数量
    pub goods_count: i32,
    /// 收货人
    pub receiver_name: String,
    /// 收货电话
    pub receiver_phone: String,
    /// 收货地址
    pub receiver_address: String,
    /// 买家备注
    pub buyer_remark: Option<String>,
}

/// Order Item DTO
/// 订单明细数据传输对象
pub struct OrderItemDTO {
    /// 订单ID
    pub order_id: i64,
    /// 商品ID
    pub spu_id: i64,
    /// SKU ID
    pub sku_id: i64,
    /// 商品标题(快照)
    pub goods_title: String,
    /// 商品图片(快照)
    pub goods_image: String,
    /// 规格描述
    pub spec_desc: Option<String>,
    /// 成交单价
    pub price: Decimal,
    /// 数量
    pub quantity: i32,
    /// 供货底价
    pub base_price: Decimal,
    /// 佣金金额
    pub commission_amount: Decimal,
    /// 结算金额
    pub settlement_amount: Decimal,
}

/// Order VO
/// 订单视图对象
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 订单编号
    pub order_no: Option<String>,
    /// 买家用户ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 店铺ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub shop_id: Option<i64>,
    /// 订单总金额
    pub total_amount: Option<f64>,
    /// 运费
    pub freight_amount: Option<f64>,
    /// 平台佣金
    pub commission_amount: Option<f64>,
    /// 佣金比例
    pub commission_rate: Option<f64>,
    /// 结算金额
    pub settlement_amount: Option<f64>,
    /// 退款金额
    pub refund_amount: Option<f64>,
    /// 商品数量
    pub goods_count: Option<i32>,
    /// 状态
    pub status: Option<i16>,
    /// 收货人
    pub receiver_name: Option<String>,
    /// 收货电话
    pub receiver_phone: Option<String>,
    /// 收货地址
    pub receiver_address: Option<String>,
    /// 买家备注
    pub buyer_remark: Option<String>,
    /// 取消原因
    pub cancel_reason: Option<String>,
    /// 支付方式
    pub pay_method: Option<String>,
    /// 支付时间
    pub pay_time: Option<String>,
    /// 发货时间
    pub delivery_time: Option<String>,
    /// 签收时间
    pub receive_time: Option<String>,
    /// 完成时间
    pub finish_time: Option<String>,
    /// 取消时间
    pub cancel_time: Option<String>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
    /// 订单明细列表
    pub items: Option<Vec<OrderItemVO>>,
}

impl From<shop_order::Model> for OrderVO {
    fn from(model: shop_order::Model) -> Self {
        Self {
            id: Some(model.id),
            order_no: Some(model.order_no),
            user_id: Some(model.user_id),
            shop_id: Some(model.shop_id),
            total_amount: model.total_amount.to_f64(),
            freight_amount: model.freight_amount.to_f64(),
            commission_amount: model.commission_amount.to_f64(),
            commission_rate: model.commission_rate.to_f64(),
            settlement_amount: model.settlement_amount.to_f64(),
            refund_amount: model.refund_amount.to_f64(),
            goods_count: Some(model.goods_count),
            status: Some(model.status),
            receiver_name: Some(model.receiver_name),
            receiver_phone: Some(model.receiver_phone),
            receiver_address: Some(model.receiver_address),
            buyer_remark: model.buyer_remark,
            cancel_reason: model.cancel_reason,
            pay_method: model.pay_method,
            pay_time: model.pay_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            delivery_time: model.delivery_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            receive_time: model.receive_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            finish_time: model.finish_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            cancel_time: model.cancel_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            items: None,
        }
    }
}

/// Order Item VO
/// 订单明细视图对象
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 订单ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    /// 商品ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub spu_id: Option<i64>,
    /// SKU ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub sku_id: Option<i64>,
    /// 商品标题
    pub goods_title: Option<String>,
    /// 商品图片
    pub goods_image: Option<String>,
    /// 规格描述
    pub spec_desc: Option<String>,
    /// 成交单价
    pub price: Option<f64>,
    /// 数量
    pub quantity: Option<i32>,
    /// 供货底价
    pub base_price: Option<f64>,
    /// 佣金金额
    pub commission_amount: Option<f64>,
    /// 结算金额
    pub settlement_amount: Option<f64>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<shop_order_item::Model> for OrderItemVO {
    fn from(model: shop_order_item::Model) -> Self {
        Self {
            id: Some(model.id),
            order_id: Some(model.order_id),
            spu_id: Some(model.spu_id),
            sku_id: Some(model.sku_id),
            goods_title: Some(model.goods_title),
            goods_image: Some(model.goods_image),
            spec_desc: model.spec_desc,
            price: model.price.to_f64(),
            quantity: Some(model.quantity),
            base_price: model.base_price.to_f64(),
            commission_amount: model.commission_amount.to_f64(),
            settlement_amount: model.settlement_amount.to_f64(),
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// Order Page Query
/// 订单分页查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderPageQuery {
    /// 页码
    pub page_num: Option<i64>,
    /// 每页数量
    pub page_size: Option<i64>,
    /// 买家用户ID
    pub user_id: Option<i64>,
    /// 店铺ID
    pub shop_id: Option<i64>,
    /// 状态
    pub status: Option<i16>,
    /// 订单编号(模糊匹配)
    pub order_no: Option<String>,
}

/// OrderModel
/// 订单数据操作模型
pub struct OrderModel;

impl OrderModel {
    /// 插入订单记录
    pub async fn insert<C: ConnectionTrait>(db: &C, form: &OrderDTO, order_no: &str) -> Result<i64, DbErr> {
        let active = shop_order::ActiveModel {
            order_no: Set(order_no.to_string()),
            user_id: Set(form.user_id),
            shop_id: Set(form.shop_id),
            total_amount: Set(form.total_amount),
            freight_amount: Set(form.freight_amount),
            settlement_amount: Set(form.total_amount - form.freight_amount),
            goods_count: Set(form.goods_count),
            status: Set(0),
            receiver_name: Set(form.receiver_name.clone()),
            receiver_phone: Set(form.receiver_phone.clone()),
            receiver_address: Set(form.receiver_address.clone()),
            buyer_remark: Set(form.buyer_remark.clone()),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        OrderEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据ID查询订单
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<shop_order::Model>, DbErr> {
        OrderEntity::find_by_id(id).one(db).await
    }

    /// 根据订单编号查询订单
    pub async fn find_by_order_no<C: ConnectionTrait>(db: &C, order_no: &str) -> Result<Option<shop_order::Model>, DbErr> {
        OrderEntity::find()
            .filter(shop_order::Column::OrderNo.eq(order_no))
            .one(db)
            .await
    }

    /// 根据用户ID查询订单列表
    pub async fn find_by_user_id<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<Vec<shop_order::Model>, DbErr> {
        OrderEntity::find()
            .filter(shop_order::Column::UserId.eq(user_id))
            .order_by_desc(shop_order::Column::Id)
            .all(db)
            .await
    }

    /// 分页查询订单列表
    pub async fn find_page<C: ConnectionTrait>(
        db: &C,
        page: i64,
        per_page: i64,
        user_id: Option<i64>,
        shop_id: Option<i64>,
        status: Option<i16>,
        order_no: Option<String>,
    ) -> Result<(Vec<shop_order::Model>, i64), DbErr> {
        let paginator = OrderEntity::find()
            .apply_if(user_id, |query, v| query.filter(shop_order::Column::UserId.eq(v)))
            .apply_if(shop_id, |query, v| query.filter(shop_order::Column::ShopId.eq(v)))
            .apply_if(status, |query, v| query.filter(shop_order::Column::Status.eq(v)))
            .apply_if(order_no, |query, v| query.filter(shop_order::Column::OrderNo.contains(v)))
            .order_by_desc(shop_order::Column::Id)
            .paginate(db, per_page as u64);

        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page(page.saturating_sub(1) as u64).await?;
        Ok((items, total))
    }

    /// 更新订单状态
    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i16) -> Result<i64, DbErr> {
        let active = shop_order::ActiveModel {
            id: Set(id),
            status: Set(status),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        OrderEntity::update_many()
            .set(active)
            .filter(shop_order::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新订单支付信息
    pub async fn update_pay_info<C: ConnectionTrait>(db: &C, id: i64, pay_method: &str, status: i16) -> Result<i64, DbErr> {
        let active = shop_order::ActiveModel {
            id: Set(id),
            status: Set(status),
            pay_method: Set(Some(pay_method.to_string())),
            pay_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        OrderEntity::update_many()
            .set(active)
            .filter(shop_order::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}

/// OrderItemModel
/// 订单明细数据操作模型
pub struct OrderItemModel;

impl OrderItemModel {
    /// 插入订单明细记录
    pub async fn insert<C: ConnectionTrait>(db: &C, form: &OrderItemDTO) -> Result<i64, DbErr> {
        let active = shop_order_item::ActiveModel {
            order_id: Set(form.order_id),
            spu_id: Set(form.spu_id),
            sku_id: Set(form.sku_id),
            goods_title: Set(form.goods_title.clone()),
            goods_image: Set(form.goods_image.clone()),
            spec_desc: Set(form.spec_desc.clone()),
            price: Set(form.price),
            quantity: Set(form.quantity),
            base_price: Set(form.base_price),
            commission_amount: Set(form.commission_amount),
            settlement_amount: Set(form.settlement_amount),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        OrderItemEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据订单ID查询订单明细列表
    pub async fn find_by_order_id<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<Vec<shop_order_item::Model>, DbErr> {
        OrderItemEntity::find()
            .filter(shop_order_item::Column::OrderId.eq(order_id))
            .all(db)
            .await
    }

    /// 批量插入订单明细记录
    pub async fn batch_insert<C: ConnectionTrait>(db: &C, items: &[OrderItemDTO]) -> Result<Vec<i64>, DbErr> {
        let mut ids = Vec::new();
        for item in items {
            let id = Self::insert(db, item).await?;
            ids.push(id);
        }
        Ok(ids)
    }
}
