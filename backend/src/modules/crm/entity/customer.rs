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
use crate::core::r#enum::industry_enum::IndustryType;
use crate::core::r#enum::lead_source_enum::LeadSource;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_customer")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub customer_no: Option<String>,
    pub company_name: Option<String>,
    pub short_name: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
    pub industry: Option<IndustryType>,
    pub level: Option<i32>,
    pub total_deal_amount: Option<Decimal>,
    pub total_deal_count: Option<i32>,
    pub last_deal_at: Option<DateTime>,
    pub source: Option<LeadSource>,
    pub currency: Option<CurrencyCode>,
    pub credit_limit: Option<Decimal>,
    pub credit_days: Option<i32>,
    pub assigned_to: Option<i64>,
    pub cooperated_at: Option<Date>,
    pub birthday_month: Option<i32>,
    pub next_follow_at: Option<DateTime>,
    pub description: Option<String>,
    pub custom_fields: Option<serde_json::Value>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
