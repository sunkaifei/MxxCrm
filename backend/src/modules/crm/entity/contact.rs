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
#[sea_orm(table_name = "mxx_crm_contact")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub customer_id: Option<i64>,
    pub name: Option<String>,
    pub title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub whatsapp: Option<String>,
    pub wechat: Option<String>,
    pub is_primary: Option<bool>,
    pub is_billing: Option<bool>,
    pub is_shipping: Option<bool>,
    pub birthday: Option<Date>,
    pub notes: Option<String>,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
