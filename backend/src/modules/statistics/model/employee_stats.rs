use sea_orm::prelude::Decimal;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::utils::string_utils::deserialize_string_to_i64;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeCustomerCountVO {
    pub employee_id: Option<i64>,
    pub employee_name: Option<String>,
    pub department_name: Option<String>,
    pub total_customers: Option<i64>,
    pub new_customers_this_month: Option<i64>,
    pub contract_customers: Option<i64>,
    pub customer_conversion_rate: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeFollowUpVO {
    pub employee_id: Option<i64>,
    pub employee_name: Option<String>,
    pub department_name: Option<String>,
    pub total_follow_up: Option<i64>,
    pub customer_follow_up: Option<i64>,
    pub opportunity_follow_up: Option<i64>,
    pub avg_follow_interval: Option<Decimal>,
    pub customers_without_follow_30_days: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeConversionVO {
    pub employee_id: Option<i64>,
    pub employee_name: Option<String>,
    pub department_name: Option<String>,
    pub total_opportunities: Option<i64>,
    pub won_opportunities: Option<i64>,
    pub lost_opportunities: Option<i64>,
    pub opportunity_win_rate: Option<Decimal>,
    pub total_contracts: Option<i64>,
    pub contract_amount: Option<Decimal>,
    pub avg_contract_amount: Option<Decimal>,
    pub avg_sales_cycle_days: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EmployeeStatsQuery {
    #[serde(default, deserialize_with = "deserialize_string_to_i64")]
    pub department_id: Option<i64>,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub month: Option<i32>,
}