use sea_orm::prelude::Decimal;
use crate::core::kit::global::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PaymentCompletionVO {
    pub year: Option<i32>,
    pub total_contract_amount: Option<Decimal>,
    pub total_payment_amount: Option<Decimal>,
    pub completion_rate: Option<Decimal>,
    pub overdue_amount: Option<Decimal>,
    pub overdue_rate: Option<Decimal>,
    pub unpaid_amount: Option<Decimal>,
    pub unpaid_rate: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PaymentMonthlyTrendVO {
    pub month: Option<i32>,
    pub contract_amount: Option<Decimal>,
    pub payment_amount: Option<Decimal>,
    pub completion_rate: Option<Decimal>,
    pub overdue_amount: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PaymentMonthlyTrendStatsVO {
    pub year: Option<i32>,
    pub months: Option<Vec<PaymentMonthlyTrendVO>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PaymentStatusAnalysisVO {
    pub status: Option<String>,
    pub status_name: Option<String>,
    pub contract_count: Option<i64>,
    pub contract_amount: Option<Decimal>,
    pub paid_amount: Option<Decimal>,
    pub percentage: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PaymentRankingVO {
    pub rank: Option<i32>,
    pub target_type: Option<String>,
    pub target_id: Option<i64>,
    pub target_name: Option<String>,
    pub contract_amount: Option<Decimal>,
    pub payment_amount: Option<Decimal>,
    pub completion_rate: Option<Decimal>,
    pub overdue_amount: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PaymentStatsQuery {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub order_by: Option<String>,
    pub limit: Option<i64>,
}