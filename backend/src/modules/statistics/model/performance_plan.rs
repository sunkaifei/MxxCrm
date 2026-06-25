//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::prelude::Decimal;
use crate::core::kit::global::{Deserialize, Serialize};

// ---- Request DTOs ----

/// 创建计划请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CreatePlanRequest {
    pub year: i32,
    pub monthly_targets: Vec<MonthlyTargetInput>,
}

/// 月度目标输入
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MonthlyTargetInput {
    pub month: i32,
    pub contract_target_amount: Option<Decimal>,
    pub payment_target_amount: Option<Decimal>,
    pub contract_target_count: Option<i32>,
}

/// 提交计划请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SubmitPlanRequest {
    pub plan_id: i64,
}

/// 审批请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ReviewPlanRequest {
    pub plan_id: i64,
    pub reason: Option<String>,
}

/// 修改申请请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ModifyPlanRequest {
    pub plan_id: i64,
    pub reason: String,
    pub monthly_targets: Vec<MonthlyTargetInput>,
}

/// 查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PlanQuery {
    pub employee_id: Option<i64>,
    pub year: Option<i32>,
    pub status: Option<i32>,
}

// ---- Response VOs ----

/// 月度目标VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MonthlyTargetVO {
    pub month: Option<i32>,
    pub contract_target_amount: Option<Decimal>,
    pub payment_target_amount: Option<Decimal>,
    pub contract_target_count: Option<i32>,
}

/// 审批记录VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ApprovalLogVO {
    pub id: Option<i64>,
    pub action: Option<i32>,
    pub operator_id: Option<i64>,
    pub operator_name: Option<String>,
    pub reason: Option<String>,
    pub previous_status: Option<i32>,
    pub new_status: Option<i32>,
    pub create_time: Option<String>,
}

/// 计划详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PlanDetailVO {
    pub id: Option<i64>,
    pub employee_id: Option<i64>,
    pub employee_name: Option<String>,
    pub year: Option<i32>,
    pub status: Option<i32>,
    pub apply_reason: Option<String>,
    pub version: Option<i32>,
    pub monthly_targets: Option<Vec<MonthlyTargetVO>>,
    pub approval_logs: Option<Vec<ApprovalLogVO>>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

/// 计划列表VO（摘要信息）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PlanListVO {
    pub id: Option<i64>,
    pub employee_id: Option<i64>,
    pub employee_name: Option<String>,
    pub year: Option<i32>,
    pub status: Option<i32>,
    pub version: Option<i32>,
    pub total_contract_target: Option<Decimal>,
    pub total_payment_target: Option<Decimal>,
    pub apply_reason: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

/// 修改申请详情VO（用于编辑时回显）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PlanModifyDetailVO {
    pub plan: Option<PlanListVO>,
    pub monthly_targets: Option<Vec<MonthlyTargetVO>>,
}