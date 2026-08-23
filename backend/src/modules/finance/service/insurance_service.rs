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
//!
//! v2：支持政策多档次（最低档/最高档/自定义档），每档独立基数、各险种比例与重大保险固定金额；
//! 员工配置通过 policy_id + policy_level_id 关联具体政策档次。

use sea_orm::*;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::RoundingStrategy;

use crate::modules::finance::entity::{
    social_insurance_policy, insurance_policy_level, employee_insurance_config,
};

// ==================== DTO ====================

/// 政策档次 DTO
#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct InsurancePolicyLevelDTO {
    pub id: Option<i64>,
    pub level_type: Option<i32>,
    pub level_name: Option<String>,
    pub base_amount: f64,
    pub base_lower: Option<f64>,
    pub base_upper: Option<f64>,
    pub pension_company_rate: Option<f64>,
    pub pension_personal_rate: Option<f64>,
    pub medical_company_rate: Option<f64>,
    pub medical_personal_rate: Option<f64>,
    pub unemployment_company_rate: Option<f64>,
    pub unemployment_personal_rate: Option<f64>,
    pub workinjury_company_rate: Option<f64>,
    pub workinjury_personal_rate: Option<f64>,
    pub maternity_company_rate: Option<f64>,
    pub maternity_personal_rate: Option<f64>,
    pub housing_fund_company_rate: Option<f64>,
    pub housing_fund_personal_rate: Option<f64>,
    pub critical_illness_company_amount: Option<f64>,
    pub critical_illness_personal_amount: Option<f64>,
}

/// 政策（表头 + 档次明细）DTO
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsurancePolicyDTO {
    pub id: Option<i64>,
    pub city_code: String,
    pub city_name: String,
    pub year: i32,
    pub base_lower: Option<f64>,
    pub base_upper: Option<f64>,
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
    pub effective_date: Option<String>,
    pub expiry_date: Option<String>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    /// 档次明细列表
    pub levels: Option<Vec<InsurancePolicyLevelDTO>>,
}

/// 员工社保配置 DTO
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeInsuranceConfigDTO {
    pub id: Option<i64>,
    pub employee_id: i64,
    pub city_code: String,
    pub policy_id: Option<i64>,
    pub policy_level_id: Option<i64>,
    pub use_policy_base: Option<bool>,
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
    pub participate_critical_illness: Option<i32>,
    pub workinjury_company_rate: Option<f64>,
    pub workinjury_personal_rate: Option<f64>,
}

/// 员工社保配置 + 关联政策/档次信息（列表展示用）
#[derive(serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeConfigWithPolicy {
    pub id: i64,
    pub employee_id: i64,
    pub employee_name: String,
    pub city_code: String,
    pub city_name: String,
    pub policy_id: Option<i64>,
    pub policy_level_id: Option<i64>,
    pub policy_year: Option<i32>,
    pub policy_effective_date: Option<String>,
    pub policy_expiry_date: Option<String>,
    pub level_name: Option<String>,
    pub level_type: Option<i32>,
    pub use_policy_base: Option<bool>,
    pub base_amount: Decimal,
    pub housing_fund_base: Option<Decimal>,
    pub housing_fund_company_rate: Option<Decimal>,
    pub housing_fund_personal_rate: Option<Decimal>,
    pub participate_pension: Option<i32>,
    pub participate_medical: Option<i32>,
    pub participate_unemployment: Option<i32>,
    pub participate_workinjury: Option<i32>,
    pub participate_maternity: Option<i32>,
    pub participate_housing_fund: Option<i32>,
    pub participate_critical_illness: Option<i32>,
    pub workinjury_company_rate: Option<Decimal>,
    pub workinjury_personal_rate: Option<Decimal>,
    pub effective_date: Option<String>,
    pub expiry_date: Option<String>,
    pub enabled: Option<i32>,
}

/// 政策 + 档次列表（列表页展示用）
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyWithLevels {
    pub policy: social_insurance_policy::Model,
    pub levels: Vec<insurance_policy_level::Model>,
}

/// 工资核算结果（含重大保险）
#[derive(serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyInsuranceResult {
    /// 个人社保合计（含重大保险）
    pub social_insurance_personal: Decimal,
    /// 单位社保合计（含重大保险）
    pub social_insurance_company: Decimal,
    /// 个人公积金
    pub housing_fund_personal: Decimal,
    /// 单位公积金
    pub housing_fund_company: Decimal,
    /// 重大保险个人固定金额
    pub critical_illness_personal: Decimal,
    /// 重大保险单位固定金额
    pub critical_illness_company: Decimal,
}

/// 险种明细项
#[derive(serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PremiumItem {
    pub company: Decimal,
    pub personal: Decimal,
    pub subtotal: Decimal,
}

/// 实时预览计算结果（前端弹窗实时展示）
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PremiumPreviewResult {
    /// 实际使用的缴费基数（clamp 后）
    pub base_amount: Decimal,
    pub pension: PremiumItem,
    pub medical: PremiumItem,
    pub unemployment: PremiumItem,
    pub workinjury: PremiumItem,
    pub maternity: PremiumItem,
    pub critical_illness: PremiumItem,
    pub housing_fund: PremiumItem,
    pub company_total: Decimal,
    pub personal_total: Decimal,
    pub grand_total: Decimal,
}

/// 预览计算请求
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewCalcDTO {
    pub policy_id: Option<i64>,
    pub level_id: Option<i64>,
    /// 自定义缴费基数（use_policy_base=false 时使用）
    pub base_amount: Option<f64>,
    pub use_policy_base: Option<bool>,
    pub housing_fund_base: Option<f64>,
    pub housing_fund_company_rate: Option<f64>,
    pub housing_fund_personal_rate: Option<f64>,
    pub participate_pension: Option<i32>,
    pub participate_medical: Option<i32>,
    pub participate_unemployment: Option<i32>,
    pub participate_workinjury: Option<i32>,
    pub participate_maternity: Option<i32>,
    pub participate_housing_fund: Option<i32>,
    pub participate_critical_illness: Option<i32>,
    pub workinjury_company_rate: Option<f64>,
    pub workinjury_personal_rate: Option<f64>,
}

// ==================== 辅助函数 ====================

fn to_dec(v: f64) -> Decimal {
    Decimal::from_f64(v).unwrap_or_default()
}

fn opt_dec(v: Option<f64>) -> Decimal {
    Decimal::from_f64(v.unwrap_or(0.0)).unwrap_or_default()
}

/// 金额四舍五入到分
fn round_money(v: Decimal) -> Decimal {
    v.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

fn opt_naive_date(s: &Option<String>) -> Option<chrono::NaiveDate> {
    s.as_deref().and_then(|v| {
        let t = v.trim();
        if t.is_empty() {
            None
        } else {
            chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d").ok()
        }
    })
}

fn date_to_string(d: Option<chrono::NaiveDate>) -> Option<String> {
    d.map(|v| v.format("%Y-%m-%d").to_string())
}

// ==================== 社保政策 CRUD ====================

/// 查询城市社保政策（含档次列表）
pub async fn get_policy_list(
    db: &DatabaseConnection,
    city_code: Option<String>,
    year: Option<i32>,
) -> Result<Vec<PolicyWithLevels>, String> {
    let mut stmt =
        social_insurance_policy::Entity::find().filter(social_insurance_policy::Column::Enabled.eq(1));
    if let Some(c) = city_code {
        stmt = stmt.filter(social_insurance_policy::Column::CityCode.eq(c));
    }
    if let Some(y) = year {
        stmt = stmt.filter(social_insurance_policy::Column::Year.eq(y));
    }
    let policies = stmt
        .order_by_desc(social_insurance_policy::Column::Year)
        .order_by_asc(social_insurance_policy::Column::Id)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut result = Vec::with_capacity(policies.len());
    for p in policies {
        let levels = insurance_policy_level::Entity::find()
            .filter(insurance_policy_level::Column::PolicyId.eq(p.id))
            .order_by_asc(insurance_policy_level::Column::LevelType)
            .order_by_asc(insurance_policy_level::Column::Id)
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        result.push(PolicyWithLevels { policy: p, levels });
    }
    Ok(result)
}

/// 新增/更新政策（表头 + 档次明细，事务）
pub async fn upsert_policy(db: &DatabaseConnection, dto: InsurancePolicyDTO) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let effective_date = opt_naive_date(&dto.effective_date);
    let expiry_date = opt_naive_date(&dto.expiry_date);

    // 政策头 upsert：有 id 则更新，否则按 city_code+year+生效日期匹配，再否则插入
    let policy_id = if let Some(pid) = dto.id {
        let existing = social_insurance_policy::Entity::find_by_id(pid)
            .one(&txn)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("政策不存在: {}", pid))?;
        let mut active: social_insurance_policy::ActiveModel = existing.into();
        active.city_name = Set(dto.city_name);
        active.year = Set(dto.year);
        active.base_lower = Set(to_dec(dto.base_lower.unwrap_or(0.0)));
        active.base_upper = Set(to_dec(dto.base_upper.unwrap_or(0.0)));
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
        active.effective_date = Set(effective_date);
        active.expiry_date = Set(expiry_date);
        if let Some(st) = dto.status {
            active.status = Set(Some(st as i16));
        }
        active.remark = Set(dto.remark);
        let updated = active.update(&txn).await.map_err(|e| e.to_string())?;
        updated.id
    } else {
        // 尝试按 city_code+year+生效日期匹配
        let existing = social_insurance_policy::Entity::find()
            .filter(social_insurance_policy::Column::CityCode.eq(&dto.city_code))
            .filter(social_insurance_policy::Column::Year.eq(dto.year))
            .filter(social_insurance_policy::Column::EffectiveDate.eq(effective_date))
            .one(&txn)
            .await
            .map_err(|e| e.to_string())?;
        if let Some(model) = existing {
            let mut active: social_insurance_policy::ActiveModel = model.into();
            active.city_name = Set(dto.city_name);
            active.base_lower = Set(to_dec(dto.base_lower.unwrap_or(0.0)));
            active.base_upper = Set(to_dec(dto.base_upper.unwrap_or(0.0)));
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
            active.effective_date = Set(effective_date);
            active.expiry_date = Set(expiry_date);
            if let Some(st) = dto.status {
                active.status = Set(Some(st as i16));
            }
            active.remark = Set(dto.remark);
            let updated = active.update(&txn).await.map_err(|e| e.to_string())?;
            updated.id
        } else {
            let active = social_insurance_policy::ActiveModel {
                city_code: Set(dto.city_code),
                city_name: Set(dto.city_name),
                year: Set(dto.year),
                base_lower: Set(to_dec(dto.base_lower.unwrap_or(0.0))),
                base_upper: Set(to_dec(dto.base_upper.unwrap_or(0.0))),
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
                effective_date: Set(effective_date),
                expiry_date: Set(expiry_date),
                status: Set(Some(dto.status.unwrap_or(1) as i16)),
                remark: Set(dto.remark),
                enabled: Set(Some(1)),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            let inserted = active.insert(&txn).await.map_err(|e| e.to_string())?;
            inserted.id
        }
    };

    // 档次明细：差量 upsert（保留已有档次 id，避免员工配置 policy_level_id 悬空）
    if let Some(levels) = dto.levels {
        if !levels.is_empty() {
            let existing_ids: Vec<i64> = insurance_policy_level::Entity::find()
                .filter(insurance_policy_level::Column::PolicyId.eq(policy_id))
                .all(&txn)
                .await
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|l| l.id)
                .collect();
            let keep_ids: Vec<i64> = levels.iter().filter_map(|lv| lv.id).collect();

            // 删除被移除的档次，并同步清空引用这些档次的员工配置（降级到政策表头计算）
            let removed_ids: Vec<i64> = existing_ids
                .iter()
                .filter(|id| !keep_ids.contains(id))
                .cloned()
                .collect();
            if !removed_ids.is_empty() {
                insurance_policy_level::Entity::delete_many()
                    .filter(insurance_policy_level::Column::Id.is_in(removed_ids.clone()))
                    .exec(&txn)
                    .await
                    .map_err(|e| e.to_string())?;
                employee_insurance_config::Entity::update_many()
                    .col_expr(
                        employee_insurance_config::Column::PolicyLevelId,
                        sea_orm::sea_query::Expr::value(Option::<i64>::None),
                    )
                    .filter(
                        employee_insurance_config::Column::PolicyLevelId.is_in(removed_ids),
                    )
                    .exec(&txn)
                    .await
                    .map_err(|e| e.to_string())?;
            }

            for lv in levels {
                let mut active = insurance_policy_level::ActiveModel {
                    policy_id: Set(policy_id),
                    level_type: Set(lv.level_type.map(|v| v as i16)),
                    level_name: Set(lv.level_name),
                    base_amount: Set(to_dec(lv.base_amount)),
                    base_lower: Set(lv.base_lower.map(to_dec)),
                    base_upper: Set(lv.base_upper.map(to_dec)),
                    pension_company_rate: Set(opt_dec(lv.pension_company_rate)),
                    pension_personal_rate: Set(opt_dec(lv.pension_personal_rate)),
                    medical_company_rate: Set(opt_dec(lv.medical_company_rate)),
                    medical_personal_rate: Set(opt_dec(lv.medical_personal_rate)),
                    unemployment_company_rate: Set(opt_dec(lv.unemployment_company_rate)),
                    unemployment_personal_rate: Set(opt_dec(lv.unemployment_personal_rate)),
                    workinjury_company_rate: Set(opt_dec(lv.workinjury_company_rate)),
                    workinjury_personal_rate: Set(opt_dec(lv.workinjury_personal_rate)),
                    maternity_company_rate: Set(opt_dec(lv.maternity_company_rate)),
                    maternity_personal_rate: Set(opt_dec(lv.maternity_personal_rate)),
                    housing_fund_company_rate: Set(opt_dec(lv.housing_fund_company_rate)),
                    housing_fund_personal_rate: Set(opt_dec(lv.housing_fund_personal_rate)),
                    critical_illness_company_amount: Set(opt_dec(lv.critical_illness_company_amount)),
                    critical_illness_personal_amount: Set(opt_dec(lv.critical_illness_personal_amount)),
                    ..Default::default()
                };
                if let Some(lid) = lv.id {
                    // 已有档次：按 id 更新（保留 id 稳定）
                    active.id = Set(lid);
                    active.update_time = Set(Some(now));
                    active.update(&txn).await.map_err(|e| e.to_string())?;
                } else {
                    // 新增档次
                    active.create_time = Set(Some(now));
                    active.update_time = Set(Some(now));
                    active.insert(&txn).await.map_err(|e| e.to_string())?;
                }
            }
        }
    }

    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(policy_id)
}

/// 删除政策（连带删除档次明细；被员工配置引用时阻止删除）
pub async fn delete_policy(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    // 引用检查：有员工配置关联该政策时不允许删除，避免配置悬空
    let ref_count = employee_insurance_config::Entity::find()
        .filter(employee_insurance_config::Column::PolicyId.eq(id))
        .filter(employee_insurance_config::Column::Enabled.eq(1))
        .count(&txn)
        .await
        .map_err(|e| e.to_string())?;
    if ref_count > 0 {
        return Err(format!(
            "该政策正被 {} 条员工社保配置引用，请先调整员工配置后再删除",
            ref_count
        ));
    }
    insurance_policy_level::Entity::delete_many()
        .filter(insurance_policy_level::Column::PolicyId.eq(id))
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;
    social_insurance_policy::Entity::delete_by_id(id)
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 员工社保配置 ====================

/// 查询所有员工社保配置（带政策/档次信息）
pub async fn get_all_employee_configs(
    db: &DatabaseConnection,
) -> Result<Vec<EmployeeConfigWithPolicy>, String> {
    let configs = employee_insurance_config::Entity::find()
        .filter(employee_insurance_config::Column::Enabled.eq(1))
        .order_by_desc(employee_insurance_config::Column::EmployeeId)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    // 批量查询员工名称（admin 表），避免 N+1
    let mut employee_name_map: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    {
        let ids: Vec<i64> = configs.iter().map(|c| c.employee_id).collect();
        if !ids.is_empty() {
            use crate::modules::system::entity::admin;
            let admins = admin::Entity::find()
                .filter(admin::Column::Id.is_in(ids))
                .all(db)
                .await
                .map_err(|e| e.to_string())?;
            for a in admins {
                let name = a
                    .nick_name
                    .clone()
                    .or_else(|| a.user_name.clone())
                    .unwrap_or_default();
                employee_name_map.insert(a.id, name);
            }
        }
    }

    let mut result = Vec::with_capacity(configs.len());
    for c in configs {
        // 城市名称：优先从政策头取
        let city_name = if let Some(pid) = c.policy_id {
            social_insurance_policy::Entity::find_by_id(pid)
                .one(db)
                .await
                .ok()
                .flatten()
                .map(|p| p.city_name)
                .unwrap_or_else(|| c.city_code.clone())
        } else {
            c.city_code.clone()
        };
        // 档次信息
        let mut policy_year: Option<i32> = None;
        let mut policy_eff: Option<String> = None;
        let mut policy_exp: Option<String> = None;
        let mut level_name: Option<String> = None;
        let mut level_type: Option<i16> = None;
        if let Some(lid) = c.policy_level_id {
            if let Some(level) = insurance_policy_level::Entity::find_by_id(lid)
                .one(db)
                .await
                .map_err(|e| e.to_string())?
            {
                level_name = level.level_name;
                level_type = level.level_type;
            }
        }
        if let Some(pid) = c.policy_id {
            if let Some(policy) = social_insurance_policy::Entity::find_by_id(pid)
                .one(db)
                .await
                .map_err(|e| e.to_string())?
            {
                policy_year = Some(policy.year);
                policy_eff = date_to_string(policy.effective_date);
                policy_exp = date_to_string(policy.expiry_date);
            }
        }

        result.push(EmployeeConfigWithPolicy {
            id: c.id,
            employee_id: c.employee_id,
            employee_name: employee_name_map
                .get(&c.employee_id)
                .cloned()
                .unwrap_or_else(|| format!("员工{}", c.employee_id)),
            city_code: c.city_code.clone(),
            city_name,
            policy_id: c.policy_id,
            policy_level_id: c.policy_level_id,
            policy_year,
            policy_effective_date: policy_eff,
            policy_expiry_date: policy_exp,
            level_name,
            level_type: level_type.map(|v| v as i32),
            use_policy_base: c.use_policy_base,
            base_amount: c.base_amount,
            housing_fund_base: c.housing_fund_base,
            housing_fund_company_rate: c.housing_fund_company_rate,
            housing_fund_personal_rate: c.housing_fund_personal_rate,
            participate_pension: c.participate_pension,
            participate_medical: c.participate_medical,
            participate_unemployment: c.participate_unemployment,
            participate_workinjury: c.participate_workinjury,
            participate_maternity: c.participate_maternity,
            participate_housing_fund: c.participate_housing_fund,
            participate_critical_illness: c.participate_critical_illness.map(|v| v as i32),
            workinjury_company_rate: c.workinjury_company_rate,
            workinjury_personal_rate: c.workinjury_personal_rate,
            effective_date: date_to_string(c.effective_date),
            expiry_date: date_to_string(c.expiry_date),
            enabled: c.enabled,
        });
    }
    Ok(result)
}

/// 新增/更新员工社保配置
pub async fn upsert_employee_config(
    db: &DatabaseConnection,
    dto: EmployeeInsuranceConfigDTO,
) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // 按 employee_id 匹配
    let existing = if let Some(pid) = dto.id {
        employee_insurance_config::Entity::find_by_id(pid)
            .one(&txn)
            .await
            .map_err(|e| e.to_string())?
    } else {
        employee_insurance_config::Entity::find()
            .filter(employee_insurance_config::Column::EmployeeId.eq(dto.employee_id))
            .filter(employee_insurance_config::Column::Enabled.eq(1))
            .one(&txn)
            .await
            .map_err(|e| e.to_string())?
    };

    if let Some(model) = existing {
        let mut active: employee_insurance_config::ActiveModel = model.into();
        active.city_code = Set(dto.city_code);
        active.policy_id = Set(dto.policy_id);
        active.policy_level_id = Set(dto.policy_level_id);
        active.use_policy_base = Set(dto.use_policy_base);
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
        active.participate_critical_illness = Set(dto.participate_critical_illness.map(|v| v as i16));
        active.workinjury_company_rate = Set(dto.workinjury_company_rate.map(to_dec));
        active.workinjury_personal_rate = Set(dto.workinjury_personal_rate.map(to_dec));
        active.update_time = Set(Some(now));
        let updated = active.update(&txn).await.map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(updated.id)
    } else {
        let active = employee_insurance_config::ActiveModel {
            employee_id: Set(dto.employee_id),
            city_code: Set(dto.city_code),
            policy_id: Set(dto.policy_id),
            policy_level_id: Set(dto.policy_level_id),
            use_policy_base: Set(dto.use_policy_base.or(Some(true))),
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
            participate_critical_illness: Set(dto.participate_critical_illness.or(Some(1)).map(|v| v as i16)),
            workinjury_company_rate: Set(dto.workinjury_company_rate.map(to_dec)),
            workinjury_personal_rate: Set(dto.workinjury_personal_rate.map(to_dec)),
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

// ==================== 社保计算引擎 ====================

/// 根据档次计算各险种明细（核心计算逻辑）
///
/// `cfg` 承载参与标志与员工级覆盖（公积金比例、工伤比例等）。
/// `use_policy_base`：true 用档次基数，false 用自定义基数并 clamp 到上下限。
fn compute_premiums(
    level: &insurance_policy_level::Model,
    use_policy_base: bool,
    custom_base: Decimal,
    cfg: &PreviewCalcDTO,
) -> PremiumPreviewResult {
    // 缴费基数
    let lower = level.base_lower.unwrap_or(Decimal::ZERO);
    let upper = level.base_upper.unwrap_or(level.base_amount);
    let base = if use_policy_base {
        level.base_amount
    } else {
        custom_base.max(lower).min(upper)
    };

    let item = |company_rate: Decimal, personal_rate: Decimal| -> PremiumItem {
        PremiumItem {
            company: round_money(base * company_rate),
            personal: round_money(base * personal_rate),
            subtotal: Decimal::ZERO,
        }
    };
    // 先算出各险种，再补 subtotal
    let mut pension = item(level.pension_company_rate, level.pension_personal_rate);
    let mut medical = item(level.medical_company_rate, level.medical_personal_rate);
    let mut unemployment = item(
        level.unemployment_company_rate,
        level.unemployment_personal_rate,
    );
    let workinjury_company_rate = cfg
        .workinjury_company_rate
        .map(to_dec)
        .unwrap_or(level.workinjury_company_rate);
    let workinjury_personal_rate = cfg
        .workinjury_personal_rate
        .map(to_dec)
        .unwrap_or(level.workinjury_personal_rate);
    let mut workinjury = item(workinjury_company_rate, workinjury_personal_rate);
    let mut maternity = item(
        level.maternity_company_rate,
        level.maternity_personal_rate,
    );
    // 重大保险：固定金额（不随基数）
    let critical_illness = PremiumItem {
        company: level.critical_illness_company_amount,
        personal: level.critical_illness_personal_amount,
        subtotal: Decimal::ZERO,
    };
    // 公积金：基数优先员工自定义 housing_fund_base，否则用社保基数
    let hf_base = cfg.housing_fund_base.map(to_dec).unwrap_or(base);
    let hf_company_rate = cfg
        .housing_fund_company_rate
        .map(to_dec)
        .unwrap_or(level.housing_fund_company_rate);
    let hf_personal_rate = cfg
        .housing_fund_personal_rate
        .map(to_dec)
        .unwrap_or(level.housing_fund_personal_rate);
    // 注意：公积金必须用 hf_base 计算，不能用社保基数 base
    let mut housing_fund = PremiumItem {
        company: round_money(hf_base * hf_company_rate),
        personal: round_money(hf_base * hf_personal_rate),
        subtotal: Decimal::ZERO,
    };

    // 应用参与标志（0=不参与，置 0）
    if cfg.participate_pension.unwrap_or(1) == 0 {
        pension = PremiumItem::default();
    }
    if cfg.participate_medical.unwrap_or(1) == 0 {
        medical = PremiumItem::default();
    }
    if cfg.participate_unemployment.unwrap_or(1) == 0 {
        unemployment = PremiumItem::default();
    }
    if cfg.participate_workinjury.unwrap_or(1) == 0 {
        workinjury = PremiumItem::default();
    }
    if cfg.participate_maternity.unwrap_or(1) == 0 {
        maternity = PremiumItem::default();
    }
    if cfg.participate_housing_fund.unwrap_or(1) == 0 {
        housing_fund = PremiumItem::default();
    }

    // 重大保险参与标志独立控制（默认参与）
    let mut critical_illness_final = critical_illness;
    if cfg.participate_critical_illness.unwrap_or(1) == 0 {
        critical_illness_final = PremiumItem::default();
    }

    // 汇总
    pension.subtotal = pension.company + pension.personal;
    medical.subtotal = medical.company + medical.personal;
    unemployment.subtotal = unemployment.company + unemployment.personal;
    workinjury.subtotal = workinjury.company + workinjury.personal;
    maternity.subtotal = maternity.company + maternity.personal;
    critical_illness_final.subtotal =
        critical_illness_final.company + critical_illness_final.personal;
    housing_fund.subtotal = housing_fund.company + housing_fund.personal;

    let company_total = pension.company
        + medical.company
        + unemployment.company
        + workinjury.company
        + maternity.company
        + critical_illness_final.company
        + housing_fund.company;
    let personal_total = pension.personal
        + medical.personal
        + unemployment.personal
        + workinjury.personal
        + maternity.personal
        + critical_illness_final.personal
        + housing_fund.personal;

    PremiumPreviewResult {
        base_amount: base,
        pension,
        medical,
        unemployment,
        workinjury,
        maternity,
        critical_illness: critical_illness_final,
        housing_fund,
        company_total: round_money(company_total),
        personal_total: round_money(personal_total),
        grand_total: round_money(company_total + personal_total),
    }
}

/// 实时预览计算：根据政策/档次/自定义参数计算各险种明细
pub async fn preview_calculation(
    db: &DatabaseConnection,
    dto: PreviewCalcDTO,
) -> Result<PremiumPreviewResult, String> {
    let level_id = dto
        .level_id
        .ok_or_else(|| "请选择政策档次".to_string())?;
    let level = insurance_policy_level::Entity::find_by_id(level_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("政策档次不存在: {}", level_id))?;

    let use_policy_base = dto.use_policy_base.unwrap_or(true);
    let custom_base = to_dec(dto.base_amount.unwrap_or(0.0));

    // 若关联了政策，校验 level 属于该政策
    if let Some(pid) = dto.policy_id {
        if level.policy_id != pid {
            return Err("所选档次不属于该政策".to_string());
        }
    }

    Ok(compute_premiums(&level, use_policy_base, custom_base, &dto))
}

/// 计算当月社保公积金（工资核算联动，兼容旧结构）
///
/// 优先级：
/// 1. 员工配置已关联 policy_level_id → 用档次明细计算（含重大保险）
/// 2. 仅关联 city_code → 降级用政策表头单档比例计算（旧结构兼容）
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

    // 查询社保政策：优先用员工配置关联的政策（须启用且生效），
    // 其次按 city_code + year 匹配状态生效的政策（避免命中同年度已停用政策）
    let mut policy_opt: Option<social_insurance_policy::Model> = None;
    if let Some(pid) = config.policy_id {
        if let Some(p) = social_insurance_policy::Entity::find_by_id(pid)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
        {
            if p.enabled.unwrap_or(1) == 1 && p.status.unwrap_or(1) == 1 {
                policy_opt = Some(p);
            }
        }
    }
    if policy_opt.is_none() {
        policy_opt = social_insurance_policy::Entity::find()
            .filter(social_insurance_policy::Column::CityCode.eq(&config.city_code))
            .filter(social_insurance_policy::Column::Year.eq(year))
            .filter(social_insurance_policy::Column::Enabled.eq(1))
            .filter(social_insurance_policy::Column::Status.eq(1))
            .order_by_desc(social_insurance_policy::Column::EffectiveDate)
            .order_by_desc(social_insurance_policy::Column::Id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?;
    }
    let policy = policy_opt.ok_or_else(|| {
        format!(
            "城市 {} 的 {} 年社保政策不存在",
            config.city_code, year
        )
    })?;

    // 构造计算参数（参与标志 + 员工级覆盖）
    let calc_cfg = PreviewCalcDTO {
        policy_id: config.policy_id.or(Some(policy.id)),
        level_id: config.policy_level_id,
        base_amount: config.base_amount.to_f64(),
        use_policy_base: config.use_policy_base.or(Some(true)),
        housing_fund_base: config.housing_fund_base.map(|d| d.to_f64().unwrap_or(0.0)),
        housing_fund_company_rate: config.housing_fund_company_rate.map(|d| d.to_f64().unwrap_or(0.0)),
        housing_fund_personal_rate: config.housing_fund_personal_rate.map(|d| d.to_f64().unwrap_or(0.0)),
        participate_pension: config.participate_pension,
        participate_medical: config.participate_medical,
        participate_unemployment: config.participate_unemployment,
        participate_workinjury: config.participate_workinjury,
        participate_maternity: config.participate_maternity,
        participate_housing_fund: config.participate_housing_fund,
        participate_critical_illness: config.participate_critical_illness.map(|v| v as i32),
        workinjury_company_rate: config.workinjury_company_rate.map(|d| d.to_f64().unwrap_or(0.0)),
        workinjury_personal_rate: config.workinjury_personal_rate.map(|d| d.to_f64().unwrap_or(0.0)),
    };

    // 优先走档次计算
    let result = if let Some(level_id) = config.policy_level_id {
        let level = insurance_policy_level::Entity::find_by_id(level_id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?;
        if let Some(level) = level {
            let preview = compute_premiums(&level, calc_cfg.use_policy_base.unwrap_or(true), config.base_amount, &calc_cfg);
            Some(preview)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(preview) = result {
        return Ok(MonthlyInsuranceResult {
            social_insurance_personal: preview.pension.personal
                + preview.medical.personal
                + preview.unemployment.personal
                + preview.workinjury.personal
                + preview.maternity.personal
                + preview.critical_illness.personal,
            social_insurance_company: preview.pension.company
                + preview.medical.company
                + preview.unemployment.company
                + preview.workinjury.company
                + preview.maternity.company
                + preview.critical_illness.company,
            housing_fund_personal: preview.housing_fund.personal,
            housing_fund_company: preview.housing_fund.company,
            critical_illness_personal: preview.critical_illness.personal,
            critical_illness_company: preview.critical_illness.company,
        });
    }

    // 降级：旧结构（政策表头单档，无档次表）
    let base = config
        .base_amount
        .max(policy.base_lower)
        .min(policy.base_upper);

    let mut social_personal = Decimal::ZERO;
    let mut social_company = Decimal::ZERO;

    if config.participate_pension.unwrap_or(1) == 1 {
        social_personal += base * policy.pension_personal_rate;
        social_company += base * policy.pension_company_rate;
    }
    if config.participate_medical.unwrap_or(1) == 1 {
        social_personal += base * policy.medical_personal_rate;
        social_company += base * policy.medical_company_rate;
    }
    if config.participate_unemployment.unwrap_or(1) == 1 {
        social_personal += base * policy.unemployment_personal_rate;
        social_company += base * policy.unemployment_company_rate;
    }
    if config.participate_workinjury.unwrap_or(1) == 1 {
        social_company += base * policy.workinjury_company_rate;
    }
    if config.participate_maternity.unwrap_or(1) == 1 {
        social_company += base * policy.maternity_company_rate;
    }

    let mut housing_personal = Decimal::ZERO;
    let mut housing_company = Decimal::ZERO;
    if config.participate_housing_fund.unwrap_or(1) == 1 {
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
        social_insurance_personal: round_money(social_personal),
        social_insurance_company: round_money(social_company),
        housing_fund_personal: round_money(housing_personal),
        housing_fund_company: round_money(housing_company),
        critical_illness_personal: Decimal::ZERO,
        critical_illness_company: Decimal::ZERO,
    })
}
