use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::core::web::response::ResultPage;

// ============ 待办汇总 ============

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct TodoSummaryVO {
    /// 逾期跟进数（客户+线索）
    pub overdue_follow_up: i64,
    /// 今日待跟进数
    pub today_follow_up: i64,
    /// 待我审批数
    pub pending_approval: i64,
    /// 待回款提醒数（7天内到期）
    pub pending_payment: i64,
    /// 即将到期合同数（30天内）
    pub expiring_contract: i64,
    /// 停滞商机数（超过N天未更新）
    pub stagnant_opportunity: i64,
    /// 待我审批的销售计划数（当前用户为 current_approver_id 且状态为待审批）
    pub pending_plan_approval: i64,
}

// ============ 跟进待办 ============

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowUpTodoQuery {
    pub page_num: u64,
    pub page_size: u64,
    /// 类型：customer=客户, lead=线索, all=全部（默认）
    pub item_type: Option<String>,
    /// 范围：overdue=逾期, today=今日, all=全部（默认 overdue+today）
    pub range_type: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FollowUpTodoVO {
    pub id: i64,
    /// customer / lead
    pub item_type: String,
    pub name: String,
    pub owner_user_id: Option<i64>,
    pub owner_user_name: Option<String>,
    pub next_follow_at: Option<String>,
    /// 逾期天数（正数=逾期，0=今日，负数=未到期）
    pub overdue_days: i32,
}

// ============ 待回款提醒 ============

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentTodoQuery {
    pub page_num: u64,
    pub page_size: u64,
    /// 天数范围（未来N天内到期），默认7
    pub days: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PaymentTodoVO {
    pub id: i64,
    pub contract_id: Option<i64>,
    pub contract_title: Option<String>,
    pub stage_name: Option<String>,
    pub plan_amount: Option<Decimal>,
    pub received_amount: Option<Decimal>,
    pub plan_date: Option<String>,
    /// 剩余天数（负数=已逾期，0=今日到期，正数=未来N天到期）
    pub remaining_days: i32,
    pub status: Option<i32>,
}

// ============ 合同到期 ============

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractTodoQuery {
    pub page_num: u64,
    pub page_size: u64,
    /// 天数范围（未来N天内到期），默认30
    pub days: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ContractTodoVO {
    pub id: i64,
    pub contract_no: Option<String>,
    pub title: Option<String>,
    pub customer_name: Option<String>,
    pub end_date: Option<String>,
    pub amount: Option<Decimal>,
    /// 剩余天数
    pub remaining_days: i32,
    pub status: Option<i32>,
    pub assigned_to: Option<i64>,
}

// ============ 商机停滞 ============

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpportunityTodoQuery {
    pub page_num: u64,
    pub page_size: u64,
    /// 停滞天数阈值（超过N天未更新），默认30
    pub days: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OpportunityTodoVO {
    pub id: i64,
    pub title: Option<String>,
    pub customer_name: Option<String>,
    pub stage: Option<i32>,
    pub stage_name: Option<String>,
    pub expected_close_date: Option<String>,
    pub update_time: Option<String>,
    /// 停滞天数
    pub stagnant_days: i32,
    pub assigned_to: Option<i64>,
}

// ============ 审批待办查询 ============

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalTodoQuery {
    pub page_num: u64,
    pub page_size: u64,
    pub business_type: Option<String>,
    pub status: Option<i32>,
    pub business_title: Option<String>,
}

pub type ApprovalTodoVO = crate::modules::approval::model::approval::ApprovalInstanceVO;
pub type ApprovalTodoResult = ResultPage<Vec<ApprovalTodoVO>>;
