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
use crate::modules::crm::entity::customer::{Entity as Customer, Column as CustomerColumn};
use crate::modules::sale::entity::invoice as invoice_entity;
use crate::modules::sale::model::invoice::{InvoiceApprovalDetailVO, InvoiceDetailVO, InvoiceListQuery, InvoiceListVO, InvoiceModel, InvoiceSaveDTO, InvoiceSaveRequest, InvoiceUpdateRequest};
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DbConn, TransactionTrait, EntityTrait, ColumnTrait, QueryFilter};
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use std::collections::{HashMap, HashSet};

pub async fn insert(db: &DbConn, form_data: &InvoiceSaveRequest, created_by: i64) -> Result<i64> {
    let txn = db.begin().await?;

    let date_prefix = format!("INV{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = InvoiceModel::get_max_invoice_no_today(&txn, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let invoice_no = format!("{}{:04}", date_prefix, seq);

    let amount = form_data.amount.unwrap_or(Decimal::from(0));
    let tax_rate = form_data.tax_rate.unwrap_or(Decimal::from(0));
    let hundred = Decimal::from(100);
    let tax_amount = form_data.tax_amount.unwrap_or(amount * tax_rate / hundred);

    let mut dto: InvoiceSaveDTO = form_data.clone().into();
    dto.invoice_no = Some(invoice_no);
    dto.status = Some(1);
    dto.tax_amount = Some(tax_amount);
    dto.create_by = Some(created_by.to_string());

    let invoice_id = InvoiceModel::insert(&txn, &dto).await?;

    txn.commit().await?;

    Ok(invoice_id)
}

pub async fn update(db: &DbConn, form_data: &InvoiceUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("发票ID不能为空"));
    }

    let existing = InvoiceModel::find_by_id(db, id).await?;
    if existing.is_none() {
        return Err(Error::from("发票不存在"));
    }

    let amount = form_data.amount.unwrap_or(Decimal::from(0));
    let tax_rate = form_data.tax_rate.unwrap_or(Decimal::from(0));
    let hundred = Decimal::from(100);
    let tax_amount = form_data.tax_amount.unwrap_or(amount * tax_rate / hundred);

    let mut dto: InvoiceSaveDTO = form_data.clone().into();
    dto.tax_amount = Some(tax_amount);
    dto.update_by = Some(updated_by.to_string());

    InvoiceModel::update_by_id(db, id, &dto).await?;

    Ok(id)
}

pub async fn batch_delete(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = InvoiceModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

pub async fn get_detail(db: &DbConn, id: i64) -> Result<InvoiceDetailVO> {
    let invoice = InvoiceModel::find_by_id(db, id).await?;
    match invoice {
        Some(i) => Ok((&i).into()),
        None => Err(Error::from("发票不存在")),
    }
}

pub async fn get_list(db: &DbConn, query: &InvoiceListQuery, current_user_id: i64) -> Result<ResultPage<Vec<InvoiceListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("all");

    let owner_user_ids_opt: Option<Vec<i64>> = match list_type {
        "my" => {
            Some(vec![current_user_id])
        }
        "subordinate" => {
            // 下属发票：获取数据权限范围内的其他用户（排除自己）
            let accessible = crate::modules::system::service::data_scope_service
                ::get_accessible_user_ids(db, current_user_id).await?;
            match accessible {
                None => {
                    // 全部数据权限：获取所有用户，排除自己
                    let all_admins = Admin::find()
                        .filter(admin::Column::Id.ne(current_user_id))
                        .all(db)
                        .await
                        .map_err(|e| Error::from(format!("查询用户列表失败: {}", e)))?;
                    Some(all_admins.iter().map(|u| u.id).collect())
                }
                Some(ids) => {
                    // 部门/仅本人权限：排除自己
                    Some(ids.into_iter().filter(|id| *id != current_user_id).collect())
                }
            }
        }
        _ => {
            // all：按多角色合并后的数据权限过滤
            crate::modules::system::service::data_scope_service
                ::get_accessible_user_ids(db, current_user_id).await?
        }
    };

    let (list, total) = if list_type == "my" {
        InvoiceModel::select_in_page(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.invoice_type,
            query.status,
            query.customer_id,
            Some(current_user_id),
        ).await?
    } else {
        InvoiceModel::select_in_page_by_owner_user_ids(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.invoice_type,
            query.status,
            query.customer_id,
            owner_user_ids_opt,
        ).await?
    };

    let customer_ids: Vec<i64> = list.iter()
        .filter_map(|c| c.customer_id)
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

    let data: Vec<InvoiceListVO> = list.iter().map(|item| {
        let mut vo: InvoiceListVO = item.into();
        if let Some(cid) = vo.customer_id {
            if let Some(name) = customer_name_map.get(&cid) {
                vo.customer_name = Some(name.clone());
            }
        }
        vo
    }).collect();
    Ok(ResultPage { items: data, total, current_page: page, page_size, total_pages: 0 })
}

// ==================== 发票审批 ====================

/// 提交发票审批
pub async fn submit_invoice(db: &DbConn, invoice_id: i64, operator_id: i64, operator_name: &str) -> Result<InvoiceDetailVO> {
    let invoice = InvoiceModel::find_by_id(db, invoice_id).await?
        .ok_or_else(|| Error::from("发票不存在"))?;

    if invoice.approval_status != Some(0) && invoice.approval_status != Some(4) {
        return Err(Error::from("当前状态不允许提交，仅草稿或已驳回状态可提交"));
    }

    let business_title = invoice.title.clone()
        .or_else(|| invoice.invoice_no.clone())
        .unwrap_or_default();
    let amount = invoice.amount.unwrap_or(Decimal::from(0));

    // 调用审批引擎提交
    let submit_req = ApprovalSubmitRequest {
        flow_code: "invoice_approval".to_string(),
        business_type: "invoice".to_string(),
        business_id: invoice_id,
        business_title: Some(business_title),
        submitter_id: operator_id,
        submitter_name: Some(operator_name.to_string()),
        extra_data: Some(serde_json::json!({ "amount": amount })),
    };
    let instance_id = ApprovalService::submit(db, &submit_req).await?;

    // 事务更新发票表
    let txn = db.begin().await?;
    let mut active: invoice_entity::ActiveModel = invoice.into_active_model();
    active.approval_status = Set(Some(1));
    active.instance_id = Set(Some(instance_id));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    active.update(&txn).await?;
    txn.commit().await?;

    get_detail(db, invoice_id).await
}

/// 审批发票通过
pub async fn approve_invoice(db: &DbConn, invoice_id: i64, operator_id: i64, operator_name: &str, reason: Option<String>) -> Result<InvoiceDetailVO> {
    let invoice = InvoiceModel::find_by_id(db, invoice_id).await?
        .ok_or_else(|| Error::from("发票不存在"))?;

    if invoice.approval_status != Some(1) && invoice.approval_status != Some(2) {
        return Err(Error::from("仅待审批或审批中状态可进行审批操作"));
    }

    let instance_id = invoice.instance_id
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

    // 事务更新发票表
    let txn = db.begin().await?;
    let mut active: invoice_entity::ActiveModel = invoice.into_active_model();
    active.approval_status = Set(Some(new_status));
    // 审批通过且审批完成时，更新发票状态为已开票（status=2）
    if new_status == 3 {
        active.status = Set(Some(2));
    }
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    active.update(&txn).await?;
    txn.commit().await?;

    get_detail(db, invoice_id).await
}

/// 驳回发票
pub async fn reject_invoice(db: &DbConn, invoice_id: i64, operator_id: i64, operator_name: &str, reason: Option<String>) -> Result<InvoiceDetailVO> {
    let invoice = InvoiceModel::find_by_id(db, invoice_id).await?
        .ok_or_else(|| Error::from("发票不存在"))?;

    if invoice.approval_status != Some(1) && invoice.approval_status != Some(2) {
        return Err(Error::from("仅待审批或审批中状态可进行驳回操作"));
    }

    let instance_id = invoice.instance_id
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

    // 事务更新发票表
    let txn = db.begin().await?;
    let mut active: invoice_entity::ActiveModel = invoice.into_active_model();
    active.approval_status = Set(Some(4));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    active.update(&txn).await?;
    txn.commit().await?;

    get_detail(db, invoice_id).await
}

/// 获取发票审批详情
pub async fn get_invoice_approval_detail(db: &DbConn, invoice_id: i64) -> Result<InvoiceApprovalDetailVO> {
    let invoice = InvoiceModel::find_by_id(db, invoice_id).await?
        .ok_or_else(|| Error::from("发票不存在"))?;

    let instance = if let Some(iid) = invoice.instance_id {
        ApprovalService::find_instance_by_id(db, iid).await?
    } else {
        None
    };

    Ok(InvoiceApprovalDetailVO {
        invoice_id: Some(invoice.id),
        invoice_no: invoice.invoice_no,
        title: invoice.title,
        customer_name: invoice.customer_name,
        amount: invoice.amount,
        approval_status: invoice.approval_status,
        instance,
    })
}
