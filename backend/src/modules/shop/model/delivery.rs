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
use crate::modules::shop::entity::shop_delivery::{self, Entity as DeliveryEntity};
use crate::utils::string_utils::serialize_option_u64_to_string;
use sea_orm::prelude::DateTime;
use sea_orm::*;

/// Delivery Request
/// 物流请求结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryRequest {
    /// 订单ID
    pub order_id: Option<i64>,
    /// 物流公司
    pub logistics_company: Option<String>,
    /// 物流单号
    pub logistics_no: Option<String>,
    /// 收货人
    pub receiver_name: Option<String>,
    /// 收货电话
    pub receiver_phone: Option<String>,
    /// 收货地址
    pub receiver_address: Option<String>,
    /// 发货备注
    pub delivery_remark: Option<String>,
    /// 物流状态
    pub delivery_status: Option<i16>,
}

/// Delivery DTO
/// 物流数据传输对象
pub struct DeliveryDTO {
    /// 订单ID
    pub order_id: i64,
    /// 物流公司
    pub logistics_company: String,
    /// 物流单号
    pub logistics_no: String,
    /// 收货人
    pub receiver_name: String,
    /// 收货电话
    pub receiver_phone: String,
    /// 收货地址
    pub receiver_address: String,
    /// 发货备注
    pub delivery_remark: Option<String>,
    /// 物流状态
    pub delivery_status: i16,
}

impl From<DeliveryRequest> for DeliveryDTO {
    fn from(req: DeliveryRequest) -> Self {
        DeliveryDTO {
            order_id: req.order_id.unwrap_or(0),
            logistics_company: req.logistics_company.unwrap_or_default(),
            logistics_no: req.logistics_no.unwrap_or_default(),
            receiver_name: req.receiver_name.unwrap_or_default(),
            receiver_phone: req.receiver_phone.unwrap_or_default(),
            receiver_address: req.receiver_address.unwrap_or_default(),
            delivery_remark: req.delivery_remark,
            delivery_status: req.delivery_status.unwrap_or(0),
        }
    }
}

/// Delivery VO
/// 物流视图对象
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 订单ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    /// 物流公司
    pub logistics_company: Option<String>,
    /// 物流单号
    pub logistics_no: Option<String>,
    /// 收货人
    pub receiver_name: Option<String>,
    /// 收货电话
    pub receiver_phone: Option<String>,
    /// 收货地址
    pub receiver_address: Option<String>,
    /// 发货备注
    pub delivery_remark: Option<String>,
    /// 物流状态
    pub delivery_status: Option<i16>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<shop_delivery::Model> for DeliveryVO {
    fn from(model: shop_delivery::Model) -> Self {
        Self {
            id: Some(model.id),
            order_id: Some(model.order_id),
            logistics_company: Some(model.logistics_company),
            logistics_no: Some(model.logistics_no),
            receiver_name: Some(model.receiver_name),
            receiver_phone: Some(model.receiver_phone),
            receiver_address: Some(model.receiver_address),
            delivery_remark: model.delivery_remark,
            delivery_status: Some(model.delivery_status),
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// DeliveryModel
/// 物流数据操作模型
pub struct DeliveryModel;

impl DeliveryModel {
    /// 插入物流记录
    pub async fn insert<C: ConnectionTrait>(db: &C, form: &DeliveryDTO) -> Result<i64, DbErr> {
        let active = shop_delivery::ActiveModel {
            order_id: Set(form.order_id),
            logistics_company: Set(form.logistics_company.clone()),
            logistics_no: Set(form.logistics_no.clone()),
            receiver_name: Set(form.receiver_name.clone()),
            receiver_phone: Set(form.receiver_phone.clone()),
            receiver_address: Set(form.receiver_address.clone()),
            delivery_remark: Set(form.delivery_remark.clone()),
            delivery_status: Set(form.delivery_status),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        DeliveryEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据订单ID查询物流列表
    pub async fn find_by_order_id<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<Vec<shop_delivery::Model>, DbErr> {
        DeliveryEntity::find()
            .filter(shop_delivery::Column::OrderId.eq(order_id))
            .all(db)
            .await
    }

    /// 根据ID查询物流
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<shop_delivery::Model>, DbErr> {
        DeliveryEntity::find_by_id(id).one(db).await
    }

    /// 更新物流状态
    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, delivery_status: i16) -> Result<i64, DbErr> {
        let active = shop_delivery::ActiveModel {
            id: Set(id),
            delivery_status: Set(delivery_status),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        DeliveryEntity::update_many()
            .set(active)
            .filter(shop_delivery::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}
