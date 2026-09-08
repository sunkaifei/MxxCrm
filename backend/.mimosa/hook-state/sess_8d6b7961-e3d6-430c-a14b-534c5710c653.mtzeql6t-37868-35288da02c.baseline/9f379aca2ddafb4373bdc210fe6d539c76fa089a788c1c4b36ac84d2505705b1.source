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
#[sea_orm(table_name = "mxx_finance_attendance_record")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub employee_id: i64,
    pub year: i32,
    pub month: i32,
    /// 应出勤天数
    pub work_days: Option<Decimal>,
    /// 实际出勤天数
    pub actual_work_days: Option<Decimal>,
    /// 迟到次数
    pub late_count: Option<i32>,
    /// 早退次数
    pub early_leave_count: Option<i32>,
    /// 旷工次数
    pub absent_count: Option<i32>,
    /// 事假天数
    pub personal_leave_days: Decimal,
    /// 病假天数
    pub sick_leave_days: Decimal,
    /// 年假天数
    pub annual_leave_days: Decimal,
    /// 工作日加班时长
    pub overtime_hours_weekday: Decimal,
    /// 周末加班时长
    pub overtime_hours_weekend: Decimal,
    /// 法定节假日加班时长
    pub overtime_hours_holiday: Decimal,
    /// 数据来源
    pub data_source: Option<i32>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
