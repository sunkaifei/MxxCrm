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

/// 网站订单（mxx_website_order）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_website_order")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 订单号（唯一）
    pub order_no: String,

    /// 用户ID
    pub user_id: i64,

    /// 站点ID
    pub website_id: Option<i64>,

    /// 商品总金额
    #[serde(default)]
    pub total_amount: Decimal,

    /// 优惠金额
    #[serde(default)]
    pub discount_amount: Option<Decimal>,

    /// 运费
    #[serde(default)]
    pub shipping_fee: Option<Decimal>,

    /// 应付金额
    #[serde(default)]
    pub pay_amount: Decimal,

    /// 订单状态：0待付款 1待发货 2待收货 3已完成 4已取消 5已关闭
    #[serde(default)]
    pub status: Option<i32>,

    /// 支付状态：0未支付 1已支付 2已退款 3部分退款
    #[serde(default)]
    pub pay_status: Option<i32>,

    /// 发货状态：0未发货 1部分发货 2已发货 3已签收
    #[serde(default)]
    pub ship_status: Option<i32>,

    /// 支付方式：1微信 2支付宝 3余额
    pub pay_type: Option<i32>,

    /// 支付时间
    pub pay_time: Option<DateTime>,

    /// 发货时间
    pub ship_time: Option<DateTime>,

    /// 完成时间
    pub finish_time: Option<DateTime>,

    /// 取消时间
    pub cancel_time: Option<DateTime>,

    /// 取消原因
    pub cancel_reason: Option<String>,

    /// 收货人姓名
    pub consignee_name: Option<String>,

    /// 收货人电话
    pub consignee_phone: Option<String>,

    /// 收货人地址
    pub consignee_address: Option<String>,

    /// 省
    pub consignee_province: Option<String>,

    /// 市
    pub consignee_city: Option<String>,

    /// 区
    pub consignee_district: Option<String>,

    /// 邮编
    pub consignee_zipcode: Option<String>,

    /// 买家备注
    pub buyer_remark: Option<String>,

    /// 卖家备注
    pub seller_remark: Option<String>,

    /// 第三方支付流水号
    pub transaction_id: Option<String>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 软删除：0未删除 1已删除
    #[serde(default)]
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
