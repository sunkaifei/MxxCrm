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
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_order_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub order_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub sku: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub unit_id: Option<i64>,
    pub quantity: Option<i32>,
    pub unit_price: Option<Decimal>,
    pub discount_rate: Option<Decimal>,
    pub discount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub delivery_date: Option<Date>,
    pub delivered_quantity: Option<Decimal>,
    pub remark: Option<String>,
    pub sort: Option<i32>,
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<i64>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
