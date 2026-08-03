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
use crate::core::r#enum::contract_status_enum::ContractStatus;
use crate::modules::approval::service::approval_service::ApprovalService;
use crate::modules::approval::model::approval::{ApprovalSubmitRequest, ApprovalProcessRequest};
use crate::modules::crm::model::contract::{ContractApprovalDetailVO, ContractApprovalLogVO, ContractApprovalRequest, ContractDetailVO, ContractListQuery, ContractListVO, ContractModel, ContractSaveDTO};
use crate::modules::crm::entity::{contract, contract_approval_log, contract::Entity as Contract, contract_approval_log::Entity as ContractApprovalLog, customer::{Entity as Customer, Column as CustomerColumn}};
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
use sea_orm::{DbConn, TransactionTrait, Set, IntoActiveModel, ActiveModelTrait, EntityTrait, ColumnTrait, QueryFilter, QueryOrder, QuerySelect, Condition};
use std::collections::{HashMap, HashSet};
use sea_orm::prelude::Decimal;
use crate::modules::sale::entity::order;

pub async fn insert(db: &DbConn, form_data: &ContractSaveDTO, created_by: i64) -> Result<i64> {
    // 数据完整性校验：客户必填、标题必填
    let customer_id = form_data.customer_id.ok_or_else(|| Error::from("客户不能为空".to_string()))?;
    let title = form_data.title.as_ref().ok_or_else(|| Error::from("合同标题不能为空".to_string()))?;

    let txn = db.begin().await?;

    // 同公司标题唯一性校验
    let existing = ContractModel::find_by_customer_and_title(&txn, customer_id, title, None).await?;
    if existing.is_some() {
        txn.rollback().await?;
        return Err(Error::from("该客户下已存在相同标题的合同".to_string()));
    }

    // 订单存在性校验：若传入 order_id，必须查询到未删除的订单
    if let Some(oid) = form_data.order_id {
        let ord = order::Entity::find_by_id(oid)
            .filter(order::Column::Deleted.eq(0))
            .one(&txn)
            .await
            .map_err(|e| Error::from(format!("查询订单失败: {}", e)))?;
        if ord.is_none() {
            txn.rollback().await?;
            return Err(Error::from(format!("关联的订单(id={})不存在或已删除", oid)));
        }
    }

    let mut dto = form_data.clone();
    dto.created_by = Some(created_by);
    dto.approval_status = Some(0);
    let result = ContractModel::insert(&txn, &dto).await?;

    txn.commit().await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &ContractSaveDTO, updated_by: i64) -> Result<i64> {
    let id = form_data.id.unwrap_or(0);
    if id == 0 {
        return Err(Error::from("合同ID不能为空".to_string()));
    }

    // 数据完整性校验：客户必填、标题必填
    let customer_id = form_data.customer_id.ok_or_else(|| Error::from("客户不能为空".to_string()))?;
    let title = form_data.title.as_ref().ok_or_else(|| Error::from("合同标题不能为空".to_string()))?;

    // 合同存在性校验
    let existing = ContractModel::find_by_id(db, id).await?;
    if existing.is_none() {
        return Err(Error::from("合同不存在".to_string()));
    }

    // 审批状态校验：仅草稿(0)或已驳回(4)允许修改；审批中(2)/已签署等不可修改
    let approval_status = existing.as_ref().unwrap().approval_status.unwrap_or(0);
    if approval_status == 2 {
        return Err(Error::from("合同审批中，不允许修改".to_string()));
    }
    if approval_status == 3 {
        return Err(Error::from("合同已审批通过，不允许修改".to_string()));
    }

    let txn = db.begin().await?;

    // 同公司标题唯一性校验（排除自身 ID）
    let dup = ContractModel::find_by_customer_and_title(&txn, customer_id, title, form_data.id).await?;
    if dup.is_some() {
        txn.rollback().await?;
        return Err(Error::from("该客户下已存在相同标题的合同".to_string()));
    }

    // 订单存在性校验：若传入 order_id，必须查询到未删除的订单
    if let Some(oid) = form_data.order_id {
        let ord = order::Entity::find_by_id(oid)
            .filter(order::Column::Deleted.eq(0))
            .one(&txn)
            .await
            .map_err(|e| Error::from(format!("查询订单失败: {}", e)))?;
        if ord.is_none() {
            txn.rollback().await?;
            return Err(Error::from(format!("关联的订单(id={})不存在或已删除", oid)));
        }
    }

    let mut dto = form_data.clone();
    dto.updated_by = Some(updated_by);
    let result = ContractModel::update_by_id(&txn, &form_data.id, &dto).await?;

    txn.commit().await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }

    // 批量删除审批状态校验：审批中(2)/已通过(3)的合同不允许删除
    let contracts_to_check = Contract::find()
        .filter(contract::Column::Id.is_in(ids_vec.clone()))
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(format!("查询合同失败: {}", e)))?;

    for c in &contracts_to_check {
        let status = c.approval_status.unwrap_or(0);
        if status == 2 {
            return Err(Error::from(format!("合同(id={})审批中，不允许删除", c.id)));
        }
        if status == 3 {
            return Err(Error::from(format!("合同(id={})已审批通过，不允许删除", c.id)));
        }
    }

    let txn = db.begin().await?;
    let result = ContractModel::batch_delete_by_ids(&txn, &ids_vec).await?;
    txn.commit().await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<ContractDetailVO> {
    let result = ContractModel::find_by_id(&db, id).await?;
    match result {
        Some(item) => {
            let mut vo: ContractDetailVO = item.into();
            let logs = ContractModel::get_approval_logs(&db, id).await?;
            let log_vos: Vec<ContractApprovalLogVO> = logs.into_iter().map(|l| l.into()).collect();
            vo.approval_logs = Some(log_vos);
            // 计算发货状态：存在关联订单且订单已发货(order_status>=5) 则视为已发货
            let shipped = order::Entity::find()
                .filter(order::Column::ContractId.eq(id))
                .filter(order::Column::Deleted.eq(0))
                .filter(order::Column::OrderStatus.gte(5))
                .one(db)
                .await?;
            vo.ship_status = if shipped.is_some() { Some(1) } else { None };
            Ok(vo)
        },
        None => Err(Error::from("合同不存在".to_string())),
    }
}

/// 根据用户ID获取其数据权限范围内的所有用户ID
///
/// 已迁移至 [`data_scope_service::get_accessible_user_ids`]，支持多角色合并。
/// 参数 `data_scope` 已弃用，内部会自动查询用户所有角色并合并权限。
async fn get_accessible_user_ids(
    db: &DbConn,
    current_user_id: i64,
    _data_scope: Option<i32>,
) -> Result<Option<Vec<i64>>> {
    crate::modules::system::service::data_scope_service::get_accessible_user_ids(db, current_user_id).await
}



pub async fn list(db: &DbConn, query: &ContractListQuery, current_user_id: i64) -> Result<ResultPage<Vec<ContractListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("all");

    let assigned_tos_opt: Option<Vec<i64>> = match list_type {
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

    let result = if list_type == "my" {
        ContractModel::select_in_page_by_assigned_tos(
            &db,
            page,
            page_size,
            query.keywords.clone(),
            query.status.clone(),
            query.customer_id,
            Some(vec![current_user_id]),
        ).await?
    } else {
        ContractModel::select_in_page_by_assigned_tos(
            &db,
            page,
            page_size,
            query.keywords.clone(),
            query.status.clone(),
            query.customer_id,
            assigned_tos_opt,
        ).await?
    };
    let list = result.0;
    let total = result.1;
    
    // 收集所有客户ID，去重
    let customer_ids: Vec<i64> = list.iter()
        .filter_map(|c| c.customer_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    
    // 批量查询客户名称（ID -> 名称）
    let customer_name_map: HashMap<i64, String> = if !customer_ids.is_empty() {
        Customer::find()
            .filter(CustomerColumn::Id.is_in(customer_ids.clone()))
            .all(db)
            .await?
            .into_iter()
            .map(|c| {
                (c.id, c.company_name.or(c.short_name).unwrap_or_default())
            })
            .collect()
    } else {
        HashMap::new()
    };
    
    let mut data: Vec<ContractListVO> = list.into_iter().map(|item| {
        let mut vo: ContractListVO = item.into();
        if let Some(cid) = vo.customer_id {
            vo.customer_name = customer_name_map.get(&cid).cloned();
        }
        vo
    }).collect();

    // 计算合同发货状态：关联订单中存在“已发货/部分发货/已签收/已完成”(order_status>=5) 即视为已发货
    let contract_ids: Vec<i64> = data.iter().filter_map(|v| v.id).collect();
    let ship_status_map: HashMap<i64, i32> = if !contract_ids.is_empty() {
        let shipped_orders = order::Entity::find()
            .filter(order::Column::ContractId.is_in(contract_ids.clone()))
            .filter(order::Column::Deleted.eq(0))
            .filter(order::Column::OrderStatus.gte(5))
            .all(db)
            .await?;
        shipped_orders
            .into_iter()
            .filter_map(|o| o.contract_id)
            .collect::<HashSet<i64>>()
            .into_iter()
            .map(|cid| (cid, 1i32))
            .collect()
    } else {
        HashMap::new()
    };
    for vo in &mut data {
        if let Some(cid) = vo.id {
            vo.ship_status = ship_status_map.get(&cid).copied();
        }
    }

    Ok(ResultPage::new(data, total, page, page_size))
}

pub async fn submit_contract(db: &DbConn, contract_id: i64, operator_id: i64, operator_name: &str) -> Result<ContractDetailVO> {
    let contract = Contract::find_by_id(contract_id)
        .filter(contract::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("合同不存在".to_string()))?;

    if contract.approval_status != Some(0) && contract.approval_status != Some(4) {
        return Err(Error::from("当前状态不允许提交，仅草稿或已驳回状态可提交".to_string()));
    }

    let total_amount = contract.total_amount.unwrap_or(Decimal::from(0));
    let previous_status = contract.approval_status;
    let title = contract.title.clone();

    // 调用审批引擎提交
    let submit_req = ApprovalSubmitRequest {
        flow_code: "contract_approval".to_string(),
        business_type: "contract".to_string(),
        business_id: contract_id,
        business_title: title,
        submitter_id: operator_id,
        submitter_name: Some(operator_name.to_string()),
        extra_data: Some(serde_json::json!({ "amount": total_amount })),
    };
    let instance_id = ApprovalService::submit(db, &submit_req).await?;

    // 更新合同表
    let txn = db.begin().await?;
    let mut active: contract::ActiveModel = contract.into_active_model();
    active.approval_status = Set(Some(1));
    active.instance_id = Set(Some(instance_id));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    active.update(&txn).await?;

    let now = chrono::Local::now().naive_local().to_owned();
    let log_payload = contract_approval_log::ActiveModel {
        contract_id: Set(contract_id),
        action: Set(1),
        operator_id: Set(operator_id),
        operator_name: Set(Some(operator_name.to_string())),
        reason: Set(None),
        previous_status: Set(previous_status),
        new_status: Set(Some(1)),
        current_stage: Set(None),
        next_stage: Set(None),
        create_time: Set(Option::from(now)),
        ..Default::default()
    };
    ContractApprovalLog::insert(log_payload).exec(&txn).await?;
    txn.commit().await?;

    find_by_id(db, contract_id).await
}

pub async fn approve_contract(db: &DbConn, req: &ContractApprovalRequest, operator_id: i64, operator_name: &str) -> Result<ContractDetailVO> {
    let contract_id = req.contract_id.ok_or_else(|| Error::from("合同ID不能为空".to_string()))?;

    let contract = Contract::find_by_id(contract_id)
        .filter(contract::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("合同不存在".to_string()))?;

    if contract.approval_status != Some(1) && contract.approval_status != Some(2) {
        return Err(Error::from("仅待审批或审批中状态可进行审批操作".to_string()));
    }

    let instance_id = contract.instance_id
        .ok_or_else(|| Error::from("审批实例不存在，请重新提交审批".to_string()))?;
    let previous_status = contract.approval_status;

    // 调用审批引擎处理（通过）
    let process_req = ApprovalProcessRequest {
        instance_id,
        action: 1,
        approver_id: operator_id,
        approver_name: Some(operator_name.to_string()),
        comment: req.reason.clone(),
    };
    ApprovalService::process(db, &process_req).await?;

    // 查询实例最新状态，判断审批是否完成
    let instance = ApprovalService::find_instance_by_id(db, instance_id).await?
        .ok_or_else(|| Error::from("审批实例不存在".to_string()))?;
    let new_status = if instance.status == 3 { 3 } else { 2 };

    // 审批通过联动：自动将合同状态置为已签署（2），sign_date 为空时设为当前日期
    let original_sign_date = contract.sign_date;
    let today_date = chrono::Local::now().naive_local().date();

    // 更新合同表
    let txn = db.begin().await?;
    let mut active: contract::ActiveModel = contract.into_active_model();
    active.approval_status = Set(Some(new_status));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    if new_status == 3 {
        active.status = Set(Some(ContractStatus::Signed));
        if original_sign_date.is_none() {
            active.sign_date = Set(Some(today_date));
        }
    }
    active.update(&txn).await?;

    let now = chrono::Local::now().naive_local().to_owned();
    let log_payload = contract_approval_log::ActiveModel {
        contract_id: Set(contract_id),
        action: Set(2),
        operator_id: Set(operator_id),
        operator_name: Set(Some(operator_name.to_string())),
        reason: Set(req.reason.clone()),
        previous_status: Set(previous_status),
        new_status: Set(Some(new_status)),
        current_stage: Set(None),
        next_stage: Set(None),
        create_time: Set(Option::from(now)),
        ..Default::default()
    };
    ContractApprovalLog::insert(log_payload).exec(&txn).await?;
    txn.commit().await?;

    find_by_id(db, contract_id).await
}

pub async fn reject_contract(db: &DbConn, req: &ContractApprovalRequest, operator_id: i64, operator_name: &str) -> Result<ContractDetailVO> {
    let contract_id = req.contract_id.ok_or_else(|| Error::from("合同ID不能为空".to_string()))?;

    let contract = Contract::find_by_id(contract_id)
        .filter(contract::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("合同不存在".to_string()))?;

    if contract.approval_status != Some(1) && contract.approval_status != Some(2) {
        return Err(Error::from("仅待审批或审批中状态可进行驳回操作".to_string()));
    }

    let instance_id = contract.instance_id
        .ok_or_else(|| Error::from("审批实例不存在，请重新提交审批".to_string()))?;
    let previous_status = contract.approval_status;

    // 调用审批引擎处理（驳回）
    let process_req = ApprovalProcessRequest {
        instance_id,
        action: 2,
        approver_id: operator_id,
        approver_name: Some(operator_name.to_string()),
        comment: req.reason.clone(),
    };
    ApprovalService::process(db, &process_req).await?;

    // 更新合同表
    let txn = db.begin().await?;
    let mut active: contract::ActiveModel = contract.into_active_model();
    active.approval_status = Set(Some(4));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    active.update(&txn).await?;

    let now = chrono::Local::now().naive_local().to_owned();
    let log_payload = contract_approval_log::ActiveModel {
        contract_id: Set(contract_id),
        action: Set(3),
        operator_id: Set(operator_id),
        operator_name: Set(Some(operator_name.to_string())),
        reason: Set(req.reason.clone()),
        previous_status: Set(previous_status),
        new_status: Set(Some(4)),
        current_stage: Set(None),
        next_stage: Set(None),
        create_time: Set(Option::from(now)),
        ..Default::default()
    };
    ContractApprovalLog::insert(log_payload).exec(&txn).await?;
    txn.commit().await?;

    find_by_id(db, contract_id).await
}

pub async fn get_approval_detail(db: &DbConn, contract_id: i64) -> Result<ContractApprovalDetailVO> {
    let contract = Contract::find_by_id(contract_id)
        .filter(contract::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("合同不存在".to_string()))?;

    let customer_name = if let Some(cid) = contract.customer_id {
        Customer::find_by_id(cid)
            .one(db)
            .await?
            .and_then(|c| c.company_name.or(c.short_name))
    } else {
        None
    };

    let instance = if let Some(iid) = contract.instance_id {
        ApprovalService::find_instance_by_id(db, iid).await?
    } else {
        None
    };

    Ok(ContractApprovalDetailVO {
        contract_id: Some(contract.id),
        contract_no: contract.contract_no,
        title: contract.title,
        customer_name,
        contract_type: contract.contract_type.map(|t| t.to_string()),
        amount: contract.amount,
        total_amount: contract.total_amount,
        currency: contract.currency,
        approval_status: contract.approval_status,
        instance,
    })
}