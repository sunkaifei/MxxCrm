//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use serde::{Deserialize, Serialize};
use rust_decimal::prelude::ToPrimitive;
use crate::modules::finance::entity::{salary_record, commission_detail};

/// 工资记录DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryRecordDTO {
    pub id: i64,
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub department_name: Option<String>,
    pub year: i32,
    pub month: i32,
    pub base_salary: f64,
    pub commission_amount: f64,
    pub performance_bonus: f64,
    pub deduction_amount: f64,
    pub total_salary: f64,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

impl From<salary_record::Model> for SalaryRecordDTO {
    fn from(model: salary_record::Model) -> Self {
        Self {
            id: model.id,
            employee_id: model.employee_id,
            employee_name: model.employee_name,
            department_name: model.department_name,
            year: model.year,
            month: model.month,
            base_salary: model.base_salary.to_f64().unwrap_or_default(),
            commission_amount: model.commission_amount.to_f64().unwrap_or_default(),
            performance_bonus: model.performance_bonus.to_f64().unwrap_or_default(),
            deduction_amount: model.deduction_amount.to_f64().unwrap_or_default(),
            total_salary: model.total_salary.to_f64().unwrap_or_default(),
            status: model.status,
            remark: model.remark,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 提成明细DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionDetailDTO {
    pub id: i64,
    pub salary_record_id: i64,
    pub contract_id: Option<i64>,
    pub contract_name: Option<String>,
    pub contract_amount: Option<f64>,
    pub payment_amount: Option<f64>,
    pub commission_base: Option<f64>,
    pub commission_rate: Option<f64>,
    pub commission_amount: Option<f64>,
    pub rule_name: Option<String>,
    pub create_time: Option<String>,
}

impl From<commission_detail::Model> for CommissionDetailDTO {
    fn from(model: commission_detail::Model) -> Self {
        Self {
            id: model.id,
            salary_record_id: model.salary_record_id,
            contract_id: model.contract_id,
            contract_name: model.contract_name,
            contract_amount: model.contract_amount.map(|d| d.to_f64().unwrap_or_default()),
            payment_amount: model.payment_amount.map(|d| d.to_f64().unwrap_or_default()),
            commission_base: model.commission_base.map(|d| d.to_f64().unwrap_or_default()),
            commission_rate: model.commission_rate.map(|d| d.to_f64().unwrap_or_default()),
            commission_amount: model.commission_amount.map(|d| d.to_f64().unwrap_or_default()),
            rule_name: model.rule_name,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 工资详情（含提成明细列表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryDetailDTO {
    #[serde(flatten)]
    pub record: SalaryRecordDTO,
    pub details: Vec<CommissionDetailDTO>,
}

/// 工资查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub employee_name: Option<String>,
    pub status: Option<i32>,
}

/// 工资核算请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryCalculateDTO {
    pub year: i32,
    pub month: i32,
}

/// 工资手动调整请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryUpdateDTO {
    pub id: i64,
    pub base_salary: Option<f64>,
    pub performance_bonus: Option<f64>,
    pub deduction_amount: Option<f64>,
    pub remark: Option<String>,
    pub updated_by: Option<i64>,
}

/// 工资汇总
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SalarySummaryDTO {
    pub total_base: f64,
    pub total_commission: f64,
    pub total_bonus: f64,
    pub total_deduction: f64,
    pub total_salary: f64,
    pub count: i64,
}

/// 批量操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryBatchDTO {
    pub ids: Vec<i64>,
    pub updated_by: Option<i64>,
}
