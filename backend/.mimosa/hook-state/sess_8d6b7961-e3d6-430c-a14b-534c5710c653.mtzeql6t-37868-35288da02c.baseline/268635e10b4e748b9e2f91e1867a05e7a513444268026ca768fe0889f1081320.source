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

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_statistics_daily_payment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub stat_date: Date,
    pub employee_id: i64,
    pub dept_id: i64,
    pub customer_id: i64,
    pub payment_count: i32,
    pub payment_amount: Decimal,
    pub contract_amount: Decimal,
    pub batch_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
