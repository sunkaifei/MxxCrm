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
use crate::modules::statistics::model::performance_plan::{
    PlanDetailVO, PlanListVO, MonthlyTargetVO, ApprovalLogVO,
    CreatePlanRequest, MonthlyTargetInput, ReviewPlanRequest, ModifyPlanRequest,
};
use crate::modules::system::entity::admin::Entity as Admin;
use sea_orm::{
    DbConn, TransactionTrait,
    EntityTrait, ColumnTrait, QueryFilter, QueryOrder, ActiveModelTrait, Set, IntoActiveModel,
};
use sea_orm::prelude::Decimal;

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

    let previous_status = plan.status;
    let new_status = 1; // submitted

    // 更新状态
    let mut active: performance_plan::ActiveModel = plan.into_active_model();
    active.status = Set(Some(new_status));
    active.update(&txn).await?;

    // 记录审批日志
    let log = plan_approval_log::ActiveModel {
        plan_id: Set(plan_id),
        action: Set(1), // submit
        operator_id: Set(operator_id),
        operator_name: Set(Some(operator_name.to_string())),
        reason: Set(None),
        previous_status: Set(previous_status),
        new_status: Set(Some(new_status)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;

    get_plan_detail(db, plan_id).await
}

/// 审批通过
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

    let previous_status = plan.status;
    let new_status = 2; // approved

    let mut active: performance_plan::ActiveModel = plan.into_active_model();
    active.status = Set(Some(new_status));
    active.update(&txn).await?;

    let log = plan_approval_log::ActiveModel {
        plan_id: Set(req.plan_id),
        action: Set(2), // approve
        operator_id: Set(operator_id),
        operator_name: Set(Some(operator_name.to_string())),
        reason: Set(req.reason.clone()),
        previous_status: Set(previous_status),
        new_status: Set(Some(new_status)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;

    get_plan_detail(db, req.plan_id).await
}

/// 驳回
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

    let previous_status = plan.status;
    let new_status = 3; // rejected

    let mut active: performance_plan::ActiveModel = plan.into_active_model();
    active.status = Set(Some(new_status));
    active.update(&txn).await?;

    let log = plan_approval_log::ActiveModel {
        plan_id: Set(req.plan_id),
        action: Set(3), // reject
        operator_id: Set(operator_id),
        operator_name: Set(Some(operator_name.to_string())),
        reason: Set(req.reason.clone()),
        previous_status: Set(previous_status),
        new_status: Set(Some(new_status)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;

    get_plan_detail(db, req.plan_id).await
}

/// 申请修改（已通过→待审批，version+1）
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

    let previous_status = plan.status;
    let new_status = 1; // submitted (back for review)
    let new_version = plan.version.unwrap_or(1) + 1;

    // 更新计划头
    let mut active: performance_plan::ActiveModel = plan.into_active_model();
    active.status = Set(Some(new_status));
    active.version = Set(Some(new_version));
    active.apply_reason = Set(Some(req.reason.clone()));
    active.update(&txn).await?;

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

    // 记录审批日志
    let log = plan_approval_log::ActiveModel {
        plan_id: Set(req.plan_id),
        action: Set(4), // modify_request
        operator_id: Set(operator_id),
        operator_name: Set(Some(operator_name.to_string())),
        reason: Set(Some(req.reason.clone())),
        previous_status: Set(previous_status),
        new_status: Set(Some(new_status)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;

    get_plan_detail(db, req.plan_id).await
}

/// 获取计划列表
pub async fn get_plan_list(db: &DbConn, employee_id: Option<i64>, year: Option<i32>, status: Option<i32>) -> Result<Vec<PlanListVO>> {
    let mut query = PerformancePlan::find()
        .filter(performance_plan::Column::Deleted.eq(0));

    if let Some(eid) = employee_id {
        query = query.filter(performance_plan::Column::EmployeeId.eq(eid));
    }
    if let Some(y) = year {
        query = query.filter(performance_plan::Column::Year.eq(y));
    }
    if let Some(s) = status {
        query = query.filter(performance_plan::Column::Status.eq(s));
    }

    query = query.order_by(performance_plan::Column::Year, sea_orm::Order::Desc)
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
        }),
        monthly_targets: Some(monthly_vos),
    })
}