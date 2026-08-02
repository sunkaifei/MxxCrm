//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 员工底薪配置表
//! 存储员工每月的底薪、绩效系数、岗位津贴等基础薪酬配置
//! 支持按年月配置（年度调薪场景），未配置的月份回退到最近一次配置

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_salary_config")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 员工 ID
    pub employee_id: i64,
    /// 生效年份
    pub year: i32,
    /// 生效月份（1-12，null 表示全年通用）
    pub month: Option<i32>,
    /// 底薪金额
    pub base_salary: Decimal,
    /// 岗位津贴
    pub position_allowance: Option<Decimal>,
    /// 绩效基数（绩效奖金 = 绩效基数 × 绩效系数）
    pub performance_base: Option<Decimal>,
    /// 绩效系数（0.0-2.0，由业绩完成率决定，null 时按业绩计划完成率自动计算）
    pub performance_coefficient: Option<Decimal>,
    /// 状态：0=禁用，1=启用
    pub status: Option<i32>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
