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
#[sea_orm(table_name = "mxx_inventory_bin_location")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub warehouse_id: Option<i64>,
    pub area_id: Option<i64>,
    pub bin_code: Option<String>,
    pub bin_name: Option<String>,
    pub bin_type: Option<i32>,
    pub row_no: Option<i32>,
    pub column_no: Option<i32>,
    pub layer_no: Option<i32>,
    pub capacity: Option<Decimal>,
    pub used_capacity: Option<Decimal>,
    pub is_active: Option<i32>,
    pub sort_order: Option<i32>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
