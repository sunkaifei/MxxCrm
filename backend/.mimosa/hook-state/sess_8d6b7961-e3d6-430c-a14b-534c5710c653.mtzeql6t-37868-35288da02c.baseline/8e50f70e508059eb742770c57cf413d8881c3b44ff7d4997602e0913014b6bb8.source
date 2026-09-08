//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_service_ticket")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub ticket_no: Option<String>,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub entitlement_id: Option<i64>,
    pub order_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub channel: Option<i32>,
    pub ticket_type: Option<i32>,
    pub status: Option<i32>,
    pub assigned_to: Option<i64>,
    pub assigned_dept: Option<i64>,
    pub sla_response_deadline: Option<DateTime>,
    pub sla_resolution_deadline: Option<DateTime>,
    pub responded_at: Option<DateTime>,
    pub resolved_at: Option<DateTime>,
    pub quota_consumed: Option<Decimal>,
    pub resolution: Option<String>,
    pub satisfaction: Option<i32>,
    pub satisfaction_remark: Option<String>,
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
