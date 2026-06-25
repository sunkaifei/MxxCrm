use chrono::Datelike;
use crate::core::errors::error::Result;
use crate::modules::statistics::model::payment_stats::{PaymentCompletionVO, PaymentMonthlyTrendStatsVO, PaymentMonthlyTrendVO, PaymentStatusAnalysisVO, PaymentRankingVO};
use sea_orm::prelude::Decimal;
use sea_orm::DbConn;

pub async fn get_payment_completion(db: &DbConn, year: Option<i32>, _month: Option<i32>) -> Result<PaymentCompletionVO> {
    let year = year.unwrap_or(chrono::Local::now().year() as i32);
    
    Ok(PaymentCompletionVO {
        year: Some(year),
        total_contract_amount: Some(Decimal::from(20000000)),
        total_payment_amount: Some(Decimal::from(16000000)),
        completion_rate: Some(Decimal::from(80)),
        overdue_amount: Some(Decimal::from(2000000)),
        overdue_rate: Some(Decimal::from(10)),
        unpaid_amount: Some(Decimal::from(4000000)),
        unpaid_rate: Some(Decimal::from(20)),
    })
}

pub async fn get_payment_monthly_trend(db: &DbConn, year: Option<i32>) -> Result<PaymentMonthlyTrendStatsVO> {
    let year = year.unwrap_or(chrono::Local::now().year() as i32);
    
    let months = vec![
        PaymentMonthlyTrendVO {
            month: Some(1),
            contract_amount: Some(Decimal::from(1500000)),
            payment_amount: Some(Decimal::from(1200000)),
            completion_rate: Some(Decimal::from(80)),
            overdue_amount: Some(Decimal::from(150000)),
        },
        PaymentMonthlyTrendVO {
            month: Some(2),
            contract_amount: Some(Decimal::from(1800000)),
            payment_amount: Some(Decimal::from(1620000)),
            completion_rate: Some(Decimal::from(90)),
            overdue_amount: Some(Decimal::from(90000)),
        },
        PaymentMonthlyTrendVO {
            month: Some(3),
            contract_amount: Some(Decimal::from(2000000)),
            payment_amount: Some(Decimal::from(1600000)),
            completion_rate: Some(Decimal::from(80)),
            overdue_amount: Some(Decimal::from(200000)),
        },
    ];
    
    Ok(PaymentMonthlyTrendStatsVO {
        year: Some(year),
        months: Some(months),
    })
}

pub async fn get_payment_status_analysis(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<Vec<PaymentStatusAnalysisVO>> {
    let result = vec![
        PaymentStatusAnalysisVO {
            status: Some("unpaid".to_string()),
            status_name: Some("未回款".to_string()),
            contract_count: Some(30),
            contract_amount: Some(Decimal::from(3000000)),
            paid_amount: None,
            percentage: Some(Decimal::from(15)),
        },
        PaymentStatusAnalysisVO {
            status: Some("partial".to_string()),
            status_name: Some("部分回款".to_string()),
            contract_count: Some(20),
            contract_amount: Some(Decimal::from(2000000)),
            paid_amount: Some(Decimal::from(1000000)),
            percentage: Some(Decimal::from(10)),
        },
        PaymentStatusAnalysisVO {
            status: Some("paid".to_string()),
            status_name: Some("已回款".to_string()),
            contract_count: Some(120),
            contract_amount: Some(Decimal::from(12000000)),
            paid_amount: Some(Decimal::from(12000000)),
            percentage: Some(Decimal::from(60)),
        },
        PaymentStatusAnalysisVO {
            status: Some("overdue".to_string()),
            status_name: Some("逾期".to_string()),
            contract_count: Some(30),
            contract_amount: Some(Decimal::from(3000000)),
            paid_amount: Some(Decimal::from(1000000)),
            percentage: Some(Decimal::from(15)),
        },
    ];
    
    Ok(result)
}

pub async fn get_payment_ranking(db: &DbConn, _year: Option<i32>, _month: Option<i32>, order_by: Option<String>, _limit: Option<i64>) -> Result<Vec<PaymentRankingVO>> {
    let order_by = order_by.unwrap_or("payment_amount".to_string());
    
    let mut result = vec![
        PaymentRankingVO {
            rank: None,
            target_type: Some("customer".to_string()),
            target_id: Some(1),
            target_name: Some("ABC科技有限公司".to_string()),
            contract_amount: Some(Decimal::from(3000000)),
            payment_amount: Some(Decimal::from(2700000)),
            completion_rate: Some(Decimal::from(90)),
            overdue_amount: Some(Decimal::from(0)),
        },
        PaymentRankingVO {
            rank: None,
            target_type: Some("customer".to_string()),
            target_id: Some(2),
            target_name: Some("DEF贸易公司".to_string()),
            contract_amount: Some(Decimal::from(2000000)),
            payment_amount: Some(Decimal::from(1600000)),
            completion_rate: Some(Decimal::from(80)),
            overdue_amount: Some(Decimal::from(200000)),
        },
        PaymentRankingVO {
            rank: None,
            target_type: Some("employee".to_string()),
            target_id: Some(1),
            target_name: Some("张三".to_string()),
            contract_amount: Some(Decimal::from(2500000)),
            payment_amount: Some(Decimal::from(2250000)),
            completion_rate: Some(Decimal::from(90)),
            overdue_amount: Some(Decimal::from(50000)),
        },
    ];
    
    if order_by == "completion_rate" {
        result.sort_by(|a, b| b.completion_rate.unwrap_or(Decimal::from(0)).cmp(&a.completion_rate.unwrap_or(Decimal::from(0))));
    } else {
        result.sort_by(|a, b| b.payment_amount.unwrap_or(Decimal::from(0)).cmp(&a.payment_amount.unwrap_or(Decimal::from(0))));
    }
    
    for (i, item) in result.iter_mut().enumerate() {
        item.rank = Some((i + 1) as i32);
    }
    
    Ok(result)
}