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
use crate::core::r#enum::industry_type_enum::IndustryType;

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
    pub address: Option<String>,
    pub website: Option<String>,
    pub industry: Option<IndustryType>,
    #[sea_orm(ignore)]
    pub region: Option<String>,
    pub level: Option<i32>,
    #[sea_orm(ignore)]
    pub currency: Option<CurrencyCode>,
    #[sea_orm(ignore)]
    pub credit_limit: Option<Decimal>,
    #[sea_orm(ignore)]
    pub credit_days: Option<i32>,
    #[sea_orm(ignore)]
    pub bank_name: Option<String>,
    #[sea_orm(ignore)]
    pub bank_account: Option<String>,
    #[sea_orm(ignore)]
    pub tax_id: Option<String>,
    #[sea_orm(ignore)]
    pub tags: Option<Vec<String>>,
    pub status: Option<i32>,
    /// 付款条款
    pub payment_terms: Option<String>,
    /// 交货条款
    pub delivery_terms: Option<String>,
    /// 备注
    pub notes: Option<String>,
    /// 是否启用
    pub is_active: Option<bool>,
    #[sea_orm(ignore)]
    pub description: Option<String>,
    #[sea_orm(ignore)]
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
