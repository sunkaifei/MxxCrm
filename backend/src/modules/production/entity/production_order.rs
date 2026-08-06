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

#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_production_order")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub mo_no: Option<String>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub quantity: Option<Decimal>,
    pub completed_quantity: Option<Decimal>,
    pub plan_start_date: Option<Date>,
    pub plan_complete_date: Option<Date>,
    pub actual_complete_date: Option<Date>,
    pub source_type: Option<String>,
    pub source_id: Option<i64>,
    pub source_no: Option<String>,
    pub workshop_id: Option<i64>,
    pub production_lead_time: Option<i32>,
    pub status: Option<i32>,
    pub cost_amount: Option<Decimal>,
    pub remark: Option<String>,
    pub deleted: Option<i32>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}