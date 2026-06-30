use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_contract_payment_plan")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub contract_id: Option<i64>,
    pub stage_name: Option<String>,
    pub payment_type: Option<i32>,
    pub plan_amount: Option<Decimal>,
    pub received_amount: Option<Decimal>,
    pub plan_date: Option<NaiveDate>,
    pub actual_date: Option<NaiveDate>,
    pub status: Option<i32>,
    pub sort: Option<i32>,
    pub remark: Option<String>,
    pub owner_user_id: Option<i64>,
    pub dept_id: Option<i64>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
