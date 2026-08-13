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
use chrono::NaiveDate;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_salary_adjustment")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub employee_id: i64,
    /// 调薪日期
    pub adjustment_date: Option<NaiveDate>,
    /// 调薪类型
    pub adjustment_type: Option<i32>,
    /// 调整前基本工资
    pub old_base_salary: Option<Decimal>,
    /// 调整后基本工资
    pub new_base_salary: Option<Decimal>,
    /// 调整前岗位津贴
    pub old_position_allowance: Option<Decimal>,
    /// 调整后岗位津贴
    pub new_position_allowance: Option<Decimal>,
    /// 调整前绩效基数
    pub old_performance_base: Option<Decimal>,
    /// 调整后绩效基数
    pub new_performance_base: Option<Decimal>,
    /// 调薪原因
    pub adjustment_reason: Option<String>,
    pub approver_id: Option<i64>,
    pub approver_name: Option<String>,
    pub approve_time: Option<DateTime>,
    /// 状态: 0=待审批 1=已通过 2=已驳回
    pub status: Option<i32>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
