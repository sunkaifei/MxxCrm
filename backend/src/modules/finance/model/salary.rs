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
use crate::modules::system::entity::admin;

/// 工资记录DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalaryRecordDTO {
    pub id: i64,
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub department_name: Option<String>,
    pub year: i32,
    pub month: i32,
    pub base_salary: f64,
    /// 岗位津贴
    pub position_allowance: f64,
    pub commission_amount: f64,
    pub performance_bonus: f64,
    pub deduction_amount: f64,
    pub total_salary: f64,
    /// 个人社保
    pub social_insurance_personal: f64,
    /// 个人公积金
    pub housing_fund_personal: f64,
    /// 单位社保
    pub social_insurance_company: f64,
    /// 单位公积金
    pub housing_fund_company: f64,
    /// 个税金额
    pub tax_amount: f64,
    /// 实发工资
    pub net_salary: f64,
    /// 团队提成金额
    pub team_commission_amount: f64,
    pub status: Option<i32>,
    /// 员工确认状态: 0=未确认, 1=已确认, 2=申请重新核算
    pub employee_confirmed: Option<i32>,
    pub confirmed_time: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
    /// 入职时间（用于列表展示）
    pub hire_date: Option<String>,
    /// 是否已核算（false=该年月未生成工资记录，占位行）
    pub calculated: bool,
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
            position_allowance: model.position_allowance.to_f64().unwrap_or_default(),
            commission_amount: model.commission_amount.to_f64().unwrap_or_default(),
            performance_bonus: model.performance_bonus.to_f64().unwrap_or_default(),
            deduction_amount: model.deduction_amount.to_f64().unwrap_or_default(),
            total_salary: model.total_salary.to_f64().unwrap_or_default(),
            social_insurance_personal: model.social_insurance_personal.to_f64().unwrap_or_default(),
            housing_fund_personal: model.housing_fund_personal.to_f64().unwrap_or_default(),
            social_insurance_company: model.social_insurance_company.to_f64().unwrap_or_default(),
            housing_fund_company: model.housing_fund_company.to_f64().unwrap_or_default(),
            tax_amount: model.tax_amount.to_f64().unwrap_or_default(),
            net_salary: model.net_salary.to_f64().unwrap_or_default(),
            team_commission_amount: model.team_commission_amount.to_f64().unwrap_or_default(),
            status: model.status,
            employee_confirmed: model.employee_confirmed,
            confirmed_time: model.confirmed_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            remark: model.remark,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            hire_date: None,
            calculated: true,
        }
    }
}

impl SalaryRecordDTO {
    /// 未核算占位记录：员工存在但该年月尚未生成工资记录
    pub fn placeholder(emp: &admin::Model, year: i32, month: i32) -> Self {
        Self {
            id: 0,
            employee_id: emp.id,
            employee_name: emp.nick_name.clone().or_else(|| emp.user_name.clone()),
            department_name: None,
            year,
            month,
            base_salary: 0.0,
            position_allowance: 0.0,
            commission_amount: 0.0,
            performance_bonus: 0.0,
            deduction_amount: 0.0,
            total_salary: 0.0,
            social_insurance_personal: 0.0,
            housing_fund_personal: 0.0,
            social_insurance_company: 0.0,
            housing_fund_company: 0.0,
            tax_amount: 0.0,
            net_salary: 0.0,
            team_commission_amount: 0.0,
            status: None,
            employee_confirmed: None,
            confirmed_time: None,
            remark: None,
            create_time: None,
            update_time: None,
            hire_date: emp.hire_date.map(|d| d.format("%Y-%m-%d").to_string()),
            calculated: false,
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct SalaryBatchDTO {
    pub ids: Vec<i64>,
    pub updated_by: Option<i64>,
}

// ===== P2-2: 工资历史趋势分析 =====

/// 趋势分析查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalaryTrendQuery {
    /// 起始年（含），默认当前年 - 2
    pub year_start: Option<i32>,
    /// 截止年（含），默认当前年
    pub year_end: Option<i32>,
    /// 起始月份（1-12），可选
    pub month_start: Option<i32>,
    /// 截止月份（1-12），可选
    pub month_end: Option<i32>,
    /// 部门名称（精确匹配，与 department_id 二选一）
    pub department_name: Option<String>,
    /// 员工ID（按单个员工分析）
    pub employee_id: Option<i64>,
    /// 员工姓名（模糊匹配）
    pub employee_name: Option<String>,
}

/// 月度趋势数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalaryTrendMonthlyPointDTO {
    /// 年份
    pub year: i32,
    /// 月份
    pub month: i32,
    /// 期间标签 "YYYY-MM"
    pub period: String,
    /// 发薪人头数（去重员工数）
    pub headcount: i64,
    /// 基本工资合计
    pub total_base: f64,
    /// 提成金额合计
    pub total_commission: f64,
    /// 绩效奖金合计
    pub total_performance: f64,
    /// 扣款合计
    pub total_deduction: f64,
    /// 团队提成合计
    pub total_team_commission: f64,
    /// 个税合计
    pub total_tax: f64,
    /// 应发工资合计
    pub total_gross: f64,
    /// 实发工资合计
    pub total_net: f64,
    /// 人均实发工资
    pub avg_net: f64,
}

/// 部门维度数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalaryTrendDeptPointDTO {
    /// 部门名称
    pub department_name: String,
    /// 人头数（去重员工数）
    pub headcount: i64,
    /// 基本工资合计
    pub total_base: f64,
    /// 提成金额合计
    pub total_commission: f64,
    /// 绩效奖金合计
    pub total_performance: f64,
    /// 应发工资合计
    pub total_gross: f64,
    /// 实发工资合计
    pub total_net: f64,
    /// 人均实发工资
    pub avg_net: f64,
}

/// 员工排名数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalaryTrendEmployeePointDTO {
    /// 员工ID
    pub employee_id: i64,
    /// 员工姓名
    pub employee_name: String,
    /// 部门名称
    pub department_name: Option<String>,
    /// 基本工资合计
    pub total_base: f64,
    /// 提成金额合计
    pub total_commission: f64,
    /// 绩效奖金合计
    pub total_performance: f64,
    /// 应发工资合计
    pub total_gross: f64,
    /// 实发工资合计
    pub total_net: f64,
    /// 参与月数
    pub months: i64,
    /// 月均实发工资
    pub avg_monthly_net: f64,
}

/// 趋势分析汇总
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SalaryTrendSummaryDTO {
    /// 总人头数（去重）
    pub total_headcount: i64,
    /// 工资记录总数
    pub total_records: i64,
    /// 月份数
    pub total_months: i64,
    /// 应发合计
    pub total_gross: f64,
    /// 实发合计
    pub total_net: f64,
    /// 基本工资合计
    pub total_base: f64,
    /// 提成金额合计
    pub total_commission: f64,
    /// 绩效奖金合计
    pub total_performance: f64,
    /// 团队提成合计
    pub total_team_commission: f64,
    /// 个税合计
    pub total_tax: f64,
    /// 月均实发
    pub avg_monthly_net: f64,
}
