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
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_shipment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub shipment_no: Option<String>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub shipment_date: Option<Date>,
    pub logistics_company: Option<String>,
    pub tracking_no: Option<String>,
    pub shipping_method: Option<i32>,
    pub receiver_name: Option<String>,
    pub receiver_phone: Option<String>,
    pub shipping_address: Option<String>,
    pub total_quantity: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
