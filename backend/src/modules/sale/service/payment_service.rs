//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::approval::service::approval_service::ApprovalService;
use crate::modules::approval::model::approval::{ApprovalSubmitRequest, ApprovalProcessRequest};
use crate::modules::crm::entity::contract_payment_plan;
use crate::modules::crm::entity::customer::{Entity as Customer, Column as CustomerColumn};
use crate::modules::sale::entity::payment as payment_entity;
use crate::modules::sale::model::payment::{
    PaymentApplyRequest, PaymentApprovalDetailVO, PaymentDetailVO, PaymentListQuery, PaymentListVO,
    PaymentModel, PaymentPlanForApplyVO, PaymentSaveDTO, PaymentSaveRequest,
    PaymentUnappliedVO, PaymentUpdateRequest,
};
use crate::modules::sale::model::payment_application::{
    PaymentApplicationModel, PaymentApplicationVO,
};
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait};
use std::collections::{HashMap, HashSet};

pub async fn insert(db: &DbConn, form_data: &PaymentSaveRequest, created_by: i64) -> Result<i64> {
    let txn = db.begin().await?;

    let date_prefix = format!("HK{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = PaymentModel::get_max_payment_no_today(&txn, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let payment_no = format!("{}{:04}", date_prefix, seq);

    let mut dto: PaymentSaveDTO = form_data.clone().into();
    dto.payment_no = Some(payment_no);
    dto.status = Some(1);
    dto.amount = Some(dto.amount.unwrap_or(Decimal::from(0)));
    dto.currency = Some(dto.currency.unwrap_or(1));
    dto.payment_method = Some(dto.payment_method.unwrap_or(1));
    dto.create_by = Some(created_by.to_string());

    let payment_id = PaymentModel::insert(&txn, &dto).await?;

    txn.commit().await?;

    Ok(payment_id)
}

pub async fn update(db: &DbConn, form_data: &PaymentUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("回款ID不能为空"));
    }

    let existing = PaymentModel::find_by_id(db, id).await?;
    if existing.is_none() {
        return Err(Error::from("回款记录不存在"));
    }

    let mut dto: PaymentSaveDTO = form_data.clone().into();
    dto.update_by = Some(updated_by.to_string());

    PaymentModel::update_by_id(db, id, &dto).await?;

    Ok(id)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = PaymentModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<PaymentDetailVO> {
    let payment = PaymentModel::find_by_id(db, id).await?;
    match payment {
        Some(p) => {
            let mut vo: PaymentDetailVO = (&p).into();
            let apps = PaymentApplicationModel::find_by_payment(db, id).await?;
            vo.applications = apps.iter().map(|m| m.into()).collect();
            Ok(vo)
        }
        None => Err(Error::from("回款记录不存在")),
    }
}

async fn get_accessible_user_ids(
    db: &DbConn,
    current_user_id: i64,
    data_scope: Option<i32>,
) -> Result<Option<Vec<i64>>> {
    match data_scope {
        Some(1) => Ok(None),
        Some(5) => Ok(Some(vec![current_user_id])),
        Some(3) | Some(4) | Some(2) | Some(0) | Some(_) => {
            let user_depts = AdminDeptMergeModel::find_by_admin_id(db, current_user_id).await
                .map_err(|e| Error::from(format!("查询用户部门失败: {}", e)))?;

            let mut target_dept_ids = Vec::new();

            if data_scope == Some(2) {
                let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
                for role in roles {
                    if role.data_scope == Some(2) {
                        if let Some(role_id) = role.id {
                            let dept_result = crate::modules::system::model::role_dept_merge::RoleDeptMergeModel::find_by_role_id(db, &Some(role_id)).await
                                .map_err(|e| Error::from(format!("查询角色部门关联失败: {}", e)))?;
                            for merge in dept_result {
                                if let Some(dept_id) = merge.dept_id {
                                    target_dept_ids.push(dept_id);
                                }
                            }
                        }
                    }
                }
            } else {
                for merge in &user_depts {
                    if let Some(dept_id) = merge.dept_id {
                        target_dept_ids.push(dept_id);
                    }
                }
            }

            if target_dept_ids.is_empty() {
                return Ok(Some(vec![current_user_id]));
            }

            let all_depts = DeptModel::find_all(db).await
                .map_err(|e| Error::from(format!("查询部门列表失败: {}", e)))?;

            let mut all_target_ids = Vec::new();
            for dept_id in &target_dept_ids {
                if data_scope == Some(4) || data_scope == Some(2) {
                    all_target_ids.extend(collect_child_dept_ids(&all_depts, *dept_id));
                } else {
                    all_target_ids.push(*dept_id);
                }
            }

            all_target_ids.sort();
            all_target_ids.dedup();

            let dept_merges = AdminDeptMergeModel::find_by_dept_id(db, all_target_ids).await
                .map_err(|e| Error::from(format!("查询部门用户失败: {}", e)))?;

            let mut user_ids: Vec<i64> = dept_merges.iter()
                .filter_map(|m| m.admin_id)
                .collect();
            user_ids.sort();
            user_ids.dedup();

            if user_ids.is_empty() {
                Ok(Some(vec![current_user_id]))
            } else {
                Ok(Some(user_ids))
            }
        }
        None => Ok(None),
    }
}

fn collect_child_dept_ids(all_depts: &[crate::modules::system::entity::dept::Model], parent_id: i64) -> Vec<i64> {
    let mut result = vec![parent_id];
    for dept in all_depts {
        if dept.parent_id == Some(parent_id) {
            result.extend(collect_child_dept_ids(all_depts, dept.id));
        }
    }
    result
}

pub async fn list(db: &DbConn, query: &PaymentListQuery, current_user_id: i64) -> Result<ResultPage<Vec<PaymentListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("all");

    let owner_user_ids_opt: Option<Vec<i64>> = match list_type {
        "my" => {
            Some(vec![current_user_id])
        }
        "subordinate" => {
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter()
                .filter_map(|r| r.data_scope)
                .min();

            match data_scope {
                Some(5) => {
                    Some(Vec::new())
                }
                Some(1) | None => {
                    let all_admins = Admin::find()
                        .filter(admin::Column::Id.ne(current_user_id))
                        .all(db)
                        .await
                        .map_err(|e| Error::from(format!("查询用户列表失败: {}", e)))?;
                    Some(all_admins.iter().map(|u| u.id).collect())
                }
                _ => {
                    let user_ids = get_accessible_user_ids(db, current_user_id, data_scope).await?
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|id| *id != current_user_id)
                        .collect::<Vec<_>>();
                    Some(user_ids)
                }
            }
        }
        _ => {
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter()
                .filter_map(|r| r.data_scope)
                .min();
            get_accessible_user_ids(db, current_user_id, data_scope).await?
        }
    };

    let (list, total) = if list_type == "my" {
        PaymentModel::select_in_page(
            db,
            page,
            page_size,
            query.payment_no.clone(),
            query.order_no.clone(),
            query.contract_id,
            query.customer_id,
            query.status,
            query.payment_method,
            Some(current_user_id),
        ).await?
    } else {
        PaymentModel::select_in_page_by_owner_user_ids(
            db,
            page,
            page_size,
            query.payment_no.clone(),
            query.order_no.clone(),
            query.contract_id,
            query.customer_id,
            query.status,
            query.payment_method,
            owner_user_ids_opt,
        ).await?
    };

    let order_ids: Vec<i64> = list.iter().filter_map(|p| p.order_id).collect();
    let orders = PaymentModel::find_orders_by_ids(db, &order_ids).await?;
    let order_map: HashMap<i64, String> = orders.into_iter()
        .filter_map(|o| o.order_no.map(|no| (o.id, no)))
        .collect();

    // 批量查询客户名称（ID -> 名称），确保历史数据 customer_name 为空时也能显示
    let customer_ids: Vec<i64> = list.iter()
        .filter_map(|p| p.customer_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let customer_name_map: HashMap<i64, String> = if !customer_ids.is_empty() {
        Customer::find()
            .filter(CustomerColumn::Id.is_in(customer_ids.clone()))
            .all(db)
            .await?
            .into_iter()
            .map(|c| (c.id, c.company_name.or(c.short_name).unwrap_or_default()))
            .collect()
    } else {
        HashMap::new()
    };

    let mut data: Vec<PaymentListVO> = list.iter().map(|item| {
        let mut vo: PaymentListVO = item.into();
        if let Some(oid) = item.order_id {
            vo.order_no = order_map.get(&oid).cloned();
        }
        // 若记录中未存客户名称，则从客户表实时取
        if vo.customer_name.as_deref().map(|s| s.is_empty()).unwrap_or(true) {
            if let Some(cid) = item.customer_id {
                if let Some(name) = customer_name_map.get(&cid) {
                    vo.customer_name = Some(name.clone());
                }
            }
        }
        vo
    }).collect();

    Ok(ResultPage { items: data, total, current_page: page, page_size, total_pages: 0 })
}

/// 确认回款：status→2，设 confirm_time/confirm_by，联动订单 paid_amount
pub async fn confirm(db: &DbConn, payment_id: i64, user_id: i64) -> Result<i64> {
    let payment = PaymentModel::find_by_id(db, payment_id).await?
        .ok_or_else(|| Error::from("回款记录不存在"))?;

    if payment.status != Some(1) {
        return Err(Error::from("回款状态不是待确认，无法确认"));
    }

    let txn = db.begin().await?;

    PaymentModel::update_confirm(&txn, payment_id, user_id).await?;

    // 联动订单 paid_amount
    if let Some(order_id) = payment.order_id {
        let amount = payment.amount.unwrap_or(Decimal::from(0));
        if amount > Decimal::from(0) {
            PaymentModel::update_order_paid_amount(&txn, order_id, amount).await?;
        }
    }

    txn.commit().await?;
    Ok(payment_id)
}

/// 驳回回款：status→3
pub async fn reject(db: &DbConn, payment_id: i64) -> Result<i64> {
    let payment = PaymentModel::find_by_id(db, payment_id).await?
        .ok_or_else(|| Error::from("回款记录不存在"))?;

    if payment.status != Some(1) {
        return Err(Error::from("回款状态不是待确认，无法驳回"));
    }

    PaymentModel::update_reject(db, payment_id).await?;
    Ok(payment_id)
}

/// 核销操作：将回款金额分配到一个或多个回款计划
pub async fn apply(db: &DbConn, req: &PaymentApplyRequest, user_id: i64) -> Result<i64> {
    let payment_id = req.payment_id.unwrap_or_default();
    if payment_id == 0 {
        return Err(Error::from("回款ID不能为空"));
    }
    if req.applications.is_empty() {
        return Err(Error::from("核销明细不能为空"));
    }

    // 1. 校验回款状态为已确认（status=2）
    let payment = PaymentModel::find_by_id(db, payment_id).await?
        .ok_or_else(|| Error::from("回款记录不存在"))?;

    if payment.status != Some(2) {
        return Err(Error::from("回款状态不是已确认，无法核销"));
    }

    // 2. 校验核销总额 ≤ 回款未核销金额
    let total_apply: Decimal = req.applications.iter()
        .map(|item| item.apply_amount)
        .sum();
    let unapplied = payment.unapplied_amount.unwrap_or(Decimal::from(0));
    if total_apply > unapplied {
        return Err(Error::from(format!("核销总额({})超过未核销金额({})", total_apply, unapplied)));
    }

    // 3. 校验每个计划的核销金额 ≤ 计划未收金额
    let mut plan_map: HashMap<i64, contract_payment_plan::Model> = HashMap::new();
    for item in &req.applications {
        if let Some(plan_id) = item.plan_id {
            if plan_map.contains_key(&plan_id) {
                continue;
            }
            let plan = contract_payment_plan::Entity::find_by_id(plan_id)
                .filter(contract_payment_plan::Column::Deleted.eq(0))
                .one(db).await?
                .ok_or_else(|| Error::from(format!("回款计划不存在: {}", plan_id)))?;

            let plan_amount = plan.plan_amount.unwrap_or(Decimal::from(0));
            let received = plan.received_amount.unwrap_or(Decimal::from(0));
            let remaining = plan_amount - received;
            if item.apply_amount > remaining {
                return Err(Error::from(format!("计划{}核销金额({})超过计划未收金额({})", plan_id, item.apply_amount, remaining)));
            }
            plan_map.insert(plan_id, plan);
        }
    }

    // 4. 事务中执行核销
    let txn = db.begin().await?;

    // 批量插入核销记录
    PaymentApplicationModel::insert_batch(&txn, payment_id, payment.contract_id, &req.applications, user_id).await?;

    // 更新 payment.applied_amount 和 unapplied_amount
    let new_applied = payment.applied_amount.unwrap_or(Decimal::from(0)) + total_apply;
    let new_unapplied = unapplied - total_apply;
    PaymentModel::update_amounts(&txn, payment_id, new_applied, new_unapplied).await?;

    // 更新 plan.received_amount 和状态
    let now = chrono::Local::now().naive_local().to_owned();
    let today = chrono::Local::now().date_naive();

    // 按计划聚合核销金额
    let mut plan_apply_map: HashMap<i64, Decimal> = HashMap::new();
    for item in &req.applications {
        if let Some(plan_id) = item.plan_id {
            *plan_apply_map.entry(plan_id).or_insert(Decimal::from(0)) += item.apply_amount;
        }
    }

    for (plan_id, apply_amt) in &plan_apply_map {
        let plan = plan_map.get(plan_id)
            .ok_or_else(|| Error::from(format!("回款计划不存在: {}", plan_id)))?;
        let plan_amount = plan.plan_amount.unwrap_or(Decimal::from(0));
        let old_received = plan.received_amount.unwrap_or(Decimal::from(0));
        let new_received = old_received + *apply_amt;
        let new_status = if new_received >= plan_amount { 4 } else if new_received > Decimal::from(0) { 1 } else { 0 };

        let mut am = contract_payment_plan::ActiveModel {
            received_amount: Set(Some(new_received)),
            status: Set(Some(new_status)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        if new_received >= plan_amount {
            am.actual_date = Set(Some(today));
        }
        contract_payment_plan::Entity::update_many()
            .set(am)
            .filter(contract_payment_plan::Column::Id.eq(*plan_id))
            .filter(contract_payment_plan::Column::Deleted.eq(0))
            .exec(&txn).await?;
    }

    txn.commit().await?;
    Ok(payment_id)
}

/// 取消核销：回滚 payment 和 plan 金额，软删除核销记录
pub async fn cancel_apply(db: &DbConn, application_id: i64) -> Result<i64> {
    // 1. 查询核销记录
    let app = PaymentApplicationModel::find_by_id(db, application_id).await?
        .ok_or_else(|| Error::from("核销记录不存在"))?;

    let apply_amount = app.apply_amount.unwrap_or(Decimal::from(0));
    let payment_id = app.payment_id.unwrap_or_default();
    if payment_id == 0 {
        return Err(Error::from("核销记录关联的回款ID无效"));
    }

    // 2. 事务中回滚
    let txn = db.begin().await?;

    // 回滚 payment.applied_amount 和 unapplied_amount
    let payment = PaymentModel::find_by_id(&txn, payment_id).await?
        .ok_or_else(|| Error::from("回款记录不存在"))?;
    let old_applied = payment.applied_amount.unwrap_or(Decimal::from(0));
    let old_unapplied = payment.unapplied_amount.unwrap_or(Decimal::from(0));
    let new_applied = old_applied - apply_amount;
    let new_unapplied = old_unapplied + apply_amount;
    PaymentModel::update_amounts(&txn, payment_id, new_applied, new_unapplied).await?;

    // 回滚 plan.received_amount 和状态
    if let Some(plan_id) = app.plan_id {
        let plan = contract_payment_plan::Entity::find_by_id(plan_id)
            .filter(contract_payment_plan::Column::Deleted.eq(0))
            .one(&txn).await?;
        if let Some(p) = plan {
            let plan_amount = p.plan_amount.unwrap_or(Decimal::from(0));
            let old_received = p.received_amount.unwrap_or(Decimal::from(0));
            let new_received = old_received - apply_amount;
            let new_status = if new_received >= plan_amount { 4 } else if new_received > Decimal::from(0) { 1 } else { 0 };
            let now = chrono::Local::now().naive_local().to_owned();

            let mut am = contract_payment_plan::ActiveModel {
                received_amount: Set(Some(new_received)),
                status: Set(Some(new_status)),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            if new_received < plan_amount {
                am.actual_date = Set(None);
            }
            contract_payment_plan::Entity::update_many()
                .set(am)
                .filter(contract_payment_plan::Column::Id.eq(plan_id))
                .filter(contract_payment_plan::Column::Deleted.eq(0))
                .exec(&txn).await?;
        }
    }

    // 软删除核销记录
    PaymentApplicationModel::delete_by_id(&txn, application_id).await?;

    txn.commit().await?;
    Ok(application_id)
}

/// 查询回款未核销金额及可核销计划列表
pub async fn get_unapplied(db: &DbConn, payment_id: i64) -> Result<PaymentUnappliedVO> {
    let payment = PaymentModel::find_by_id(db, payment_id).await?
        .ok_or_else(|| Error::from("回款记录不存在"))?;

    let mut plans: Vec<PaymentPlanForApplyVO> = Vec::new();
    if let Some(contract_id) = payment.contract_id {
        let plan_list = contract_payment_plan::Entity::find()
            .filter(contract_payment_plan::Column::ContractId.eq(contract_id))
            .filter(contract_payment_plan::Column::Deleted.eq(0))
            .order_by_asc(contract_payment_plan::Column::Sort)
            .all(db).await?;

        for p in plan_list {
            let plan_amount = p.plan_amount.unwrap_or(Decimal::from(0));
            let received = p.received_amount.unwrap_or(Decimal::from(0));
            let remaining = plan_amount - received;
            if remaining > Decimal::from(0) {
                plans.push(PaymentPlanForApplyVO {
                    id: Some(p.id),
                    stage_name: p.stage_name,
                    plan_amount: Some(plan_amount),
                    received_amount: Some(received),
                    unapplied_amount: Some(remaining),
                    status: p.status,
                });
            }
        }
    }

    Ok(PaymentUnappliedVO {
        payment_id: Some(payment_id),
        amount: payment.amount,
        applied_amount: payment.applied_amount,
        unapplied_amount: payment.unapplied_amount,
        contract_id: payment.contract_id,
        plans,
    })
}

/// 查询回款的核销明细列表
pub async fn get_applications(db: &DbConn, payment_id: i64) -> Result<Vec<PaymentApplicationVO>> {
    let list = PaymentApplicationModel::find_by_payment(db, payment_id).await?;
    Ok(list.iter().map(|m| m.into()).collect())
}

// ==================== 回款审批 ====================

/// 提交回款审批
pub async fn submit_payment(db: &DbConn, payment_id: i64, operator_id: i64, operator_name: &str) -> Result<PaymentDetailVO> {
    let payment = PaymentModel::find_by_id(db, payment_id).await?
        .ok_or_else(|| Error::from("回款记录不存在"))?;

    if payment.approval_status != Some(0) && payment.approval_status != Some(4) {
        return Err(Error::from("当前状态不允许提交，仅草稿或已驳回状态可提交"));
    }

    let business_title = payment.payment_no.clone()
        .or_else(|| payment.customer_name.clone());
    let amount = payment.amount.unwrap_or(Decimal::from(0));

    // 调用审批引擎提交
    let submit_req = ApprovalSubmitRequest {
        flow_code: "payment_approval".to_string(),
        business_type: "payment".to_string(),
        business_id: payment_id,
        business_title,
        submitter_id: operator_id,
        submitter_name: Some(operator_name.to_string()),
        extra_data: Some(serde_json::json!({ "amount": amount })),
    };
    let instance_id = ApprovalService::submit(db, &submit_req).await?;

    // 事务更新回款表
    let txn = db.begin().await?;
    let mut active: payment_entity::ActiveModel = payment.into_active_model();
    active.approval_status = Set(Some(1));
    active.instance_id = Set(Some(instance_id));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    active.update(&txn).await?;
    txn.commit().await?;

    find_by_id(db, payment_id).await
}

/// 审批通过回款
pub async fn approve_payment(db: &DbConn, payment_id: i64, operator_id: i64, operator_name: &str, reason: Option<String>) -> Result<PaymentDetailVO> {
    let payment = PaymentModel::find_by_id(db, payment_id).await?
        .ok_or_else(|| Error::from("回款记录不存在"))?;

    if payment.approval_status != Some(1) && payment.approval_status != Some(2) {
        return Err(Error::from("仅待审批或审批中状态可进行审批操作"));
    }

    let instance_id = payment.instance_id
        .ok_or_else(|| Error::from("审批实例不存在，请重新提交审批"))?;

    // 调用审批引擎处理（通过）
    let process_req = ApprovalProcessRequest {
        instance_id,
        action: 1,
        approver_id: operator_id,
        approver_name: Some(operator_name.to_string()),
        comment: reason,
    };
    ApprovalService::process(db, &process_req).await?;

    // 查询实例最新状态，判断审批是否完成
    let instance = ApprovalService::find_instance_by_id(db, instance_id).await?
        .ok_or_else(|| Error::from("审批实例不存在"))?;
    let new_status = if instance.status == 3 { 3 } else { 2 };

    // 审批完成时需联动订单 paid_amount，提前保存原值
    let order_id = payment.order_id;
    let amount = payment.amount.unwrap_or(Decimal::from(0));

    // 事务更新回款表
    let txn = db.begin().await?;
    let mut active: payment_entity::ActiveModel = payment.into_active_model();
    active.approval_status = Set(Some(new_status));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));

    // 审批完成时执行确认联动：更新订单 paid_amount
    if new_status == 3 {
        active.status = Set(Some(2));
        active.confirm_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
        active.confirm_by = Set(Some(operator_id));
    }
    active.update(&txn).await?;

    if new_status == 3 {
        if let Some(oid) = order_id {
            if amount > Decimal::from(0) {
                PaymentModel::update_order_paid_amount(&txn, oid, amount).await?;
            }
        }
    }

    txn.commit().await?;

    find_by_id(db, payment_id).await
}

/// 驳回回款
pub async fn reject_payment(db: &DbConn, payment_id: i64, operator_id: i64, operator_name: &str, reason: Option<String>) -> Result<PaymentDetailVO> {
    let payment = PaymentModel::find_by_id(db, payment_id).await?
        .ok_or_else(|| Error::from("回款记录不存在"))?;

    if payment.approval_status != Some(1) && payment.approval_status != Some(2) {
        return Err(Error::from("仅待审批或审批中状态可进行驳回操作"));
    }

    let instance_id = payment.instance_id
        .ok_or_else(|| Error::from("审批实例不存在，请重新提交审批"))?;

    // 调用审批引擎处理（驳回）
    let process_req = ApprovalProcessRequest {
        instance_id,
        action: 2,
        approver_id: operator_id,
        approver_name: Some(operator_name.to_string()),
        comment: reason,
    };
    ApprovalService::process(db, &process_req).await?;

    // 事务更新回款表
    let txn = db.begin().await?;
    let mut active: payment_entity::ActiveModel = payment.into_active_model();
    active.approval_status = Set(Some(4));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    active.update(&txn).await?;
    txn.commit().await?;

    find_by_id(db, payment_id).await
}

/// 获取回款审批详情
pub async fn get_payment_approval_detail(db: &DbConn, payment_id: i64) -> Result<PaymentApprovalDetailVO> {
    let payment = PaymentModel::find_by_id(db, payment_id).await?
        .ok_or_else(|| Error::from("回款记录不存在"))?;

    let instance = if let Some(iid) = payment.instance_id {
        ApprovalService::find_instance_by_id(db, iid).await?
    } else {
        None
    };

    Ok(PaymentApprovalDetailVO {
        payment_id: Some(payment.id),
        payment_no: payment.payment_no,
        customer_name: payment.customer_name,
        amount: payment.amount,
        approval_status: payment.approval_status,
        instance,
    })
}

