use crate::core::errors::error::Result;
use crate::modules::statistics::model::customer_stats::{CustomerTypeStatsVO, CustomerSourceStatsVO, CustomerIndustryStatsVO, CustomerFunnelStatsVO, CustomerFunnelVO};
use sea_orm::prelude::Decimal;
use sea_orm::DbConn;

pub async fn get_customer_type_stats(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<Vec<CustomerTypeStatsVO>> {
    let result = vec![
        CustomerTypeStatsVO {
            customer_type: Some("潜在客户".to_string()),
            total_count: Some(500),
            contract_count: Some(80),
            conversion_rate: Some(Decimal::from(16)),
        },
        CustomerTypeStatsVO {
            customer_type: Some("普通客户".to_string()),
            total_count: Some(300),
            contract_count: Some(120),
            conversion_rate: Some(Decimal::from(40)),
        },
        CustomerTypeStatsVO {
            customer_type: Some("VIP客户".to_string()),
            total_count: Some(100),
            contract_count: Some(60),
            conversion_rate: Some(Decimal::from(60)),
        },
        CustomerTypeStatsVO {
            customer_type: Some("SVIP客户".to_string()),
            total_count: Some(50),
            contract_count: Some(40),
            conversion_rate: Some(Decimal::from(80)),
        },
    ];
    
    Ok(result)
}

pub async fn get_customer_source_stats(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<Vec<CustomerSourceStatsVO>> {
    let result = vec![
        CustomerSourceStatsVO {
            source: Some("展会".to_string()),
            total_count: Some(200),
            contract_count: Some(60),
            conversion_rate: Some(Decimal::from(30)),
        },
        CustomerSourceStatsVO {
            source: Some("线上广告".to_string()),
            total_count: Some(300),
            contract_count: Some(45),
            conversion_rate: Some(Decimal::from(15)),
        },
        CustomerSourceStatsVO {
            source: Some("老客户推荐".to_string()),
            total_count: Some(150),
            contract_count: Some(75),
            conversion_rate: Some(Decimal::from(50)),
        },
        CustomerSourceStatsVO {
            source: Some("官网".to_string()),
            total_count: Some(100),
            contract_count: Some(20),
            conversion_rate: Some(Decimal::from(20)),
        },
    ];
    
    Ok(result)
}

pub async fn get_customer_industry_stats(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<Vec<CustomerIndustryStatsVO>> {
    let result = vec![
        CustomerIndustryStatsVO {
            industry: Some("IT/互联网".to_string()),
            total_count: Some(250),
            contract_count: Some(100),
            conversion_rate: Some(Decimal::from(40)),
            contract_amount: Some(Decimal::from(5000000)),
        },
        CustomerIndustryStatsVO {
            industry: Some("制造业".to_string()),
            total_count: Some(200),
            contract_count: Some(80),
            conversion_rate: Some(Decimal::from(40)),
            contract_amount: Some(Decimal::from(4000000)),
        },
        CustomerIndustryStatsVO {
            industry: Some("零售业".to_string()),
            total_count: Some(150),
            contract_count: Some(45),
            conversion_rate: Some(Decimal::from(30)),
            contract_amount: Some(Decimal::from(2250000)),
        },
    ];
    
    Ok(result)
}

pub async fn get_customer_funnel(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<CustomerFunnelStatsVO> {
    let funnel = vec![
        CustomerFunnelVO {
            stage: Some("线索".to_string()),
            count: Some(1000),
            amount: None,
            rate: Some(Decimal::from(100)),
        },
        CustomerFunnelVO {
            stage: Some("客户".to_string()),
            count: Some(500),
            amount: None,
            rate: Some(Decimal::from(50)),
        },
        CustomerFunnelVO {
            stage: Some("商机".to_string()),
            count: Some(300),
            amount: Some(Decimal::from(15000000)),
            rate: Some(Decimal::from(30)),
        },
        CustomerFunnelVO {
            stage: Some("合同".to_string()),
            count: Some(150),
            amount: Some(Decimal::from(7500000)),
            rate: Some(Decimal::from(15)),
        },
    ];
    
    Ok(CustomerFunnelStatsVO {
        total_leads: Some(1000),
        total_customers: Some(500),
        total_opportunities: Some(300),
        total_contracts: Some(150),
        lead_to_customer_rate: Some(Decimal::from(50)),
        customer_to_opportunity_rate: Some(Decimal::from(60)),
        opportunity_to_contract_rate: Some(Decimal::from(50)),
        overall_conversion_rate: Some(Decimal::from(15)),
        funnel: Some(funnel),
    })
}