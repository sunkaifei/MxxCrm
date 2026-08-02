//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 个税计算服务，实现累计预扣法

use sea_orm::*;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};

use crate::modules::finance::entity::{tax_rate, employee_tax_config, salary_tax_detail};

// ==================== DTO ====================

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRateDTO {
    pub level: i32,
    pub min_amount: f64,
    pub max_amount: Option<f64>,
    pub rate: f64,
    pub quick_deduction: f64,
    pub tax_type: i32,
    pub effective_date: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeTaxConfigDTO {
    pub employee_id: i64,
    pub year: i32,
    pub tax_threshold: Option<f64>,
    pub children_education: Option<f64>,
    pub continuing_education: Option<f64>,
    pub housing_loan: Option<f64>,
    pub housing_rent: Option<f64>,
    pub supporting_elderly: Option<f64>,
    pub infant_care: Option<f64>,
    pub serious_illness: Option<f64>,
    pub other_deduction: Option<f64>,
    pub foreigner_allowance: Option<f64>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyTaxResult {
    pub monthly_tax: Decimal,
    pub cumulative_income: Decimal,
    pub cumulative_taxable: Decimal,
    pub applicable_rate: Decimal,
    pub quick_deduction: Decimal,
    pub cumulative_tax_should: Decimal,
    pub cumulative_tax_paid: Decimal,
    pub monthly_threshold: Decimal,
    pub monthly_special_deduction: Decimal,
    pub monthly_other_deduction: Decimal,
}

// ==================== 辅助函数 ====================

fn to_dec(v: f64) -> Decimal {
    Decimal::from_f64(v).unwrap_or_default()
}

fn parse_datetime_str(s: &str) -> Option<chrono::NaiveDateTime> {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .or_else(|| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
        })
}

/// 根据金额在税率表中查找适用税率与速算扣除数
fn find_rate(rates: &[tax_rate::Model], amount: Decimal) -> (Decimal, Decimal) {
    for r in rates {
        let in_range = amount >= r.min_amount
            && match r.max_amount {
                Some(m) => amount < m,
                None => true,
            };
        if in_range {
            return (r.rate, r.quick_deduction);
        }
    }
    (Decimal::ZERO, Decimal::ZERO)
}

// ==================== 税率表 CRUD ====================

/// 查询税率表（tax_type: 1=综合所得累计, 2=年终奖月度）
pub async fn get_tax_rate_list(
    db: &DatabaseConnection,
    tax_type: Option<i32>,
) -> Result<Vec<tax_rate::Model>, String> {
    let mut stmt = tax_rate::Entity::find().filter(tax_rate::Column::Enabled.eq(1));
    if let Some(t) = tax_type {
        stmt = stmt.filter(tax_rate::Column::TaxType.eq(t));
    }
    stmt.order_by_asc(tax_rate::Column::Level)
        .all(db)
        .await
        .map_err(|e| e.to_string())
}

/// 新增/更新税率
pub async fn upsert_tax_rate(db: &DatabaseConnection, dto: TaxRateDTO) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // 按 tax_type + level 匹配
    let existing = tax_rate::Entity::find()
        .filter(tax_rate::Column::TaxType.eq(dto.tax_type))
        .filter(tax_rate::Column::Level.eq(dto.level))
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?;

    let effective_date = dto.effective_date.as_deref().and_then(parse_datetime_str).map(|dt| dt.date());

    if let Some(model) = existing {
        let mut active: tax_rate::ActiveModel = model.into();
        active.min_amount = Set(to_dec(dto.min_amount));
        active.max_amount = Set(dto.max_amount.map(to_dec));
        active.rate = Set(to_dec(dto.rate));
        active.quick_deduction = Set(to_dec(dto.quick_deduction));
        active.effective_date = Set(effective_date);
        let updated = active.update(&txn).await.map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(updated.id)
    } else {
        let active = tax_rate::ActiveModel {
            level: Set(dto.level),
            min_amount: Set(to_dec(dto.min_amount)),
            max_amount: Set(dto.max_amount.map(to_dec)),
            rate: Set(to_dec(dto.rate)),
            quick_deduction: Set(to_dec(dto.quick_deduction)),
            tax_type: Set(dto.tax_type),
            effective_date: Set(effective_date),
            enabled: Set(Some(1)),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        let inserted = active.insert(&txn).await.map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(inserted.id)
    }
}

/// 删除税率
pub async fn delete_tax_rate(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    tax_rate::Entity::delete_by_id(id)
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 员工个税配置 ====================

/// 查询员工个税配置
pub async fn get_employee_tax_config(
    db: &DatabaseConnection,
    employee_id: i64,
    year: i32,
) -> Result<Option<employee_tax_config::Model>, String> {
    employee_tax_config::Entity::find()
        .filter(employee_tax_config::Column::EmployeeId.eq(employee_id))
        .filter(employee_tax_config::Column::Year.eq(year))
        .one(db)
        .await
        .map_err(|e| e.to_string())
}

/// 查询所有员工个税配置列表（支持按年份过滤）
pub async fn get_employee_tax_config_list(
    db: &DatabaseConnection,
    year: Option<i32>,
) -> Result<Vec<employee_tax_config::Model>, String> {
    let mut stmt = employee_tax_config::Entity::find();
    if let Some(y) = year {
        stmt = stmt.filter(employee_tax_config::Column::Year.eq(y));
    }
    stmt.order_by_desc(employee_tax_config::Column::EmployeeId)
        .all(db)
        .await
        .map_err(|e| e.to_string())
}

/// 新增/更新员工个税配置（含7项专项附加扣除）
pub async fn upsert_employee_tax_config(
    db: &DatabaseConnection,
    dto: EmployeeTaxConfigDTO,
) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let existing = employee_tax_config::Entity::find()
        .filter(employee_tax_config::Column::EmployeeId.eq(dto.employee_id))
        .filter(employee_tax_config::Column::Year.eq(dto.year))
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?;

    let threshold = dto.tax_threshold.map(to_dec).unwrap_or(Decimal::from(5000));
    let children_education = dto.children_education.map(to_dec).unwrap_or_default();
    let continuing_education = dto.continuing_education.map(to_dec).unwrap_or_default();
    let housing_loan = dto.housing_loan.map(to_dec).unwrap_or_default();
    let housing_rent = dto.housing_rent.map(to_dec).unwrap_or_default();
    let supporting_elderly = dto.supporting_elderly.map(to_dec).unwrap_or_default();
    let infant_care = dto.infant_care.map(to_dec).unwrap_or_default();
    let serious_illness = dto.serious_illness.map(to_dec).unwrap_or_default();
    let other_deduction = dto.other_deduction.map(to_dec).unwrap_or_default();
    let foreigner_allowance = dto.foreigner_allowance.map(to_dec).unwrap_or_default();

    if let Some(model) = existing {
        let mut active: employee_tax_config::ActiveModel = model.into();
        active.tax_threshold = Set(threshold);
        active.children_education = Set(children_education);
        active.continuing_education = Set(continuing_education);
        active.housing_loan = Set(housing_loan);
        active.housing_rent = Set(housing_rent);
        active.supporting_elderly = Set(supporting_elderly);
        active.infant_care = Set(infant_care);
        active.serious_illness = Set(serious_illness);
        active.other_deduction = Set(other_deduction);
        active.foreigner_allowance = Set(foreigner_allowance);
        active.update_time = Set(Some(now));
        let updated = active.update(&txn).await.map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(updated.id)
    } else {
        let active = employee_tax_config::ActiveModel {
            employee_id: Set(dto.employee_id),
            year: Set(dto.year),
            tax_threshold: Set(threshold),
            children_education: Set(children_education),
            continuing_education: Set(continuing_education),
            housing_loan: Set(housing_loan),
            housing_rent: Set(housing_rent),
            supporting_elderly: Set(supporting_elderly),
            infant_care: Set(infant_care),
            serious_illness: Set(serious_illness),
            other_deduction: Set(other_deduction),
            foreigner_allowance: Set(foreigner_allowance),
            cumulative_income: Set(Decimal::ZERO),
            cumulative_threshold_deduction: Set(Decimal::ZERO),
            cumulative_special_deduction: Set(Decimal::ZERO),
            cumulative_other_deduction: Set(Decimal::ZERO),
            cumulative_tax_paid: Set(Decimal::ZERO),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        let inserted = active.insert(&txn).await.map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(inserted.id)
    }
}

// ==================== 个税计算 ====================

/// 计算当月个税（累计预扣法）
///
/// 算法：
/// - 累计应纳税所得额 = 累计收入 - 累计减除费用(5000*月数) - 累计专项附加扣除 - 累计其他扣除
/// - 查税率表(tax_type=1)找适用税率和速算扣除数
/// - 累计应纳税额 = 累计应纳税所得额 × 税率 - 速算扣除数
/// - 当月应纳税额 = 累计应纳税额 - 累计已缴税额
/// - 当月应纳税额 < 0 时取 0
pub async fn calculate_monthly_tax(
    db: &DatabaseConnection,
    employee_id: i64,
    year: i32,
    month: i32,
    monthly_income: f64,
) -> Result<MonthlyTaxResult, String> {
    let config = employee_tax_config::Entity::find()
        .filter(employee_tax_config::Column::EmployeeId.eq(employee_id))
        .filter(employee_tax_config::Column::Year.eq(year))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "员工个税配置不存在".to_string())?;

    let income = to_dec(monthly_income);
    let monthly_threshold = config.tax_threshold;

    // P2-1: 外籍员工津补贴免税处理（财税〔1994〕148号、财税〔2018〕164号）
    // 外籍员工（含港澳台）可在"专项附加扣除"与"津补贴免税"中二选一，互斥：
    //   - 选择"津补贴免税"时：津补贴部分（住房/伙食/洗衣/搬迁/语言培训/子女教育津贴等）
    //     作为免税收入从应税收入中扣减，不再享受专项附加扣除
    //   - 选择"专项附加扣除"时：按内地员工规则处理，foreigner_allowance 不参与
    // 通过 foreigner_allowance > 0 判定员工选择"津补贴免税"方式
    let is_foreigner_exemption = config.foreigner_allowance > Decimal::ZERO;

    // 当月应税收入：外籍员工需扣减免税津补贴部分（不低于0）
    let monthly_taxable_income = if is_foreigner_exemption {
        let taxable = income - config.foreigner_allowance;
        if taxable < Decimal::ZERO { Decimal::ZERO } else { taxable }
    } else {
        income
    };

    // 当月专项附加扣除 = 7 项之和（外籍员工选择津补贴免税时为0）
    let monthly_special_deduction = if is_foreigner_exemption {
        Decimal::ZERO
    } else {
        config.children_education
            + config.continuing_education
            + config.housing_loan
            + config.housing_rent
            + config.supporting_elderly
            + config.infant_care
            + config.serious_illness
    };

    // 当月其他扣除 = 仅其他扣除（外籍津补贴已作为免税收入扣减，不在此重复）
    let monthly_other_deduction = config.other_deduction;

    // 累计值 = 配置中已累计（截至上月）+ 当月
    let cumulative_income = config.cumulative_income + monthly_taxable_income;
    let cumulative_threshold = monthly_threshold * Decimal::from(month); // 5000 * 月数
    let cumulative_special = config.cumulative_special_deduction + monthly_special_deduction;
    let cumulative_other = config.cumulative_other_deduction + monthly_other_deduction;

    // 累计应纳税所得额
    let cumulative_taxable =
        cumulative_income - cumulative_threshold - cumulative_special - cumulative_other;

    let cumulative_tax_paid = config.cumulative_tax_paid;

    // 累计应纳税所得额 <= 0，当月不纳税
    if cumulative_taxable <= Decimal::ZERO {
        return Ok(MonthlyTaxResult {
            monthly_tax: Decimal::ZERO,
            cumulative_income,
            cumulative_taxable,
            applicable_rate: Decimal::ZERO,
            quick_deduction: Decimal::ZERO,
            cumulative_tax_should: Decimal::ZERO,
            cumulative_tax_paid,
            monthly_threshold,
            monthly_special_deduction,
            monthly_other_deduction,
        });
    }

    // 查税率表（tax_type=1 综合所得累计）
    let rates = get_tax_rate_list(db, Some(1)).await?;
    let (rate, quick_deduction) = find_rate(&rates, cumulative_taxable);

    let cumulative_tax_should = cumulative_taxable * rate - quick_deduction;
    let mut monthly_tax = cumulative_tax_should - cumulative_tax_paid;
    if monthly_tax < Decimal::ZERO {
        monthly_tax = Decimal::ZERO;
    }

    Ok(MonthlyTaxResult {
        monthly_tax,
        cumulative_income,
        cumulative_taxable,
        applicable_rate: rate,
        quick_deduction,
        cumulative_tax_should,
        cumulative_tax_paid,
        monthly_threshold,
        monthly_special_deduction,
        monthly_other_deduction,
    })
}

/// 保存个税明细到 mxx_finance_salary_tax_detail 表，并更新员工个税配置的累计值
pub async fn save_tax_detail(
    db: &DatabaseConnection,
    salary_record_id: i64,
    employee_id: i64,
    year: i32,
    month: i32,
    result: MonthlyTaxResult,
) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // 查询配置以计算当月收入（= 累计收入 - 上月累计收入）
    let config = employee_tax_config::Entity::find()
        .filter(employee_tax_config::Column::EmployeeId.eq(employee_id))
        .filter(employee_tax_config::Column::Year.eq(year))
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "员工个税配置不存在".to_string())?;

    // 在 config.into() 消费前保存所需的累计值
    let prev_cumulative_income = config.cumulative_income;
    let prev_cumulative_special = config.cumulative_special_deduction;
    let prev_cumulative_other = config.cumulative_other_deduction;
    let prev_cumulative_tax_paid = config.cumulative_tax_paid;

    let monthly_income = result.cumulative_income - prev_cumulative_income;

    // 保存个税明细
    let detail = salary_tax_detail::ActiveModel {
        salary_record_id: Set(salary_record_id),
        employee_id: Set(employee_id),
        year: Set(year),
        month: Set(month),
        monthly_income: Set(Some(monthly_income)),
        monthly_threshold: Set(Some(result.monthly_threshold)),
        monthly_special_deduction: Set(Some(result.monthly_special_deduction)),
        monthly_other_deduction: Set(Some(result.monthly_other_deduction)),
        cumulative_income: Set(Some(result.cumulative_income)),
        cumulative_taxable: Set(Some(result.cumulative_taxable)),
        applicable_rate: Set(Some(result.applicable_rate)),
        quick_deduction: Set(Some(result.quick_deduction)),
        cumulative_tax_should: Set(Some(result.cumulative_tax_should)),
        cumulative_tax_paid: Set(Some(result.cumulative_tax_paid)),
        monthly_tax: Set(Some(result.monthly_tax)),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    let inserted = detail.insert(&txn).await.map_err(|e| e.to_string())?;

    // 更新员工个税配置的累计值（截至当月的累计）
    let mut active: employee_tax_config::ActiveModel = config.into();
    active.cumulative_income = Set(result.cumulative_income);
    active.cumulative_threshold_deduction =
        Set(result.monthly_threshold * Decimal::from(month));
    active.cumulative_special_deduction =
        Set(prev_cumulative_special + result.monthly_special_deduction);
    active.cumulative_other_deduction =
        Set(prev_cumulative_other + result.monthly_other_deduction);
    // 累计已预缴税额 = 上月累计 + 当月应纳税额
    active.cumulative_tax_paid = Set(prev_cumulative_tax_paid + result.monthly_tax);
    active.update_time = Set(Some(now));
    active.update(&txn).await.map_err(|e| e.to_string())?;

    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(inserted.id)
}

// ==================== 个税明细查询 ====================

/// 查询员工某年个税明细
pub async fn get_tax_detail_list(
    db: &DatabaseConnection,
    employee_id: i64,
    year: i32,
) -> Result<Vec<salary_tax_detail::Model>, String> {
    salary_tax_detail::Entity::find()
        .filter(salary_tax_detail::Column::EmployeeId.eq(employee_id))
        .filter(salary_tax_detail::Column::Year.eq(year))
        .order_by_asc(salary_tax_detail::Column::Month)
        .all(db)
        .await
        .map_err(|e| e.to_string())
}

// ==================== 年终奖计税 ====================

/// 年终奖单独计税（除以12找税率）
pub async fn calculate_annual_bonus_tax(
    db: &DatabaseConnection,
    bonus_amount: f64,
) -> Result<Decimal, String> {
    let bonus = to_dec(bonus_amount);
    if bonus <= Decimal::ZERO {
        return Ok(Decimal::ZERO);
    }

    // 除以12找适用税率
    let monthly_equiv = bonus / Decimal::from(12);
    let rates = get_tax_rate_list(db, Some(2)).await?;
    let (rate, quick_deduction) = find_rate(&rates, monthly_equiv);

    let tax = bonus * rate - quick_deduction;
    if tax < Decimal::ZERO {
        Ok(Decimal::ZERO)
    } else {
        Ok(tax)
    }
}

// ==================== P2-2: 年终奖双轨计税 ====================

/// 年终奖计税方式
/// - 1 = 单独计税（财税〔2018〕164号：除以12找税率，单独计算）
/// - 2 = 并入综合所得（与当月工资合并，按累计预扣法计算税差）
pub const BONUS_TAX_MODE_SEPARATE: i32 = 1;
pub const BONUS_TAX_MODE_MERGED: i32 = 2;

/// 年终奖双轨计税结果
#[derive(serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AnnualBonusTaxResult {
    /// 计税方式: 1=单独计税 2=并入综合所得
    pub tax_mode: i32,
    /// 年终奖应纳税额
    pub bonus_tax: Decimal,
    /// 计税方式说明
    pub mode_desc: String,
    /// 单独计税应纳税额（用于对比参考）
    pub separate_tax: Decimal,
    /// 并入综合所得应纳税额（用于对比参考）
    pub merged_tax: Decimal,
}

/// P2-2: 年终奖双轨计税
///
/// 根据财税〔2018〕164号，居民个人取得全年一次性奖金，可选择：
/// - 单独计税：不并入当年综合所得，以奖金全额除以12找税率，单独计算纳税
/// - 并入综合所得：与当年综合所得合并，按累计预扣法计算
///
/// 算法：
/// - `tax_mode=1`（单独计税）：调用 `calculate_annual_bonus_tax`，与工资个税独立
/// - `tax_mode=2`（并入综合所得）：
///   1. 计算不含奖金的当月个税 tax_without_bonus
///   2. 计算含奖金的当月个税 tax_with_bonus
///   3. 奖金部分个税 = tax_with_bonus - tax_without_bonus
///
/// 注：调用方需确保 mode=2 时不再为同一笔工资单独调用 `calculate_monthly_tax` +
/// `save_tax_detail`（避免重复计税）；建议 mode=2 时直接以 (工资+奖金) 作为
/// monthly_income 调用 `calculate_monthly_tax` 并保存，本函数仅返回税差供参考。
pub async fn calculate_annual_bonus_tax_dual(
    db: &DatabaseConnection,
    employee_id: i64,
    year: i32,
    month: i32,
    bonus_amount: f64,
    monthly_income: f64,
    tax_mode: i32,
) -> Result<AnnualBonusTaxResult, String> {
    let bonus = to_dec(bonus_amount);

    // 始终计算两种方式的税额，便于前端展示对比
    let separate_tax = if bonus > Decimal::ZERO {
        calculate_annual_bonus_tax(db, bonus_amount).await?
    } else {
        Decimal::ZERO
    };

    let merged_tax = if bonus > Decimal::ZERO {
        // 并入综合所得：计算含/不含奖金的税差
        // calculate_monthly_tax 不修改状态（仅 save_tax_detail 写入），可安全调用两次
        let tax_without = calculate_monthly_tax(
            db, employee_id, year, month, monthly_income,
        ).await.map(|r| r.monthly_tax).unwrap_or(Decimal::ZERO);

        let combined_income = monthly_income + bonus_amount;
        let tax_with = calculate_monthly_tax(
            db, employee_id, year, month, combined_income,
        ).await.map(|r| r.monthly_tax).unwrap_or(Decimal::ZERO);

        let diff = tax_with - tax_without;
        if diff < Decimal::ZERO { Decimal::ZERO } else { diff }
    } else {
        Decimal::ZERO
    };

    // 按选定方式返回
    let (bonus_tax, mode_desc) = match tax_mode {
        BONUS_TAX_MODE_MERGED => (merged_tax, "并入综合所得".to_string()),
        _ => (separate_tax, "单独计税".to_string()),
    };

    Ok(AnnualBonusTaxResult {
        tax_mode,
        bonus_tax,
        mode_desc,
        separate_tax,
        merged_tax,
    })
}
