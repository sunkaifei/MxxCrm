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

/// 发货单（mxx_website_delivery）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_website_delivery")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 订单ID
    pub order_id: i64,

    /// 订单号
    pub order_no: Option<String>,

    /// 物流单号
    pub delivery_no: Option<String>,

    /// 物流公司
    pub delivery_company: Option<String>,

    /// 配送方式：1快递 2自提 3同城
    #[serde(default)]
    pub delivery_type: Option<i32>,

    /// 状态：0已发货 1已签收 2已退回
    #[serde(default)]
    pub status: Option<i32>,

    /// 发货人ID（管理员）
    pub shipper_id: Option<i64>,

    /// 发货人姓名
    pub shipper_name: Option<String>,

    /// 收货人姓名
    pub consignee_name: Option<String>,

    /// 收货人电话
    pub consignee_phone: Option<String>,

    /// 收货人地址
    pub consignee_address: Option<String>,

    /// 件数
    #[serde(default)]
    pub item_count: Option<i32>,

    /// 备注
    pub remark: Option<String>,

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
