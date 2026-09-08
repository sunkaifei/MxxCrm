//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::{NaiveDate, NaiveDateTime};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 员工薪资档案（mxx_hr_employee_salary）：财务录入定薪后生成，作为月薪核算基数
#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_hr_employee_salary")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 员工ID（mxx_system_admin.id）
    pub employee_id: i64,
    /// 定薪基数
    pub base_salary: Decimal,
    /// 试用期月数
    pub probation_months: Option<i32>,
    /// 试用期工资比例（如 0.80）
    pub probation_ratio: Option<Decimal>,
    /// 生效日期
    pub effective_date: Option<NaiveDate>,
    /// 1入职定薪 2调薪
    pub source: Option<i32>,
    /// 来源审批实例ID
    pub approval_instance_id: Option<i64>,
    /// 1生效 0停用
    pub status: Option<i32>,
    pub create_by: Option<String>,
    pub create_time: Option<NaiveDateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<NaiveDateTime>,
    /// 是否删除
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
