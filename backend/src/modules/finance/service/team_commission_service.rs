//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 团队提成服务
//! 负责按管理者维度计算团队提成，并将结果写回 salary_record.team_commission_amount
//!

use sea_orm::*;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::collections::{HashMap, HashSet};

use crate::modules::finance::entity::{salary_record, commission_rule};
use crate::modules::crm::entity::{contract, contract_payment_plan};
use crate::modules::system::entity::admin;
use crate::modules::finance::service::tax_service;

/// P2-3: 默认管理者岗位系数（按管理层级差异化）
/// - level 1（直属上级/团队长/主管）: 3%
/// - level 2（部门经理）: 5%
/// - level 3（总监/高管）: 8%
fn default_coefficient_by_level(level: usize) -> Decimal {
    match level {
        1 => Decimal::new(3, 2), // 0.03 团队长/主管
        2 => Decimal::new(5, 2), // 0.05 部门经理
        3 => Decimal::new(8, 2), // 0.08 总监/高管
        _ => Decimal::new(3, 2), // 默认按团队长
    }
}

/// P2-3: 按 commission_rule.rule_type 映射岗位系数
/// rule_type: 3=部门经理, 4=总监, 5=团队长
fn coefficient_by_rule_type(rule_type: Option<i32>) -> Decimal {
    match rule_type {
        Some(3) => Decimal::new(5, 2), // 部门经理 5%
        Some(4) => Decimal::new(8, 2), // 总监 8%
        Some(5) => Decimal::new(3, 2), // 团队长 3%
        _ => Decimal::new(5, 2),       // 默认 5%
    }
}

/// 向上查找管理者的最大层级
const MAX_MANAGER_LEVELS: usize = 3;

/// 团队提成列表项
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamCommissionListDTO {
    pub id: i64,
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub department_name: Option<String>,
    pub year: i32,
    pub month: i32,
    pub team_commission_amount: f64,
    pub base_salary: f64,
    pub total_salary: f64,
    pub status: Option<i32>,
}

/// 团队提成汇总项
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamCommissionSummaryDTO {
    pub manager_id: i64,
    pub manager_name: Option<String>,
    pub team_total_payment: f64,
    pub commission_amount: f64,
    pub subordinates_count: i64,
}

/// 月度团队提成结算
///
/// 计算逻辑：
/// 1. 查询当月所有已回款的合同回款计划（actual_date 在当月且 received >= plan）
/// 2. 按合同 assigned_to 找到员工
/// 3. 沿 direct_manager_id 向上找各级管理者，最多 3 级
/// 4. 根据提成规则表 commission_rule 中 calc_base_type=2（团队提成）的规则计算
/// 5. 管理者团队提成 = 下属团队总回款额 × 管理者岗位系数（默认 0.05）
/// 6. 将结果写入 salary_record.team_commission_amount
pub async fn calc_monthly_settlement(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
) -> Result<i64, String> {
    // 计算月份起止日期
    let month_start = chrono::NaiveDate::from_ymd_opt(year, month as u32, 1)
        .ok_or_else(|| "日期格式错误".to_string())?;
    let next_month = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1)
    }
    .ok_or_else(|| "日期格式错误".to_string())?;
    let month_end = next_month - chrono::Duration::days(1);

    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // 1. 查询当月已回款的回款计划
    let payment_plans: Vec<contract_payment_plan::Model> = contract_payment_plan::Entity::find()
        .filter(contract_payment_plan::Column::ActualDate.gte(month_start))
        .filter(contract_payment_plan::Column::ActualDate.lte(month_end))
        .filter(contract_payment_plan::Column::Deleted.eq(0))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;

    // 过滤完全回款的计划（received >= plan 且 plan > 0）
    let fully_paid_plans: Vec<&contract_payment_plan::Model> = payment_plans
        .iter()
        .filter(|p| {
            let received = p.received_amount.unwrap_or_default();
            let plan = p.plan_amount.unwrap_or_default();
            received >= plan && !plan.is_zero()
        })
        .collect();

    if fully_paid_plans.is_empty() {
        txn.commit().await.map_err(|e| e.to_string())?;
        return Ok(0);
    }

    // 2. 查询相关合同
    let contract_ids: Vec<i64> = fully_paid_plans
        .iter()
        .filter_map(|p| p.contract_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let mut contract_map: HashMap<i64, contract::Model> = HashMap::new();
    if !contract_ids.is_empty() {
        let contracts = contract::Entity::find()
            .filter(contract::Column::Id.is_in(contract_ids))
            .all(&txn)
            .await
            .map_err(|e| e.to_string())?;
        for c in contracts {
            contract_map.insert(c.id, c);
        }
    }

    // 3. 按员工聚合回款额（employee_id -> total_payment）
    let mut employee_payment: HashMap<i64, Decimal> = HashMap::new();
    for plan in &fully_paid_plans {
        if let Some(contract_id) = plan.contract_id {
            if let Some(contract_model) = contract_map.get(&contract_id) {
                if let Some(employee_id) = contract_model.assigned_to {
                    let payment = plan.received_amount.unwrap_or_default();
                    *employee_payment.entry(employee_id).or_insert(Decimal::ZERO) += payment;
                }
            }
        }
    }

    if employee_payment.is_empty() {
        txn.commit().await.map_err(|e| e.to_string())?;
        return Ok(0);
    }

    // 4. 查询涉及的员工信息（用于获取 direct_manager_id）
    let employee_ids: Vec<i64> = employee_payment.keys().cloned().collect();
    let admins = admin::Entity::find()
        .filter(admin::Column::Id.is_in(employee_ids.clone()))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;
    let mut admin_map: HashMap<i64, admin::Model> = HashMap::new();
    for a in admins {
        admin_map.insert(a.id, a);
    }

    // 5. P2-3: 查询启用的团队提成规则（calc_base_type=2）及其阶梯
    let team_rules: Vec<commission_rule::Model> = commission_rule::Entity::find()
        .filter(commission_rule::Column::CalcBaseType.eq(2))
        .filter(commission_rule::Column::Enabled.eq(1))
        .filter(commission_rule::Column::Deleted.eq(0))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;

    // P2-3: 查询团队提成规则的阶梯配置（用于覆盖默认系数）
    let team_rule_ids: Vec<i64> = team_rules.iter().map(|r| r.id).collect();
    let mut rule_tier_map: HashMap<i64, Vec<crate::modules::finance::entity::commission_tier::Model>> = HashMap::new();
    if !team_rule_ids.is_empty() {
        use crate::modules::finance::entity::commission_tier;
        let tiers = commission_tier::Entity::find()
            .filter(commission_tier::Column::RuleId.is_in(team_rule_ids))
            .order_by_asc(commission_tier::Column::MinAmount)
            .all(&txn)
            .await
            .map_err(|e| e.to_string())?;
        for t in tiers {
            rule_tier_map.entry(t.rule_id).or_default().push(t);
        }
    }

    // 6. 对每个员工沿 direct_manager_id 向上遍历最多 3 级，累计各管理者的下属团队回款额
    // P2-3: 同时记录管理者所在层级（1=团队长, 2=经理, 3=总监），用于差异化系数
    // manager_payment: manager_id -> (total_payment, min_level)
    // 同一管理者可能通过多个下属链路被触达，取最浅层级（最小 level）作为其岗位系数依据
    let mut manager_payment: HashMap<i64, (Decimal, usize)> = HashMap::new();

    for (employee_id, payment) in &employee_payment {
        let mut current_id = *employee_id;
        let mut visited: HashSet<i64> = HashSet::new();
        visited.insert(current_id);

        for level in 1..=MAX_MANAGER_LEVELS {
            // 取当前员工的直属上级
            let manager_id = match admin_map.get(&current_id) {
                Some(a) => a.direct_manager_id,
                None => None,
            };
            let manager_id = match manager_id {
                Some(mid) => mid,
                None => break,
            };
            // 防止循环引用
            if !visited.insert(manager_id) {
                break;
            }
            // P2-3: 累计该管理者的团队回款，并记录最浅层级
            let entry = manager_payment.entry(manager_id).or_insert((Decimal::ZERO, level));
            entry.0 += *payment;
            if level < entry.1 {
                entry.1 = level;
            }
            // 继续向上
            current_id = manager_id;
            // 加载新的上级到 admin_map
            if !admin_map.contains_key(&current_id) {
                if let Some(a) = admin::Entity::find_by_id(current_id)
                    .one(&txn)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    admin_map.insert(current_id, a);
                } else {
                    break;
                }
            }
        }
    }

    if manager_payment.is_empty() {
        txn.commit().await.map_err(|e| e.to_string())?;
        return Ok(0);
    }

    // 7. 查询管理者在该年月的 salary_record，更新 team_commission_amount
    let manager_ids: Vec<i64> = manager_payment.keys().cloned().collect();
    let salary_records: Vec<salary_record::Model> = salary_record::Entity::find()
        .filter(salary_record::Column::EmployeeId.is_in(manager_ids.clone()))
        .filter(salary_record::Column::Year.eq(year))
        .filter(salary_record::Column::Month.eq(month))
        .filter(salary_record::Column::Deleted.eq(0))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;

    let now = Utc::now().naive_utc();
    let mut updated_count: i64 = 0;

    for record in salary_records {
        let manager_id = record.employee_id;
        let (team_payment, manager_level) = match manager_payment.get(&manager_id) {
            Some(v) => *v,
            None => continue,
        };

        // P2-3: 按管理者层级差异化岗位系数
        // 优先级：commission_rule 阶梯配置 > rule_type 映射 > 管理层级默认值
        // 1. 尝试从团队提成规则的阶梯中找匹配的 commission_rate
        // 2. 若规则有 rule_type 但无阶梯，用 coefficient_by_rule_type
        // 3. 否则按管理层级默认系数
        let mut coefficient = default_coefficient_by_level(manager_level);

        // P2-3: 按管理层级映射期望的 rule_type
        // level 1=团队长(rule_type 5), level 2=部门经理(rule_type 3), level 3=总监(rule_type 4)
        let expected_rule_type: Option<i32> = match manager_level {
            1 => Some(5),
            2 => Some(3),
            3 => Some(4),
            _ => None,
        };

        // 查找匹配的团队提成规则（优先匹配 rule_type，其次取全公司通用规则）
        let mut matched_rule: Option<&commission_rule::Model> = None;
        // 第一轮：精确匹配 rule_type
        if let Some(rt) = expected_rule_type {
            for r in &team_rules {
                if r.rule_type == Some(rt) {
                    matched_rule = Some(r);
                    break;
                }
            }
        }
        // 第二轮：取任意一条全公司通用规则（apply_scope=2）
        if matched_rule.is_none() {
            for r in &team_rules {
                if r.apply_scope.unwrap_or(2) == 2 {
                    matched_rule = Some(r);
                    break;
                }
            }
        }

        if let Some(rule) = matched_rule {
            // 优先使用阶梯配置的 commission_rate（取第一个阶梯作为基础系数）
            if let Some(tiers) = rule_tier_map.get(&rule.id) {
                if let Some(first_tier) = tiers.first() {
                    if first_tier.commission_rate > Decimal::ZERO {
                        coefficient = first_tier.commission_rate;
                    }
                }
            }
            // 若阶梯未覆盖（系数仍为默认层级值），且规则有 rule_type，用 rule_type 映射
            if coefficient == default_coefficient_by_level(manager_level) {
                if rule.rule_type.is_some() {
                    coefficient = coefficient_by_rule_type(rule.rule_type);
                }
            }
        }

        let commission = team_payment * coefficient;
        log::info!(
            "[team_commission] 管理者{} 层级={} 系数={} 团队回款={} 提成={}",
            manager_id, manager_level, coefficient, team_payment, commission
        );

        // 在消费 record 之前先计算新的应发工资
        // total_salary = base_salary + commission_amount + performance_bonus + team_commission_amount - deduction_amount
        let new_total = record.base_salary
            + record.commission_amount
            + record.performance_bonus
            + commission
            - record.deduction_amount;

        // P0-2 修复：团队提成归集后重算个税和实发工资
        // 之前只更新 total_salary 未重算 tax_amount 和 net_salary，导致管理者个税申报数据错误
        let personal_insurance = record.social_insurance_personal;
        let personal_housing = record.housing_fund_personal;
        // 注：tax_service::calculate_monthly_tax 接收 &DatabaseConnection 而非事务，但内部 save_tax_detail
        // 会写入个税累计值。这里使用 db 而非 txn 以匹配签名；个税重算与工资更新不在同一事务，
        // 但失败时按原值保留，不影响工资记录正确性。
        let new_tax_amount: Decimal = match tax_service::calculate_monthly_tax(
            db,
            manager_id,
            record.year,
            record.month,
            new_total.to_f64().unwrap_or(0.0),
        ).await {
            Ok(result) => result.monthly_tax,
            Err(e) => {
                log::warn!("[team_commission] 员工{}个税重算失败，按原值保留: {}", manager_id, e);
                record.tax_amount
            }
        };
        let new_net = new_total - personal_insurance - personal_housing - new_tax_amount;

        let mut active: salary_record::ActiveModel = record.into();
        active.team_commission_amount = Set(commission);
        active.total_salary = Set(new_total);
        active.tax_amount = Set(new_tax_amount);
        active.net_salary = Set(new_net);
        active.update_time = Set(Some(now));
        active.update(&txn).await.map_err(|e| e.to_string())?;
        updated_count += 1;
        // 已更新，跳过下面再次按 manager_id 处理
        manager_payment.remove(&manager_id);
    }

    // 对于没有 salary_record 的管理者（理论上不应发生），跳过
    // 这里不做新建，避免与 salary_service 的核算逻辑冲突

    txn.commit().await.map_err(|e| e.to_string())?;

    // v2 分轮计算：category=3/4/5 在 category=2 提交后独立执行
    // 失败时记录日志但不影响 category=2 已完成的结果
    if let Err(e) = calc_team_bonus_round(db, year, month).await {
        log::warn!("[team_commission] category=3 团队激励奖金计算失败: {}", e);
    }
    if let Err(e) = calc_reallocation_round(db, year, month).await {
        log::warn!("[team_commission] category=5 总提成再分配归集失败: {}", e);
    }
    if let Err(e) = calc_pool_fund_round(db, year, month).await {
        log::warn!("[team_commission] category=4 团建资金池存入失败: {}", e);
    }

    Ok(updated_count)
}

/// Step 3: 团队激励奖金计算（category=3）
///
/// 逻辑：
/// 1. 查询当月启用的 category=3 规则
/// 2. 对每条规则，按 beneficiary_role 找到对应管理者
/// 3. 汇总该管理者团队当月回款
/// 4. 检查是否达到 bonus_target，达标则发放 bonus_fixed_amount
/// 5. 写入 commission_result (category=3)
/// 6. 累加到 salary_record.bonus_amount
async fn calc_team_bonus_round(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
) -> Result<i64, String> {
    use crate::modules::finance::entity::commission_result;
    use crate::modules::crm::entity::{contract, contract_payment_plan};

    // 查询启用的 category=3 规则
    let bonus_rules: Vec<commission_rule::Model> = commission_rule::Entity::find()
        .filter(commission_rule::Column::CommissionCategory.eq(3))
        .filter(commission_rule::Column::Enabled.eq(1))
        .filter(commission_rule::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    if bonus_rules.is_empty() {
        return Ok(0);
    }

    // 计算月份起止
    let (month_start, month_end) = month_range(year, month)?;

    // 查询当月已回款计划
    let payment_plans: Vec<contract_payment_plan::Model> = contract_payment_plan::Entity::find()
        .filter(contract_payment_plan::Column::ActualDate.gte(month_start))
        .filter(contract_payment_plan::Column::ActualDate.lte(month_end))
        .filter(contract_payment_plan::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let fully_paid: Vec<&contract_payment_plan::Model> = payment_plans
        .iter()
        .filter(|p| {
            let received = p.received_amount.unwrap_or_default();
            let plan = p.plan_amount.unwrap_or_default();
            received >= plan && !plan.is_zero()
        })
        .collect();

    if fully_paid.is_empty() {
        return Ok(0);
    }

    // 查询合同
    let contract_ids: Vec<i64> = fully_paid.iter().filter_map(|p| p.contract_id).collect::<HashSet<_>>().into_iter().collect();
    let mut contract_map: HashMap<i64, contract::Model> = HashMap::new();
    if !contract_ids.is_empty() {
        let contracts = contract::Entity::find()
            .filter(contract::Column::Id.is_in(contract_ids))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for c in contracts {
            contract_map.insert(c.id, c);
        }
    }

    // 按员工聚合回款额
    let mut employee_payment: HashMap<i64, Decimal> = HashMap::new();
    for plan in &fully_paid {
        if let Some(contract_id) = plan.contract_id {
            if let Some(contract_model) = contract_map.get(&contract_id) {
                if let Some(employee_id) = contract_model.assigned_to {
                    let payment = plan.received_amount.unwrap_or_default();
                    *employee_payment.entry(employee_id).or_insert(Decimal::ZERO) += payment;
                }
            }
        }
    }

    // 沿管理链向上找各级管理者，聚合团队回款
    let employee_ids: Vec<i64> = employee_payment.keys().cloned().collect();
    let mut admin_map: HashMap<i64, admin::Model> = HashMap::new();
    if !employee_ids.is_empty() {
        let admins = admin::Entity::find()
            .filter(admin::Column::Id.is_in(employee_ids.clone()))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for a in admins {
            admin_map.insert(a.id, a);
        }
    }

    // manager_team_payment: manager_id -> 团队总回款
    let mut manager_team_payment: HashMap<i64, Decimal> = HashMap::new();
    for (employee_id, payment) in &employee_payment {
        let mut current_id = *employee_id;
        let mut visited: HashSet<i64> = HashSet::new();
        visited.insert(current_id);
        for _ in 1..=MAX_MANAGER_LEVELS {
            let manager_id = match admin_map.get(&current_id) {
                Some(a) => a.direct_manager_id,
                None => None,
            };
            let manager_id = match manager_id {
                Some(mid) => mid,
                None => break,
            };
            if !visited.insert(manager_id) {
                break;
            }
            *manager_team_payment.entry(manager_id).or_insert(Decimal::ZERO) += *payment;
            current_id = manager_id;
            if !admin_map.contains_key(&current_id) {
                if let Some(a) = admin::Entity::find_by_id(current_id).one(db).await.map_err(|e| e.to_string())? {
                    admin_map.insert(current_id, a);
                } else {
                    break;
                }
            }
        }
    }

    let now = chrono::Utc::now().naive_utc();
    let mut processed_count: i64 = 0;

    for rule in &bonus_rules {
        let beneficiary_role = rule.beneficiary_role;
        let target = rule.bonus_target.unwrap_or(Decimal::ZERO);
        let fixed_amount = rule.bonus_fixed_amount.unwrap_or(Decimal::ZERO);
        if target <= Decimal::ZERO || fixed_amount <= Decimal::ZERO {
            continue;
        }

        // 按 beneficiary_role 筛选管理者
        for (manager_id, team_payment) in &manager_team_payment {
            // 检查团队回款是否达标
            if *team_payment < target {
                continue;
            }

            // 检查该管理者是否符合 beneficiary_role
            // 简化处理：所有管理者都视为符合（实际应根据岗位匹配）
            // TODO: 根据 admin_post_merge 精确匹配 beneficiary_role

            let manager_name = admin_map.get(manager_id).and_then(|a| a.nick_name.clone().or(a.user_name.clone()));

            // 写入 commission_result
            let result = commission_result::ActiveModel {
                salary_record_id: Set(None),
                contract_id: Set(None),
                contract_name: Set(None),
                rule_id: Set(rule.id),
                rule_name: Set(rule.rule_name.clone()),
                rule_type: Set(rule.rule_type.unwrap_or(3)),
                commission_category: Set(Some(3)),
                beneficiary_role: Set(Some(beneficiary_role)),
                manager_level: Set(None),
                allocate_status: Set(Some(0)),
                allocated_amount: Set(Some(Decimal::ZERO)),
                pool_id: Set(None),
                cost_amount: Set(None),
                user_id: Set(*manager_id),
                user_name: Set(manager_name),
                user_post_id: Set(None),
                department_id: Set(None),
                calc_base_amount: Set(*team_payment),
                tier_min_amount: Set(None),
                tier_max_amount: Set(None),
                commission_rate: Set(Decimal::ZERO),
                share_ratio: Set(None),
                commission_amount: Set(fixed_amount),
                trigger_condition: Set(1),
                trigger_source_id: Set(None),
                period_year: Set(year),
                period_month: Set(month),
                settled: Set(0),
                remark: Set(Some(format!("团队回款 {:.2} 达标门槛 {:.2}", team_payment, target))),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            result.insert(db).await.map_err(|e| e.to_string())?;

            // 累加到 salary_record.bonus_amount
            let sr = salary_record::Entity::find()
                .filter(salary_record::Column::EmployeeId.eq(*manager_id))
                .filter(salary_record::Column::Year.eq(year))
                .filter(salary_record::Column::Month.eq(month))
                .filter(salary_record::Column::Deleted.eq(0))
                .one(db)
                .await
                .map_err(|e| e.to_string())?;

            if let Some(sr_model) = sr {
                let current_bonus = sr_model.bonus_amount;
                let mut sr_active: salary_record::ActiveModel = sr_model.into();
                sr_active.bonus_amount = Set(current_bonus + fixed_amount);
                sr_active.update_time = Set(Some(now));
                sr_active.update(db).await.map_err(|e| e.to_string())?;
                processed_count += 1;
            }
        }
    }

    log::info!("[team_commission] category=3 团队激励奖金计算完成 year={} month={} 处理记录={}", year, month, processed_count);
    Ok(processed_count)
}

/// Step 4: 总提成再分配归集（category=5）
///
/// 逻辑：
/// 1. 查询当月启用的 category=5 规则
/// 2. 对每条规则，按 beneficiary_role 找到管理者
/// 3. 计算归集金额 = 团队回款 × 归集比例
/// 4. 写入 commission_result (category=5, allocate_status=1=待分配)
/// 5. 不写入 salary_record（等待管理者手动分配）
async fn calc_reallocation_round(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
) -> Result<i64, String> {
    use crate::modules::finance::entity::{commission_result, commission_tier};
    use crate::modules::crm::entity::{contract, contract_payment_plan};

    let rules: Vec<commission_rule::Model> = commission_rule::Entity::find()
        .filter(commission_rule::Column::CommissionCategory.eq(5))
        .filter(commission_rule::Column::Enabled.eq(1))
        .filter(commission_rule::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    if rules.is_empty() {
        return Ok(0);
    }

    let (month_start, month_end) = month_range(year, month)?;

    let payment_plans: Vec<contract_payment_plan::Model> = contract_payment_plan::Entity::find()
        .filter(contract_payment_plan::Column::ActualDate.gte(month_start))
        .filter(contract_payment_plan::Column::ActualDate.lte(month_end))
        .filter(contract_payment_plan::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let fully_paid: Vec<&contract_payment_plan::Model> = payment_plans
        .iter()
        .filter(|p| {
            let received = p.received_amount.unwrap_or_default();
            let plan = p.plan_amount.unwrap_or_default();
            received >= plan && !plan.is_zero()
        })
        .collect();

    if fully_paid.is_empty() {
        return Ok(0);
    }

    let contract_ids: Vec<i64> = fully_paid.iter().filter_map(|p| p.contract_id).collect::<HashSet<_>>().into_iter().collect();
    let mut contract_map: HashMap<i64, contract::Model> = HashMap::new();
    if !contract_ids.is_empty() {
        let contracts = contract::Entity::find()
            .filter(contract::Column::Id.is_in(contract_ids))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for c in contracts {
            contract_map.insert(c.id, c);
        }
    }

    let mut employee_payment: HashMap<i64, Decimal> = HashMap::new();
    for plan in &fully_paid {
        if let Some(contract_id) = plan.contract_id {
            if let Some(contract_model) = contract_map.get(&contract_id) {
                if let Some(employee_id) = contract_model.assigned_to {
                    let payment = plan.received_amount.unwrap_or_default();
                    *employee_payment.entry(employee_id).or_insert(Decimal::ZERO) += payment;
                }
            }
        }
    }

    let employee_ids: Vec<i64> = employee_payment.keys().cloned().collect();
    let mut admin_map: HashMap<i64, admin::Model> = HashMap::new();
    if !employee_ids.is_empty() {
        let admins = admin::Entity::find()
            .filter(admin::Column::Id.is_in(employee_ids.clone()))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for a in admins {
            admin_map.insert(a.id, a);
        }
    }

    // 聚合各管理者的团队回款
    let mut manager_team_payment: HashMap<i64, Decimal> = HashMap::new();
    for (employee_id, payment) in &employee_payment {
        let mut current_id = *employee_id;
        let mut visited: HashSet<i64> = HashSet::new();
        visited.insert(current_id);
        for _ in 1..=MAX_MANAGER_LEVELS {
            let manager_id = match admin_map.get(&current_id) {
                Some(a) => a.direct_manager_id,
                None => None,
            };
            let manager_id = match manager_id {
                Some(mid) => mid,
                None => break,
            };
            if !visited.insert(manager_id) {
                break;
            }
            *manager_team_payment.entry(manager_id).or_insert(Decimal::ZERO) += *payment;
            current_id = manager_id;
            if !admin_map.contains_key(&current_id) {
                if let Some(a) = admin::Entity::find_by_id(current_id).one(db).await.map_err(|e| e.to_string())? {
                    admin_map.insert(current_id, a);
                } else {
                    break;
                }
            }
        }
    }

    let now = chrono::Utc::now().naive_utc();
    let mut processed_count: i64 = 0;

    for rule in &rules {
        // 获取归集比例（从阶梯表第一档，或默认 0）
        let rate = get_rule_rate(db, rule.id).await?;

        if rate <= Decimal::ZERO {
            continue;
        }

        for (manager_id, team_payment) in &manager_team_payment {
            let allocate_amount = *team_payment * rate;
            if allocate_amount <= Decimal::ZERO {
                continue;
            }

            let manager_name = admin_map.get(manager_id).and_then(|a| a.nick_name.clone().or(a.user_name.clone()));

            // 写入 commission_result，状态=待分配
            let result = commission_result::ActiveModel {
                salary_record_id: Set(None),
                contract_id: Set(None),
                contract_name: Set(None),
                rule_id: Set(rule.id),
                rule_name: Set(rule.rule_name.clone()),
                rule_type: Set(rule.rule_type.unwrap_or(5)),
                commission_category: Set(Some(5)),
                beneficiary_role: Set(Some(rule.beneficiary_role)),
                manager_level: Set(None),
                allocate_status: Set(Some(1)), // 1=待分配
                allocated_amount: Set(Some(Decimal::ZERO)),
                pool_id: Set(None),
                cost_amount: Set(None),
                user_id: Set(*manager_id),
                user_name: Set(manager_name),
                user_post_id: Set(None),
                department_id: Set(None),
                calc_base_amount: Set(*team_payment),
                tier_min_amount: Set(None),
                tier_max_amount: Set(None),
                commission_rate: Set(rate),
                share_ratio: Set(None),
                commission_amount: Set(allocate_amount),
                trigger_condition: Set(1),
                trigger_source_id: Set(None),
                period_year: Set(year),
                period_month: Set(month),
                settled: Set(0),
                remark: Set(Some(format!("团队回款 {:.2} × 归集比例 {:.4} = {:.2}", team_payment, rate, allocate_amount))),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            result.insert(db).await.map_err(|e| e.to_string())?;
            processed_count += 1;
        }
    }

    log::info!("[team_commission] category=5 总提成再分配归集完成 year={} month={} 待分配记录={}", year, month, processed_count);
    Ok(processed_count)
}

/// Step 5: 团建资金池存入（category=4）
///
/// 逻辑：
/// 1. 查询当月启用的 category=4 规则
/// 2. 对每条规则，计算存入金额 = 团队回款 × 提取比例
/// 3. 写入 commission_result (category=4, pool_id=xxx)
/// 4. 调用 commission_pool_service::deposit_from_commission 存入资金池
/// 5. 不写入 salary_record
async fn calc_pool_fund_round(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
) -> Result<i64, String> {
    use crate::modules::finance::entity::{commission_result, commission_tier};
    use crate::modules::crm::entity::{contract, contract_payment_plan};
    use crate::modules::finance::service::commission_pool_service;

    let rules: Vec<commission_rule::Model> = commission_rule::Entity::find()
        .filter(commission_rule::Column::CommissionCategory.eq(4))
        .filter(commission_rule::Column::Enabled.eq(1))
        .filter(commission_rule::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    if rules.is_empty() {
        return Ok(0);
    }

    let (month_start, month_end) = month_range(year, month)?;

    let payment_plans: Vec<contract_payment_plan::Model> = contract_payment_plan::Entity::find()
        .filter(contract_payment_plan::Column::ActualDate.gte(month_start))
        .filter(contract_payment_plan::Column::ActualDate.lte(month_end))
        .filter(contract_payment_plan::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let fully_paid: Vec<&contract_payment_plan::Model> = payment_plans
        .iter()
        .filter(|p| {
            let received = p.received_amount.unwrap_or_default();
            let plan = p.plan_amount.unwrap_or_default();
            received >= plan && !plan.is_zero()
        })
        .collect();

    if fully_paid.is_empty() {
        return Ok(0);
    }

    let contract_ids: Vec<i64> = fully_paid.iter().filter_map(|p| p.contract_id).collect::<HashSet<_>>().into_iter().collect();
    let mut contract_map: HashMap<i64, contract::Model> = HashMap::new();
    if !contract_ids.is_empty() {
        let contracts = contract::Entity::find()
            .filter(contract::Column::Id.is_in(contract_ids))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for c in contracts {
            contract_map.insert(c.id, c);
        }
    }

    let mut employee_payment: HashMap<i64, Decimal> = HashMap::new();
    for plan in &fully_paid {
        if let Some(contract_id) = plan.contract_id {
            if let Some(contract_model) = contract_map.get(&contract_id) {
                if let Some(employee_id) = contract_model.assigned_to {
                    let payment = plan.received_amount.unwrap_or_default();
                    *employee_payment.entry(employee_id).or_insert(Decimal::ZERO) += payment;
                }
            }
        }
    }

    let employee_ids: Vec<i64> = employee_payment.keys().cloned().collect();
    let mut admin_map: HashMap<i64, admin::Model> = HashMap::new();
    if !employee_ids.is_empty() {
        let admins = admin::Entity::find()
            .filter(admin::Column::Id.is_in(employee_ids.clone()))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for a in admins {
            admin_map.insert(a.id, a);
        }
    }

    // 聚合各管理者的团队回款
    let mut manager_team_payment: HashMap<i64, Decimal> = HashMap::new();
    for (employee_id, payment) in &employee_payment {
        let mut current_id = *employee_id;
        let mut visited: HashSet<i64> = HashSet::new();
        visited.insert(current_id);
        for _ in 1..=MAX_MANAGER_LEVELS {
            let manager_id = match admin_map.get(&current_id) {
                Some(a) => a.direct_manager_id,
                None => None,
            };
            let manager_id = match manager_id {
                Some(mid) => mid,
                None => break,
            };
            if !visited.insert(manager_id) {
                break;
            }
            *manager_team_payment.entry(manager_id).or_insert(Decimal::ZERO) += *payment;
            current_id = manager_id;
            if !admin_map.contains_key(&current_id) {
                if let Some(a) = admin::Entity::find_by_id(current_id).one(db).await.map_err(|e| e.to_string())? {
                    admin_map.insert(current_id, a);
                } else {
                    break;
                }
            }
        }
    }

    let now = chrono::Utc::now().naive_utc();
    let mut processed_count: i64 = 0;

    for rule in &rules {
        let pool_id = match rule.pool_id {
            Some(pid) => pid,
            None => {
                log::warn!("[team_commission] category=4 规则 {} 未配置 pool_id，跳过", rule.id);
                continue;
            }
        };

        let rate = get_rule_rate(db, rule.id).await?;
        if rate <= Decimal::ZERO {
            continue;
        }

        for (manager_id, team_payment) in &manager_team_payment {
            let deposit_amount = *team_payment * rate;
            if deposit_amount <= Decimal::ZERO {
                continue;
            }

            let manager_name = admin_map.get(manager_id).and_then(|a| a.nick_name.clone().or(a.user_name.clone()));

            // 写入 commission_result
            let result = commission_result::ActiveModel {
                salary_record_id: Set(None),
                contract_id: Set(None),
                contract_name: Set(None),
                rule_id: Set(rule.id),
                rule_name: Set(rule.rule_name.clone()),
                rule_type: Set(rule.rule_type.unwrap_or(4)),
                commission_category: Set(Some(4)),
                beneficiary_role: Set(Some(rule.beneficiary_role)),
                manager_level: Set(None),
                allocate_status: Set(Some(0)),
                allocated_amount: Set(Some(Decimal::ZERO)),
                pool_id: Set(Some(pool_id)),
                cost_amount: Set(None),
                user_id: Set(*manager_id),
                user_name: Set(manager_name),
                user_post_id: Set(None),
                department_id: Set(None),
                calc_base_amount: Set(*team_payment),
                tier_min_amount: Set(None),
                tier_max_amount: Set(None),
                commission_rate: Set(rate),
                share_ratio: Set(None),
                commission_amount: Set(deposit_amount),
                trigger_condition: Set(1),
                trigger_source_id: Set(None),
                period_year: Set(year),
                period_month: Set(month),
                settled: Set(0),
                remark: Set(Some(format!("团队回款 {:.2} × 提取比例 {:.4} = {:.2} 存入资金池", team_payment, rate, deposit_amount))),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            result.insert(db).await.map_err(|e| e.to_string())?;

            // 存入资金池
            if let Err(e) = commission_pool_service::deposit_from_commission(
                db, pool_id, deposit_amount,
                Some(rule.id), Some(*manager_id),
                Some(year), Some(month),
            ).await {
                log::warn!("[team_commission] 资金池 {} 存入失败: {}", pool_id, e);
            }

            processed_count += 1;
        }
    }

    log::info!("[team_commission] category=4 团建资金池存入完成 year={} month={} 处理记录={}", year, month, processed_count);
    Ok(processed_count)
}

/// 计算月份起止日期
fn month_range(year: i32, month: i32) -> Result<(chrono::NaiveDate, chrono::NaiveDate), String> {
    let month_start = chrono::NaiveDate::from_ymd_opt(year, month as u32, 1)
        .ok_or_else(|| "日期格式错误".to_string())?;
    let next_month = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1)
    }
    .ok_or_else(|| "日期格式错误".to_string())?;
    let month_end = next_month - chrono::Duration::days(1);
    Ok((month_start, month_end))
}

/// 获取规则的比例（从阶梯表第一档，或默认 0）
async fn get_rule_rate(db: &DatabaseConnection, rule_id: i64) -> Result<Decimal, String> {
    use crate::modules::finance::entity::commission_tier;
    let tiers = commission_tier::Entity::find()
        .filter(commission_tier::Column::RuleId.eq(rule_id))
        .order_by_asc(commission_tier::Column::Sort)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(first_tier) = tiers.first() {
        if first_tier.commission_rate > Decimal::ZERO {
            return Ok(first_tier.commission_rate);
        }
    }
    Ok(Decimal::ZERO)
}

/// 查询团队提成列表
///
/// 从 salary_record 的 team_commission_amount 字段读取
/// 只返回团队提成金额 > 0 的记录
pub async fn get_team_commission_list(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
    manager_id: Option<i64>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<TeamCommissionListDTO>, i64), String> {
    let mut stmt = salary_record::Entity::find()
        .filter(salary_record::Column::Year.eq(year))
        .filter(salary_record::Column::Month.eq(month))
        .filter(salary_record::Column::Deleted.eq(0))
        .filter(salary_record::Column::TeamCommissionAmount.gt(Decimal::ZERO));

    if let Some(mid) = manager_id {
        stmt = stmt.filter(salary_record::Column::EmployeeId.eq(mid));
    }

    stmt = stmt.order_by_desc(salary_record::Column::TeamCommissionAmount);

    let page = std::cmp::max(page, 1);
    let page_size = std::cmp::max(page_size, 1);
    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?;

    let dto_list: Vec<TeamCommissionListDTO> = items
        .into_iter()
        .map(|m| TeamCommissionListDTO {
            id: m.id,
            employee_id: m.employee_id,
            employee_name: m.employee_name,
            department_name: m.department_name,
            year: m.year,
            month: m.month,
            team_commission_amount: m.team_commission_amount.to_f64().unwrap_or_default(),
            base_salary: m.base_salary.to_f64().unwrap_or_default(),
            total_salary: m.total_salary.to_f64().unwrap_or_default(),
            status: m.status,
        })
        .collect();

    Ok((dto_list, total))
}

/// 团队提成汇总（按管理者分组统计，去重聚合）
pub async fn get_team_summary(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
) -> Result<Vec<TeamCommissionSummaryDTO>, String> {
    let records = salary_record::Entity::find()
        .filter(salary_record::Column::Year.eq(year))
        .filter(salary_record::Column::Month.eq(month))
        .filter(salary_record::Column::Deleted.eq(0))
        .filter(salary_record::Column::TeamCommissionAmount.gt(Decimal::ZERO))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    if records.is_empty() {
        return Ok(Vec::new());
    }

    // 按 manager_id 聚合（防止同一管理者多条记录导致重复）
    // 优先取 status 最大（已发放优先）且 id 最大（最新）的记录作为代表
    let mut grouped: HashMap<i64, (Decimal, i32)> = HashMap::new(); // manager_id -> (commission, status)
    for r in &records {
        let mid = r.employee_id;
        let commission = r.team_commission_amount;
        let status = r.status.unwrap_or(0);
        match grouped.get(&mid) {
            Some((existing_comm, existing_status)) => {
                // 已发放(status=2)优先，否则取金额更大的
                let should_replace = status > *existing_status
                    || (status == *existing_status && commission > *existing_comm);
                if should_replace {
                    grouped.insert(mid, (commission, status));
                }
            }
            None => {
                grouped.insert(mid, (commission, status));
            }
        }
    }

    let manager_ids: Vec<i64> = grouped.keys().cloned().collect();
    let mut admin_map: HashMap<i64, admin::Model> = HashMap::new();
    if !manager_ids.is_empty() {
        let admins = admin::Entity::find()
            .filter(admin::Column::Id.is_in(manager_ids.clone()))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for a in admins {
            admin_map.insert(a.id, a);
        }
    }

    // 统计每个管理者的下属人数（直接下属），一次性查询避免 N+1
    let mut subordinate_counts: HashMap<i64, i64> = HashMap::new();
    if !manager_ids.is_empty() {
        // 查询所有直属上级在 manager_ids 中的员工
        let subordinates = admin::Entity::find()
            .filter(admin::Column::DirectManagerId.is_in(manager_ids.clone()))
            .filter(admin::Column::Deleted.eq(0))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for sub in subordinates {
            if let Some(mid) = sub.direct_manager_id {
                *subordinate_counts.entry(mid).or_insert(0) += 1;
            }
        }
    }

    let coeff = default_coefficient_by_level(2); // P2-3: 汇总估算用经理级系数 0.05
    let mut result: Vec<TeamCommissionSummaryDTO> = grouped
        .into_iter()
        .map(|(manager_id, (commission, _))| {
            let manager_name = admin_map
                .get(&manager_id)
                .and_then(|a| a.nick_name.clone().or_else(|| a.user_name.clone()));
            let team_total_payment = if coeff.is_zero() {
                Decimal::ZERO
            } else {
                commission / coeff
            };
            TeamCommissionSummaryDTO {
                manager_id,
                manager_name,
                team_total_payment: team_total_payment.to_f64().unwrap_or_default(),
                commission_amount: commission.to_f64().unwrap_or_default(),
                subordinates_count: subordinate_counts.get(&manager_id).copied().unwrap_or(0),
            }
        })
        .collect();

    // 按提成金额降序
    result.sort_by(|a, b| b.commission_amount.partial_cmp(&a.commission_amount).unwrap_or(std::cmp::Ordering::Equal));

    Ok(result)
}
