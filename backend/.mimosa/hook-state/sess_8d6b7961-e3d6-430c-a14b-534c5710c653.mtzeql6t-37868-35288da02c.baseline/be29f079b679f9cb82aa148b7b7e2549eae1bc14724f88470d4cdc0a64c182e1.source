//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 工资核算日志表
//! 记录每次核算（手动/自动）的执行情况，便于排查问题和审计

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_salary_calc_log")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 核算年份
    pub year: i32,
    /// 核算月份
    pub month: i32,
    /// 触发方式：0=手动，1=定时任务自动
    pub trigger_type: Option<i32>,
    /// 核算结果：0=失败，1=成功
    pub result: Option<i32>,
    /// 生成工资记录数
    pub generated_count: Option<i64>,
    /// 失败原因（result=0 时填写）
    pub error_message: Option<String>,
    /// 执行耗时（毫秒）
    pub elapsed_ms: Option<i64>,
    /// 操作人 ID（自动核算时为 0）
    pub operator_id: Option<i64>,
    /// 操作人姓名
    pub operator_name: Option<String>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
