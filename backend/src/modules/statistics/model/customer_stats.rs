use sea_orm::prelude::Decimal;
use crate::core::kit::global::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerTypeStatsVO {
    pub customer_type: Option<String>,
    pub total_count: Option<i64>,
    pub contract_count: Option<i64>,
    pub conversion_rate: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerSourceStatsVO {
    pub source: Option<String>,
    pub total_count: Option<i64>,
    pub contract_count: Option<i64>,
    pub conversion_rate: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerIndustryStatsVO {
    pub industry: Option<String>,
    pub total_count: Option<i64>,
    pub contract_count: Option<i64>,
    pub conversion_rate: Option<Decimal>,
    pub contract_amount: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerFunnelVO {
    pub stage: Option<String>,
    pub count: Option<i64>,
    pub amount: Option<Decimal>,
    pub rate: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerFunnelStatsVO {
    pub total_leads: Option<i64>,
    pub total_customers: Option<i64>,
    pub total_opportunities: Option<i64>,
    pub total_contracts: Option<i64>,
    pub lead_to_customer_rate: Option<Decimal>,
    pub customer_to_opportunity_rate: Option<Decimal>,
    pub opportunity_to_contract_rate: Option<Decimal>,
    pub overall_conversion_rate: Option<Decimal>,
    pub funnel: Option<Vec<CustomerFunnelVO>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerStatsQuery {
    pub year: Option<i32>,
    pub month: Option<i32>,
}