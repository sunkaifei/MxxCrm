//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Error;
use crate::core::errors::error::Result;
use crate::modules::statistics::entity::performance_plan::{self, Entity as PerformancePlan};
use crate::modules::statistics::entity::plan_monthly_target::{self, Entity as PlanMonthlyTarget};
use crate::modules::statistics::entity::plan_approval_log::{self, Entity as PlanApprovalLog};
use crate::modules::statistics::entity::plan_approval_node::{self, Entity as PlanApprovalNode};
use crate::modules::statistics::model::performance_plan::{
    PlanDetailVO, PlanListVO, MonthlyTargetVO, ApprovalLogVO, ApprovalNodeVO,
    CreatePlanRequest, MonthlyTargetInput, ReviewPlanRequest, ModifyPlanRequest,
    UpdatePlanTargetsRequest, PlanProgressSummaryVO, ProgressItemVO,
    PlanCoverageVO, PlanCoverageSummaryVO,
};
use crate::modules::statistics::service::employee_stats_service;
use crate::modules::statistics::service::stats_range::StatsScope;
use crate::modules::system::entity::admin::Entity as Admin;
use crate::modules::crm::entity::contract::{self, Entity as Contract};
use sea_orm::{
    ConnectionTrait, DbConn, Statement, TransactionTrait,
    EntityTrait, ColumnTrait, QueryFilter, QueryOrder, ActiveModelTrait, Set, IntoActiveModel,
};
use sea_orm::prelude::Decimal;

/// 年度计划覆盖度：数据权限范围内全员 × 当年计划状态（集中管理视角）
/// 口径与员工全景榜一致：approved = 当年存在已通过(status=2)计划 ⇒ 销售身份
pub async fn get_plan_coverage(
    db: &DbConn,
    scope: &StatsScope,
    year: i32,
) -> Result<PlanCoverageSummaryVO> {
    // 员工名单（含数据权限收缩与部门名映射，复用员工统计加载器）
    let admins = employee_stats_service::load_admins(db, scope).await?;

    // 该年度全部计划的月度目标汇总（单条 SQL，避免逐计划 N+1）
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(
            sea_orm::DbBackend::Postgres,
            r#"SELECT p.id AS plan_id, p.employee_id AS employee_id, p.status AS status,
                      COALESCE(SUM(t.contract_target_amount), 0) AS total_contract_target,
                      COALESCE(SUM(t.payment_target_amount), 0) AS total_payment_target
               FROM mxx_statistics_performance_plan p
               LEFT JOIN mxx_statistics_plan_monthly_target t ON t.plan_id = p.id AND t.deleted = 0
               WHERE p.deleted = 0 AND p.year = $1 AND p.employee_id IS NOT NULL
               GROUP BY p.id, p.employee_id, p.status
               ORDER BY p.id DESC"#,
            [year.into()],
        ))
        .await?;

    // 归并到人：ORDER BY id DESC 保证首见即最新一条计划的状态；
    // approved 口径与员工全景榜对齐：当年任意一条 status=2 即视为已通过销售
    let mut latest_status: std::collections::HashMap<i64, i32> = std::collections::HashMap::new();
    let mut approved_set: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut contract_sum: std::collections::HashMap<i64, Decimal> = std::collections::HashMap::new();
    let mut payment_sum: std::collections::HashMap<i64, Decimal> = std::collections::HashMap::new();
    for r in rows {
        let emp = r.try_get::<i64>("", "employee_id").unwrap_or(0);
        if emp <= 0 {
            continue;
        }
        if let Ok(Some(s)) = r.try_get::<Option<i32>>("", "status") {
            latest_status.entry(emp).or_insert(s);
            if s == 2 {
                approved_set.insert(emp);
            }
        }
        let add = |map: &mut std::collections::HashMap<i64, Decimal>, col: &str| -> () {
            if let Ok(v) = r.try_get::<Decimal>("", col) {
                map.entry(emp).and_modify(|x| *x += v).or_insert(v);
            }
        };
        add(&mut contract_sum, "total_contract_target");
        add(&mut payment_sum, "total_payment_target");
    }

    let mut items: Vec<PlanCoverageVO> = Vec::with_capacity(admins.len());
    let mut with_plan_count = 0_i64;
    for (id, name, dept_name) in admins {
        let has_plan = latest_status.contains_key(&id);
        if has_plan {
            with_plan_count += 1;
        }
        let approved = approved_set.contains(&id);
        let trim = |m: &std::collections::HashMap<i64, Decimal>| m.get(&id).copied().filter(|d| !d.is_zero());
        items.push(PlanCoverageVO {
            employee_id: id,
            name,
            dept_name,
            has_plan,
            approved,
            plan_status: latest_status.get(&id).copied(),
            total_contract_target: trim(&contract_sum),
            total_payment_target: trim(&payment_sum),
        });
    }
    // 未建计划者排最前（管理动作优先补缺），其次待通过，最后已通过
    items.sort_by(|a, b| {
        let rank = |v: &PlanCoverageVO| if !v.has_plan { 0 } else if !v.approved { 1 } else { 2 };
        rank(a).cmp(&rank(b)).then_with(|| a.name.cmp(&b.name))
    });

    let total_employees = items.len() as i64;
    let approved_count = items.iter().filter(|v| v.approved).count() as i64;
    let coverage_rate = if total_employees > 0 {
        Some((Decimal::from(approved_count) * Decimal::from(100) / Decimal::from(total_employees)).round_dp(2))
    } else {
        None
    };

    Ok(PlanCoverageSummaryVO {
        year,
        total_employees,
        with_plan_count,
        approved_count,
        coverage_rate,
        items,
    })
}

/// 创建草稿计划
pub async fn create_plan(db: &DbConn, employee_id: i64, req: &CreatePlanRequest) -> Result<PlanDetailVO> {
    let txn = db.begin().await?;

    // 检查同一年是否已有计划
    let existing = PerformancePlan::find()
        .filter(performance_plan::Column::EmployeeId.eq(employee_id))
        .filter(performance_plan::Column::Year.eq(req.year))
        .filter(performance_plan::Column::Deleted.eq(0))
        .one(&txn)
        .await?;

    if existing.is_some() {
        return Err(crate::core::errors::error::Error::BadRequest("该年份已有业绩计划，请勿重复创建".to_string()));
    }

    // 创建计划头
    let plan = performance_plan::ActiveModel {
        employee_id: Set(employee_id),
        year: Set(req.year),
        status: Set(Some(0)), // draft
        apply_reason: Set(None),
        version: Set(Some(1)),
        ..Default::default()
    };
    let plan_result = plan.insert(&txn).await?;

    // 创建月度目标
    for mt in &req.monthly_targets {
        let monthly = plan_monthly_target::ActiveModel {
            plan_id: Set(plan_result.id),
            month: Set(mt.month),
            contract_target_amount: Set(mt.contract_target_amount),
            payment_target_amount: Set(mt.payment_target_amount),
            contract_target_count: Set(mt.contract_target_count),
            ..Default::default()
        };
        monthly.insert(&txn).await?;
    }

    txn.commit().await?;

    get_plan_detail(db, plan_result.id).await
}

/// 提交计划（草稿→待审批）
/// 提交时按 direct_manager_id 链向上遍历计算审批链，按金额阈值确定层级数
/// 金额阈值：< 100万=1级审批（直属上级），100万-500万=2级，≥500万=3级
pub async fn submit_plan(db: &DbConn, plan_id: i64, operator_id: i64, operator_name: &str) -> Result<PlanDetailVO> {
    let txn = db.begin().await?;

    let plan = PerformancePlan::find_by_id(plan_id)
        .filter(performance_plan::Column::Deleted.eq(0))
        .one(&txn)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::BadRequest("计划不存在".to_string()))?;

    if plan.status != Some(0) && plan.status != Some(3) {
        return Err(crate::core::errors::error::Error::BadRequest("当前状态不允许提交，仅草稿或已驳回状态可提交".to_string()));
    }

    // 冻结检查
    if plan.is_frozen.unwrap_or(0) == 1 {
        return Err(crate::core::errors::error::Error::BadRequest("该计划已冻结，不可修改或提交".to_string()));
    }

    let previous_status = plan.status;
    let new_status = 1; // submitted
    let now = chrono::Local::now().naive_local();

    // ===== 计算年目标总金额，确定审批层级数 =====
    let targets = PlanMonthlyTarget::find()
        .filter(plan_monthly_target::Column::PlanId.eq(plan_id))
        .filter(plan_monthly_target::Column::Deleted.eq(0))
        .all(&txn)
        .await?;
    let total_amount: Decimal = targets.iter()
        .map(|t| t.contract_target_amount.unwrap_or(Decimal::from(0)))
        .sum();
    let total_amount_f64 = total_amount.to_string().parse::<f64>().unwrap_or(0.0);

    // 金额阈值：100万=1000000，500万=5000000
    let max_levels = if total_amount_f64 >= 5_000_000.0 {
        3
    } else if total_amount_f64 >= 1_000_000.0 {
        2
    } else {
        1
    };

    // ===== 沿 direct_manager_id 链向上收集审批人 =====
    let employee_id = plan.employee_id;
    let all_admins = Admin::find()
        .filter(crate::modules::system::entity::admin::Column::Deleted.eq(0))
        .filter(crate::modules::system::entity::admin::Column::Status.eq(1))
        .all(&txn)
        .await?;
    let admin_map: std::collections::HashMap<i64, &crate::modules::system::entity::admin::Model> =
        all_admins.iter().map(|a| (a.id, a)).collect();

    // 向上遍历 direct_manager_id 链，收集审批人（去重，排除提交人自己）
    let mut approver_chain: Vec<(i64, String)> = Vec::new();
    let mut visited: std::collections::HashSet<i64> = std::collections::HashSet::new();
    visited.insert(employee_id);

    let mut current_id = employee_id;
    loop {
        let current_admin = match admin_map.get(&current_id) {
            Some(a) => a,
            None => break,
        };
        let manager_id = match current_admin.direct_manager_id {
            Some(mid) if mid > 0 => mid,
            _ => break, // 无上级，到达组织顶层
        };
        // 防自审批：跳过提交人自己
        if manager_id == employee_id {
            break;
        }
        // 去重
        if visited.contains(&manager_id) {
            break;
        }
        visited.insert(manager_id);
        let manager_name = admin_map.get(&manager_id)
            .and_then(|a| a.user_name.clone())
            .unwrap_or_else(|| format!("用户{}", manager_id));
        approver_chain.push((manager_id, manager_name));

        current_id = manager_id;
        // 达到所需层级数即可停止
        if approver_chain.len() >= max_levels {
            break;
        }
    }

    if approver_chain.is_empty() {
        // 无上级审批人（可能是顶层管理者自己），自动通过
        let plan_year = plan.year;
        let mut active: performance_plan::ActiveModel = plan.into_active_model();
        active.status = Set(Some(2)); // approved
        active.current_approver_id = Set(None);
        active.current_approver_name = Set(None);
        active.approval_level = Set(Some(0));
        active.total_levels = Set(Some(0));
        active.submit_time = Set(Some(now));
        active.update(&txn).await?;

        let log = plan_approval_log::ActiveModel {
            plan_id: Set(plan_id),
            action: Set(1), // submit
            operator_id: Set(operator_id),
            operator_name: Set(Some(operator_name.to_string())),
            reason: Set(Some("无上级审批人，系统自动通过".to_string())),
            previous_status: Set(previous_status),
            new_status: Set(Some(2)),
            current_level: Set(Some(0)),
            ..Default::default()
        };
        log.insert(&txn).await?;

        txn.commit().await?;
        // 通知提交人计划已自动通过
        let _ = send_plan_notice(db, employee_id, operator_id, "销售计划已自动通过",
            &format!("您的 {} 年销售计划因无上级审批人，已自动通过。", plan_year)).await;
        return get_plan_detail(db, plan_id).await;
    }

    let total_levels = approver_chain.len() as i32;
    let first_approver = &approver_chain[0];
    let plan_year = plan.year;

    // ===== 写入审批节点快照 =====
    for (idx, (approver_id, approver_name)) in approver_chain.iter().enumerate() {
        let node = plan_approval_node::ActiveModel {
            plan_id: Set(plan_id),
            level: Set((idx + 1) as i32),
            approver_id: Set(*approver_id),
            approver_name: Set(Some(approver_name.clone())),
            status: Set(Some(0)), // pending
            comment: Set(None),
            ..Default::default()
        };
        node.insert(&txn).await?;
    }

    // ===== 更新计划状态 =====
    let mut active: performance_plan::ActiveModel = plan.into_active_model();
    active.status = Set(Some(new_status));
    active.current_approver_id = Set(Some(first_approver.0));
    active.current_approver_name = Set(Some(first_approver.1.clone()));
    active.approval_level = Set(Some(1));
    active.total_levels = Set(Some(total_levels));
    active.submit_time = Set(Some(now));
    active.update(&txn).await?;

    // ===== 记录审批日志 =====
    let log = plan_approval_log::ActiveModel {
        plan_id: Set(plan_id),
        action: Set(1), // submit
        operator_id: Set(operator_id),
        operator_name: Set(Some(operator_name.to_string())),
        reason: Set(None),
        previous_status: Set(previous_status),
        new_status: Set(Some(new_status)),
        current_level: Set(Some(1)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;

    // 通知第一级审批人
    let _ = send_plan_notice(db, first_approver.0, operator_id,
        &format!("有新的销售计划待您审批（第1级/共{}级）", total_levels),
        &format!("员工 {} 提交了 {} 年销售计划，年合同目标总额 {}，请及时审批。", operator_name, plan_year, total_amount))
        .await;

    get_plan_detail(db, plan_id).await
}

/// 审批通过
/// 校验当前操作人是 plan.current_approver_id，防越权审批
/// 多级审批：当前级通过后流转到下一级审批人；最后一级通过后 status 改为 approved
pub async fn approve_plan(db: &DbConn, req: &ReviewPlanRequest, operator_id: i64, operator_name: &str) -> Result<PlanDetailVO> {
    let txn = db.begin().await?;

    let plan = PerformancePlan::find_by_id(req.plan_id)
        .filter(performance_plan::Column::Deleted.eq(0))
        .one(&txn)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::BadRequest("计划不存在".to_string()))?;

    if plan.status != Some(1) {
        return Err(crate::core::errors::error::Error::BadRequest("仅待审批状态可进行审批操作".to_string()));
    }

    // ===== 校验当前操作人是当前审批人 =====
    let current_approver_id = plan.current_approver_id.unwrap_or(0);
    if current_approver_id == 0 {
        return Err(crate::core::errors::error::Error::BadRequest("该计划无当前审批人，无法审批".to_string()));
    }
    if current_approver_id != operator_id {
        return Err(crate::core::errors::error::Error::BadRequest(
            format!("您不是当前审批人（当前审批人ID={}），无法审批此计划", current_approver_id)
        ));
    }

    let previous_status = plan.status;
    let current_level = plan.approval_level.unwrap_or(1);
    let total_levels = plan.total_levels.unwrap_or(1);
    let employee_id = plan.employee_id;
    let plan_year = plan.year;

    // ===== 更新当前审批节点状态 =====
    let current_node = PlanApprovalNode::find()
        .filter(plan_approval_node::Column::PlanId.eq(req.plan_id))
        .filter(plan_approval_node::Column::Level.eq(current_level))
        .filter(plan_approval_node::Column::Deleted.eq(0))
        .one(&txn)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::BadRequest("审批节点不存在".to_string()))?;

    let mut node_active: plan_approval_node::ActiveModel = current_node.into_active_model();
    node_active.status = Set(Some(1)); // approved
    node_active.comment = Set(req.reason.clone());
    node_active.update(&txn).await?;

    // ===== 判断是否还有下一级 =====
    let next_level = current_level + 1;
    let has_next = next_level <= total_levels;

    let (new_status, new_approver_id, new_approver_name, new_level) = if has_next {
        // 流转到下一级审批人
        let next_node = PlanApprovalNode::find()
            .filter(plan_approval_node::Column::PlanId.eq(req.plan_id))
            .filter(plan_approval_node::Column::Level.eq(next_level))
            .filter(plan_approval_node::Column::Deleted.eq(0))
            .one(&txn)
            .await?;

        if let Some(nn) = next_node {
            (1, Some(nn.approver_id), nn.approver_name.clone(), Some(next_level))
        } else {
            // 下一级节点不存在（数据异常），直接通过
            (2, None, None, Some(total_levels))
        }
    } else {
        // 最后一级，审批完成
        (2, None, None, Some(total_levels))
    };

    // ===== 更新计划状态 =====
    let mut active: performance_plan::ActiveModel = plan.into_active_model();
    active.status = Set(Some(new_status));
    active.current_approver_id = Set(new_approver_id);
    active.current_approver_name = Set(new_approver_name.clone());
    active.approval_level = Set(new_level);
    active.update(&txn).await?;

    // ===== 记录审批日志 =====
    let log = plan_approval_log::ActiveModel {
        plan_id: Set(req.plan_id),
        action: Set(2), // approve
        operator_id: Set(operator_id),
        operator_name: Set(Some(operator_name.to_string())),
        reason: Set(req.reason.clone()),
        previous_status: Set(previous_status),
        new_status: Set(Some(new_status)),
        current_level: Set(Some(current_level)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;

    // ===== 通知 =====
    if new_status == 2 {
        // 最终通过，通知提交人
        let _ = send_plan_notice(db, employee_id, operator_id,
            &format!("您的 {} 年销售计划已审批通过", plan_year),
            &format!("您的 {} 年销售计划已通过全部审批（共{}级），现在可以开始执行。", plan_year, total_levels))
            .await;
    } else if let (Some(next_id), Some(next_name)) = (new_approver_id, new_approver_name.as_ref()) {
        // 流转到下一级，通知下一级审批人
        let _ = send_plan_notice(db, next_id, operator_id,
            &format!("有销售计划待您审批（第{}级/共{}级）", next_level, total_levels),
            &format!("员工提交的 {} 年销售计划已通过第{}级审批，现需您进行第{}级审批。", plan_year, current_level, next_level))
            .await;
    }

    get_plan_detail(db, req.plan_id).await
}

/// 驳回
/// 校验当前操作人是 plan.current_approver_id，reason 必填
pub async fn reject_plan(db: &DbConn, req: &ReviewPlanRequest, operator_id: i64, operator_name: &str) -> Result<PlanDetailVO> {
    let txn = db.begin().await?;

    let plan = PerformancePlan::find_by_id(req.plan_id)
        .filter(performance_plan::Column::Deleted.eq(0))
        .one(&txn)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::BadRequest("计划不存在".to_string()))?;

    if plan.status != Some(1) {
        return Err(crate::core::errors::error::Error::BadRequest("仅待审批状态可进行审批操作".to_string()));
    }

    // ===== 校验当前操作人是当前审批人 =====
    let current_approver_id = plan.current_approver_id.unwrap_or(0);
    if current_approver_id == 0 {
        return Err(crate::core::errors::error::Error::BadRequest("该计划无当前审批人，无法审批".to_string()));
    }
    if current_approver_id != operator_id {
        return Err(crate::core::errors::error::Error::BadRequest(
            format!("您不是当前审批人（当前审批人ID={}），无法审批此计划", current_approver_id)
        ));
    }

    // ===== 驳回原因必填 =====
    let reason = req.reason.as_ref().map(|s| s.trim()).unwrap_or("");
    if reason.is_empty() {
        return Err(crate::core::errors::error::Error::BadRequest("驳回时必须填写原因".to_string()));
    }

    let previous_status = plan.status;
    let current_level = plan.approval_level.unwrap_or(1);
    let employee_id = plan.employee_id;
    let plan_year = plan.year;
    let new_status = 3; // rejected

    // ===== 更新当前审批节点状态 =====
    let current_node = PlanApprovalNode::find()
        .filter(plan_approval_node::Column::PlanId.eq(req.plan_id))
        .filter(plan_approval_node::Column::Level.eq(current_level))
        .filter(plan_approval_node::Column::Deleted.eq(0))
        .one(&txn)
        .await?;

    if let Some(node) = current_node {
        let mut node_active: plan_approval_node::ActiveModel = node.into_active_model();
        node_active.status = Set(Some(2)); // rejected
        node_active.comment = Set(req.reason.clone());
        node_active.update(&txn).await?;
    }

    // ===== 更新计划状态 =====
    let mut active: performance_plan::ActiveModel = plan.into_active_model();
    active.status = Set(Some(new_status));
    active.current_approver_id = Set(None);
    active.current_approver_name = Set(None);
    active.update(&txn).await?;

    // ===== 记录审批日志 =====
    let log = plan_approval_log::ActiveModel {
        plan_id: Set(req.plan_id),
        action: Set(3), // reject
        operator_id: Set(operator_id),
        operator_name: Set(Some(operator_name.to_string())),
        reason: Set(req.reason.clone()),
        previous_status: Set(previous_status),
        new_status: Set(Some(new_status)),
        current_level: Set(Some(current_level)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;

    // 通知提交人计划被驳回
    let _ = send_plan_notice(db, employee_id, operator_id,
        &format!("您的 {} 年销售计划被驳回", plan_year),
        &format!("您的 {} 年销售计划在第{}级审批被驳回。驳回原因：{}", plan_year, current_level, reason))
        .await;

    get_plan_detail(db, req.plan_id).await
}

/// 申请修改（已通过→待审批，version+1）
/// 修改时记录原月度目标快照，重新计算审批链
pub async fn modify_plan(db: &DbConn, req: &ModifyPlanRequest, operator_id: i64, operator_name: &str) -> Result<PlanDetailVO> {
    let txn = db.begin().await?;

    let plan = PerformancePlan::find_by_id(req.plan_id)
        .filter(performance_plan::Column::Deleted.eq(0))
        .one(&txn)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::BadRequest("计划不存在".to_string()))?;

    if plan.status != Some(2) {
        return Err(crate::core::errors::error::Error::BadRequest("仅已通过状态可申请修改".to_string()));
    }

    // 冻结检查
    if plan.is_frozen.unwrap_or(0) == 1 {
        return Err(crate::core::errors::error::Error::BadRequest("该计划已冻结，不可修改".to_string()));
    }

    let previous_status = plan.status;
    let new_status = 1; // submitted (back for review)
    let new_version = plan.version.unwrap_or(1) + 1;
    let employee_id = plan.employee_id;
    let plan_year = plan.year;
    let now = chrono::Local::now().naive_local();

    // ===== 记录原月度目标快照 =====
    let old_targets = PlanMonthlyTarget::find()
        .filter(plan_monthly_target::Column::PlanId.eq(req.plan_id))
        .filter(plan_monthly_target::Column::Deleted.eq(0))
        .all(&txn)
        .await?;
    let snapshot_json = serde_json::to_string(&old_targets.iter().map(|t| {
        serde_json::json!({
            "month": t.month,
            "contract_target_amount": t.contract_target_amount,
            "payment_target_amount": t.payment_target_amount,
            "contract_target_count": t.contract_target_count,
        })
    }).collect::<Vec<_>>()).unwrap_or_else(|_| "[]".to_string());

    // 软删除旧月度目标
    for t in old_targets {
        let mut tm: plan_monthly_target::ActiveModel = t.into_active_model();
        tm.deleted = Set(Some(1));
        tm.update(&txn).await?;
    }

    // 插入新月度目标
    for mt in &req.monthly_targets {
        let monthly = plan_monthly_target::ActiveModel {
            plan_id: Set(req.plan_id),
            month: Set(mt.month),
            contract_target_amount: Set(mt.contract_target_amount),
            payment_target_amount: Set(mt.payment_target_amount),
            contract_target_count: Set(mt.contract_target_count),
            ..Default::default()
        };
        monthly.insert(&txn).await?;
    }

    // ===== 软删除旧审批节点，重新计算审批链 =====
    let old_nodes = PlanApprovalNode::find()
        .filter(plan_approval_node::Column::PlanId.eq(req.plan_id))
        .filter(plan_approval_node::Column::Deleted.eq(0))
        .all(&txn)
        .await?;
    for n in old_nodes {
        let mut nm: plan_approval_node::ActiveModel = n.into_active_model();
        nm.deleted = Set(Some(1));
        nm.update(&txn).await?;
    }

    // 计算新总金额确定层级数
    let new_total: Decimal = req.monthly_targets.iter()
        .map(|t| t.contract_target_amount.unwrap_or(Decimal::from(0)))
        .sum();
    let new_total_f64 = new_total.to_string().parse::<f64>().unwrap_or(0.0);
    let max_levels = if new_total_f64 >= 5_000_000.0 { 3 }
        else if new_total_f64 >= 1_000_000.0 { 2 }
        else { 1 };

    // 重新遍历 direct_manager_id 链
    let all_admins = Admin::find()
        .filter(crate::modules::system::entity::admin::Column::Deleted.eq(0))
        .filter(crate::modules::system::entity::admin::Column::Status.eq(1))
        .all(&txn)
        .await?;
    let admin_map: std::collections::HashMap<i64, &crate::modules::system::entity::admin::Model> =
        all_admins.iter().map(|a| (a.id, a)).collect();

    let mut approver_chain: Vec<(i64, String)> = Vec::new();
    let mut visited: std::collections::HashSet<i64> = std::collections::HashSet::new();
    visited.insert(employee_id);
    let mut current_id = employee_id;
    loop {
        let current_admin = match admin_map.get(&current_id) {
            Some(a) => a,
            None => break,
        };
        let manager_id = match current_admin.direct_manager_id {
            Some(mid) if mid > 0 && mid != employee_id && !visited.contains(&mid) => mid,
            _ => break,
        };
        visited.insert(manager_id);
        let manager_name = admin_map.get(&manager_id)
            .and_then(|a| a.user_name.clone())
            .unwrap_or_else(|| format!("用户{}", manager_id));
        approver_chain.push((manager_id, manager_name));
        current_id = manager_id;
        if approver_chain.len() >= max_levels { break; }
    }

    let (new_status_final, first_approver_id, first_approver_name, total_levels) = if approver_chain.is_empty() {
        (2, None, None, 0)
    } else {
        let total_levels = approver_chain.len() as i32;
        let first = &approver_chain[0];
        // 写入新审批节点
        for (idx, (aid, aname)) in approver_chain.iter().enumerate() {
            let node = plan_approval_node::ActiveModel {
                plan_id: Set(req.plan_id),
                level: Set((idx + 1) as i32),
                approver_id: Set(*aid),
                approver_name: Set(Some(aname.clone())),
                status: Set(Some(0)),
                ..Default::default()
            };
            node.insert(&txn).await?;
        }
        (1, Some(first.0), Some(first.1.clone()), total_levels)
    };

    // 更新计划头
    let mut active: performance_plan::ActiveModel = plan.into_active_model();
    active.status = Set(Some(new_status_final));
    active.version = Set(Some(new_version));
    active.apply_reason = Set(Some(req.reason.clone()));
    active.current_approver_id = Set(first_approver_id);
    active.current_approver_name = Set(first_approver_name.clone());
    active.approval_level = Set(if new_status_final == 2 { Some(0) } else { Some(1) });
    active.total_levels = Set(Some(total_levels));
    active.submit_time = Set(Some(now));
    active.update(&txn).await?;

    // 记录审批日志（含快照）
    let log = plan_approval_log::ActiveModel {
        plan_id: Set(req.plan_id),
        action: Set(4), // modify_request
        operator_id: Set(operator_id),
        operator_name: Set(Some(operator_name.to_string())),
        reason: Set(Some(req.reason.clone())),
        previous_status: Set(previous_status),
        new_status: Set(Some(new_status_final)),
        current_level: Set(Some(if new_status_final == 2 { 0 } else { 1 })),
        snapshot: Set(Some(snapshot_json)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;

    // 通知审批人
    if new_status_final == 1 {
        if let Some(aid) = first_approver_id {
            let _ = send_plan_notice(db, aid, operator_id,
                &format!("有修改后的销售计划待您审批（第1级/共{}级）", total_levels),
                &format!("员工 {} 修改了 {} 年销售计划（版本{}），请及时审批。", operator_name, plan_year, new_version))
                .await;
        }
    } else {
        // 无审批人，自动通过
        let _ = send_plan_notice(db, employee_id, operator_id,
            &format!("您的 {} 年销售计划修改已自动通过", plan_year),
            &format!("您的 {} 年销售计划修改申请因无上级审批人，已自动通过。", plan_year))
            .await;
    }

    get_plan_detail(db, req.plan_id).await
}

/// 更新草稿/驳回状态的月度目标（不改变状态，不记录审批日志）
/// 仅 status=0(草稿) 或 status=3(驳回) 可直接更新
pub async fn update_plan_targets(
    db: &DbConn,
    req: &UpdatePlanTargetsRequest,
    _operator_id: i64,
    _operator_name: &str,
) -> Result<PlanDetailVO> {
    let txn = db.begin().await?;

    let plan = PerformancePlan::find_by_id(req.plan_id)
        .filter(performance_plan::Column::Deleted.eq(0))
        .one(&txn)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::BadRequest("计划不存在".to_string()))?;

    if plan.status != Some(0) && plan.status != Some(3) {
        return Err(crate::core::errors::error::Error::BadRequest(
            "仅草稿或已驳回状态可直接更新目标，已通过状态请走申请修改流程".to_string(),
        ));
    }

    // 软删除旧月度目标
    let old_targets = PlanMonthlyTarget::find()
        .filter(plan_monthly_target::Column::PlanId.eq(req.plan_id))
        .filter(plan_monthly_target::Column::Deleted.eq(0))
        .all(&txn)
        .await?;

    for t in old_targets {
        let mut tm: plan_monthly_target::ActiveModel = t.into_active_model();
        tm.deleted = Set(Some(1));
        tm.update(&txn).await?;
    }

    // 插入新月度目标
    for mt in &req.monthly_targets {
        let monthly = plan_monthly_target::ActiveModel {
            plan_id: Set(req.plan_id),
            month: Set(mt.month),
            contract_target_amount: Set(mt.contract_target_amount),
            payment_target_amount: Set(mt.payment_target_amount),
            contract_target_count: Set(mt.contract_target_count),
            ..Default::default()
        };
        monthly.insert(&txn).await?;
    }

    txn.commit().await?;

    get_plan_detail(db, req.plan_id).await
}

/// 递归获取所有下属员工 ID（通过 direct_manager_id 链）
async fn collect_subordinate_ids(db: &DbConn, manager_id: i64) -> Vec<i64> {
    let all_admins = Admin::find()
        .filter(crate::modules::system::entity::admin::Column::Deleted.eq(0))
        .filter(crate::modules::system::entity::admin::Column::Status.eq(1))
        .all(db)
        .await
        .unwrap_or_default();

    let mut result: Vec<i64> = Vec::new();
    let mut queue: Vec<i64> = vec![manager_id];
    let mut visited: std::collections::HashSet<i64> = std::collections::HashSet::new();
    visited.insert(manager_id);

    while let Some(current) = queue.pop() {
        for a in &all_admins {
            if let Some(mgr) = a.direct_manager_id {
                if mgr == current && !visited.contains(&a.id) {
                    visited.insert(a.id);
                    result.push(a.id);
                    queue.push(a.id);
                }
            }
        }
    }

    result
}

/// 获取进度汇总（个人 + 团队，自下而上汇总模式）
pub async fn get_plan_progress_summary(
    db: &DbConn,
    employee_id: i64,
    year: i32,
) -> Result<PlanProgressSummaryVO> {
    // ===== 个人进度 =====
    let personal = build_progress_for_employees(db, &[employee_id], year).await?;

    // ===== 团队进度（个人 + 所有下属）=====
    let sub_ids = collect_subordinate_ids(db, employee_id).await;
    let mut team_ids: Vec<i64> = vec![employee_id];
    team_ids.extend(sub_ids);

    let team = if team_ids.len() > 1 {
        build_progress_for_employees(db, &team_ids, year).await?
    } else {
        ProgressItemVO::default()
    };

    Ok(PlanProgressSummaryVO { personal, team })
}

/// 为一组员工构建进度汇总
async fn build_progress_for_employees(
    db: &DbConn,
    employee_ids: &[i64],
    year: i32,
) -> Result<ProgressItemVO> {
    // 目标：查询这些员工该年的已通过(status=2)计划月度目标，累加
    let plans = PerformancePlan::find()
        .filter(performance_plan::Column::Deleted.eq(0))
        .filter(performance_plan::Column::Year.eq(year))
        .filter(performance_plan::Column::EmployeeId.is_in(employee_ids.to_vec()))
        .all(db)
        .await?;

    let plan_ids: Vec<i64> = plans.iter().map(|p| p.id).collect();
    let approved_count = plans.iter().filter(|p| p.status == Some(2)).count() as i32;

    let mut total_contract_target = Decimal::from(0);
    let mut total_payment_target = Decimal::from(0);

    if !plan_ids.is_empty() {
        let targets = PlanMonthlyTarget::find()
            .filter(plan_monthly_target::Column::PlanId.is_in(plan_ids.clone()))
            .filter(plan_monthly_target::Column::Deleted.eq(0))
            .all(db)
            .await?;
        for t in targets {
            total_contract_target += t.contract_target_amount.unwrap_or(Decimal::from(0));
            total_payment_target += t.payment_target_amount.unwrap_or(Decimal::from(0));
        }
    }

    // 实际：查询这些员工作为 assigned_to 的已签订合同金额
    let signed_statuses = vec![2_i32, 3, 4, 5];
    let year_start = chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or_default();
    let year_end = chrono::NaiveDate::from_ymd_opt(year, 12, 31).unwrap_or_default();

    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::Status.is_in(signed_statuses))
        .filter(contract::Column::AssignedTo.is_in(employee_ids.to_vec()))
        .filter(contract::Column::SignDate.between(year_start, year_end))
        .all(db)
        .await?;

    let actual_amount: Decimal = contracts
        .iter()
        .map(|c| c.amount.unwrap_or(Decimal::from(0)))
        .sum();

    let completion_rate = if total_contract_target > Decimal::from(0) {
        Some(actual_amount * Decimal::from(100) / total_contract_target)
    } else {
        Some(Decimal::from(0))
    };

    Ok(ProgressItemVO {
        target_amount: Some(total_contract_target),
        actual_amount: Some(actual_amount),
        completion_rate,
        member_count: Some(employee_ids.len() as i32),
        approved_count: Some(approved_count),
    })
}

/// 获取计划列表
/// pending_my_approval=true 时查询当前用户作为 current_approver_id 的待审计划
pub async fn get_plan_list(
    db: &DbConn,
    employee_id: Option<i64>,
    year: Option<i32>,
    status: Option<i32>,
    pending_my_approval: Option<bool>,
    current_user_id: i64,
) -> Result<Vec<PlanListVO>> {
    let mut query = PerformancePlan::find()
        .filter(performance_plan::Column::Deleted.eq(0));

    // 待我审批模式：查询当前用户作为 current_approver_id 且状态为待审批(1)的计划
    if pending_my_approval.unwrap_or(false) {
        query = query
            .filter(performance_plan::Column::CurrentApproverId.eq(current_user_id))
            .filter(performance_plan::Column::Status.eq(1));
    } else if let Some(eid) = employee_id {
        query = query.filter(performance_plan::Column::EmployeeId.eq(eid));
    }

    if let Some(y) = year {
        query = query.filter(performance_plan::Column::Year.eq(y));
    }
    if let Some(s) = status {
        query = query.filter(performance_plan::Column::Status.eq(s));
    }

    query = query.order_by(performance_plan::Column::SubmitTime, sea_orm::Order::Desc)
        .order_by(performance_plan::Column::Id, sea_orm::Order::Desc);

    let plans = query.all(db).await?;

    // 获取员工姓名
    let admin_map = Admin::find()
        .filter(crate::modules::system::entity::admin::Column::Deleted.eq(0))
        .all(db)
        .await?
        .into_iter()
        .map(|a| (a.id, a.user_name))
        .collect::<std::collections::HashMap<i64, Option<String>>>();

    let mut result = Vec::new();
    for p in plans {
        // 获取月度目标汇总
        let targets = PlanMonthlyTarget::find()
            .filter(plan_monthly_target::Column::PlanId.eq(p.id))
            .filter(plan_monthly_target::Column::Deleted.eq(0))
            .all(db)
            .await?;

        let total_contract: Decimal = targets.iter()
            .map(|t| t.contract_target_amount.unwrap_or(Decimal::from(0)))
            .sum();
        let total_payment: Decimal = targets.iter()
            .map(|t| t.payment_target_amount.unwrap_or(Decimal::from(0)))
            .sum();

        result.push(PlanListVO {
            id: Some(p.id),
            employee_id: Some(p.employee_id),
            employee_name: admin_map.get(&p.employee_id).cloned().flatten(),
            year: Some(p.year),
            status: p.status,
            version: p.version,
            total_contract_target: Some(total_contract),
            total_payment_target: Some(total_payment),
            apply_reason: p.apply_reason,
            create_time: p.create_time.map(|t| t.to_string()),
            update_time: p.update_time.map(|t| t.to_string()),
            current_approver_id: p.current_approver_id,
            current_approver_name: p.current_approver_name,
            approval_level: p.approval_level,
            total_levels: p.total_levels,
            submit_time: p.submit_time.map(|t| t.to_string()),
            is_frozen: p.is_frozen,
        });
    }

    Ok(result)
}

/// 获取计划详情（含月度目标和审批记录）
pub async fn get_plan_detail(db: &DbConn, plan_id: i64) -> Result<PlanDetailVO> {
    let plan = PerformancePlan::find_by_id(plan_id)
        .filter(performance_plan::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::BadRequest("计划不存在".to_string()))?;

    // 获取员工姓名
    let admin_map = Admin::find()
        .filter(crate::modules::system::entity::admin::Column::Deleted.eq(0))
        .all(db)
        .await?
        .into_iter()
        .map(|a| (a.id, a.user_name))
        .collect::<std::collections::HashMap<i64, Option<String>>>();

    // 获取月度目标
    let monthly_targets = PlanMonthlyTarget::find()
        .filter(plan_monthly_target::Column::PlanId.eq(plan_id))
        .filter(plan_monthly_target::Column::Deleted.eq(0))
        .order_by(plan_monthly_target::Column::Month, sea_orm::Order::Asc)
        .all(db)
        .await?;

    let monthly_vos: Vec<MonthlyTargetVO> = monthly_targets.into_iter().map(|t| MonthlyTargetVO {
        month: Some(t.month),
        contract_target_amount: t.contract_target_amount,
        payment_target_amount: t.payment_target_amount,
        contract_target_count: t.contract_target_count,
    }).collect();

    // 获取审批记录
    let logs = PlanApprovalLog::find()
        .filter(plan_approval_log::Column::PlanId.eq(plan_id))
        .filter(plan_approval_log::Column::Deleted.eq(0))
        .order_by(plan_approval_log::Column::CreateTime, sea_orm::Order::Asc)
        .all(db)
        .await?;

    let log_vos: Vec<ApprovalLogVO> = logs.into_iter().map(|l| ApprovalLogVO {
        id: Some(l.id),
        action: Some(l.action),
        operator_id: Some(l.operator_id),
        operator_name: l.operator_name,
        reason: l.reason,
        previous_status: l.previous_status,
        new_status: l.new_status,
        create_time: l.create_time.map(|t| t.to_string()),
    }).collect();

    // 获取审批节点链
    let nodes = PlanApprovalNode::find()
        .filter(plan_approval_node::Column::PlanId.eq(plan_id))
        .filter(plan_approval_node::Column::Deleted.eq(0))
        .order_by(plan_approval_node::Column::Level, sea_orm::Order::Asc)
        .all(db)
        .await?;

    let node_vos: Vec<ApprovalNodeVO> = nodes.into_iter().map(|n| ApprovalNodeVO {
        id: Some(n.id),
        level: Some(n.level),
        approver_id: Some(n.approver_id),
        approver_name: n.approver_name,
        status: n.status,
        comment: n.comment,
        create_time: n.create_time.map(|t| t.to_string()),
        update_time: n.update_time.map(|t| t.to_string()),
    }).collect();

    Ok(PlanDetailVO {
        id: Some(plan.id),
        employee_id: Some(plan.employee_id),
        employee_name: admin_map.get(&plan.employee_id).cloned().flatten(),
        year: Some(plan.year),
        status: plan.status,
        apply_reason: plan.apply_reason,
        version: plan.version,
        monthly_targets: Some(monthly_vos),
        approval_logs: Some(log_vos),
        create_time: plan.create_time.map(|t| t.to_string()),
        update_time: plan.update_time.map(|t| t.to_string()),
        current_approver_id: plan.current_approver_id,
        current_approver_name: plan.current_approver_name,
        approval_level: plan.approval_level,
        total_levels: plan.total_levels,
        submit_time: plan.submit_time.map(|t| t.to_string()),
        is_frozen: plan.is_frozen,
        approval_nodes: Some(node_vos),
    })
}

/// 获取计划修改详情（用于编辑回显）
pub async fn get_plan_modify_detail(db: &DbConn, plan_id: i64) -> Result<crate::modules::statistics::model::performance_plan::PlanModifyDetailVO> {
    let plan = PerformancePlan::find_by_id(plan_id)
        .filter(performance_plan::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::BadRequest("计划不存在".to_string()))?;

    let monthly_targets = PlanMonthlyTarget::find()
        .filter(plan_monthly_target::Column::PlanId.eq(plan_id))
        .filter(plan_monthly_target::Column::Deleted.eq(0))
        .order_by(plan_monthly_target::Column::Month, sea_orm::Order::Asc)
        .all(db)
        .await?;

    let monthly_vos: Vec<MonthlyTargetVO> = monthly_targets.into_iter().map(|t| MonthlyTargetVO {
        month: Some(t.month),
        contract_target_amount: t.contract_target_amount,
        payment_target_amount: t.payment_target_amount,
        contract_target_count: t.contract_target_count,
    }).collect();

    Ok(crate::modules::statistics::model::performance_plan::PlanModifyDetailVO {
        plan: Some(PlanListVO {
            id: Some(plan.id),
            employee_id: Some(plan.employee_id),
            employee_name: None,
            year: Some(plan.year),
            status: plan.status,
            version: plan.version,
            total_contract_target: None,
            total_payment_target: None,
            apply_reason: plan.apply_reason,
            create_time: plan.create_time.map(|t| t.to_string()),
            update_time: plan.update_time.map(|t| t.to_string()),
            current_approver_id: plan.current_approver_id,
            current_approver_name: plan.current_approver_name,
            approval_level: plan.approval_level,
            total_levels: plan.total_levels,
            submit_time: plan.submit_time.map(|t| t.to_string()),
            is_frozen: plan.is_frozen,
        }),
        monthly_targets: Some(monthly_vos),
    })
}

/// 发送销售计划站内信通知
/// 复用客户转移的通知模式：NoticeModel::insert + notice_service::update_by_id_publish
async fn send_plan_notice(
    db: &DbConn,
    to_user_id: i64,
    operator_id: i64,
    title: &str,
    content: &str,
) -> Result<()> {
    use crate::modules::system::model::notice::{NoticeModel, NoticeSaveDTO};

    let now = chrono::Local::now().naive_local();
    let save_dto = NoticeSaveDTO {
        id: None,
        title: Some(title.to_string()),
        content: Some(format!("<p>{}</p>", content)),
        r#type: Some(3), // 3=系统消息
        level: Some("high".to_string()),
        target_type: Some(2), // 2=指定用户
        target_user_ids: Some(to_user_id.to_string()),
        publisher_id: Some(operator_id),
        publish_status: Some(0),
        publish_time: Some(now),
        revoke_time: None,
        create_by: Some(operator_id),
        create_time: Some(now),
        update_by: Some(operator_id),
        update_time: Some(now),
    };

    let notice_id = NoticeModel::insert(db, &save_dto).await?;
    let _ = crate::modules::system::service::notice_service::update_by_id_publish(
        db,
        &Some(notice_id),
        &Some(operator_id),
    )
    .await;
    Ok(())
}