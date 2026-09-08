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
#[sea_orm(table_name = "mxx_sale_tax_invoice")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub invoice_id: Option<i64>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub tax_invoice_no: Option<String>,
    pub tax_invoice_code: Option<String>,
    pub platform: Option<i32>,
    pub platform_invoice_id: Option<String>,
    pub invoice_category: Option<i32>,
    pub status: Option<i32>,
    pub amount: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub pdf_url: Option<String>,
    pub buyer_name: Option<String>,
    pub buyer_tax_no: Option<String>,
    pub buyer_address: Option<String>,
    pub buyer_bank_account: Option<String>,
    pub issue_time: Option<DateTime>,
    pub void_time: Option<DateTime>,
    pub void_reason: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
