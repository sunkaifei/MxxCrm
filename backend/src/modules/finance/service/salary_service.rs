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
use chrono::{Utc, Datelike};
use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use std::collections::HashMap;

use crate::modules::finance::entity::{
    salary_record, salary_config, salary_calc_log,
    commission_detail, commission_rule, commission_tier,
};
use crate::modules::finance::model::salary::{
    SalaryRecordDTO, SalaryDetailDTO, CommissionDetailDTO, SalaryQuery, SalaryUpdateDTO, SalarySummaryDTO,
    SalaryTrendQuery, SalaryTrendMonthlyPointDTO, SalaryTrendDeptPointDTO,
    SalaryTrendEmployeePointDTO, SalaryTrendSummaryDTO,
};
use crate::modules::crm::entity::{contract, contract_payment_plan};
use crate::modules::system::entity::{admin, admin_dept_merge, admin_post_merge, dept, post};
use crate::modules::statistics::entity::{performance_plan, plan_monthly_target};

/// 分页列表
pub async fn get_list(
    db: &DatabaseConnection,
    query: SalaryQuery,
    user_id: i64,
) -> Result<(Vec<SalaryRecordDTO>, i64), String> {
    let mut stmt = salary_record::Entity::find()
        .filter(salary_record::Column::Deleted.eq(0));

    // 数据权限过滤
    let (scope, allowed_ids) = resolve_data_scope(db, user_id).await?;
    match scope {
        SalaryDataScope::All => {},
        SalaryDataScope::SelfAndSubordinates | SalaryDataScope::SelfOnly => {
            stmt = stmt.filter(salary_record::Column::EmployeeId.is_in(allowed_ids));
        }
    }

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
/// trigger_type: 0=手动触发，1=定时任务自动触发
/// operator_id/operator_name: 操作人信息（自动触发时为 0/"系统"）
pub async fn calculate(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
    trigger_type: i32,
    operator_id: i64,
    operator_name: &str,
) -> Result<i64, String> {
    let start_time = std::time::Instant::now();

    // 记录核算日志（开始）
    let log_id = insert_calc_log(
        db, year, month, trigger_type, operator_id, operator_name,
    ).await?;

    // 执行核算，捕获结果写入日志
    match calculate_inner(db, year, month).await {
        Ok(count) => {
            // 工资记录已提交，归集团队提成到管理者的 salary_record.team_commission_amount
            // 团队提成服务使用独立事务，基于当月已回款合同沿 direct_manager_id 向上计算
            let _ = crate::modules::finance::service::team_commission_service::calc_monthly_settlement(
                db, year, month,
            ).await;
            let elapsed = start_time.elapsed().as_millis() as i64;
            update_calc_log_success(db, log_id, count, elapsed).await.ok();

            // 核算成功后通知所有财务角色用户
            let finance_users = find_users_by_role_key(db, "finance").await.unwrap_or_default();
            let notify_title = format!("{}年{}月工资核算已完成", year, month);
            let notify_content = format!("{}年{}月工资核算已完成，共生成{}条工资记录", year, month, count);
            for fin_id in finance_users {
                let _ = NotificationService::send_system_notification(
                    db, fin_id,
                    notify_title.clone(),
                    notify_content.clone(),
                    2, // 通知类型 2=审批通知
                    Some("/finance/salary".to_string()),
                ).await;
            }

            Ok(count)
        }
        Err(e) => {
            let elapsed = start_time.elapsed().as_millis() as i64;
            update_calc_log_failure(db, log_id, &e, elapsed).await.ok();
            Err(e)
        }
    }
}

/// 核算内部实现
async fn calculate_inner(db: &DatabaseConnection, year: i32, month: i32) -> Result<i64, String> {
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

    // 6. 批量查询员工底薪配置（优先精确匹配 year+month，其次 year+month=null 全年配置）
    let mut salary_config_map: HashMap<i64, salary_config::Model> = HashMap::new();
    let configs = salary_config::Entity::find()
        .filter(salary_config::Column::EmployeeId.is_in(employee_ids.clone()))
        .filter(salary_config::Column::Year.eq(year))
        .filter(salary_config::Column::Status.eq(1))
        .filter(salary_config::Column::Deleted.eq(0))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;
    // 优先 month 精确匹配，其次 month=null 的全年配置
    let mut fallback_configs: HashMap<i64, salary_config::Model> = HashMap::new();
    for cfg in configs {
        if let Some(m) = cfg.month {
            if m == month {
                salary_config_map.insert(cfg.employee_id, cfg);
            }
        } else {
            fallback_configs.insert(cfg.employee_id, cfg);
        }
    }
    for emp_id in &employee_ids {
        salary_config_map.entry(*emp_id).or_insert_with(|| {
            fallback_configs.get(emp_id).cloned().unwrap_or_default()
        });
    }

    // 7. 批量查询员工业绩计划完成率（用于绩效系数计算）
    // 绩效系数 = min(业绩完成率, 1.5)，无计划数据时系数为 0
    let mut performance_rate_map: HashMap<i64, Decimal> = HashMap::new();
    let plans = performance_plan::Entity::find()
        .filter(performance_plan::Column::EmployeeId.is_in(employee_ids.clone()))
        .filter(performance_plan::Column::Year.eq(year))
        .filter(performance_plan::Column::Status.eq(2)) // 已审批通过的计划
        .filter(performance_plan::Column::Deleted.eq(0))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;
    if !plans.is_empty() {
        let plan_ids: Vec<i64> = plans.iter().map(|p| p.id).collect();
        let targets = plan_monthly_target::Entity::find()
            .filter(plan_monthly_target::Column::PlanId.is_in(plan_ids))
            .filter(plan_monthly_target::Column::Deleted.eq(0))
            .all(&txn)
            .await
            .map_err(|e| e.to_string())?;
        // 计算每个员工的月度目标（当月）
        let mut plan_target_map: HashMap<i64, Decimal> = HashMap::new(); // employee_id -> month_target
        let plan_emp_map: HashMap<i64, i64> = plans.iter()
            .map(|p| (p.id, p.employee_id))
            .collect();
        for t in &targets {
            if t.month == month {
                if let Some(emp_id) = plan_emp_map.get(&t.plan_id) {
                    let target = t.payment_target_amount.unwrap_or(Decimal::ZERO);
                    *plan_target_map.entry(*emp_id).or_insert(Decimal::ZERO) += target;
                }
            }
        }
        // 计算完成率 = 当月实际回款 / 当月目标
        for (emp_id, contracts) in &employee_contracts {
            let actual: Decimal = contracts.iter()
                .map(|(_, _, payment, _)| *payment)
                .sum();
            if let Some(target) = plan_target_map.get(emp_id) {
                if *target > Decimal::ZERO {
                    let rate = actual / target;
                    // 完成率上限 1.5（150%）
                    let cap = Decimal::from_f64(1.5).unwrap_or(Decimal::ONE);
                    let capped = std::cmp::min(rate, cap);
                    performance_rate_map.insert(*emp_id, capped);
                }
            }
        }
    }

    // 8. 为每个员工匹配提成规则并计算
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
            let (commission_amount, commission_rate, rule_name, commission_base) = if let Some(rule) = &matched_rule {
                let tiers = rule_tiers_map.get(&rule.id).cloned().unwrap_or_default();
                // 根据规则 calc_base_field 决定提成基数
                let base = resolve_commission_base(&rule.calc_base_field, contract_amount, payment_amount);
                let (amt, rate) = calculate_tier_commission(&tiers, base, rule.tier_mode);
                (amt, rate, rule.rule_name.clone().unwrap_or_default(), base)
            } else {
                (Decimal::ZERO, Decimal::ZERO, String::new(), payment_amount)
            };

            total_commission += commission_amount;

            let detail_model = commission_detail::ActiveModel {
                salary_record_id: Set(0), // 稍后更新
                contract_id: Set(Some(contract_id)),
                contract_name: Set(Some(contract_name)),
                contract_amount: Set(Some(contract_amount)),
                payment_amount: Set(Some(payment_amount)),
                commission_base: Set(Some(commission_base)),
                commission_rate: Set(Some(commission_rate)),
                commission_amount: Set(Some(commission_amount)),
                rule_name: Set(if rule_name.is_empty() { None } else { Some(rule_name) }),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            detail_models.push(detail_model);
        }

        // 从底薪配置读取（含岗位津贴）
        let config = salary_config_map.get(&employee_id);
        let base_salary = config.map(|c| c.base_salary).unwrap_or(Decimal::ZERO);
        let position_allowance = config.and_then(|c| c.position_allowance).unwrap_or(Decimal::ZERO);
        let base_with_allowance = base_salary + position_allowance;

        // 绩效奖金计算：
        // - 若配置了 performance_coefficient（手动系数），直接使用
        // - 否则按业绩计划完成率自动计算：绩效奖金 = 绩效基数 × min(完成率, 1.5)
        // - 无业绩计划时绩效为 0
        let performance_bonus = if let Some(cfg) = config {
            let perf_base = cfg.performance_base.unwrap_or(Decimal::ZERO);
            if perf_base > Decimal::ZERO {
                if let Some(manual_coeff) = cfg.performance_coefficient {
                    perf_base * manual_coeff
                } else if let Some(rate) = performance_rate_map.get(&employee_id) {
                    perf_base * rate
                } else {
                    Decimal::ZERO
                }
            } else {
                Decimal::ZERO
            }
        } else {
            Decimal::ZERO
        };

        // 扣款：从考勤记录计算（若有），否则为 0
        let mut deduction_amount = Decimal::ZERO;
        if let Ok(attendance_result) = crate::modules::finance::service::attendance_service::calculate_deduction(db, employee_id, year, month).await {
            deduction_amount = Decimal::from_f64(attendance_result.deduction_amount).unwrap_or(Decimal::ZERO);
        }

        // 社保公积金计算（若有员工配置）
        // P1-6 修复：未配置时记录告警日志，便于管理员排查
        let insurance_result = match crate::modules::finance::service::insurance_service::calculate_monthly_insurance(db, employee_id, year, month).await {
            Ok(r) => r,
            Err(e) => {
                log::warn!("[salary] 员工{}({}) 社保配置缺失，按0处理：{}", employee_id, employee_name, e);
                Default::default()
            }
        };

        // 团队提成：当前员工的工资记录中 team_commission_amount=0
        // 管理者的团队提成由 calculate() 调用 team_commission_service::calc_monthly_settlement 在核算后统一归集
        let team_commission_amount = Decimal::ZERO;

        // 应发工资 = 底薪 + 岗位津贴 + 提成 + 绩效 + 团队提成 - 扣款
        let total_salary = base_with_allowance + total_commission + performance_bonus + team_commission_amount - deduction_amount;

        // 个税计算：应纳税所得额 = 应发工资 - 个人社保 - 个人公积金 - 5000(起征点) - 专项附加扣除
        // P1-6 修复：未配置时记录告警日志
        let taxable_income = total_salary - insurance_result.social_insurance_personal - insurance_result.housing_fund_personal;
        let tax_result = match crate::modules::finance::service::tax_service::calculate_monthly_tax(
            db, employee_id, year, month, taxable_income.to_f64().unwrap_or(0.0),
        ).await {
            Ok(r) => r,
            Err(e) => {
                log::warn!("[salary] 员工{}({}) 个税配置缺失，按0处理：{}", employee_id, employee_name, e);
                Default::default()
            }
        };

        // P1-3: 自定义工资项引擎接入 calculate 主流程
        // 查询启用的自定义项，按 calc_mode 求值，并按 item_type/is_pretax 参与 total_salary 和个税计算
        let custom_items = match crate::modules::finance::entity::salary_item::Entity::find()
            .filter(crate::modules::finance::entity::salary_item::Column::Enabled.eq(1))
            .all(db).await {
            Ok(items) => items,
            Err(e) => {
                log::warn!("[salary] 员工{} 自定义项查询失败：{}", employee_id, e);
                Vec::new()
            }
        };

        let mut custom_add_pretax = Decimal::ZERO;   // 税前增项
        let mut custom_sub_pretax = Decimal::ZERO;   // 税前减项
        let mut custom_add_posttax = Decimal::ZERO;  // 税后增项
        let mut custom_sub_posttax = Decimal::ZERO;  // 税后减项
        let mut custom_taxable_pretax = Decimal::ZERO; // 应税的自定义项（用于重算个税）

        let mut context: HashMap<String, Decimal> = HashMap::new();
        context.insert("baseSalary".to_string(), base_salary);
        context.insert("positionAllowance".to_string(), position_allowance);
        context.insert("commission".to_string(), total_commission);
        context.insert("performanceBonus".to_string(), performance_bonus);
        context.insert("deduction".to_string(), deduction_amount);
        context.insert("totalSalary".to_string(), total_salary);
        context.insert("socialPersonal".to_string(), insurance_result.social_insurance_personal);
        context.insert("housingPersonal".to_string(), insurance_result.housing_fund_personal);

        let mut custom_value_models: Vec<crate::modules::finance::entity::salary_item_value::ActiveModel> = Vec::new();

        for item in &custom_items {
            let amount = if item.calc_mode.unwrap_or(1) == 2 {
                // 公式模式
                let formula = item.formula.as_deref().unwrap_or("");
                crate::modules::finance::service::salary_item_service::calculate_formula(
                    formula, &context, item.default_value,
                )
            } else {
                // 手动模式：使用默认值（实际值由 save_item_values 后续覆盖）
                item.default_value
            };

            if amount == Decimal::ZERO { continue; }

            let item_type = item.item_type.unwrap_or(1); // 1=增项, 2=减项
            let is_pretax = item.is_pretax.unwrap_or(1) == 1; // 1=税前, 0=税后
            let is_taxable = item.is_taxable.unwrap_or(1) == 1; // 1=应税, 0=非应税

            match (item_type, is_pretax) {
                (1, true) => {
                    custom_add_pretax += amount;
                    if is_taxable { custom_taxable_pretax += amount; }
                }
                (2, true) => {
                    custom_sub_pretax += amount;
                    if is_taxable { custom_taxable_pretax -= amount; }
                }
                (1, false) => custom_add_posttax += amount,
                (2, false) => custom_sub_posttax += amount,
                _ => {}
            }

            // 收集自定义项值模型
            custom_value_models.push(crate::modules::finance::entity::salary_item_value::ActiveModel {
                salary_record_id: Set(0), // 后面更新
                item_id: Set(item.id),
                item_code: Set(Some(item.item_code.clone())),
                item_name: Set(Some(item.item_name.clone())),
                amount: Set(amount),
                is_taxable: Set(Some(if is_taxable { 1 } else { 0 })),
                ..Default::default()
            });
        }

        // 重算 total_salary（含税前自定义项）
        let total_salary = total_salary + custom_add_pretax - custom_sub_pretax;

        // 若有应税自定义项，重算个税
        let tax_result = if custom_taxable_pretax != Decimal::ZERO {
            let new_taxable = total_salary - insurance_result.social_insurance_personal - insurance_result.housing_fund_personal;
            match crate::modules::finance::service::tax_service::calculate_monthly_tax(
                db, employee_id, year, month, new_taxable.to_f64().unwrap_or(0.0),
            ).await {
                Ok(r) => r,
                Err(_) => tax_result,
            }
        } else {
            tax_result
        };

        // 实发工资 = 应发工资 - 个人社保 - 个人公积金 - 个税 + 税后增项 - 税后减项
        let net_salary = total_salary - insurance_result.social_insurance_personal - insurance_result.housing_fund_personal - tax_result.monthly_tax + custom_add_posttax - custom_sub_posttax;

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
            social_insurance_personal: Set(insurance_result.social_insurance_personal),
            housing_fund_personal: Set(insurance_result.housing_fund_personal),
            social_insurance_company: Set(insurance_result.social_insurance_company),
            housing_fund_company: Set(insurance_result.housing_fund_company),
            tax_amount: Set(tax_result.monthly_tax),
            net_salary: Set(net_salary),
            team_commission_amount: Set(team_commission_amount),
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

        // 保存自定义项值
        for mut cv in custom_value_models {
            cv.salary_record_id = Set(salary_id);
            let _ = cv.insert(&txn).await;
        }

        // 保存个税明细（在主事务内，保证原子性）
        let _ = crate::modules::finance::service::tax_service::save_tax_detail_in_conn(
            &txn, salary_id, employee_id, year, month, tax_result,
        ).await;

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
/// tier_mode: 0=单档命中(命中的阶梯按其比例全额计算), 1=累进(分段累计,类似个税), 2=超额递增(超额部分按高档率)
fn calculate_tier_commission(
    tiers: &[commission_tier::Model],
    base: Decimal,
    tier_mode: Option<i32>,
) -> (Decimal, Decimal) {
    if tiers.is_empty() || base <= Decimal::ZERO {
        return (Decimal::ZERO, Decimal::ZERO);
    }

    // 按 min_amount 升序排序（拷贝一份避免修改原切片）
    let mut sorted: Vec<&commission_tier::Model> = tiers.iter().collect();
    sorted.sort_by(|a, b| a.min_amount.cmp(&b.min_amount));

    let mode = tier_mode.unwrap_or(0);
    match mode {
        1 => {
            // 累进模式：分段累计，类似个税计算
            // 每段金额 = min(本档上限, base) - 本档下限，若为正则按本档比例计算
            let mut total = Decimal::ZERO;
            for tier in &sorted {
                let lower = tier.min_amount;
                let upper = tier.max_amount.unwrap_or(Decimal::MAX);
                if base <= lower {
                    break;
                }
                let segment = if base < upper { base - lower } else { upper - lower };
                if segment > Decimal::ZERO {
                    total += segment * tier.commission_rate;
                }
            }
            // 返回平均税率（用于记录）
            let avg_rate = if base > Decimal::ZERO {
                total / base
            } else {
                Decimal::ZERO
            };
            (total, avg_rate)
        }
        2 => {
            // 超额递增模式：超出部分按更高档率计算（类似超额累进）
            // 与"累进"在数学上等价，区分主要为业务语义（超额递增强调超出阈值部分）
            let mut total = Decimal::ZERO;
            let mut prev_upper = Decimal::ZERO;
            for tier in &sorted {
                let lower = tier.min_amount.max(prev_upper);
                let upper = tier.max_amount.unwrap_or(Decimal::MAX);
                if base <= lower {
                    break;
                }
                let segment = if base < upper { base - lower } else { upper - lower };
                if segment > Decimal::ZERO {
                    total += segment * tier.commission_rate;
                }
                prev_upper = upper;
            }
            let avg_rate = if base > Decimal::ZERO {
                total / base
            } else {
                Decimal::ZERO
            };
            (total, avg_rate)
        }
        _ => {
            // 默认单档命中：找到第一个命中的阶梯，按该档比例全额计算
            for tier in &sorted {
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
    }
}

/// 根据规则 calc_base_field 选择提成基数
/// - "contract_amount" 或 "contract": 使用合同总额
/// - "net_amount": 使用净回款额（payment_amount - 退款，暂用 payment_amount）
/// - "profit": 使用毛利（需 cost 字段，暂未支持，回退到 payment_amount）
/// - 其他或 None: 使用回款额（默认）
fn resolve_commission_base(
    calc_base_field: &Option<String>,
    contract_amount: Decimal,
    payment_amount: Decimal,
) -> Decimal {
    match calc_base_field.as_deref() {
        Some("contract_amount") | Some("contract") => contract_amount,
        Some("net_amount") => payment_amount, // 暂未扣减退款
        Some("profit") => payment_amount,     // 暂无成本字段
        _ => payment_amount,
    }
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

// ==================== 核算日志 ====================

/// 插入核算日志（开始状态）
async fn insert_calc_log(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
    trigger_type: i32,
    operator_id: i64,
    operator_name: &str,
) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let log = salary_calc_log::ActiveModel {
        year: Set(year),
        month: Set(month),
        trigger_type: Set(Some(trigger_type)),
        result: Set(None),
        generated_count: Set(Some(0)),
        error_message: Set(None),
        elapsed_ms: Set(None),
        operator_id: Set(Some(operator_id)),
        operator_name: Set(Some(operator_name.to_string())),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    let inserted = log.insert(db).await.map_err(|e| e.to_string())?;
    Ok(inserted.id)
}

/// 更新核算日志为成功
async fn update_calc_log_success(
    db: &DatabaseConnection,
    log_id: i64,
    count: i64,
    elapsed_ms: i64,
) -> Result<(), String> {
    let mut model: salary_calc_log::ActiveModel = salary_calc_log::Entity::find_by_id(log_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "核算日志不存在".to_string())?
        .into();
    model.result = Set(Some(1));
    model.generated_count = Set(Some(count));
    model.elapsed_ms = Set(Some(elapsed_ms));
    model.update(db).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 更新核算日志为失败
async fn update_calc_log_failure(
    db: &DatabaseConnection,
    log_id: i64,
    error: &str,
    elapsed_ms: i64,
) -> Result<(), String> {
    let mut model: salary_calc_log::ActiveModel = salary_calc_log::Entity::find_by_id(log_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "核算日志不存在".to_string())?
        .into();
    model.result = Set(Some(0));
    model.error_message = Set(Some(error.to_string()));
    model.elapsed_ms = Set(Some(elapsed_ms));
    model.update(db).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 查询核算日志列表
pub async fn get_calc_log_list(
    db: &DatabaseConnection,
    year: Option<i32>,
    month: Option<i32>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<salary_calc_log::Model>, i64), String> {
    let mut stmt = salary_calc_log::Entity::find();
    if let Some(y) = year {
        stmt = stmt.filter(salary_calc_log::Column::Year.eq(y));
    }
    if let Some(m) = month {
        stmt = stmt.filter(salary_calc_log::Column::Month.eq(m));
    }
    stmt = stmt.order_by_desc(salary_calc_log::Column::CreateTime);

    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?;
    Ok((items, total))
}

// ==================== 底薪配置 CRUD ====================

/// 查询底薪配置列表
pub async fn get_config_list(
    db: &DatabaseConnection,
    employee_id: Option<i64>,
    year: Option<i32>,
) -> Result<Vec<serde_json::Value>, String> {
    let mut stmt = salary_config::Entity::find()
        .filter(salary_config::Column::Deleted.eq(0));
    if let Some(eid) = employee_id {
        stmt = stmt.filter(salary_config::Column::EmployeeId.eq(eid));
    }
    if let Some(y) = year {
        stmt = stmt.filter(salary_config::Column::Year.eq(y));
    }
    let configs = stmt
        .order_by_asc(salary_config::Column::EmployeeId)
        .order_by_asc(salary_config::Column::Year)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    // 批量查询员工姓名
    let emp_ids: Vec<i64> = configs.iter().map(|c| c.employee_id).collect::<Vec<_>>();
    let mut name_map: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    if !emp_ids.is_empty() {
        let admins = admin::Entity::find()
            .filter(admin::Column::Id.is_in(emp_ids))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for a in admins {
            let name = a.nick_name.or(a.user_name).unwrap_or_default();
            name_map.insert(a.id, name);
        }
    }

    let result: Vec<serde_json::Value> = configs
        .into_iter()
        .map(|c| {
            let emp_name = name_map.get(&c.employee_id).cloned().unwrap_or_default();
            serde_json::json!({
                "id": c.id,
                "employeeId": c.employee_id,
                "employeeName": emp_name,
                "year": c.year,
                "month": c.month,
                "baseSalary": c.base_salary,
                "positionAllowance": c.position_allowance,
                "performanceBase": c.performance_base,
                "performanceCoefficient": c.performance_coefficient,
                "status": c.status,
            })
        })
        .collect();

    Ok(result)
}

/// 新增/更新底薪配置
pub async fn upsert_config(
    db: &DatabaseConnection,
    employee_id: i64,
    year: i32,
    month: Option<i32>,
    base_salary: f64,
    position_allowance: Option<f64>,
    performance_base: Option<f64>,
    performance_coefficient: Option<f64>,
) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // 查找是否已存在
    let mut existing = salary_config::Entity::find()
        .filter(salary_config::Column::EmployeeId.eq(employee_id))
        .filter(salary_config::Column::Year.eq(year))
        .filter(salary_config::Column::Deleted.eq(0))
        .all(&txn)
        .await
        .map_err(|e| e.to_string())?;
    // 精确匹配 month
    let matched = existing.iter().find(|c| c.month == month).cloned();

    if let Some(mut model) = matched {
        let mut active: salary_config::ActiveModel = model.clone().into();
        active.base_salary = Set(Decimal::from_f64(base_salary).unwrap_or_default());
        active.position_allowance = Set(position_allowance.map(|v| Decimal::from_f64(v).unwrap_or_default()));
        active.performance_base = Set(performance_base.map(|v| Decimal::from_f64(v).unwrap_or_default()));
        active.performance_coefficient = Set(performance_coefficient.map(|v| Decimal::from_f64(v).unwrap_or_default()));
        active.update_time = Set(Some(now));
        let updated = active.update(&txn).await.map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(updated.id)
    } else {
        let active = salary_config::ActiveModel {
            employee_id: Set(employee_id),
            year: Set(year),
            month: Set(month),
            base_salary: Set(Decimal::from_f64(base_salary).unwrap_or_default()),
            position_allowance: Set(position_allowance.map(|v| Decimal::from_f64(v).unwrap_or_default())),
            performance_base: Set(performance_base.map(|v| Decimal::from_f64(v).unwrap_or_default())),
            performance_coefficient: Set(performance_coefficient.map(|v| Decimal::from_f64(v).unwrap_or_default())),
            status: Set(Some(1)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        let inserted = active.insert(&txn).await.map_err(|e| e.to_string())?;
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(inserted.id)
    }
}

/// 删除底薪配置
pub async fn delete_config(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    salary_config::Entity::update_many()
        .filter(salary_config::Column::Id.eq(id))
        .col_expr(salary_config::Column::Deleted, Expr::value(1))
        .col_expr(salary_config::Column::UpdateTime, Expr::value(now))
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 数据权限 ====================

// 数据权限范围
pub enum SalaryDataScope {
    All,
    SelfAndSubordinates,
    SelfOnly,
}

/// 解析当前用户的数据权限范围
/// 返回 (scope, allowed_employee_ids)
/// - All: 全量（超管/财务/总经理/老板）
/// - SelfAndSubordinates: 自己+下属（有下属的管理岗）
/// - SelfOnly: 仅自己（普通员工）
pub async fn resolve_data_scope(
    db: &DatabaseConnection,
    user_id: i64,
) -> Result<(SalaryDataScope, Vec<i64>), String> {
    use crate::modules::system::service::role_service;

    // 1. 查用户
    let admin_user = admin::Entity::find_by_id(user_id)
        .one(db).await
        .map_err(|e| e.to_string())?
        .ok_or("用户不存在".to_string())?;

    // 2. 超管 → All
    if admin_user.user_type == Some(1) {
        return Ok((SalaryDataScope::All, vec![]));
    }

    // 3. 查角色 keys
    let role_keys = role_service::user_by_role_group(db, &Some(user_id))
        .await.map_err(|e| e.to_string())?;
    let full_scope_roles = ["finance", "general_manager", "boss", "cw"];
    if role_keys.iter().any(|k| full_scope_roles.contains(&k.as_str())) {
        return Ok((SalaryDataScope::All, vec![]));
    }

    // 4. 查下属（BFS 递归遍历 direct_manager_id）
    let subordinates = find_all_subordinates(db, user_id).await?;
    if subordinates.is_empty() {
        Ok((SalaryDataScope::SelfOnly, vec![user_id]))
    } else {
        let mut ids = subordinates;
        ids.push(user_id);
        Ok((SalaryDataScope::SelfAndSubordinates, ids))
    }
}

/// 递归查找所有下属（BFS）
pub async fn find_all_subordinates(
    db: &DatabaseConnection,
    manager_id: i64,
) -> Result<Vec<i64>, String> {
    let mut result = Vec::new();
    let mut queue = vec![manager_id];
    let mut visited = std::collections::HashSet::new();
    visited.insert(manager_id);

    while let Some(current) = queue.pop() {
        let subs = admin::Entity::find()
            .filter(admin::Column::DirectManagerId.eq(current))
            .filter(admin::Column::Deleted.eq(0))
            .all(db).await
            .map_err(|e| e.to_string())?;
        for s in subs {
            if visited.insert(s.id) {
                result.push(s.id);
                queue.push(s.id);
            }
        }
    }
    Ok(result)
}

// ==================== 工资确认/申诉 ====================

use crate::modules::finance::entity::salary_confirm;
use crate::modules::message::service::notification_service::NotificationService;

/// DTO: 工资确认/申诉请求
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalaryConfirmDTO {
    pub salary_record_id: i64,
    pub action: i32,       // 1=确认, 2=申请重新核算
    pub reason: Option<String>,
}

/// DTO: 财务处理申诉
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalaryConfirmHandleDTO {
    pub confirm_id: i64,
    pub action: i32,       // 1=同意重新核算, 2=驳回
    pub remark: Option<String>,
}

/// 员工提交工资确认/申诉
pub async fn submit_confirm(
    db: &DatabaseConnection,
    user_id: i64,
    user_name: &str,
    dto: SalaryConfirmDTO,
) -> Result<i64, String> {
    // 查工资记录
    let record = salary_record::Entity::find_by_id(dto.salary_record_id)
        .one(db).await.map_err(|e| e.to_string())?
        .ok_or("工资记录不存在".to_string())?;

    // 只能确认自己的工资
    if record.employee_id != user_id {
        return Err("只能确认自己的工资记录".to_string());
    }

    // 校验 action
    if dto.action != 1 && dto.action != 2 {
        return Err("无效的操作类型".to_string());
    }

    // 申请重新核算需要理由
    if dto.action == 2 && dto.reason.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
        return Err("申请重新核算必须填写理由".to_string());
    }

    // 更新工资记录的确认状态 + 创建确认/申诉记录（事务保证原子性）
    let confirmed_status = if dto.action == 1 { 1 } else { 2 };
    let now = Utc::now().naive_utc();
    let record_id = record.id;
    let record_year = record.year;
    let record_month = record.month;
    let reason_clone = dto.reason.clone();
    let user_name_owned = user_name.to_string();
    let confirm_record_id = db.transaction::<_, i64, String>(|txn| {
        Box::pin(async move {
            let mut active: salary_record::ActiveModel = record.into();
            active.employee_confirmed = sea_orm::Set(Some(confirmed_status));
            active.confirmed_time = sea_orm::Set(Some(now));
            active.update(txn).await.map_err(|e| e.to_string())?;

            let confirm = salary_confirm::ActiveModel {
                salary_record_id: sea_orm::Set(record_id),
                employee_id: sea_orm::Set(user_id),
                employee_name: sea_orm::Set(Some(user_name_owned)),
                year: sea_orm::Set(record_year),
                month: sea_orm::Set(record_month),
                action: sea_orm::Set(dto.action),
                reason: sea_orm::Set(dto.reason.clone()),
                status: sea_orm::Set(Some(if dto.action == 1 { 1 } else { 0 })),
                handler_id: sea_orm::Set(None),
                handler_name: sea_orm::Set(None),
                handle_time: sea_orm::Set(None),
                handle_remark: sea_orm::Set(None),
                create_time: sea_orm::Set(Some(now)),
                ..Default::default()
            };
            let result = confirm.insert(txn).await.map_err(|e| e.to_string())?;
            Ok(result.id)
        })
    })
    .await
    .map_err(|e| e.to_string())?;

    // 如果是申请重新核算，通知所有财务角色用户
    if dto.action == 2 {
        // 查找所有财务角色用户
        let finance_users = find_users_by_role_key(db, "finance").await.unwrap_or_default();
        for fin_id in finance_users {
            let _ = NotificationService::send_system_notification(
                db, fin_id,
                format!("工资重新核算申请 - {}年{}月", record_year, record_month),
                format!("员工 {} 申请重新核算 {}年{}月工资，理由：{}",
                    user_name, record_year, record_month,
                    reason_clone.as_deref().unwrap_or("无")),
                2, // 通知类型 2=审批通知
                Some("/finance/salary".to_string()),
            ).await;
        }
    }

    Ok(confirm_record_id)
}

/// 财务处理申诉
pub async fn handle_confirm(
    db: &DatabaseConnection,
    handler_id: i64,
    handler_name: &str,
    dto: SalaryConfirmHandleDTO,
) -> Result<(), String> {
    let confirm = salary_confirm::Entity::find_by_id(dto.confirm_id)
        .one(db).await.map_err(|e| e.to_string())?
        .ok_or("申诉记录不存在".to_string())?;

    if confirm.status != Some(0) {
        return Err("该申诉已处理".to_string());
    }

    let now = Utc::now().naive_utc();
    let new_status = if dto.action == 1 { 1 } else { 2 };

    let mut active: salary_confirm::ActiveModel = confirm.clone().into();
    active.status = sea_orm::Set(Some(new_status));
    active.handler_id = sea_orm::Set(Some(handler_id));
    active.handler_name = sea_orm::Set(Some(handler_name.to_string()));
    active.handle_time = sea_orm::Set(Some(now));
    active.handle_remark = sea_orm::Set(dto.remark.clone());
    active.update(db).await.map_err(|e| e.to_string())?;

    // 同意重新核算 → 执行核算
    if dto.action == 1 {
        // 先获取申诉关联的工资记录信息（year/month/employee_id），用于核算后定位新记录
        let old_record = salary_record::Entity::find_by_id(confirm.salary_record_id)
            .one(db).await.map_err(|e| e.to_string())?;

        let (year, month, employee_id) = if let Some(ref r) = old_record {
            (r.year, r.month, r.employee_id)
        } else {
            // 旧记录可能已被删除，从 confirm 记录中获取
            (confirm.year, confirm.month, confirm.employee_id)
        };

        // 调用核算（会删除该年月 status=0 的旧记录并重建）
        calculate(db, year, month, 0, handler_id, handler_name).await?;

        // 找到重新核算后该员工的新工资记录（最新的那条）
        let new_record = salary_record::Entity::find()
            .filter(salary_record::Column::EmployeeId.eq(employee_id))
            .filter(salary_record::Column::Year.eq(year))
            .filter(salary_record::Column::Month.eq(month))
            .filter(salary_record::Column::Status.eq(0))
            .filter(salary_record::Column::Deleted.eq(0))
            .order_by_desc(salary_record::Column::Id)
            .one(db).await.map_err(|e| e.to_string())?;

        if let Some(new_rec) = new_record {
            // 更新申诉记录关联到新工资记录 + 重置新工资记录确认状态（事务保证原子性）
            let new_rec_id = new_rec.id;
            let confirm_clone = confirm.clone();
            db.transaction::<_, (), String>(|txn| {
                Box::pin(async move {
                    let mut c_active: salary_confirm::ActiveModel = confirm_clone.into();
                    c_active.salary_record_id = sea_orm::Set(new_rec_id);
                    c_active.update(txn).await.map_err(|e| e.to_string())?;

                    let mut rec_active: salary_record::ActiveModel = new_rec.into();
                    rec_active.employee_confirmed = sea_orm::Set(Some(0));
                    rec_active.confirmed_time = sea_orm::Set(None);
                    rec_active.update(txn).await.map_err(|e| e.to_string())?;
                    Ok(())
                })
            })
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    // 通知员工处理结果
    let result_text = if dto.action == 1 { "已同意重新核算" } else { "已驳回" };
    let _ = NotificationService::send_system_notification(
        db, confirm.employee_id,
        format!("工资申诉处理结果 - {}年{}月", confirm.year, confirm.month),
        format!("您的工资重新核算申请{}，备注：{}",
            result_text,
            dto.remark.as_deref().unwrap_or("无")),
        2,
        Some("/finance/salary".to_string()),
    ).await;

    Ok(())
}

/// 查询我的确认/申诉记录
pub async fn get_my_confirms(
    db: &DatabaseConnection,
    user_id: i64,
    page: i64,
    page_size: i64,
) -> Result<(Vec<salary_confirm::Model>, i64), String> {
    let stmt = salary_confirm::Entity::find()
        .filter(salary_confirm::Column::EmployeeId.eq(user_id))
        .order_by_desc(salary_confirm::Column::CreateTime);

    let page = std::cmp::max(page, 1);
    let page_size = std::cmp::max(page_size, 1);
    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator.fetch_page((page - 1) as u64).await.map_err(|e| e.to_string())?;
    Ok((items, total))
}

/// 查询待处理申诉列表（财务用）
pub async fn get_pending_confirms(
    db: &DatabaseConnection,
    page: i64,
    page_size: i64,
) -> Result<(Vec<salary_confirm::Model>, i64), String> {
    let stmt = salary_confirm::Entity::find()
        .filter(salary_confirm::Column::Status.eq(0))
        .filter(salary_confirm::Column::Action.eq(2))
        .order_by_asc(salary_confirm::Column::CreateTime);

    let page = std::cmp::max(page, 1);
    let page_size = std::cmp::max(page_size, 1);
    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator.fetch_page((page - 1) as u64).await.map_err(|e| e.to_string())?;
    Ok((items, total))
}

/// V7-6: 申诉列表筛选查询参数
#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PendingConfirmQuery {
    pub employee_id: Option<i64>,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub status: Option<i32>,
    pub page: i64,
    pub page_size: i64,
}

/// V7-6: 申诉列表筛选条件（支持 employee_id/year/month/status）
/// 注：status=None 时默认仅查待处理（status=0 且 action=2）；status=Some(x) 时按指定 status 查询
pub async fn get_pending_confirms_filtered(
    db: &DatabaseConnection,
    query: PendingConfirmQuery,
) -> Result<(Vec<salary_confirm::Model>, i64), String> {
    let mut stmt = salary_confirm::Entity::find();

    // status 默认行为：未传则查待处理（0），传了按传入值
    match query.status {
        Some(s) => stmt = stmt.filter(salary_confirm::Column::Status.eq(s)),
        None => stmt = stmt.filter(salary_confirm::Column::Status.eq(0)),
    }
    // 待处理（status=0）时仅看 action=2（申诉），其他状态显示全部
    if query.status.is_none() {
        stmt = stmt.filter(salary_confirm::Column::Action.eq(2));
    }

    if let Some(eid) = query.employee_id {
        stmt = stmt.filter(salary_confirm::Column::EmployeeId.eq(eid));
    }
    if let Some(y) = query.year {
        stmt = stmt.filter(salary_confirm::Column::Year.eq(y));
    }
    if let Some(m) = query.month {
        stmt = stmt.filter(salary_confirm::Column::Month.eq(m));
    }

    // 待处理按创建时间升序（先申请的先处理），其他按降序
    if query.status.is_none() {
        stmt = stmt.order_by_asc(salary_confirm::Column::CreateTime);
    } else {
        stmt = stmt.order_by_desc(salary_confirm::Column::CreateTime);
    }

    let page = std::cmp::max(query.page, 1);
    let page_size = std::cmp::max(query.page_size, 1);
    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator.fetch_page((page - 1) as u64).await.map_err(|e| e.to_string())?;
    Ok((items, total))
}

/// 查找拥有指定角色 key 的所有用户 ID
async fn find_users_by_role_key(db: &DatabaseConnection, role_key: &str) -> Result<Vec<i64>, String> {
    use crate::modules::system::entity::{admin_role_merge, role, admin};

    // 查角色
    let role = role::Entity::find()
        .filter(role::Column::RoleKey.eq(role_key))
        .filter(role::Column::Deleted.eq(0))
        .one(db).await.map_err(|e| e.to_string())?;

    let role = match role {
        Some(r) => r,
        None => return Ok(vec![]),
    };

    // 查关联用户
    let merges = admin_role_merge::Entity::find()
        .filter(admin_role_merge::Column::RoleId.eq(role.id))
        .all(db).await.map_err(|e| e.to_string())?;

    let mut user_ids = Vec::new();
    for merge in merges {
        if let Some(uid) = merge.admin_id {
            // 验证用户存在且未删除
            if let Some(u) = admin::Entity::find_by_id(uid).one(db).await.map_err(|e| e.to_string())? {
                if u.deleted.unwrap_or(0) == 0 && u.status.unwrap_or(1) == 1 {
                    user_ids.push(uid);
                }
            }
        }
    }
    Ok(user_ids)
}

// ===== V8-1: 工资单审批流对接 approval 引擎 =====
//
// 策略：保留现有直接 approve/batch_approve 接口（向后兼容），新增审批流对接入口。
// 流程：submit_salary_approval -> approval 引擎审批 -> sync_salary_approval_status 同步状态
// 审批流模板 flow_code = "salary_approval"，需在 approval 模块预先配置（business_type="salary"）
// 审批通过（instance.status=2）后同步工资记录为已审核；驳回（status=4）保持待审核并记日志

use crate::modules::approval::service::approval_service::ApprovalService;
use crate::modules::approval::model::approval::ApprovalSubmitRequest;
use crate::modules::approval::entity::approval_instance;

/// 工资审批流 flow_code 常量
pub const SALARY_APPROVAL_FLOW_CODE: &str = "salary_approval";
/// 工资审批流 business_type 常量
pub const SALARY_APPROVAL_BUSINESS_TYPE: &str = "salary";

/// 提交月度工资到审批流（逐条提交，封装为批量）
///
/// - 查询指定年月所有 status=0（待审核）的工资记录
/// - 逐条调用 ApprovalService::submit 创建审批实例
/// - extra_data 中记录 salary_record_id 便于回调
/// - 返回 (成功数, 失败数, 失败详情)
pub async fn submit_salary_approval(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
    submitter_id: i64,
    submitter_name: &str,
) -> Result<(usize, Vec<String>), String> {
    let records = salary_record::Entity::find()
        .filter(salary_record::Column::Year.eq(year))
        .filter(salary_record::Column::Month.eq(month))
        .filter(salary_record::Column::Status.eq(0))
        .filter(salary_record::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    if records.is_empty() {
        return Err(format!("{}年{}月没有待审核的工资记录", year, month));
    }

    let mut success = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for r in &records {
        let emp_name = r.employee_name.clone().unwrap_or_else(|| format!("员工{}", r.employee_id));
        let title = format!("{}年{}月 {} 工资审批", r.year, r.month, emp_name);
        let extra = serde_json::json!({
            "salary_record_id": r.id,
            "employee_id": r.employee_id,
            "employee_name": emp_name,
            "year": r.year,
            "month": r.month,
            "total_salary": r.total_salary,
        });

        let req = ApprovalSubmitRequest {
            flow_code: SALARY_APPROVAL_FLOW_CODE.to_string(),
            business_type: SALARY_APPROVAL_BUSINESS_TYPE.to_string(),
            business_id: r.id,
            business_title: Some(title),
            submitter_id,
            submitter_name: Some(submitter_name.to_string()),
            extra_data: Some(extra),
        };

        match ApprovalService::submit(db, &req).await {
            Ok(_instance_id) => success += 1,
            Err(e) => failures.push(format!("员工{}({}): {}", emp_name, r.employee_id, e)),
        }
    }

    Ok((success, failures))
}

/// 同步工资审批状态到工资记录
///
/// - 查询指定年月所有 business_type="salary" 的审批实例
/// - 审批通过（status=2）：更新对应工资记录为已审核（status=1）
/// - 审批驳回（status=4）：保持待审核，记录驳回原因到 remark
/// - 返回 (同步通过数, 同步驳回数)
pub async fn sync_salary_approval_status(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
) -> Result<(usize, usize), String> {
    // 查询该月份所有工资审批实例
    let instances = approval_instance::Entity::find()
        .filter(approval_instance::Column::BusinessType.eq(SALARY_APPROVAL_BUSINESS_TYPE))
        .filter(approval_instance::Column::Status.is_in(vec![2, 4])) // 2=通过, 4=驳回
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut approved = 0usize;
    let mut rejected = 0usize;
    let now = Utc::now().naive_utc();

    for inst in &instances {
        let salary_record_id = match inst.business_id {
            Some(id) => id,
            None => continue,
        };

        // 从 extra_data 解析 year/month，只处理指定月份的
        if let Some(extra) = &inst.extra_data {
            let inst_year = extra.get("year").and_then(|v| v.as_i64());
            let inst_month = extra.get("month").and_then(|v| v.as_i64());
            if let (Some(y), Some(m)) = (inst_year, inst_month) {
                if y != year as i64 || m != month as i64 {
                    continue;
                }
            }
        }

        let record = match salary_record::Entity::find_by_id(salary_record_id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
        {
            Some(r) => r,
            None => continue,
        };

        // 只处理待审核（status=0）的记录
        if record.status.unwrap_or(0) != 0 {
            continue;
        }

        let inst_status = inst.status.unwrap_or(0);
        if inst_status == 2 {
            // 审批通过：更新工资为已审核
            let mut model: salary_record::ActiveModel = record.into();
            model.status = Set(Some(1));
            model.update_time = Set(Some(now));
            model.update(db).await.map_err(|e| e.to_string())?;
            approved += 1;
        } else if inst_status == 4 {
            // 审批驳回：保持待审核，记录到 remark
            let mut model: salary_record::ActiveModel = record.into();
            let remark = format!("{} 审批驳回", now.format("%Y-%m-%d"));
            model.remark = Set(Some(remark));
            model.update_time = Set(Some(now));
            model.update(db).await.map_err(|e| e.to_string())?;
            rejected += 1;
        }
    }

    Ok((approved, rejected))
}

// ==================== P2-2: 工资历史趋势分析 ====================
//
// 提供四个维度的聚合查询，供前端趋势分析页（finance/salary-analysis/index.vue）使用：
//   1. get_trend_monthly        —— 按月聚合的时间序列（折线图）
//   2. get_trend_by_department  —— 按部门聚合（柱状图 / 饼图）
//   3. get_trend_by_employee    —— 员工排名（TopN 柱状图）
//   4. get_trend_summary        —— 周期汇总卡片
//
// 所有查询都遵循数据权限范围（resolve_data_scope），普通员工只能看到自己的数据。

/// 解析查询参数的默认值，并返回 (year_start, year_end, month_start, month_end)
fn normalize_trend_range(query: &SalaryTrendQuery) -> (i32, i32, Option<i32>, Option<i32>) {
    let current_year = chrono::Utc::now().year();
    let year_start = query.year_start.unwrap_or(current_year - 2);
    let year_end = query.year_end.unwrap_or(current_year);
    (year_start, year_end, query.month_start, query.month_end)
}

/// 构建趋势查询的基础过滤条件（年月范围 + 员工/部门筛选 + 软删除过滤）
fn apply_trend_filters(
    mut stmt: sea_orm::Select<salary_record::Entity>,
    query: &SalaryTrendQuery,
) -> sea_orm::Select<salary_record::Entity> {
    let (year_start, year_end, month_start, month_end) = normalize_trend_range(query);

    stmt = stmt
        .filter(salary_record::Column::Year.gte(year_start))
        .filter(salary_record::Column::Year.lte(year_end))
        .filter(salary_record::Column::Deleted.eq(0));

    if let Some(ms) = month_start {
        stmt = stmt.filter(salary_record::Column::Month.gte(ms));
    }
    if let Some(me) = month_end {
        stmt = stmt.filter(salary_record::Column::Month.lte(me));
    }
    if let Some(eid) = query.employee_id {
        stmt = stmt.filter(salary_record::Column::EmployeeId.eq(eid));
    }
    if let Some(name) = &query.employee_name {
        if !name.is_empty() {
            stmt = stmt.filter(salary_record::Column::EmployeeName.contains(name));
        }
    }
    if let Some(dept) = &query.department_name {
        if !dept.is_empty() {
            stmt = stmt.filter(salary_record::Column::DepartmentName.eq(dept));
        }
    }
    stmt
}

/// P2-2: 月度趋势时间序列
/// 按 (year, month) 升序聚合，用于折线图展示工资各科目随时间的变化
pub async fn get_trend_monthly(
    db: &DatabaseConnection,
    query: SalaryTrendQuery,
    user_id: i64,
) -> Result<Vec<SalaryTrendMonthlyPointDTO>, String> {
    let mut stmt = apply_trend_filters(salary_record::Entity::find(), &query);

    // 数据权限过滤
    let (scope, allowed_ids) = resolve_data_scope(db, user_id).await?;
    if let SalaryDataScope::SelfAndSubordinates | SalaryDataScope::SelfOnly = scope {
        stmt = stmt.filter(salary_record::Column::EmployeeId.is_in(allowed_ids));
    }

    stmt = stmt
        .order_by_asc(salary_record::Column::Year)
        .order_by_asc(salary_record::Column::Month);

    let records = stmt.all(db).await.map_err(|e| e.to_string())?;

    // 按 (year, month) 分组聚合
    let mut bucket: HashMap<(i32, i32), Vec<&salary_record::Model>> = HashMap::new();
    for r in &records {
        bucket.entry((r.year, r.month)).or_default().push(r);
    }

    let mut sorted_keys: Vec<(i32, i32)> = bucket.keys().cloned().collect();
    sorted_keys.sort();

    let mut result = Vec::with_capacity(sorted_keys.len());
    for key in sorted_keys {
        let (year, month) = key;
        let rows = bucket.remove(&key).unwrap_or_default();
        let headcount: i64 = rows
            .iter()
            .map(|r| r.employee_id)
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        let total_base: Decimal = rows.iter().map(|r| r.base_salary).sum();
        let total_commission: Decimal = rows.iter().map(|r| r.commission_amount).sum();
        let total_performance: Decimal = rows.iter().map(|r| r.performance_bonus).sum();
        let total_deduction: Decimal = rows.iter().map(|r| r.deduction_amount).sum();
        let total_team_commission: Decimal = rows.iter().map(|r| r.team_commission_amount).sum();
        let total_tax: Decimal = rows.iter().map(|r| r.tax_amount).sum();
        let total_gross: Decimal = rows.iter().map(|r| r.total_salary).sum();
        let total_net: Decimal = rows.iter().map(|r| r.net_salary).sum();
        let avg_net = if headcount > 0 {
            total_net / Decimal::from(headcount)
        } else {
            Decimal::ZERO
        };

        result.push(SalaryTrendMonthlyPointDTO {
            year,
            month,
            period: format!("{:04}-{:02}", year, month),
            headcount,
            total_base: total_base.to_f64().unwrap_or_default(),
            total_commission: total_commission.to_f64().unwrap_or_default(),
            total_performance: total_performance.to_f64().unwrap_or_default(),
            total_deduction: total_deduction.to_f64().unwrap_or_default(),
            total_team_commission: total_team_commission.to_f64().unwrap_or_default(),
            total_tax: total_tax.to_f64().unwrap_or_default(),
            total_gross: total_gross.to_f64().unwrap_or_default(),
            total_net: total_net.to_f64().unwrap_or_default(),
            avg_net: avg_net.to_f64().unwrap_or_default(),
        });
    }

    Ok(result)
}

/// P2-2: 部门维度聚合
/// 按部门分组统计，用于部门工资对比柱状图
pub async fn get_trend_by_department(
    db: &DatabaseConnection,
    query: SalaryTrendQuery,
    user_id: i64,
) -> Result<Vec<SalaryTrendDeptPointDTO>, String> {
    let mut stmt = apply_trend_filters(salary_record::Entity::find(), &query);

    let (scope, allowed_ids) = resolve_data_scope(db, user_id).await?;
    if let SalaryDataScope::SelfAndSubordinates | SalaryDataScope::SelfOnly = scope {
        stmt = stmt.filter(salary_record::Column::EmployeeId.is_in(allowed_ids));
    }

    let records = stmt.all(db).await.map_err(|e| e.to_string())?;

    // 按部门名称分组（None → "未分配"）
    let mut bucket: HashMap<String, Vec<&salary_record::Model>> = HashMap::new();
    for r in &records {
        let dept_name = r.department_name.clone().unwrap_or_else(|| "未分配".to_string());
        bucket.entry(dept_name).or_default().push(r);
    }

    let mut result: Vec<SalaryTrendDeptPointDTO> = bucket
        .into_iter()
        .map(|(dept_name, rows)| {
            let headcount: i64 = rows
                .iter()
                .map(|r| r.employee_id)
                .collect::<std::collections::HashSet<_>>()
                .len() as i64;
            let total_base: Decimal = rows.iter().map(|r| r.base_salary).sum();
            let total_commission: Decimal = rows.iter().map(|r| r.commission_amount).sum();
            let total_performance: Decimal = rows.iter().map(|r| r.performance_bonus).sum();
            let total_gross: Decimal = rows.iter().map(|r| r.total_salary).sum();
            let total_net: Decimal = rows.iter().map(|r| r.net_salary).sum();
            let avg_net = if headcount > 0 {
                total_net / Decimal::from(headcount)
            } else {
                Decimal::ZERO
            };
            SalaryTrendDeptPointDTO {
                department_name: dept_name,
                headcount,
                total_base: total_base.to_f64().unwrap_or_default(),
                total_commission: total_commission.to_f64().unwrap_or_default(),
                total_performance: total_performance.to_f64().unwrap_or_default(),
                total_gross: total_gross.to_f64().unwrap_or_default(),
                total_net: total_net.to_f64().unwrap_or_default(),
                avg_net: avg_net.to_f64().unwrap_or_default(),
            }
        })
        .collect();

    // 按实发合计降序
    result.sort_by(|a, b| b.total_net.partial_cmp(&a.total_net).unwrap_or(std::cmp::Ordering::Equal));
    Ok(result)
}

/// P2-2: 员工维度排名
/// 按员工聚合，按实发工资降序，用于 TopN 排名图表
pub async fn get_trend_by_employee(
    db: &DatabaseConnection,
    query: SalaryTrendQuery,
    user_id: i64,
    limit: Option<i64>,
) -> Result<Vec<SalaryTrendEmployeePointDTO>, String> {
    let mut stmt = apply_trend_filters(salary_record::Entity::find(), &query);

    let (scope, allowed_ids) = resolve_data_scope(db, user_id).await?;
    if let SalaryDataScope::SelfAndSubordinates | SalaryDataScope::SelfOnly = scope {
        stmt = stmt.filter(salary_record::Column::EmployeeId.is_in(allowed_ids));
    }

    let records = stmt.all(db).await.map_err(|e| e.to_string())?;

    // 按 employee_id 分组
    let mut bucket: HashMap<i64, Vec<&salary_record::Model>> = HashMap::new();
    for r in &records {
        bucket.entry(r.employee_id).or_default().push(r);
    }

    let mut result: Vec<SalaryTrendEmployeePointDTO> = bucket
        .into_iter()
        .map(|(emp_id, rows)| {
            let first = rows.first();
            let employee_name = first
                .and_then(|r| r.employee_name.clone())
                .unwrap_or_else(|| format!("员工{}", emp_id));
            let department_name = first.and_then(|r| r.department_name.clone());
            let months: i64 = rows
                .iter()
                .map(|r| (r.year, r.month))
                .collect::<std::collections::HashSet<_>>()
                .len() as i64;
            let total_base: Decimal = rows.iter().map(|r| r.base_salary).sum();
            let total_commission: Decimal = rows.iter().map(|r| r.commission_amount).sum();
            let total_performance: Decimal = rows.iter().map(|r| r.performance_bonus).sum();
            let total_gross: Decimal = rows.iter().map(|r| r.total_salary).sum();
            let total_net: Decimal = rows.iter().map(|r| r.net_salary).sum();
            let avg_monthly_net = if months > 0 {
                total_net / Decimal::from(months)
            } else {
                Decimal::ZERO
            };
            SalaryTrendEmployeePointDTO {
                employee_id: emp_id,
                employee_name,
                department_name,
                total_base: total_base.to_f64().unwrap_or_default(),
                total_commission: total_commission.to_f64().unwrap_or_default(),
                total_performance: total_performance.to_f64().unwrap_or_default(),
                total_gross: total_gross.to_f64().unwrap_or_default(),
                total_net: total_net.to_f64().unwrap_or_default(),
                months,
                avg_monthly_net: avg_monthly_net.to_f64().unwrap_or_default(),
            }
        })
        .collect();

    // 按实发合计降序
    result.sort_by(|a, b| b.total_net.partial_cmp(&a.total_net).unwrap_or(std::cmp::Ordering::Equal));

    // 截取 TopN
    if let Some(n) = limit {
        if n > 0 {
            result.truncate(n as usize);
        }
    }
    Ok(result)
}

/// P2-2: 周期汇总（KPI 卡片）
/// 全量统计所选时间范围内的工资数据
pub async fn get_trend_summary(
    db: &DatabaseConnection,
    query: SalaryTrendQuery,
    user_id: i64,
) -> Result<SalaryTrendSummaryDTO, String> {
    let mut stmt = apply_trend_filters(salary_record::Entity::find(), &query);

    let (scope, allowed_ids) = resolve_data_scope(db, user_id).await?;
    if let SalaryDataScope::SelfAndSubordinates | SalaryDataScope::SelfOnly = scope {
        stmt = stmt.filter(salary_record::Column::EmployeeId.is_in(allowed_ids));
    }

    let records = stmt.all(db).await.map_err(|e| e.to_string())?;

    let total_records = records.len() as i64;
    let total_headcount: i64 = records
        .iter()
        .map(|r| r.employee_id)
        .collect::<std::collections::HashSet<_>>()
        .len() as i64;
    let total_months: i64 = records
        .iter()
        .map(|r| (r.year, r.month))
        .collect::<std::collections::HashSet<_>>()
        .len() as i64;

    let total_base: Decimal = records.iter().map(|r| r.base_salary).sum();
    let total_commission: Decimal = records.iter().map(|r| r.commission_amount).sum();
    let total_performance: Decimal = records.iter().map(|r| r.performance_bonus).sum();
    let total_team_commission: Decimal = records.iter().map(|r| r.team_commission_amount).sum();
    let total_tax: Decimal = records.iter().map(|r| r.tax_amount).sum();
    let total_gross: Decimal = records.iter().map(|r| r.total_salary).sum();
    let total_net: Decimal = records.iter().map(|r| r.net_salary).sum();
    let avg_monthly_net = if total_months > 0 {
        total_net / Decimal::from(total_months)
    } else {
        Decimal::ZERO
    };

    Ok(SalaryTrendSummaryDTO {
        total_headcount,
        total_records,
        total_months,
        total_gross: total_gross.to_f64().unwrap_or_default(),
        total_net: total_net.to_f64().unwrap_or_default(),
        total_base: total_base.to_f64().unwrap_or_default(),
        total_commission: total_commission.to_f64().unwrap_or_default(),
        total_performance: total_performance.to_f64().unwrap_or_default(),
        total_team_commission: total_team_commission.to_f64().unwrap_or_default(),
        total_tax: total_tax.to_f64().unwrap_or_default(),
        avg_monthly_net: avg_monthly_net.to_f64().unwrap_or_default(),
    })
}
