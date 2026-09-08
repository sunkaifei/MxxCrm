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
#[sea_orm(table_name = "mxx_purchase_supplier")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub supplier_no: Option<String>,
    #[sea_orm(column_name = "name")]
    pub company_name: Option<String>,
    pub short_name: Option<String>,
    #[sea_orm(column_name = "contact_person")]
    pub contact_name: Option<String>,
    #[sea_orm(column_name = "phone")]
    pub contact_phone: Option<String>,
    #[sea_orm(column_name = "email")]
    pub contact_email: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub level: Option<i32>,
    pub currency: Option<i32>,
    pub credit_limit: Option<Decimal>,
    pub credit_days: Option<i32>,
    pub bank_name: Option<String>,
    pub bank_account: Option<String>,
    pub tax_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<i32>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub notes: Option<String>,
    pub is_active: Option<bool>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
