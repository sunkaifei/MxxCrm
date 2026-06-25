use crate::core::errors::error::Result;
use crate::modules::statistics::model::contract_stats::{ContractRankingVO, ContractTypeDistributionVO, ContractStatusAnalysisVO};
use sea_orm::prelude::Decimal;
use sea_orm::DbConn;

pub async fn get_contract_ranking(db: &DbConn, _year: Option<i32>, _month: Option<i32>, order_by: Option<String>, _order_type: Option<String>, _limit: Option<i64>) -> Result<Vec<ContractRankingVO>> {
    let order_by = order_by.unwrap_or("amount".to_string());
    
    let mut result = vec![
        ContractRankingVO {
            rank: None,
            target_type: Some("customer".to_string()),
            target_id: Some(1),
            target_name: Some("ABC科技有限公司".to_string()),
            contract_count: Some(12),
            contract_amount: Some(Decimal::from(3000000)),
            payment_amount: Some(Decimal::from(2400000)),
            payment_rate: Some(Decimal::from(80)),
        },
        ContractRankingVO {
            rank: None,
            target_type: Some("customer".to_string()),
            target_id: Some(2),
            target_name: Some("DEF贸易公司".to_string()),
            contract_count: Some(8),
            contract_amount: Some(Decimal::from(2000000)),
            payment_amount: Some(Decimal::from(1600000)),
            payment_rate: Some(Decimal::from(80)),
        },
        ContractRankingVO {
            rank: None,
            target_type: Some("employee".to_string()),
            target_id: Some(1),
            target_name: Some("张三".to_string()),
            contract_count: Some(25),
            contract_amount: Some(Decimal::from(2500000)),
            payment_amount: Some(Decimal::from(2000000)),
            payment_rate: Some(Decimal::from(80)),
        },
    ];
    
    if order_by == "count" {
        result.sort_by(|a, b| b.contract_count.unwrap_or(0).cmp(&a.contract_count.unwrap_or(0)));
    } else {
        result.sort_by(|a, b| b.contract_amount.unwrap_or(Decimal::from(0)).cmp(&a.contract_amount.unwrap_or(Decimal::from(0))));
    }
    
    for (i, item) in result.iter_mut().enumerate() {
        item.rank = Some((i + 1) as i32);
    }
    
    Ok(result)
}

pub async fn get_contract_type_distribution(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<Vec<ContractTypeDistributionVO>> {
    let result = vec![
        ContractTypeDistributionVO {
            contract_type: Some("销售合同".to_string()),
            contract_count: Some(150),
            contract_amount: Some(Decimal::from(15000000)),
            percentage: Some(Decimal::from(75)),
        },
        ContractTypeDistributionVO {
            contract_type: Some("服务合同".to_string()),
            contract_count: Some(40),
            contract_amount: Some(Decimal::from(4000000)),
            percentage: Some(Decimal::from(20)),
        },
        ContractTypeDistributionVO {
            contract_type: Some("合作协议".to_string()),
            contract_count: Some(10),
            contract_amount: Some(Decimal::from(1000000)),
            percentage: Some(Decimal::from(5)),
        },
    ];
    
    Ok(result)
}

pub async fn get_contract_status_analysis(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<Vec<ContractStatusAnalysisVO>> {
    let result = vec![
        ContractStatusAnalysisVO {
            status: Some("pending".to_string()),
            status_name: Some("待审批".to_string()),
            contract_count: Some(20),
            contract_amount: Some(Decimal::from(2000000)),
            percentage: Some(Decimal::from(10)),
        },
        ContractStatusAnalysisVO {
            status: Some("approved".to_string()),
            status_name: Some("已审批".to_string()),
            contract_count: Some(80),
            contract_amount: Some(Decimal::from(8000000)),
            percentage: Some(Decimal::from(40)),
        },
        ContractStatusAnalysisVO {
            status: Some("executing".to_string()),
            status_name: Some("执行中".to_string()),
            contract_count: Some(50),
            contract_amount: Some(Decimal::from(5000000)),
            percentage: Some(Decimal::from(25)),
        },
        ContractStatusAnalysisVO {
            status: Some("completed".to_string()),
            status_name: Some("已完成".to_string()),
            contract_count: Some(40),
            contract_amount: Some(Decimal::from(4000000)),
            percentage: Some(Decimal::from(20)),
        },
        ContractStatusAnalysisVO {
            status: Some("cancelled".to_string()),
            status_name: Some("已取消".to_string()),
            contract_count: Some(10),
            contract_amount: Some(Decimal::from(1000000)),
            percentage: Some(Decimal::from(5)),
        },
    ];
    
    Ok(result)
}