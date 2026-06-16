//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mxx_shop_delivery")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    pub id: i64,
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
    /// 物流状态: 0=待揽件, 1=运输中, 2=已签收
    pub delivery_status: i16,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
