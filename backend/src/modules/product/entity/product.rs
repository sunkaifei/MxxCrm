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
#[sea_orm(table_name = "mxx_product_product")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub category_id: Option<i64>,
    pub brand: Option<String>,
    pub origin: Option<String>,
    pub material: Option<String>,
    pub weight: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub purchase_price: Option<Decimal>,
    pub sale_price: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub stock: Option<i32>,
    pub min_stock: Option<i32>,
    pub max_stock: Option<i32>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub images: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub custom_fields: Option<serde_json::Value>,
    pub created_by: Option<i64>,
    pub created_at: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub updated_at: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
