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
#[sea_orm(table_name = "mxx_purchase_return")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub return_no: Option<String>,
    pub receipt_id: Option<i64>,
    pub po_id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub return_date: Option<Date>,
    pub total_amount: Option<Decimal>,
    pub reason: Option<String>,
    pub status: Option<i32>,
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