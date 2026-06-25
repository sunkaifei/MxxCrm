use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::approval::service::approval_service::ApprovalService;
use crate::modules::approval::model::approval::{ApprovalSubmitRequest, ApprovalProcessRequest};
use crate::modules::crm::model::contract::{ContractApprovalDetailVO, ContractApprovalLogVO, ContractApprovalRequest, ContractDetailVO, ContractListQuery, ContractListVO, ContractModel, ContractSaveDTO};
use crate::modules::crm::entity::{contract, contract_approval_log, contract::Entity as Contract, contract_approval_log::Entity as ContractApprovalLog, customer::{Entity as Customer, Column as CustomerColumn}};
use sea_orm::{DbConn, TransactionTrait, Set, IntoActiveModel, ActiveModelTrait, EntityTrait, ColumnTrait, QueryFilter, QueryOrder, Condition};
use sea_orm::prelude::Decimal;

pub async fn insert(db: &DbConn, form_data: &ContractSaveDTO, created_by: i64) -> Result<i64> {
    let mut dto = form_data.clone();
    dto.created_by = Some(created_by);
    dto.approval_status = Some(0);
    let result = ContractModel::insert(&db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &ContractSaveDTO, updated_by: i64) -> Result<i64> {
    let mut dto = form_data.clone();
    dto.updated_by = Some(updated_by);
    let result = ContractModel::update_by_id(&db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = ContractModel::batch_delete_by_ids(&db, &ids_vec).await?;
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
            Ok(vo)
        },
        None => Err(Error::from("合同不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &ContractListQuery) -> Result<ResultPage<Vec<ContractListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    let (list, total) = ContractModel::select_in_page(
        &db,
        page,
        page_size,
        query.keywords.clone(),
        query.status.clone(),
        query.customer_id,
    ).await?;
    
    let data: Vec<ContractListVO> = list.into_iter().map(|item| item.into()).collect();
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

    // 更新合同表
    let txn = db.begin().await?;
    let mut active: contract::ActiveModel = contract.into_active_model();
    active.approval_status = Set(Some(new_status));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
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