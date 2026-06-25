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
use crate::core::r#enum::currency_code_enum::CurrencyCode;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_quotation")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub quotation_no: Option<String>,
    pub customer_id: Option<i64>,
    pub contact_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub title: Option<String>,
    pub items: Option<serde_json::Value>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub tax_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub grand_total: Option<Decimal>,
    pub valid_until: Option<Date>,
    pub status: Option<String>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub delivery_date: Option<Date>,
    pub port_of_loading: Option<String>,
    pub port_of_destination: Option<String>,
    pub bank_info: Option<String>,
    pub notes: Option<String>,
    pub assigned_to: Option<i64>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}