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

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_company_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub company_name: Option<String>,
    pub credit_code: Option<String>,
    pub legal_person: Option<String>,
    pub legal_phone: Option<String>,
    pub register_address: Option<String>,
    pub business_scope: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub logo_url: Option<String>,
    pub tax_number: Option<String>,
    pub invoice_title: Option<String>,
    pub remark: Option<String>,
    pub deleted: Option<i32>,
    pub create_by: Option<i64>,
    pub update_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
