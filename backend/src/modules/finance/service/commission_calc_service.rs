//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::Datelike;

use crate::modules::finance::entity::{commission_rule, commission_tier, commission_result};
use crate::modules::crm::entity::{contract, contract_commission_member, contract_payment_plan};
use crate::modules::system::entity::{admin, admin_post_merge, admin_dept_merge};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionResultVO {
    pub id: Option<i64>,
    pub contract_id: Option<i64>,
    pub contract_name: Option<String>,
    pub rule_id: i64,
    pub rule_name: Option<String>,
    pub rule_type: i32,
    pub user_id: i64,
    pub user_name: Option<String>,
    pub calc_base_amount: f64,
    pub commission_rate: f64,
    pub share_ratio: Option<f64>,
    pub commission_amount: f64,
    pub trigger_condition: i32,
    pub period_year: i32,
    pub period_month: i32,
}

pub async fn calc_on_contract_sign(
    db: &DatabaseConnection,
    contract_id: i64,
) -> Result<Vec<commission_result::Model>, String> {
    let contract_model = contract::Entity::find_by_id(contract_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "合同不存在".to_string())?;

    let plan = _get_contract_plan(db, contract_id).await?;
    let plan = match plan {
        Some(p) => p,
        None => return Ok(Vec::new()),
    };

    let calc_amount = contract_model.total_amount.unwrap_or_default();

    let results = _calc_for_contract(
        db,
        &contract_model,
        &plan,
        2,
        calc_amount,
        Some(contract_id),
    )
    .await?;

    let mut inserted = Vec::new();
    for result in results {
        let active_model = _result_to_active_model(result);
        let res = active_model.insert(db).await.map_err(|e| e.to_string())?;
        let inserted_model = commission_result::Entity::find_by_id(res.id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "插入提成结果失败".to_string())?;
        inserted.push(inserted_model);
    }

    Ok(inserted)
}

pub async fn calc_on_payment(
    db: &DatabaseConnection,
    payment_id: i64,
    payment_amount: Decimal,
) -> Result<Vec<commission_result::Model>, String> {
    let payment = contract_payment_plan::Entity::find_by_id(payment_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "回款记录不存在".to_string())?;

    let contract_id = payment.contract_id.ok_or_else(|| "回款记录无关联合同".to_string())?;

    let contract_model = contract::Entity::find_by_id(contract_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "合同不存在".to_string())?;

    let plan = _get_contract_plan(db, contract_id).await?;
    let plan = match plan {
        Some(p) => p,
        None => return Ok(Vec::new()),
    };

    let trigger_condition = plan.trigger_condition.unwrap_or(1);

    let results = _calc_for_contract(
        db,
        &contract_model,
        &plan,
        trigger_condition,
        payment_amount,
        Some(payment_id),
    )
    .await?;

    let mut inserted = Vec::new();
    for result in results {
        let active_model = _result_to_active_model(result);
        let res = active_model.insert(db).await.map_err(|e| e.to_string())?;
        let inserted_model = commission_result::Entity::find_by_id(res.id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "插入提成结果失败".to_string())?;
        inserted.push(inserted_model);
    }

    Ok(inserted)
}

pub async fn calc_monthly_settlement(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
) -> Result<Vec<commission_result::Model>, String> {
    // 委托调用 team_commission_service::calc_monthly_settlement 完成实际结算
    // 该函数将团队提成金额写回 salary_record.team_commission_amount，并返回更新的记录数
    let updated_count = crate::modules::finance::service::team_commission_service::calc_monthly_settlement(
        db, year, month,
    )
    .await?;

    log::info!(
        "[commission_calc] 月度团队提成结算完成 year={} month={} 更新记录数={}",
        year, month, updated_count
    );

    // 原函数签名返回 Vec<commission_result::Model>，但团队提成结果已直接写入
    // salary_record.team_commission_amount，不再单独生成 commission_result 记录。
    // controller 仅需知道是否成功，因此返回空 Vec 表示成功且无需返回明细。
    Ok(Vec::new())
}

pub async fn preview_contract_commission(
    db: &DatabaseConnection,
    contract_id: i64,
) -> Result<Vec<CommissionResultVO>, String> {
    let contract_model = contract::Entity::find_by_id(contract_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "合同不存在".to_string())?;

    let plan = _get_contract_plan(db, contract_id).await?;
    let plan = match plan {
        Some(p) => p,
        None => return Ok(Vec::new()),
    };

    let calc_amount = contract_model.total_amount.unwrap_or_default();
    let trigger_condition = plan.trigger_condition.unwrap_or(2);

    let results = _calc_for_contract(
        db,
        &contract_model,
        &plan,
        trigger_condition,
        calc_amount,
        None,
    )
    .await?;

    let vo_list = results
        .into_iter()
        .map(|r| CommissionResultVO {
            id: None,
            contract_id: r.contract_id,
            contract_name: r.contract_name,
            rule_id: r.rule_id,
            rule_name: r.rule_name,
            rule_type: r.rule_type,
            user_id: r.user_id,
            user_name: r.user_name,
            calc_base_amount: r.calc_base_amount.to_f64().unwrap_or_default(),
            commission_rate: r.commission_rate.to_f64().unwrap_or_default(),
            share_ratio: r.share_ratio.map(|d| d.to_f64().unwrap_or_default()),
            commission_amount: r.commission_amount.to_f64().unwrap_or_default(),
            trigger_condition: r.trigger_condition,
            period_year: r.period_year,
            period_month: r.period_month,
        })
        .collect();

    Ok(vo_list)
}

async fn _get_contract_plan(
    db: &DatabaseConnection,
    contract_id: i64,
) -> Result<Option<commission_rule::Model>, String> {
    let contract_model = contract::Entity::find_by_id(contract_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "合同不存在".to_string())?;

    if let Some(rule_id) = contract_model.commission_rule_id {
        let rule = commission_rule::Entity::find_by_id(rule_id)
            .filter(commission_rule::Column::Enabled.eq(1))
            .filter(commission_rule::Column::Deleted.eq(0))
            .one(db)
            .await
            .map_err(|e| e.to_string())?;
        if rule.is_some() {
            return Ok(rule);
        }
    }

    let default_plan = crate::modules::finance::model::commission_rule::CommissionRuleModel::find_default(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(default_plan)
}

async fn _find_tier_rate(
    db: &DatabaseConnection,
    rule_id: i64,
    amount: Decimal,
) -> Result<(Decimal, Option<Decimal>, Option<Decimal>), String> {
    let tiers = commission_tier::Entity::find()
        .filter(commission_tier::Column::RuleId.eq(rule_id))
        .order_by_asc(commission_tier::Column::Sort)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    for tier in tiers {
        let min = tier.min_amount;
        let max = tier.max_amount;

        let matches = match max {
            Some(max_val) => amount >= min && amount < max_val,
            None => amount >= min,
        };

        if matches {
            return Ok((tier.commission_rate, Some(min), max));
        }
    }

    Ok((Decimal::from(0), None, None))
}

async fn _calc_for_contract(
    db: &DatabaseConnection,
    contract_model: &contract::Model,
    plan: &commission_rule::Model,
    trigger_condition: i32,
    calc_amount: Decimal,
    source_id: Option<i64>,
) -> Result<Vec<commission_result::Model>, String> {
    let rule_type = plan.rule_type.unwrap_or(1);
    let category = plan.commission_category;
    let now = chrono::Utc::now().naive_utc();
    let period_year = now.date().year();
    let period_month = now.date().month() as i32;

    let (rate, tier_min, tier_max) = _find_tier_rate(db, plan.id, calc_amount).await?;

    let mut results = Vec::new();

    // v2 分发逻辑：基于 commission_category 决定计算路径
    // 即时计算（合同签订/回款触发）仅处理 category=1（个人提成）和 category=6（利润提成）
    // category=2/3/4/5（管理分润/团队奖金/资金池/再分配）由月度结算 calc_monthly_settlement 统一处理
    let should_calc_now = matches!(category, 1 | 6);
    if !should_calc_now {
        return Ok(results);
    }

    // 计算基数：category=6（利润提成）使用毛利，其他使用回款/合同额
    // 注：合同表暂无成本字段，cost_amount 暂为 0，毛利 = 回款额 - 0 = 回款额
    let (calc_base, cost_amount) = if category == 6 {
        let cost = Decimal::ZERO; // TODO: 合同表扩展成本字段后替换
        let profit = calc_amount - cost;
        if profit < Decimal::ZERO {
            (Decimal::ZERO, Some(cost))
        } else {
            (profit, Some(cost))
        }
    } else {
        (calc_amount, None)
    };

    // 根据 rule_type 区分单人 vs 团队分成（保留旧逻辑兼容）
    match rule_type {
        1 => {
            let user_id = contract_model.assigned_to.unwrap_or(0);
            let mut user_name = None;
            let mut user_post_id = None;
            let mut department_id = None;

            if user_id > 0 {
                if let Some(user) = admin::Entity::find_by_id(user_id)
                    .one(db)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    user_name = user.nick_name.or(user.user_name);
                }

                // 查询用户岗位（admin_post_merge 取第一条）
                if let Some(merge) = admin_post_merge::Entity::find()
                    .filter(admin_post_merge::Column::AdminId.eq(user_id))
                    .one(db)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    user_post_id = merge.post_id;
                }

                // 查询用户部门（admin_dept_merge 取第一条）
                if let Some(merge) = admin_dept_merge::Entity::find()
                    .filter(admin_dept_merge::Column::AdminId.eq(user_id))
                    .one(db)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    department_id = merge.dept_id;
                }
            }

            let mut commission_amount = calc_base * rate;
            // v2: 应用单笔封顶
            if let Some(cap) = plan.commission_cap {
                if commission_amount > cap {
                    commission_amount = cap;
                }
            }

            let result = commission_result::Model {
                id: 0,
                salary_record_id: None,
                contract_id: Some(contract_model.id),
                contract_name: contract_model.title.clone(),
                rule_id: plan.id,
                rule_name: plan.rule_name.clone(),
                rule_type,
                commission_category: Some(category),
                beneficiary_role: Some(plan.beneficiary_role),
                manager_level: None,
                allocate_status: Some(0),
                allocated_amount: Some(Decimal::ZERO),
                pool_id: None,
                cost_amount,
                user_id,
                user_name,
                user_post_id,
                department_id,
                calc_base_amount: calc_base,
                tier_min_amount: tier_min,
                tier_max_amount: tier_max,
                commission_rate: rate,
                share_ratio: None,
                commission_amount,
                trigger_condition,
                trigger_source_id: source_id,
                period_year,
                period_month,
                settled: 0,
                remark: None,
                create_time: Some(now),
            };

            results.push(result);
        }
        2 => {
            let members = contract_commission_member::Entity::find()
                .filter(contract_commission_member::Column::ContractId.eq(contract_model.id))
                .order_by_asc(contract_commission_member::Column::Sort)
                .all(db)
                .await
                .map_err(|e| e.to_string())?;

            for member in members {
                let share_ratio = member.share_ratio;
                let mut commission_amount = calc_base * rate * share_ratio;
                // v2: 应用单笔封顶（按个人封顶）
                if let Some(cap) = plan.commission_cap {
                    if commission_amount > cap {
                        commission_amount = cap;
                    }
                }

                let result = commission_result::Model {
                    id: 0,
                    salary_record_id: None,
                    contract_id: Some(contract_model.id),
                    contract_name: contract_model.title.clone(),
                    rule_id: plan.id,
                    rule_name: plan.rule_name.clone(),
                    rule_type,
                    commission_category: Some(category),
                    beneficiary_role: Some(plan.beneficiary_role),
                    manager_level: None,
                    allocate_status: Some(0),
                    allocated_amount: Some(Decimal::ZERO),
                    pool_id: None,
                    cost_amount,
                    user_id: member.user_id,
                    user_name: member.user_name.clone(),
                    user_post_id: None,
                    department_id: None,
                    calc_base_amount: calc_base,
                    tier_min_amount: tier_min,
                    tier_max_amount: tier_max,
                    commission_rate: rate,
                    share_ratio: Some(share_ratio),
                    commission_amount,
                    trigger_condition,
                    trigger_source_id: source_id,
                    period_year,
                    period_month,
                    settled: 0,
                    remark: None,
                    create_time: Some(now),
                };

                results.push(result);
            }
        }
        _ => {
            // 旧 rule_type=3/4/5 已由月度结算统一处理，即时触发跳过
        }
    }

    Ok(results)
}

fn _result_to_active_model(result: commission_result::Model) -> commission_result::ActiveModel {
    commission_result::ActiveModel {
        salary_record_id: Set(result.salary_record_id),
        contract_id: Set(result.contract_id),
        contract_name: Set(result.contract_name),
        rule_id: Set(result.rule_id),
        rule_name: Set(result.rule_name),
        rule_type: Set(result.rule_type),
        commission_category: Set(result.commission_category),
        beneficiary_role: Set(result.beneficiary_role),
        manager_level: Set(result.manager_level),
        allocate_status: Set(result.allocate_status),
        allocated_amount: Set(result.allocated_amount),
        pool_id: Set(result.pool_id),
        cost_amount: Set(result.cost_amount),
        user_id: Set(result.user_id),
        user_name: Set(result.user_name),
        user_post_id: Set(result.user_post_id),
        department_id: Set(result.department_id),
        calc_base_amount: Set(result.calc_base_amount),
        tier_min_amount: Set(result.tier_min_amount),
        tier_max_amount: Set(result.tier_max_amount),
        commission_rate: Set(result.commission_rate),
        share_ratio: Set(result.share_ratio),
        commission_amount: Set(result.commission_amount),
        trigger_condition: Set(result.trigger_condition),
        trigger_source_id: Set(result.trigger_source_id),
        period_year: Set(result.period_year),
        period_month: Set(result.period_month),
        settled: Set(result.settled),
        remark: Set(result.remark),
        create_time: Set(result.create_time),
        ..Default::default()
    }
}
