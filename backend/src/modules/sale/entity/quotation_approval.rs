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
#[sea_orm(table_name = "mxx_sale_quotation_approval")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub quotation_id: Option<i64>,
    pub version: Option<i32>,
    pub approval_type: Option<i32>,
    pub approval_status: Option<i32>,
    pub approver_id: Option<i64>,
    pub approver_name: Option<String>,
    pub original_amount: Option<Decimal>,
    pub adjusted_amount: Option<Decimal>,
    pub approval_remark: Option<String>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
