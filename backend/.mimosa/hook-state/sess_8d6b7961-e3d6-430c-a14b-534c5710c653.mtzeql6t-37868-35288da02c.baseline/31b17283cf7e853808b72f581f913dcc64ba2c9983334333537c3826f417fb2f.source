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
#[sea_orm(table_name = "mxx_finance_salary_confirm")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub salary_record_id: i64,
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub year: i32,
    pub month: i32,
    /// 1=确认无误, 2=申请重新核算
    pub action: i32,
    /// 申请理由
    pub reason: Option<String>,
    /// 0=待处理, 1=已处理(同意重算), 2=已驳回
    pub status: Option<i32>,
    pub handler_id: Option<i64>,
    pub handler_name: Option<String>,
    pub handle_time: Option<DateTime>,
    pub handle_remark: Option<String>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
