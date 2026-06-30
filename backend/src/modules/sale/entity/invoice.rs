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
#[sea_orm(table_name = "mxx_sale_invoice")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub invoice_no: Option<String>,
    pub title: Option<String>,
    pub invoice_type: Option<i32>,
    pub contract_id: Option<i64>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub tax_no: Option<String>,
    pub invoice_date: Option<Date>,
    pub due_date: Option<Date>,
    pub amount: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub status: Option<i32>,
    pub buyer_name: Option<String>,
    pub buyer_tax_no: Option<String>,
    pub buyer_address: Option<String>,
    pub buyer_bank: Option<String>,
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
