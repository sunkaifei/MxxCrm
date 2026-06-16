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
use crate::core::r#enum::industry_enum::IndustryType;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_purchase_supplier")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub supplier_no: Option<String>,
    pub company_name: Option<String>,
    pub short_name: Option<String>,
    pub country: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
    pub industry: Option<IndustryType>,
    pub region: Option<String>,
    pub level: Option<String>,
    pub currency: Option<String>,
    pub credit_limit: Option<Decimal>,
    pub credit_days: Option<i32>,
    pub bank_name: Option<String>,
    pub bank_account: Option<String>,
    pub tax_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub description: Option<String>,
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
