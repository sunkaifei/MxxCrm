//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 订单项（mxx_website_order_item）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_website_order_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 订单ID
    pub order_id: i64,

    /// 产品ID
    pub product_id: i64,

    /// SKU ID
    pub sku_id: Option<i64>,

    /// 产品名称（快照）
    pub product_name: Option<String>,

    /// 产品图片（快照）
    pub product_image: Option<String>,

    /// SKU 编码
    pub sku_code: Option<String>,

    /// SKU 规格（JSON）
    pub sku_specs: Option<String>,

    /// 单价
    #[serde(default)]
    pub price: Decimal,

    /// 数量
    #[serde(default)]
    pub quantity: i32,

    /// 小计金额
    #[serde(default)]
    pub total_amount: Decimal,

    /// 退款状态：0未退 1申请中 2已退款 3拒绝退款
    #[serde(default)]
    pub refund_status: Option<i32>,

    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
