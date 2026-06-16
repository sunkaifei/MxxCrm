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
use crate::modules::shop::entity::shop_refund::{self, Entity as RefundEntity};
use crate::utils::string_utils::serialize_option_u64_to_string;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::{DateTime, Decimal};
use sea_orm::*;

/// Refund Request
/// 退款请求结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RefundRequest {
    /// 订单ID
    pub order_id: Option<i64>,
    /// 订单明细ID
    pub order_item_id: Option<i64>,
    /// 退款类型
    pub refund_type: Option<i16>,
    /// 退款原因
    pub refund_reason: Option<String>,
    /// 退款金额
    pub refund_amount: Option<f64>,
}

/// Refund DTO
/// 退款数据传输对象
pub struct RefundDTO {
    /// 订单ID
    pub order_id: i64,
    /// 订单明细ID
    pub order_item_id: i64,
    /// 退款单号
    pub refund_no: String,
    /// 买家用户ID
    pub user_id: i64,
    /// 店铺ID
    pub shop_id: i64,
    /// 退款类型
    pub refund_type: i16,
    /// 退款原因
    pub refund_reason: String,
    /// 退款金额
    pub refund_amount: Decimal,
    /// 退款状态
    pub refund_status: i16,
}

/// Refund VO
/// 退款视图对象
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RefundVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 订单ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    /// 订单明细ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_item_id: Option<i64>,
    /// 退款单号
    pub refund_no: Option<String>,
    /// 买家用户ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 店铺ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub shop_id: Option<i64>,
    /// 退款类型
    pub refund_type: Option<i16>,
    /// 退款原因
    pub refund_reason: Option<String>,
    /// 退款金额
    pub refund_amount: Option<f64>,
    /// 退款状态
    pub refund_status: Option<i16>,
    /// 审核备注
    pub audit_remark: Option<String>,
    /// 审核时间
    pub audit_time: Option<String>,
    /// 退回原因
    pub reject_reason: Option<String>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<shop_refund::Model> for RefundVO {
    fn from(model: shop_refund::Model) -> Self {
        Self {
            id: Some(model.id),
            order_id: Some(model.order_id),
            order_item_id: Some(model.order_item_id),
            refund_no: Some(model.refund_no),
            user_id: Some(model.user_id),
            shop_id: Some(model.shop_id),
            refund_type: Some(model.refund_type),
            refund_reason: Some(model.refund_reason),
            refund_amount: model.refund_amount.to_f64(),
            refund_status: Some(model.refund_status),
            audit_remark: model.audit_remark,
            audit_time: model.audit_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            reject_reason: model.reject_reason,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// RefundModel
/// 退款数据操作模型
pub struct RefundModel;

impl RefundModel {
    /// 插入退款记录
    pub async fn insert<C: ConnectionTrait>(db: &C, form: &RefundDTO) -> Result<i64, DbErr> {
        let active = shop_refund::ActiveModel {
            order_id: Set(form.order_id),
            order_item_id: Set(form.order_item_id),
            refund_no: Set(form.refund_no.clone()),
            user_id: Set(form.user_id),
            shop_id: Set(form.shop_id),
            refund_type: Set(form.refund_type),
            refund_reason: Set(form.refund_reason.clone()),
            refund_amount: Set(form.refund_amount),
            refund_status: Set(form.refund_status),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        RefundEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据ID查询退款
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<shop_refund::Model>, DbErr> {
        RefundEntity::find_by_id(id).one(db).await
    }

    /// 根据订单ID查询退款列表
    pub async fn find_by_order_id<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<Vec<shop_refund::Model>, DbErr> {
        RefundEntity::find()
            .filter(shop_refund::Column::OrderId.eq(order_id))
            .all(db)
            .await
    }

    /// 根据用户ID查询退款列表
    pub async fn find_by_user_id<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<Vec<shop_refund::Model>, DbErr> {
        RefundEntity::find()
            .filter(shop_refund::Column::UserId.eq(user_id))
            .order_by_desc(shop_refund::Column::Id)
            .all(db)
            .await
    }

    /// 审核退款
    pub async fn audit<C: ConnectionTrait>(db: &C, id: i64, refund_status: i16, audit_remark: Option<String>, reject_reason: Option<String>) -> Result<i64, DbErr> {
        let active = shop_refund::ActiveModel {
            id: Set(id),
            refund_status: Set(refund_status),
            audit_remark: Set(audit_remark),
            reject_reason: Set(reject_reason),
            audit_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        RefundEntity::update_many()
            .set(active)
            .filter(shop_refund::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}
