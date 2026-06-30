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
use sea_orm::sea_query::Expr;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use std::collections::HashMap;

use crate::modules::finance::entity::{salary_record, commission_detail, commission_rule, commission_tier};
use crate::modules::finance::model::salary::{
    SalaryRecordDTO, SalaryDetailDTO, CommissionDetailDTO, SalaryQuery, SalaryUpdateDTO, SalarySummaryDTO,
};
use crate::modules::crm::entity::{contract, contract_payment_plan};
use crate::modules::system::entity::{admin, admin_dept_merge, admin_post_merge, dept, post};

/// 分页列表
pub async fn get_list(
    db: &DatabaseConnection,
    query: SalaryQuery,
) -> Result<(Vec<SalaryRecordDTO>, i64), String> {
    let mut stmt = salary_record::Entity::find()
        .filter(salary_record::Column::Deleted.eq(0));

    if let Some(year) = query.year {
        stmt = stmt.filter(salary_record::Column::Year.eq(year));
    }
    if let Some(month) = query.month {
        stmt = stmt.filter(salary_record::Column::Month.eq(month));
    }
    if let Some(status) = query.status {
        stmt = stmt.filter(salary_record::Column::Status.eq(status));
    }
    if let Some(employee_name) = &query.employee_name {
        stmt = stmt.filter(salary_record::Column::EmployeeName.contains(employee_name));
    }

    stmt = stmt.order_by_desc(salary_record::Column::Year)
        .order_by_desc(salary_record::Column::Month)
        .order_by_desc(salary_record::Column::CreateTime);

    let page = std::cmp::max(query.page.unwrap_or(1), 1);
    let page_size = std::cmp::max(query.page_size.unwrap_or(20), 1);

    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?;

    let dto_list: Vec<SalaryRecordDTO> = items.into_iter().map(SalaryRecordDTO::from).collect();

    Ok((dto_list, total))
}

/// 详情含提成明细
pub async fn get_detail(db: &DatabaseConnection, id: i64) -> Result<SalaryDetailDTO, String> {
    let record = salary_record::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "工资记录不存在".to_string())?;

    let dto: SalaryRecordDTO = record.into();

    let details = commission_detail::Entity::find()
        .filter(commission_detail::Column::SalaryRecordId.eq(id))
        .order_by_desc(commission_detail::Column::CreateTime)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let detail_dtos: Vec<CommissionDetailDTO> = details.into_iter().map(CommissionDetailDTO::from).collect();

    Ok(SalaryDetailDTO {
        record: dto,
        details: detail_dtos,
    })
}

/// 执行月度核算（核心逻辑）
pub async fn calculate(db: &DatabaseConnection, year: i32, month: i32) -> Result<i64, String> {
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

    // 1. 先删除该年月已有的"待审核"(status=0)工资记录及其提成明细
    let pending_records: Vec<salary_record::Model> = salary_record::Entity::find()
        .filter(salary_record::Column::Year.eq(year))
        .filter(salary_record::Column::Month.eq(month))
        .filter(salary_record::Column::Status.eq(0))
        .filter(salary_record::Column::Deleted.eq(0))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;

    let pending_ids: Vec<i64> = pending_records.iter().map(|r| r.id).collect();
    if !pending_ids.is_empty() {
        // 删除提成明细
        commission_detail::Entity::delete_many()
            .filter(commission_detail::Column::SalaryRecordId.is_in(pending_ids.clone()))
            .exec(&txn)
            .await
            .map_err(|e| e.to_string())?;
        // 删除工资记录
        salary_record::Entity::delete_many()
            .filter(salary_record::Column::Id.is_in(pending_ids))
            .exec(&txn)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 2. 查询当月完全回款的合同回款计划（received_amount >= plan_amount 且 actual_date 在指定月份）
    let payment_plans: Vec<contract_payment_plan::Model> = contract_payment_plan::Entity::find()
        .filter(contract_payment_plan::Column::ActualDate.gte(month_start))
        .filter(contract_payment_plan::Column::ActualDate.lte(month_end))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;

    // 过滤完全回款的计划
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

    // 3. 获取相关合同信息
    let contract_ids: Vec<i64> = fully_paid_plans
        .iter()
        .filter_map(|p| p.contract_id)
        .collect::<Vec<_>>();
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

    // 4. 按合同负责人(assigned_to)分组
    let mut employee_contracts: HashMap<i64, Vec<(i64, Decimal, Decimal, String)>> = HashMap::new();
    // key: employee_id, value: Vec<(contract_id, contract_amount, payment_amount, contract_name)>
    for plan in fully_paid_plans {
        if let Some(contract_id) = plan.contract_id {
            if let Some(contract_model) = contract_map.get(&contract_id) {
                if let Some(employee_id) = contract_model.assigned_to {
                    let contract_name = contract_model.title.clone().unwrap_or_default();
                    let contract_amount = contract_model.amount.unwrap_or_default();
                    let payment_amount = plan.received_amount.unwrap_or_default();
                    employee_contracts
                        .entry(employee_id)
                        .or_default()
                        .push((contract_id, contract_amount, payment_amount, contract_name));
                }
            }
        }
    }

    if employee_contracts.is_empty() {
        txn.commit().await.map_err(|e| e.to_string())?;
        return Ok(0);
    }

    // 5. 批量查询员工信息
    let employee_ids: Vec<i64> = employee_contracts.keys().cloned().collect();
    let admins = admin::Entity::find()
        .filter(admin::Column::Id.is_in(employee_ids.clone()))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;
    let mut admin_map: HashMap<i64, admin::Model> = HashMap::new();
    for a in admins {
        admin_map.insert(a.id, a);
    }

    // 查询员工部门关系
    let dept_merges = admin_dept_merge::Entity::find()
        .filter(admin_dept_merge::Column::AdminId.is_in(employee_ids.clone()))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;
    let mut admin_dept_map: HashMap<i64, i64> = HashMap::new();
    for dm in dept_merges {
        if let (Some(admin_id), Some(dept_id)) = (dm.admin_id, dm.dept_id) {
            admin_dept_map.entry(admin_id).or_insert(dept_id);
        }
    }

    // 查询员工岗位关系
    let post_merges = admin_post_merge::Entity::find()
        .filter(admin_post_merge::Column::AdminId.is_in(employee_ids.clone()))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;
    let mut admin_post_map: HashMap<i64, i64> = HashMap::new();
    for pm in post_merges {
        if let (Some(admin_id), Some(post_id)) = (pm.admin_id, pm.post_id) {
            admin_post_map.entry(admin_id).or_insert(post_id);
        }
    }

    // 查询所有启用的提成规则（含阶梯）
    let rules: Vec<commission_rule::Model> = commission_rule::Entity::find()
        .filter(commission_rule::Column::Enabled.eq(1))
        .filter(commission_rule::Column::Deleted.eq(0))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;

    let rule_ids: Vec<i64> = rules.iter().map(|r| r.id).collect();
    let mut rule_tiers_map: HashMap<i64, Vec<commission_tier::Model>> = HashMap::new();
    if !rule_ids.is_empty() {
        let all_tiers = commission_tier::Entity::find()
            .filter(commission_tier::Column::RuleId.is_in(rule_ids))
            .order_by_asc(commission_tier::Column::Sort)
            .all(&txn)
            .await
            .map_err(|e| e.to_string())?;
        for t in all_tiers {
            rule_tiers_map.entry(t.rule_id).or_default().push(t);
        }
    }

    let now = Utc::now().naive_utc();
    let mut generated_count: i64 = 0;

    // 6. 为每个员工匹配提成规则并计算
    for (employee_id, contracts) in employee_contracts.into_iter() {
        let admin_model = match admin_map.get(&employee_id) {
            Some(a) => a.clone(),
            None => continue,
        };
        let employee_name = admin_model
            .nick_name
            .clone()
            .or_else(|| admin_model.user_name.clone())
            .unwrap_or_default();

        let dept_id = admin_dept_map.get(&employee_id).cloned();
        let post_id = admin_post_map.get(&employee_id).cloned();

        // 查询部门名称
        let department_name = if let Some(did) = dept_id {
            dept::Entity::find_by_id(did)
                .one(&txn)
                .await
                .map_err(|e| e.to_string())?
                .and_then(|d| d.dept_name)
        } else {
            None
        };

        // 匹配提成规则（最优匹配）
        let matched_rule = match_rule(&rules, dept_id, post_id, month_start);

        let mut total_commission = Decimal::ZERO;
        let mut detail_models: Vec<commission_detail::ActiveModel> = Vec::new();

        for (contract_id, contract_amount, payment_amount, contract_name) in contracts {
            let (commission_amount, commission_rate, rule_name) = if let Some(rule) = &matched_rule {
                let tiers = rule_tiers_map.get(&rule.id).cloned().unwrap_or_default();
                let (amt, rate) = calculate_tier_commission(&tiers, payment_amount);
                (amt, rate, rule.rule_name.clone().unwrap_or_default())
            } else {
                (Decimal::ZERO, Decimal::ZERO, String::new())
            };

            total_commission += commission_amount;

            let detail_model = commission_detail::ActiveModel {
                salary_record_id: Set(0), // 稍后更新
                contract_id: Set(Some(contract_id)),
                contract_name: Set(Some(contract_name)),
                contract_amount: Set(Some(contract_amount)),
                payment_amount: Set(Some(payment_amount)),
                commission_base: Set(Some(payment_amount)),
                commission_rate: Set(Some(commission_rate)),
                commission_amount: Set(Some(commission_amount)),
                rule_name: Set(if rule_name.is_empty() { None } else { Some(rule_name) }),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            detail_models.push(detail_model);
        }

        // 创建工资记录
        let base_salary = Decimal::ZERO;
        let performance_bonus = Decimal::ZERO;
        let deduction_amount = Decimal::ZERO;
        let total_salary = base_salary + total_commission + performance_bonus - deduction_amount;

        let salary_model = salary_record::ActiveModel {
            employee_id: Set(employee_id),
            employee_name: Set(Some(employee_name)),
            department_name: Set(department_name),
            year: Set(year),
            month: Set(month),
            base_salary: Set(base_salary),
            commission_amount: Set(total_commission),
            performance_bonus: Set(performance_bonus),
            deduction_amount: Set(deduction_amount),
            total_salary: Set(total_salary),
            status: Set(Some(0)),
            remark: Set(None),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };

        let inserted_salary = salary_model.insert(&txn).await.map_err(|e| e.to_string())?;
        let salary_id = inserted_salary.id;

        // 更新明细的 salary_record_id 并插入
        for mut detail in detail_models {
            detail.salary_record_id = Set(salary_id);
            detail.insert(&txn).await.map_err(|e| e.to_string())?;
        }

        generated_count += 1;
    }

    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(generated_count)
}

/// 匹配提成规则（最优匹配）
/// 优先级: 部门+岗位 > 仅部门 > 仅岗位 > 通用规则
fn match_rule(
    rules: &[commission_rule::Model],
    dept_id: Option<i64>,
    post_id: Option<i64>,
    current_date: chrono::NaiveDate,
) -> Option<commission_rule::Model> {
    let mut best: Option<(&commission_rule::Model, i32)> = None;

    for rule in rules {
        // 检查生效日期
        if let Some(ed) = rule.effective_date {
            if ed > current_date {
                continue;
            }
        }
        // 检查失效日期
        if let Some(exp) = rule.expiry_date {
            if exp < current_date {
                continue;
            }
        }

        let rule_dept = rule.department_id;
        let rule_post = rule.post_id;

        let score = match (rule_dept, rule_post, dept_id, post_id) {
            // 部门+岗位都匹配
            (Some(rd), Some(rp), Some(d), Some(p)) if rd == d && rp == p => 4,
            // 仅部门匹配（规则只指定部门）
            (Some(rd), None, Some(d), _) if rd == d => 3,
            // 仅岗位匹配（规则只指定岗位）
            (None, Some(rp), _, Some(p)) if rp == p => 2,
            // 通用规则（都不指定）
            (None, None, _, _) => 1,
            _ => 0,
        };

        if score > 0 {
            if let Some((_, prev_score)) = best {
                if score > prev_score {
                    best = Some((rule, score));
                }
            } else {
                best = Some((rule, score));
            }
        }
    }

    best.map(|(r, _)| r.clone())
}

/// 按阶梯计算提成
fn calculate_tier_commission(
    tiers: &[commission_tier::Model],
    base: Decimal,
) -> (Decimal, Decimal) {
    for tier in tiers {
        let min = tier.min_amount;
        let max = tier.max_amount;

        let in_range = base >= min && match max {
            Some(m) => base < m,
            None => true,
        };

        if in_range {
            let amount = base * tier.commission_rate;
            return (amount, tier.commission_rate);
        }
    }
    (Decimal::ZERO, Decimal::ZERO)
}

/// 手动调整
pub async fn update(db: &DatabaseConnection, dto: SalaryUpdateDTO) -> Result<(), String> {
    let mut model: salary_record::ActiveModel = salary_record::Entity::find_by_id(dto.id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "工资记录不存在".to_string())?
        .into();

    let now = Utc::now().naive_utc();

    if let Some(base_salary) = dto.base_salary {
        model.base_salary = Set(Decimal::from_f64(base_salary).unwrap_or_default());
    }
    if let Some(performance_bonus) = dto.performance_bonus {
        model.performance_bonus = Set(Decimal::from_f64(performance_bonus).unwrap_or_default());
    }
    if let Some(deduction_amount) = dto.deduction_amount {
        model.deduction_amount = Set(Decimal::from_f64(deduction_amount).unwrap_or_default());
    }
    if let Some(remark) = dto.remark {
        model.remark = Set(Some(remark));
    }

    // 重新计算应发工资
    let existing = salary_record::Entity::find_by_id(dto.id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "工资记录不存在".to_string())?;

    let base = dto.base_salary.map(|v| Decimal::from_f64(v).unwrap_or_default()).unwrap_or(existing.base_salary);
    let commission = existing.commission_amount;
    let bonus = dto.performance_bonus.map(|v| Decimal::from_f64(v).unwrap_or_default()).unwrap_or(existing.performance_bonus);
    let deduction = dto.deduction_amount.map(|v| Decimal::from_f64(v).unwrap_or_default()).unwrap_or(existing.deduction_amount);
    let total = base + commission + bonus - deduction;
    model.total_salary = Set(total);

    if let Some(uid) = dto.updated_by {
        model.updated_by = Set(Some(uid));
    }
    model.update_time = Set(Some(now));

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    model.update(&txn).await.map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 审核（状态: 0=待审核 -> 1=已审核）
pub async fn approve(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    let record = salary_record::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "工资记录不存在".to_string())?;

    let status = record.status.unwrap_or(0);
    if status != 0 {
        return Err("只有待审核状态的工资记录才能审核".to_string());
    }

    let now = Utc::now().naive_utc();
    let mut model: salary_record::ActiveModel = record.into();
    model.status = Set(Some(1));
    model.update_time = Set(Some(now));

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    model.update(&txn).await.map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 批量审核
pub async fn batch_approve(db: &DatabaseConnection, ids: Vec<i64>) -> Result<(), String> {
    if ids.is_empty() {
        return Err("请选择要审核的记录".to_string());
    }

    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    salary_record::Entity::update_many()
        .filter(salary_record::Column::Id.is_in(ids.clone()))
        .filter(salary_record::Column::Status.eq(0))
        .col_expr(salary_record::Column::Status, Expr::value(1))
        .col_expr(salary_record::Column::UpdateTime, Expr::value(now))
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;

    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 发放（状态: 1=已审核 -> 2=已发放）
pub async fn pay(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    let record = salary_record::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "工资记录不存在".to_string())?;

    let status = record.status.unwrap_or(0);
    if status != 1 {
        return Err("只有已审核状态的工资记录才能发放".to_string());
    }

    let now = Utc::now().naive_utc();
    let mut model: salary_record::ActiveModel = record.into();
    model.status = Set(Some(2));
    model.update_time = Set(Some(now));

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    model.update(&txn).await.map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 批量发放
pub async fn batch_pay(db: &DatabaseConnection, ids: Vec<i64>) -> Result<(), String> {
    if ids.is_empty() {
        return Err("请选择要发放的记录".to_string());
    }

    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    salary_record::Entity::update_many()
        .filter(salary_record::Column::Id.is_in(ids.clone()))
        .filter(salary_record::Column::Status.eq(1))
        .col_expr(salary_record::Column::Status, Expr::value(2))
        .col_expr(salary_record::Column::UpdateTime, Expr::value(now))
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;

    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 汇总
pub async fn get_summary(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
) -> Result<SalarySummaryDTO, String> {
    let records = salary_record::Entity::find()
        .filter(salary_record::Column::Year.eq(year))
        .filter(salary_record::Column::Month.eq(month))
        .filter(salary_record::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let count = records.len() as i64;
    let total_base: Decimal = records.iter().map(|r| r.base_salary).sum();
    let total_commission: Decimal = records.iter().map(|r| r.commission_amount).sum();
    let total_bonus: Decimal = records.iter().map(|r| r.performance_bonus).sum();
    let total_deduction: Decimal = records.iter().map(|r| r.deduction_amount).sum();
    let total_salary: Decimal = records.iter().map(|r| r.total_salary).sum();

    Ok(SalarySummaryDTO {
        total_base: total_base.to_f64().unwrap_or_default(),
        total_commission: total_commission.to_f64().unwrap_or_default(),
        total_bonus: total_bonus.to_f64().unwrap_or_default(),
        total_deduction: total_deduction.to_f64().unwrap_or_default(),
        total_salary: total_salary.to_f64().unwrap_or_default(),
        count,
    })
}
