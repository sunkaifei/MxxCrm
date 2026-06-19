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
use crate::core::r#enum::lead_source_enum::LeadSource;
use crate::core::r#enum::lead_status_enum::LeadStatus;
use crate::core::r#enum::currency_code_enum::CurrencyCode;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_lead")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub company_name: Option<String>,
    pub contact_name: Option<String>,
    pub title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
    pub industry: Option<IndustryType>,
    pub source: Option<LeadSource>,
    pub source_detail: Option<String>,
    pub status: Option<LeadStatus>,
    pub level: Option<String>,
    pub tags: Option<Vec<String>>,
    pub budget: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub next_follow_at: Option<DateTime>,
    pub assigned_to: Option<i64>,
    pub converted_to_customer_id: Option<i64>,
    pub converted_at: Option<DateTime>,
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
