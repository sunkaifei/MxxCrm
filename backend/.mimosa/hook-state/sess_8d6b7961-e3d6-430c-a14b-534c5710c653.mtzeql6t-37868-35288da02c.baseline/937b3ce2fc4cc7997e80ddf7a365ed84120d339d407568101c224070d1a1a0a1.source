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

/// P1-2 批量设置参保方案请求：员工清单 + 统一应用的参保方案
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSetInsuranceConfigDTO {
    /// 目标员工ID清单（非空）
    pub employee_ids: Vec<i64>,
    /// 参保城市编码
    pub city_code: String,
    /// 城市政策ID
    pub policy_id: i64,
    /// 缴费档次ID（空则由后端取政策默认档次）
    pub policy_level_id: Option<i64>,
    pub use_policy_base: Option<bool>,
    /// 自定义缴费基数（use_policy_base=false 时使用）
    pub base_amount: Option<f64>,
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

/// 批量设置失败明细项
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSetFailedItem {
    pub employee_id: i64,
    pub reason: String,
}

/// 批量设置结果：成功/失败清单 + 总数
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSetResult {
    pub total: usize,
    pub success: Vec<i64>,
    pub failed: Vec<BatchSetFailedItem>,
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
    /// 配置来源：manual=手工配置 inherited=继承档案城市默认方案
    pub source: Option<String>,
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

/// 员工社保配置覆盖度明细 VO
#[derive(serde::Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct InsuranceCoverageEmployee {
    pub employee_id: i64,
    pub employee_name: String,
    /// 是否存在手工社保配置（enabled=1）
    pub has_config: bool,
    /// 档案参保地（work_city_code），缺失人员恒为空
    pub work_city_code: Option<String>,
    /// 能否核算社保（有配置或有参保地）
    pub covered: bool,
}

/// 员工社保配置覆盖度 VO（以参与工资核算的员工为分母）
#[derive(serde::Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct InsuranceCoverageVO {
    /// 参与工资核算的员工总数（salary_enabled=1 且在职）
    pub total: i64,
    /// 已手工配置人数
    pub configured: i64,
    /// 无手工配置但有参保地（继承城市政策）人数
    pub inherited: i64,
    /// 既无配置又无参保地人数（核算时会被阻止出账）
    pub missing: i64,
    /// 缺失员工明细
    pub missing_list: Vec<InsuranceCoverageEmployee>,
}

/// 查询员工社保配置覆盖度：口径与工资核算预检一致（配置 = enabled=1 的行；参保地 = work_city_code 非空白）
pub async fn get_insurance_coverage(
    db: &DatabaseConnection,
) -> Result<InsuranceCoverageVO, String> {
    use crate::modules::system::entity::admin;

    let admins = admin::Entity::find()
        .filter(admin::Column::SalaryEnabled.eq(1))
        .filter(admin::Column::Status.eq(1))
        .filter(admin::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let ids: Vec<i64> = admins.iter().map(|a| a.id).collect();
    let mut configured_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    if !ids.is_empty() {
        let cfgs = employee_insurance_config::Entity::find()
            .filter(employee_insurance_config::Column::EmployeeId.is_in(ids))
            .filter(employee_insurance_config::Column::Enabled.eq(1))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for c in cfgs {
            configured_ids.insert(c.employee_id);
        }
    }

    let mut vo = InsuranceCoverageVO::default();
    vo.total = admins.len() as i64;
    let mut missing: Vec<InsuranceCoverageEmployee> = Vec::new();
    for a in &admins {
        let has_config = configured_ids.contains(&a.id);
        let city = a.work_city_code.clone().filter(|s| !s.trim().is_empty());
        if has_config {
            vo.configured += 1;
        } else if city.is_some() {
            vo.inherited += 1;
        } else {
            missing.push(InsuranceCoverageEmployee {
                employee_id: a.id,
                employee_name: a.nick_name.clone()
                    .or_else(|| a.user_name.clone())
                    .unwrap_or_default(),
                has_config,
                work_city_code: city,
                covered: false,
            });
        }
    }
    missing.sort_by_key(|e| e.employee_id);
    vo.missing = missing.len() as i64;
    vo.missing_list = missing;
    Ok(vo)
}

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
            source: c.source,
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
        // P1-2: 可选项仅在显式传入时更新，避免批量设置（未传参与标志/基数模式/比例）
        //       将以 NULL 覆盖员工已有自定义值；None 语义 = 保持不变
        if let Some(v) = dto.use_policy_base {
            active.use_policy_base = Set(Some(v));
        }
        active.base_amount = Set(to_dec(dto.base_amount));
        if let Some(v) = dto.housing_fund_base {
            active.housing_fund_base = Set(Some(to_dec(v)));
        }
        if let Some(v) = dto.housing_fund_company_rate {
            active.housing_fund_company_rate = Set(Some(to_dec(v)));
        }
        if let Some(v) = dto.housing_fund_personal_rate {
            active.housing_fund_personal_rate = Set(Some(to_dec(v)));
        }
        if let Some(v) = dto.participate_pension {
            active.participate_pension = Set(Some(v));
        }
        if let Some(v) = dto.participate_medical {
            active.participate_medical = Set(Some(v));
        }
        if let Some(v) = dto.participate_unemployment {
            active.participate_unemployment = Set(Some(v));
        }
        if let Some(v) = dto.participate_workinjury {
            active.participate_workinjury = Set(Some(v));
        }
        if let Some(v) = dto.participate_maternity {
            active.participate_maternity = Set(Some(v));
        }
        if let Some(v) = dto.participate_housing_fund {
            active.participate_housing_fund = Set(Some(v));
        }
        if let Some(v) = dto.participate_critical_illness {
            active.participate_critical_illness = Set(Some(v as i16));
        }
        if let Some(v) = dto.workinjury_company_rate {
            active.workinjury_company_rate = Set(Some(to_dec(v)));
        }
        if let Some(v) = dto.workinjury_personal_rate {
            active.workinjury_personal_rate = Set(Some(to_dec(v)));
        }
        // P1-1：页面手工保存/编辑的配置一律标记为 manual
        active.source = Set(Some("manual".to_string()));
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
            // P1-1：页面手工新增的配置一律标记为 manual
            source: Set(Some("manual".to_string())),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        let inserted = active.insert(&txn).await.map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(inserted.id)
    }
}

/// P1-2：批量设置参保方案——逐条复用单条 upsert（每条独立事务），单条失败不影响其余，返回成功/失败清单
///
/// 批量设置仅用于覆盖手工方案；新增员工默认走档案参保地继承逻辑，无需人工干预。
/// 未显式选择档次时自动取政策最低档（level_type=0，与档案继承 build_inherited_config 口径一致）。
pub async fn batch_set_employee_config(
    db: &DatabaseConnection,
    dto: BatchSetInsuranceConfigDTO,
) -> Result<BatchSetResult, String> {
    if dto.employee_ids.is_empty() {
        return Err("员工清单为空，未执行批量设置".to_string());
    }
    let total = dto.employee_ids.len();
    // P1-2: 批量未显式选择档次时，取该政策最低档（level_type=0）作为默认档，
    // 与页面预览"默认档次"文案及继承路径口径一致，避免写入 policy_level_id=NULL
    // 后算薪降级到政策表头单档（多档政策表头比例可能为 0）导致金额失真
    let resolved_level_id = match dto.policy_level_id {
        Some(lid) => Some(lid),
        None => insurance_policy_level::Entity::find()
            .filter(insurance_policy_level::Column::PolicyId.eq(dto.policy_id))
            .filter(insurance_policy_level::Column::LevelType.eq(0i16))
            .one(db)
            .await
            .map_err(|e| format!("查询政策默认档次失败：{}", e))?
            .map(|l| l.id),
    };
    let mut success: Vec<i64> = Vec::new();
    let mut failed: Vec<BatchSetFailedItem> = Vec::new();
    for employee_id in dto.employee_ids {
        let single = EmployeeInsuranceConfigDTO {
            id: None,
            employee_id,
            city_code: dto.city_code.clone(),
            policy_id: Some(dto.policy_id),
            policy_level_id: resolved_level_id,
            // P1-2: 批量"绑定档次"语义 = 按档次基数缴费；未显式传 usePolicyBase 时置 true，
            // 防止覆盖已有"自定义基数"（use_policy_base=false）员工后 base_amount 被写 0 又钳制到政策下限
            use_policy_base: dto.use_policy_base.or(Some(true)),
            base_amount: dto.base_amount.unwrap_or(0.0),
            housing_fund_base: dto.housing_fund_base,
            housing_fund_company_rate: dto.housing_fund_company_rate,
            housing_fund_personal_rate: dto.housing_fund_personal_rate,
            participate_pension: dto.participate_pension,
            participate_medical: dto.participate_medical,
            participate_unemployment: dto.participate_unemployment,
            participate_workinjury: dto.participate_workinjury,
            participate_maternity: dto.participate_maternity,
            participate_housing_fund: dto.participate_housing_fund,
            participate_critical_illness: dto.participate_critical_illness,
            workinjury_company_rate: dto.workinjury_company_rate,
            workinjury_personal_rate: dto.workinjury_personal_rate,
        };
        match upsert_employee_config(db, single).await {
            Ok(_) => success.push(employee_id),
            Err(e) => failed.push(BatchSetFailedItem {
                employee_id,
                reason: e,
            }),
        }
    }
    Ok(BatchSetResult {
        total,
        success,
        failed,
    })
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

/// P1-1：按城市 + 年份 + 月份匹配生效中的社保政策
///
/// 匹配规则（同一年内不同月份可适用不同生效月的政策）：
/// 1. 优先取 `effective_month <= month` 的政策中生效月最大的（如政策 4 月生效，核算 6 月则命中）；
/// 2. 无月份限定（effective_month 为空，视为全年适用）的政策作为兜底；
/// 3. 仅返回启用且状态生效的政策，避免命中已停用政策。
async fn find_policy_for_month<C: ConnectionTrait>(
    conn: &C,
    city_code: &str,
    year: i32,
    month: i32,
) -> Result<Option<social_insurance_policy::Model>, String> {
    // 1) 月份限定政策：取生效月 <= 核算月中的最大者
    let monthly = social_insurance_policy::Entity::find()
        .filter(social_insurance_policy::Column::CityCode.eq(city_code))
        .filter(social_insurance_policy::Column::Year.eq(year))
        .filter(social_insurance_policy::Column::Enabled.eq(1))
        .filter(social_insurance_policy::Column::Status.eq(1))
        .filter(social_insurance_policy::Column::EffectiveMonth.is_not_null())
        .filter(social_insurance_policy::Column::EffectiveMonth.lte(month))
        .order_by_desc(social_insurance_policy::Column::EffectiveMonth)
        .order_by_desc(social_insurance_policy::Column::EffectiveDate)
        .order_by_desc(social_insurance_policy::Column::Id)
        .one(conn)
        .await
        .map_err(|e| e.to_string())?;
    if monthly.is_some() {
        return Ok(monthly);
    }
    // 2) 全年适用政策（未设生效月）兜底
    social_insurance_policy::Entity::find()
        .filter(social_insurance_policy::Column::CityCode.eq(city_code))
        .filter(social_insurance_policy::Column::Year.eq(year))
        .filter(social_insurance_policy::Column::Enabled.eq(1))
        .filter(social_insurance_policy::Column::Status.eq(1))
        .filter(social_insurance_policy::Column::EffectiveMonth.is_null())
        .order_by_desc(social_insurance_policy::Column::EffectiveDate)
        .order_by_desc(social_insurance_policy::Column::Id)
        .one(conn)
        .await
        .map_err(|e| e.to_string())
}

/// P1-1：按政策构造"继承默认方案"的员工虚拟配置（仅供算薪内存计算，不落库）
///
/// 参与标志默认全部参加；基数默认跟随政策档次。
/// [Assumption — 待业务确认] 默认档次取政策中的最低档（level_type=0）；
/// 政策无档次明细（旧结构）时降级为表头单档计算，基数取政策下限。
async fn build_inherited_config<C: ConnectionTrait>(
    conn: &C,
    policy: &social_insurance_policy::Model,
) -> Result<employee_insurance_config::Model, String> {
    // 默认档次：政策下的最低档（level_type=0）
    let level = insurance_policy_level::Entity::find()
        .filter(insurance_policy_level::Column::PolicyId.eq(policy.id))
        .filter(insurance_policy_level::Column::LevelType.eq(0i16))
        .one(conn)
        .await
        .map_err(|e| e.to_string())?;
    Ok(employee_insurance_config::Model {
        id: 0,
        employee_id: 0,
        city_code: policy.city_code.clone(),
        policy_id: Some(policy.id),
        policy_level_id: level.as_ref().map(|l| l.id),
        use_policy_base: Some(true),
        base_amount: level
            .as_ref()
            .map(|l| l.base_amount)
            .unwrap_or(policy.base_lower),
        housing_fund_base: None,
        housing_fund_company_rate: None,
        housing_fund_personal_rate: None,
        participate_pension: Some(1),
        participate_medical: Some(1),
        participate_unemployment: Some(1),
        participate_workinjury: Some(1),
        participate_maternity: Some(1),
        participate_housing_fund: Some(1),
        participate_critical_illness: Some(1),
        workinjury_company_rate: None,
        workinjury_personal_rate: None,
        effective_date: None,
        expiry_date: None,
        enabled: Some(1),
        source: Some("inherited".to_string()),
        create_time: None,
        update_time: None,
    })
}

/// 计算当月社保公积金（工资核算联动，兼容旧结构）
///
/// P1-1 配置优先级（手工配置 > 档案城市默认政策；两者皆无则显式失败阻止出账）：
/// 1. 员工手工社保配置（manual）已关联 policy_level_id → 用档次明细计算（含重大保险）
/// 2. 仅关联 city_code → 降级用政策表头单档比例计算（旧结构兼容）
/// 3. 无手工配置 → 继承档案参保地（work_city_code）的默认政策与最低档（inherited，不落库）
///
/// month 实际参与政策匹配：优先取 effective_month <= 核算月的最新政策，
/// 无月份限定（全年适用）的政策兜底，支持同一年内不同月份的不同配置。
pub(crate) async fn calculate_monthly_insurance<C: ConnectionTrait>(
    conn: &C,
    employee_id: i64,
    year: i32,
    month: i32,
) -> Result<MonthlyInsuranceResult, String> {
    // P1-1：month 实际参与政策匹配（同一年内不同月份可适用不同生效月的政策）；
    // 配置优先级：手工配置（manual）> 档案参保地默认方案（继承，不落库）；
    // 两者皆无时显式报错，阻止出账（不再静默按 0）。
    let admin = {
        use crate::modules::system::entity::admin;
        admin::Entity::find_by_id(employee_id)
            .one(conn)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("员工{}不存在，无法核算社保", employee_id))?
    };
    let employee_name = admin
        .nick_name
        .clone()
        .or_else(|| admin.user_name.clone())
        .unwrap_or_else(|| format!("员工{}", employee_id));

    // 查询员工手工社保配置（enabled=1）
    let manual_cfg = employee_insurance_config::Entity::find()
        .filter(employee_insurance_config::Column::EmployeeId.eq(employee_id))
        .filter(employee_insurance_config::Column::Enabled.eq(1))
        .one(conn)
        .await
        .map_err(|e| e.to_string())?;

    // 确定配置与适用政策
    let (config, policy) = match manual_cfg {
        Some(cfg) => {
            // 手工配置路径：优先用配置关联的政策（须启用且生效），
            // 其次按 city_code + year + month 匹配状态生效的政策（避免命中同年度已停用政策）
            let mut policy_opt: Option<social_insurance_policy::Model> = None;
            if let Some(pid) = cfg.policy_id {
                if let Some(p) = social_insurance_policy::Entity::find_by_id(pid)
                    .one(conn)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    if p.enabled.unwrap_or(1) == 1 && p.status.unwrap_or(1) == 1 {
                        policy_opt = Some(p);
                    }
                }
            }
            if policy_opt.is_none() {
                policy_opt = find_policy_for_month(conn, &cfg.city_code, year, month).await?;
            }
            let policy = policy_opt.ok_or_else(|| {
                format!(
                    "员工{}的手工社保配置（城市 {}）在 {} 年 {} 月无生效社保政策，已阻止出账",
                    employee_name, cfg.city_code, year, month
                )
            })?;
            (cfg, policy)
        }
        None => {
            // 继承路径：员工未手工配置社保 → 按档案参保地（work_city_code）匹配当年城市政策
            let city = admin
                .work_city_code
                .clone()
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| {
                    format!(
                        "员工{}未手工配置社保且档案未设置参保地（work_city_code），无法核算社保，已阻止出账",
                        employee_name
                    )
                })?;
            let policy = find_policy_for_month(conn, &city, year, month)
                .await?
                .ok_or_else(|| {
                    format!(
                        "城市 {} 的 {} 年 {} 月社保政策不存在，员工{}无法核算社保，已阻止出账",
                        city, year, month, employee_name
                    )
                })?;
            let cfg = build_inherited_config(conn, &policy).await?;
            (cfg, policy)
        }
    };

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
            .one(conn)
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
