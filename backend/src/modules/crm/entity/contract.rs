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
use crate::core::r#enum::contract_status_enum::ContractStatus;
use crate::core::r#enum::currency_code_enum::CurrencyCode;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_contract")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub contract_no: Option<String>,
    pub customer_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    #[sea_orm(column_name = "name")]
    pub title: Option<String>,
    pub contract_type: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub tax_amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub status: Option<ContractStatus>,
    pub start_date: Option<Date>,
    pub end_date: Option<Date>,
    pub sign_date: Option<Date>,
    pub payment_terms: Option<String>,
    pub contract_file: Option<String>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub updated_at: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
