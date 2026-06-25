use sea_orm::prelude::Decimal;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::statistics::entity::performance_target::{self, Entity as PerformanceTarget};
use crate::utils::string_utils::{deserialize_string_to_u64, deserialize_string_to_i64};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PerformanceTargetSaveRequest {
    pub employee_id: Option<i64>,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub contract_target_amount: Option<Decimal>,
    pub payment_target_amount: Option<Decimal>,
    pub contract_target_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PerformanceTargetBatchSaveRequest {
    pub targets: Option<Vec<PerformanceTargetSaveRequest>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PerformanceTargetQuery {
    #[serde(default, deserialize_with = "deserialize_string_to_i64")]
    pub employee_id: Option<i64>,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub month: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PerformanceTargetVO {
    pub id: Option<i64>,
    pub employee_id: Option<i64>,
    pub employee_name: Option<String>,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub contract_target_amount: Option<Decimal>,
    pub payment_target_amount: Option<Decimal>,
    pub contract_target_count: Option<i32>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

impl From<performance_target::Model> for PerformanceTargetVO {
    fn from(model: performance_target::Model) -> Self {
        PerformanceTargetVO {
            id: Some(model.id),
            employee_id: model.employee_id,
            employee_name: None,
            year: model.year,
            month: model.month,
            contract_target_amount: model.contract_target_amount,
            payment_target_amount: model.payment_target_amount,
            contract_target_count: model.contract_target_count,
            create_time: model.create_time.map(|t| t.to_string()),
            update_time: model.update_time.map(|t| t.to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MonthlyPerformanceVO {
    pub month: Option<i32>,
    pub contract_target: Option<Decimal>,
    pub payment_target: Option<Decimal>,
    pub contract_actual: Option<Decimal>,
    pub payment_actual: Option<Decimal>,
    pub contract_completion_rate: Option<Decimal>,
    pub payment_completion_rate: Option<Decimal>,
    pub contract_count: Option<i64>,
    pub payment_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MonthlyPerformanceStatsVO {
    pub year: Option<i32>,
    pub total_contract_target: Option<Decimal>,
    pub total_payment_target: Option<Decimal>,
    pub total_contract_actual: Option<Decimal>,
    pub total_payment_actual: Option<Decimal>,
    pub contract_completion_rate: Option<Decimal>,
    pub payment_completion_rate: Option<Decimal>,
    pub months: Option<Vec<MonthlyPerformanceVO>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PerformanceRankingQuery {
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub month: Option<i32>,
    #[serde(default)]
    pub order_by: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_to_i64")]
    pub department_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PerformanceRankingVO {
    pub rank: Option<i32>,
    pub employee_id: Option<i64>,
    pub employee_name: Option<String>,
    pub department_name: Option<String>,
    pub contract_amount: Option<Decimal>,
    pub contract_count: Option<i64>,
    pub payment_amount: Option<Decimal>,
    pub payment_count: Option<i64>,
    pub contract_target: Option<Decimal>,
    pub payment_target: Option<Decimal>,
    pub contract_completion_rate: Option<Decimal>,
    pub payment_completion_rate: Option<Decimal>,
}