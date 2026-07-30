//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 费用类型字典实体层
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_expense_type")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub type_name: Option<String>,
    pub type_code: Option<String>,
    pub parent_id: Option<i64>,
    pub sort: Option<i32>,
    /// 状态：0=禁用,1=启用
    pub status: Option<i32>,
    /// 是否系统内置：0=否,1=是
    pub is_system: Option<i32>,
    pub create_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
