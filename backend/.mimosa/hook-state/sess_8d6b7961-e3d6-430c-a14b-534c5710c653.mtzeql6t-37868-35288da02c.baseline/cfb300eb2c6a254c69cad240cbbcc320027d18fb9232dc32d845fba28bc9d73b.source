//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_exchange_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub exchange_id: Option<i64>,
    pub original_order_item_id: Option<i64>,
    pub original_product_id: Option<i64>,
    pub original_product_name: Option<String>,
    pub original_qty: Option<Decimal>,
    pub new_product_id: Option<i64>,
    pub new_product_name: Option<String>,
    pub new_qty: Option<Decimal>,
    pub new_unit_price: Option<Decimal>,
    pub price_diff: Option<Decimal>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
