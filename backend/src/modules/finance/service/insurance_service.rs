//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 社保公积金服务

use sea_orm::*;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

use crate::modules::finance::entity::{social_insurance_policy, employee_insurance_config};

// ==================== DTO ====================

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsurancePolicyDTO {
    pub city_code: String,
    pub city_name: String,
    pub year: i32,
    pub base_lower: f64,
    pub base_upper: f64,
    pub pension_company_rate: Option<f64>,
    pub pension_personal_rate: Option<f64>,
    pub medical_company_rate: Option<f64>,
    pub medical_personal_rate: Option<f64>,
    pub unemployment_company_rate: Option<f64>,
    pub unemployment_personal_rate: Option<f64>,
    pub workinjury_company_rate: Option<f64>,
    pub maternity_company_rate: Option<f64>,
    pub housing_fund_company_rate: Option<f64>,
    pub housing_fund_personal_rate: Option<f64>,
    pub effective_month: Option<i32>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeInsuranceConfigDTO {
    pub employee_id: i64,
    pub city_code: String,
    pub base_amount: f64,
    pub housing_fund_base: Option<f64>,
    pub housing_fund_company_rate: Option<f64>,
    pub housing_fund_personal_rate: Option<f64>,
    pub participate_pension: Option<i32>,
    pub participate_medical: Option<i32>,
    pub participate_unemployment: Option<i32>,
    pub participate_workinjury: Option<i32>,
    pub participate_maternity: Option<i32>,
    pub participate_housing_fund: Option<i32>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyInsuranceResult {
    pub social_insurance_personal: Decimal,
    pub social_insurance_company: Decimal,
    pub housing_fund_personal: Decimal,
    pub housing_fund_company: Decimal,
}

// ==================== 辅助函数 ====================

fn to_dec(v: f64) -> Decimal {
    Decimal::from_f64(v).unwrap_or_default()
}

fn opt_dec(v: Option<f64>) -> Decimal {
    Decimal::from_f64(v.unwrap_or(0.0)).unwrap_or_default()
}

// ==================== 社保政策 CRUD ====================

/// 查询城市社保政策
pub async fn get_policy_list(
    db: &DatabaseConnection,
    city_code: Option<String>,
    year: Option<i32>,
) -> Result<Vec<social_insurance_policy::Model>, String> {
    let mut stmt =
        social_insurance_policy::Entity::find().filter(social_insurance_policy::Column::Enabled.eq(1));
    if let Some(c) = city_code {
        stmt = stmt.filter(social_insurance_policy::Column::CityCode.eq(c));
    }
    if let Some(y) = year {
        stmt = stmt.filter(social_insurance_policy::Column::Year.eq(y));
    }
    stmt.order_by_desc(social_insurance_policy::Column::Year)
        .all(db)
        .await
        .map_err(|e| e.to_string())
}

/// 新增/更新政策
pub async fn upsert_policy(db: &DatabaseConnection, dto: InsurancePolicyDTO) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // 按 city_code + year 匹配
    let existing = social_insurance_policy::Entity::find()
        .filter(social_insurance_policy::Column::CityCode.eq(&dto.city_code))
        .filter(social_insurance_policy::Column::Year.eq(dto.year))
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(model) = existing {
        let mut active: social_insurance_policy::ActiveModel = model.into();
        active.city_name = Set(dto.city_name);
        active.base_lower = Set(to_dec(dto.base_lower));
        active.base_upper = Set(to_dec(dto.base_upper));
        active.pension_company_rate = Set(opt_dec(dto.pension_company_rate));
        active.pension_personal_rate = Set(opt_dec(dto.pension_personal_rate));
        active.medical_company_rate = Set(opt_dec(dto.medical_company_rate));
        active.medical_personal_rate = Set(opt_dec(dto.medical_personal_rate));
        active.unemployment_company_rate = Set(opt_dec(dto.unemployment_company_rate));
        active.unemployment_personal_rate = Set(opt_dec(dto.unemployment_personal_rate));
        active.workinjury_company_rate = Set(opt_dec(dto.workinjury_company_rate));
        active.maternity_company_rate = Set(opt_dec(dto.maternity_company_rate));
        active.housing_fund_company_rate = Set(opt_dec(dto.housing_fund_company_rate));
        active.housing_fund_personal_rate = Set(opt_dec(dto.housing_fund_personal_rate));
        active.effective_month = Set(dto.effective_month);
        let updated = active.update(&txn).await.map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(updated.id)
    } else {
        let active = social_insurance_policy::ActiveModel {
            city_code: Set(dto.city_code),
            city_name: Set(dto.city_name),
            year: Set(dto.year),
            base_lower: Set(to_dec(dto.base_lower)),
            base_upper: Set(to_dec(dto.base_upper)),
            pension_company_rate: Set(opt_dec(dto.pension_company_rate)),
            pension_personal_rate: Set(opt_dec(dto.pension_personal_rate)),
            medical_company_rate: Set(opt_dec(dto.medical_company_rate)),
            medical_personal_rate: Set(opt_dec(dto.medical_personal_rate)),
            unemployment_company_rate: Set(opt_dec(dto.unemployment_company_rate)),
            unemployment_personal_rate: Set(opt_dec(dto.unemployment_personal_rate)),
            workinjury_company_rate: Set(opt_dec(dto.workinjury_company_rate)),
            maternity_company_rate: Set(opt_dec(dto.maternity_company_rate)),
            housing_fund_company_rate: Set(opt_dec(dto.housing_fund_company_rate)),
            housing_fund_personal_rate: Set(opt_dec(dto.housing_fund_personal_rate)),
            effective_month: Set(dto.effective_month),
            enabled: Set(Some(1)),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        let inserted = active.insert(&txn).await.map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(inserted.id)
    }
}

/// 删除政策
pub async fn delete_policy(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    social_insurance_policy::Entity::delete_by_id(id)
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 员工社保配置 ====================

/// 查询员工社保配置
pub async fn get_employee_config_list(
    db: &DatabaseConnection,
    employee_id: i64,
) -> Result<Vec<employee_insurance_config::Model>, String> {
    employee_insurance_config::Entity::find()
        .filter(employee_insurance_config::Column::EmployeeId.eq(employee_id))
        .filter(employee_insurance_config::Column::Enabled.eq(1))
        .all(db)
        .await
        .map_err(|e| e.to_string())
}

/// 查询所有员工社保配置列表
pub async fn get_all_employee_configs(
    db: &DatabaseConnection,
) -> Result<Vec<employee_insurance_config::Model>, String> {
    employee_insurance_config::Entity::find()
        .filter(employee_insurance_config::Column::Enabled.eq(1))
        .order_by_desc(employee_insurance_config::Column::EmployeeId)
        .all(db)
        .await
        .map_err(|e| e.to_string())
}

/// 新增/更新员工社保配置
pub async fn upsert_employee_config(
    db: &DatabaseConnection,
    dto: EmployeeInsuranceConfigDTO,
) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // 按 employee_id 匹配
    let existing = employee_insurance_config::Entity::find()
        .filter(employee_insurance_config::Column::EmployeeId.eq(dto.employee_id))
        .filter(employee_insurance_config::Column::Enabled.eq(1))
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(model) = existing {
        let mut active: employee_insurance_config::ActiveModel = model.into();
        active.city_code = Set(dto.city_code);
        active.base_amount = Set(to_dec(dto.base_amount));
        active.housing_fund_base = Set(dto.housing_fund_base.map(to_dec));
        active.housing_fund_company_rate = Set(dto.housing_fund_company_rate.map(to_dec));
        active.housing_fund_personal_rate = Set(dto.housing_fund_personal_rate.map(to_dec));
        active.participate_pension = Set(dto.participate_pension);
        active.participate_medical = Set(dto.participate_medical);
        active.participate_unemployment = Set(dto.participate_unemployment);
        active.participate_workinjury = Set(dto.participate_workinjury);
        active.participate_maternity = Set(dto.participate_maternity);
        active.participate_housing_fund = Set(dto.participate_housing_fund);
        active.update_time = Set(Some(now));
        let updated = active.update(&txn).await.map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(updated.id)
    } else {
        let active = employee_insurance_config::ActiveModel {
            employee_id: Set(dto.employee_id),
            city_code: Set(dto.city_code),
            base_amount: Set(to_dec(dto.base_amount)),
            housing_fund_base: Set(dto.housing_fund_base.map(to_dec)),
            housing_fund_company_rate: Set(dto.housing_fund_company_rate.map(to_dec)),
            housing_fund_personal_rate: Set(dto.housing_fund_personal_rate.map(to_dec)),
            participate_pension: Set(dto.participate_pension.or(Some(1))),
            participate_medical: Set(dto.participate_medical.or(Some(1))),
            participate_unemployment: Set(dto.participate_unemployment.or(Some(1))),
            participate_workinjury: Set(dto.participate_workinjury.or(Some(1))),
            participate_maternity: Set(dto.participate_maternity.or(Some(1))),
            participate_housing_fund: Set(dto.participate_housing_fund.or(Some(1))),
            enabled: Set(Some(1)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        let inserted = active.insert(&txn).await.map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(inserted.id)
    }
}

// ==================== 社保计算 ====================

/// 计算当月社保公积金
///
/// - 根据员工配置的 city_code 和年份查政策
/// - 缴费基数 = clamp(base_amount, base_lower, base_upper)
/// - 个人社保 = 养老个人 + 医疗个人 + 失业个人
/// - 单位社保 = 养老单位 + 医疗单位 + 失业单位 + 工伤单位 + 生育单位
/// - 个人公积金 = base × housing_fund_personal_rate
/// - 单位公积金 = base × housing_fund_company_rate
/// - 根据 participate_xxx 字段决定是否参与各项
pub async fn calculate_monthly_insurance(
    db: &DatabaseConnection,
    employee_id: i64,
    year: i32,
    _month: i32,
) -> Result<MonthlyInsuranceResult, String> {
    // 查询员工社保配置
    let config = employee_insurance_config::Entity::find()
        .filter(employee_insurance_config::Column::EmployeeId.eq(employee_id))
        .filter(employee_insurance_config::Column::Enabled.eq(1))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "员工社保配置不存在".to_string())?;

    // 查询城市社保政策
    let policy = social_insurance_policy::Entity::find()
        .filter(social_insurance_policy::Column::CityCode.eq(&config.city_code))
        .filter(social_insurance_policy::Column::Year.eq(year))
        .filter(social_insurance_policy::Column::Enabled.eq(1))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            format!(
                "城市 {} 的 {} 年社保政策不存在",
                config.city_code, year
            )
        })?;

    // 缴费基数 = clamp(base_amount, base_lower, base_upper)
    let base = config
        .base_amount
        .max(policy.base_lower)
        .min(policy.base_upper);

    let mut social_personal = Decimal::ZERO;
    let mut social_company = Decimal::ZERO;

    // 养老保险
    if config.participate_pension.unwrap_or(1) == 1 {
        social_personal += base * policy.pension_personal_rate;
        social_company += base * policy.pension_company_rate;
    }
    // 医疗保险
    if config.participate_medical.unwrap_or(1) == 1 {
        social_personal += base * policy.medical_personal_rate;
        social_company += base * policy.medical_company_rate;
    }
    // 失业保险
    if config.participate_unemployment.unwrap_or(1) == 1 {
        social_personal += base * policy.unemployment_personal_rate;
        social_company += base * policy.unemployment_company_rate;
    }
    // 工伤保险（仅单位）
    if config.participate_workinjury.unwrap_or(1) == 1 {
        social_company += base * policy.workinjury_company_rate;
    }
    // 生育保险（仅单位）
    if config.participate_maternity.unwrap_or(1) == 1 {
        social_company += base * policy.maternity_company_rate;
    }

    // 住房公积金
    let mut housing_personal = Decimal::ZERO;
    let mut housing_company = Decimal::ZERO;
    if config.participate_housing_fund.unwrap_or(1) == 1 {
        // P1-4 修复：公积金基数也需应用上下限 clamp（复用社保基数上下限）
        let raw_hf_base = config.housing_fund_base.unwrap_or(base);
        let hf_base = raw_hf_base.max(policy.base_lower).min(policy.base_upper);
        let hf_company_rate = config
            .housing_fund_company_rate
            .unwrap_or(policy.housing_fund_company_rate);
        let hf_personal_rate = config
            .housing_fund_personal_rate
            .unwrap_or(policy.housing_fund_personal_rate);
        housing_personal = hf_base * hf_personal_rate;
        housing_company = hf_base * hf_company_rate;
    }

    Ok(MonthlyInsuranceResult {
        social_insurance_personal: social_personal,
        social_insurance_company: social_company,
        housing_fund_personal: housing_personal,
        housing_fund_company: housing_company,
    })
}
