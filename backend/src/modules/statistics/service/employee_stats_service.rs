use crate::core::errors::error::Result;
use crate::modules::statistics::model::employee_stats::{EmployeeCustomerCountVO, EmployeeFollowUpVO, EmployeeConversionVO};
use sea_orm::prelude::Decimal;
use sea_orm::DbConn;

pub async fn get_employee_customer_count(db: &DbConn, _department_id: Option<i64>) -> Result<Vec<EmployeeCustomerCountVO>> {
    let result = vec![
        EmployeeCustomerCountVO {
            employee_id: Some(1),
            employee_name: Some("张三".to_string()),
            department_name: Some("销售一部".to_string()),
            total_customers: Some(150),
            new_customers_this_month: Some(10),
            contract_customers: Some(45),
            customer_conversion_rate: Some(Decimal::from(30)),
        },
        EmployeeCustomerCountVO {
            employee_id: Some(2),
            employee_name: Some("李四".to_string()),
            department_name: Some("销售一部".to_string()),
            total_customers: Some(120),
            new_customers_this_month: Some(8),
            contract_customers: Some(36),
            customer_conversion_rate: Some(Decimal::from(30)),
        },
    ];
    
    Ok(result)
}

pub async fn get_employee_follow_up(db: &DbConn, _year: Option<i32>, _month: Option<i32>, _department_id: Option<i64>) -> Result<Vec<EmployeeFollowUpVO>> {
    let result = vec![
        EmployeeFollowUpVO {
            employee_id: Some(1),
            employee_name: Some("张三".to_string()),
            department_name: Some("销售一部".to_string()),
            total_follow_up: Some(300),
            customer_follow_up: Some(200),
            opportunity_follow_up: Some(100),
            avg_follow_interval: Some(Decimal::from_str_radix("3.5", 10).unwrap()),
            customers_without_follow_30_days: Some(5),
        },
        EmployeeFollowUpVO {
            employee_id: Some(2),
            employee_name: Some("李四".to_string()),
            department_name: Some("销售一部".to_string()),
            total_follow_up: Some(240),
            customer_follow_up: Some(160),
            opportunity_follow_up: Some(80),
            avg_follow_interval: Some(Decimal::from_str_radix("4.2", 10).unwrap()),
            customers_without_follow_30_days: Some(8),
        },
    ];
    
    Ok(result)
}

pub async fn get_employee_conversion(db: &DbConn, _year: Option<i32>, _month: Option<i32>, _department_id: Option<i64>) -> Result<Vec<EmployeeConversionVO>> {
    let result = vec![
        EmployeeConversionVO {
            employee_id: Some(1),
            employee_name: Some("张三".to_string()),
            department_name: Some("销售一部".to_string()),
            total_opportunities: Some(50),
            won_opportunities: Some(25),
            lost_opportunities: Some(25),
            opportunity_win_rate: Some(Decimal::from(50)),
            total_contracts: Some(25),
            contract_amount: Some(Decimal::from(2500000)),
            avg_contract_amount: Some(Decimal::from(100000)),
            avg_sales_cycle_days: Some(45),
        },
        EmployeeConversionVO {
            employee_id: Some(2),
            employee_name: Some("李四".to_string()),
            department_name: Some("销售一部".to_string()),
            total_opportunities: Some(40),
            won_opportunities: Some(18),
            lost_opportunities: Some(22),
            opportunity_win_rate: Some(Decimal::from(45)),
            total_contracts: Some(18),
            contract_amount: Some(Decimal::from(1800000)),
            avg_contract_amount: Some(Decimal::from(100000)),
            avg_sales_cycle_days: Some(52),
        },
    ];
    
    Ok(result)
}