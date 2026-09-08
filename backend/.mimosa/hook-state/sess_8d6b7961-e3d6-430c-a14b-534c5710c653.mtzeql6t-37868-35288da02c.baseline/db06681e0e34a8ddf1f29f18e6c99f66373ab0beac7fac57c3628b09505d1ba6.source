use sea_orm::prelude::Decimal;
use crate::core::kit::global::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomerTypeStatsVO {
    pub customer_type: Option<String>,
    pub total_count: Option<i64>,
    pub contract_count: Option<i64>,
    pub conversion_rate: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomerSourceStatsVO {
    pub source: Option<String>,
    pub total_count: Option<i64>,
    pub contract_count: Option<i64>,
    pub conversion_rate: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomerIndustryStatsVO {
    pub industry: Option<String>,
    pub total_count: Option<i64>,
    pub contract_count: Option<i64>,
    pub conversion_rate: Option<Decimal>,
    pub contract_amount: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomerFunnelVO {
    pub stage: Option<String>,
    pub count: Option<i64>,
    pub amount: Option<Decimal>,
    pub rate: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
pub struct CustomerStatsQuery {
    pub year: Option<i32>,
    pub month: Option<i32>,
    /// 自定义起始日期（YYYY-MM-DD），优先级高于 year/month
    pub start_date: Option<String>,
    /// 自定义结束日期（YYYY-MM-DD）
    pub end_date: Option<String>,
}