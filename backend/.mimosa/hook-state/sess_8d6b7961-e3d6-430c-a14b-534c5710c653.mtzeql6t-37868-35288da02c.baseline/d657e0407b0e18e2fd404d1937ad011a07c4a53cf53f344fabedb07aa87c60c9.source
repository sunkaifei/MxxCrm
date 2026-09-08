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

/// 入职定薪审批环节数据（mxx_hr_hire_salary_data）：按 instance_id + 节点记录各环节填写的结构化字段
#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_hr_hire_salary_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 审批实例ID
    pub instance_id: i64,
    /// 环节节点key
    pub node_key: Option<String>,
    /// 1部门经理 2人事 3CEO 4财务
    pub stage: i32,
    /// 建议工资（部门经理填）
    pub suggested_salary: Option<Decimal>,
    /// 试用期月数（部门经理填）
    pub probation_months: Option<i32>,
    /// 工作能力评估（部门经理填，仅审批人可见）
    pub ability_assessment: Option<String>,
    /// 带宽评估：1带宽内 2超带宽（人事填）
    pub band_status: Option<i32>,
    /// 超带宽原因（人事填）
    pub band_reason: Option<String>,
    /// 谈定工资（人事填，与候选人协商确定的月工资金额）
    pub negotiated_salary: Option<Decimal>,
    /// 试用期工资比例（人事填）
    pub probation_ratio: Option<Decimal>,
    /// CEO终审意见/特批说明（CEO填）
    pub ceo_opinion: Option<String>,
    /// 最终定薪（财务填）
    pub final_salary: Option<Decimal>,
    /// 生效日期（财务填）
    pub effective_date: Option<NaiveDate>,
    /// 填写人
    pub approver_id: Option<i64>,
    /// 审批意见
    pub comment: Option<String>,
    pub create_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
