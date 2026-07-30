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

/// 更新月度目标请求（草稿/驳回状态直接更新，不走审批流）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct UpdatePlanTargetsRequest {
    pub plan_id: i64,
    pub monthly_targets: Vec<MonthlyTargetInput>,
}

/// 查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PlanQuery {
    pub employee_id: Option<i64>,
    pub year: Option<i32>,
    pub status: Option<i32>,
    /// 待我审批模式：true=查询当前登录用户作为 current_approver_id 的待审计划
    #[serde(default)]
    pub pending_my_approval: Option<bool>,
}

// ---- Response VOs ----

/// 月度目标VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyTargetVO {
    pub month: Option<i32>,
    pub contract_target_amount: Option<Decimal>,
    pub payment_target_amount: Option<Decimal>,
    pub contract_target_count: Option<i32>,
}

/// 审批记录VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
    /// 当前审批人 ID
    pub current_approver_id: Option<i64>,
    /// 当前审批人姓名
    pub current_approver_name: Option<String>,
    /// 当前审批层级
    pub approval_level: Option<i32>,
    /// 总审批层级数
    pub total_levels: Option<i32>,
    /// 提交时间
    pub submit_time: Option<String>,
    /// 是否已冻结
    pub is_frozen: Option<i32>,
    /// 审批节点链
    pub approval_nodes: Option<Vec<ApprovalNodeVO>>,
}

/// 审批节点VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalNodeVO {
    pub id: Option<i64>,
    pub level: Option<i32>,
    pub approver_id: Option<i64>,
    pub approver_name: Option<String>,
    pub status: Option<i32>,
    pub comment: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

/// 计划列表VO（摘要信息）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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
    /// 当前审批人 ID
    pub current_approver_id: Option<i64>,
    /// 当前审批人姓名
    pub current_approver_name: Option<String>,
    /// 当前审批层级
    pub approval_level: Option<i32>,
    /// 总审批层级数
    pub total_levels: Option<i32>,
    /// 提交时间
    pub submit_time: Option<String>,
    /// 是否已冻结
    pub is_frozen: Option<i32>,
}

/// 修改申请详情VO（用于编辑时回显）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlanModifyDetailVO {
    pub plan: Option<PlanListVO>,
    pub monthly_targets: Option<Vec<MonthlyTargetVO>>,
}

// ---- 进度汇总 ----

/// 单层进度项
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProgressItemVO {
    pub target_amount: Option<Decimal>,
    pub actual_amount: Option<Decimal>,
    pub completion_rate: Option<Decimal>,
    pub member_count: Option<i32>,
    pub approved_count: Option<i32>,
}

/// 进度汇总VO（个人 + 团队）
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlanProgressSummaryVO {
    pub personal: ProgressItemVO,
    /// 团队汇总（无下属时各字段为 0）
    pub team: ProgressItemVO,
}