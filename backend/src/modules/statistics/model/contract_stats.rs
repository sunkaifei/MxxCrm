use sea_orm::prelude::Decimal;
use crate::core::kit::global::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContractRankingVO {
    pub rank: Option<i32>,
    pub target_type: Option<String>,
    pub target_id: Option<i64>,
    pub target_name: Option<String>,
    pub contract_count: Option<i64>,
    pub contract_amount: Option<Decimal>,
    pub payment_amount: Option<Decimal>,
    pub payment_rate: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContractTypeDistributionVO {
    pub contract_type: Option<String>,
    pub contract_count: Option<i64>,
    pub contract_amount: Option<Decimal>,
    pub percentage: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContractStatusAnalysisVO {
    pub status: Option<String>,
    pub status_name: Option<String>,
    pub contract_count: Option<i64>,
    pub contract_amount: Option<Decimal>,
    pub percentage: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContractStatsQuery {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub order_by: Option<String>,
    pub order_type: Option<String>,
    pub limit: Option<i64>,
}