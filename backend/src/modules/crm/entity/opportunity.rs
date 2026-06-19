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
use crate::core::r#enum::lead_source_enum::LeadSource;
use crate::core::r#enum::opportunity_stage_enum::OpportunityStage;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_opportunity")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    #[sea_orm(ignore)]
    pub opportunity_no: Option<String>,
    pub customer_id: Option<i64>,
    #[sea_orm(ignore)]
    pub contact_id: Option<i64>,
    #[sea_orm(ignore)]
    pub lead_id: Option<i64>,
    #[sea_orm(column_name = "name")]
    pub title: Option<String>,
    pub description: Option<String>,
    pub stage: Option<OpportunityStage>,
    pub probability: Option<i32>,
    pub amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub expected_close_date: Option<Date>,
    pub assigned_to: Option<i64>,
    #[sea_orm(ignore)]
    pub source: Option<LeadSource>,
    #[sea_orm(ignore)]
    pub tags: Option<Vec<String>>,
    #[sea_orm(ignore)]
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
