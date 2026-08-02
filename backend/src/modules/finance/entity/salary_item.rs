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
#[sea_orm(table_name = "mxx_finance_salary_item")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub item_code: String,
    pub item_name: String,
    /// 项目类型
    pub item_type: Option<i32>,
    /// 计算方式
    pub calc_mode: Option<i32>,
    pub formula: Option<String>,
    pub default_value: Decimal,
    /// 是否计税
    pub is_taxable: Option<i32>,
    /// 是否税前
    pub is_pretax: Option<i32>,
    pub sort: Option<i32>,
    pub enabled: Option<i32>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
