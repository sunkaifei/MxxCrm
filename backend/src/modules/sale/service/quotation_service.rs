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
use crate::modules::approval::model::approval::{ApprovalProcessRequest, ApprovalSubmitRequest};
use crate::modules::approval::service::approval_service::ApprovalService;
use crate::modules::company::service::code_rule_service;
use crate::modules::crm::entity::customer;
use crate::modules::sale::model::order::{OrderItemModel, OrderItemSaveDTO, OrderModel, OrderSaveDTO};
use crate::modules::sale::model::quotation::{
    QuotationApprovalModel, QuotationDetailVO, QuotationItemModel, QuotationListQuery,
    QuotationListVO, QuotationModel, QuotationSaveDTO, QuotationSaveRequest,
    QuotationUpdateRequest, recalculate_amounts,
};
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::{DbConn, Set, TransactionTrait, ActiveModelTrait, IntoActiveModel, ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;

pub async fn insert(db: &DbConn, form_data: &QuotationSaveRequest, created_by: String) -> Result<i64> {
    let items = form_data.items.clone().unwrap_or_default();
    if items.is_empty() {
        return Err(Error::from("报价单明细不能为空"));
    }

    let txn = db.begin().await?;

    if let (Some(customer_id), Some(title)) = (form_data.customer_id, form_data.title.as_deref()) {
        let existing = QuotationModel::find_by_customer_and_title(&txn, customer_id, title, None).await?;
        if existing.is_some() {
            txn.rollback().await?;
            return Err(Error::from("该客户下已存在相同标题的报价单"));
        }
    }

    // 调用编码模块生成报价编号（如未配置规则则使用默认规则）
    let quotation_no = match code_rule_service::generate_code(&txn, "quotation", None, None, None).await {
        Ok(code) => code,
        Err(_) => {
            let date_prefix = format!("QT{}", chrono::Local::now().format("%Y%m%d"));
            let max_seq = QuotationModel::get_max_quotation_no_today(&txn, &date_prefix).await?;
            let seq = max_seq.unwrap_or(0) + 1;
            format!("{}{:04}", date_prefix, seq)
        }
    };

    let mut dto: QuotationSaveDTO = form_data.clone().into();
    dto.quotation_no = Some(quotation_no);
    dto.status = Some(1);
    dto.approval_status = Some(1);
    dto.current_version = Some(1);
    dto.create_by = Some(created_by.clone());

    let quotation_id = QuotationModel::insert(&txn, &dto).await?;
    QuotationItemModel::insert_batch(&txn, quotation_id, &items).await?;
    recalculate_amounts(&txn, quotation_id).await?;

    txn.commit().await?;

    Ok(quotation_id)
}

pub async fn update(db: &DbConn, form_data: &QuotationUpdateRequest, updated_by: String) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("报价单ID不能为空"));
    }
    let items = form_data.items.clone().unwrap_or_default();
    if items.is_empty() {
        return Err(Error::from("报价单明细不能为空"));
    }

    let existing = QuotationModel::find_by_id(db, id).await?;
    let existing = existing.ok_or_else(|| Error::from("报价单不存在"))?;

    // 审批中(approval_status=2)不允许修改
    if existing.approval_status == Some(2) {
        return Err(Error::from("报价单审批中，不允许修改"));
    }

    let txn = db.begin().await?;

    if let (Some(customer_id), Some(title)) = (form_data.customer_id, form_data.title.as_deref()) {
        let duplicate = QuotationModel::find_by_customer_and_title(&txn, customer_id, title, form_data.id).await?;
        if duplicate.is_some() {
            txn.rollback().await?;
            return Err(Error::from("该客户下已存在相同标题的报价单"));
        }
    }

    let mut dto: QuotationSaveDTO = form_data.clone().into();
    dto.update_by = Some(updated_by.clone());

    QuotationModel::update_by_id(&txn, &form_data.id, &dto).await?;
    QuotationItemModel::delete_by_quotation_id(&txn, id).await?;
    QuotationItemModel::insert_batch(&txn, id, &items).await?;
    recalculate_amounts(&txn, id).await?;

    txn.commit().await?;

    Ok(id)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }

    let txn = db.begin().await?;

    for &id in ids_vec {
        QuotationItemModel::delete_by_quotation_id(&txn, id).await?;
    }
    let result = QuotationModel::batch_delete_by_ids(&txn, ids_vec).await?;

    txn.commit().await?;

    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<QuotationDetailVO> {
    let main = QuotationModel::find_by_id(db, id).await?
        .ok_or_else(|| Error::from("报价单不存在".to_string()))?;
    let items = QuotationItemModel::find_by_quotation_id(db, id).await?;
    let approvals = QuotationApprovalModel::find_by_quotation_id(db, id).await?;
    Ok((main, items, approvals).into())
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

pub async fn list(db: &DbConn, query: &QuotationListQuery, current_user_id: i64) -> Result<ResultPage<Vec<QuotationListVO>>> {
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
        QuotationModel::select_in_page(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.customer_id,
            query.status,
            query.approval_status,
            query.start_date.clone(),
            query.end_date.clone(),
        ).await?
    } else {
        QuotationModel::select_in_page_by_owner_user_ids(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.customer_id,
            query.status,
            query.approval_status,
            query.start_date.clone(),
            query.end_date.clone(),
            owner_user_ids_opt,
        ).await?
    };

    let mut customer_map: HashMap<i64, String> = HashMap::new();
    let customer_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.customer_id)
        .collect();
    if !customer_ids.is_empty() {
        let customers = customer::Entity::find()
            .filter(customer::Column::Id.is_in(customer_ids))
            .all(db)
            .await?;
        for c in customers {
            if let Some(name) = c.company_name {
                customer_map.insert(c.id, name);
            }
        }
    }

    let data: Vec<QuotationListVO> = list.into_iter().map(|item| {
        let cid = item.customer_id;
        let mut vo: QuotationListVO = item.into();
        if let Some(c) = cid {
            vo.customer_name = customer_map.get(&c).cloned();
        }
        vo
    }).collect();
    Ok(ResultPage { items: data, total, current_page: page, page_size, total_pages: 0 })
}

pub async fn update_status(db: &DbConn, id: i64, status: i32) -> Result<i64> {
    let result = QuotationModel::update_status(db, id, status).await?;
    Ok(result)
}

/// 提交审批：调用通用审批引擎，更新报价单状态为审批中
pub async fn submit_approval(
    db: &DbConn,
    id: i64,
    operator_id: i64,
    operator_name: &str,
    remark: Option<String>,
) -> Result<QuotationDetailVO> {
    let quotation = QuotationModel::find_by_id(db, id).await?
        .ok_or_else(|| Error::from("报价单不存在".to_string()))?;

    if quotation.approval_status != Some(1) && quotation.approval_status != Some(4) {
        return Err(Error::from("当前状态不允许提交，仅草稿或已驳回状态可提交".to_string()));
    }

    let grand_total = quotation.grand_total.unwrap_or_else(|| Decimal::from(0));
    let title = quotation.title.clone().unwrap_or_else(|| quotation.quotation_no.clone().unwrap_or_default());

    let submit_req = ApprovalSubmitRequest {
        flow_code: "quotation_approval".to_string(),
        business_type: "quotation".to_string(),
        business_id: id,
        business_title: Some(title.clone()),
        submitter_id: operator_id,
        submitter_name: Some(operator_name.to_string()),
        extra_data: Some(serde_json::json!({ "amount": grand_total })),
    };
    let instance_id = ApprovalService::submit(db, &submit_req).await?;

    let txn = db.begin().await?;
    let mut active: crate::modules::sale::entity::quotation::ActiveModel = quotation.into_active_model();
    active.approval_status = Set(Some(2));
    active.instance_id = Set(Some(instance_id));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    active.update(&txn).await?;
    txn.commit().await?;

    let _ = remark;
    find_by_id(db, id).await
}

/// 审批通过
pub async fn approve(
    db: &DbConn,
    id: i64,
    operator_id: i64,
    operator_name: &str,
    remark: Option<String>,
) -> Result<QuotationDetailVO> {
    let quotation = QuotationModel::find_by_id(db, id).await?
        .ok_or_else(|| Error::from("报价单不存在".to_string()))?;

    if quotation.approval_status != Some(2) {
        return Err(Error::from("仅审批中状态可进行审批操作".to_string()));
    }

    let instance_id = quotation.instance_id
        .ok_or_else(|| Error::from("审批实例不存在，请重新提交审批".to_string()))?;

    let process_req = ApprovalProcessRequest {
        instance_id,
        action: 1,
        approver_id: operator_id,
        approver_name: Some(operator_name.to_string()),
        comment: remark.clone(),
    };
    ApprovalService::process(db, &process_req).await?;

    let instance = ApprovalService::find_instance_by_id(db, instance_id).await?
        .ok_or_else(|| Error::from("审批实例不存在".to_string()))?;
    let new_approval_status = if instance.status == 3 { 3 } else { 2 };
    let new_status = if instance.status == 3 { Some(3) } else { quotation.status };

    let txn = db.begin().await?;
    QuotationModel::update_status_and_approval(&txn, id, new_status, Some(new_approval_status)).await?;
    txn.commit().await?;

    find_by_id(db, id).await
}

/// 审批驳回
pub async fn reject(
    db: &DbConn,
    id: i64,
    operator_id: i64,
    operator_name: &str,
    remark: Option<String>,
) -> Result<QuotationDetailVO> {
    let quotation = QuotationModel::find_by_id(db, id).await?
        .ok_or_else(|| Error::from("报价单不存在".to_string()))?;

    if quotation.approval_status != Some(2) {
        return Err(Error::from("仅审批中状态可进行驳回操作".to_string()));
    }

    let instance_id = quotation.instance_id
        .ok_or_else(|| Error::from("审批实例不存在，请重新提交审批".to_string()))?;

    let process_req = ApprovalProcessRequest {
        instance_id,
        action: 2,
        approver_id: operator_id,
        approver_name: Some(operator_name.to_string()),
        comment: remark.clone(),
    };
    ApprovalService::process(db, &process_req).await?;

    let txn = db.begin().await?;
    QuotationModel::update_status_and_approval(&txn, id, Some(1), Some(4)).await?;
    txn.commit().await?;

    find_by_id(db, id).await
}

pub async fn convert_to_order(db: &DbConn, quotation_id: i64, created_by: String) -> Result<i64> {
    let detail = QuotationModel::find_by_id(db, quotation_id).await?
        .ok_or_else(|| Error::from("报价单不存在".to_string()))?;

    if detail.approval_status != Some(3) {
        return Err(Error::from("只有审批通过的报价单才能转为订单".to_string()));
    }

    let items = QuotationItemModel::find_by_quotation_id(db, quotation_id).await?;
    if items.is_empty() {
        return Err(Error::from("报价单明细不能为空".to_string()));
    }

    let txn = db.begin().await?;

    let date_prefix = format!("SO{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = OrderModel::get_max_order_no_today(&txn, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let order_no = format!("{}{:04}", date_prefix, seq);

    let created_by_i64 = created_by.parse::<i64>()
        .map_err(|_| Error::from("创建人ID格式错误".to_string()))?;

    let grand_total = detail.grand_total.unwrap_or_else(|| Decimal::from(0));
    let product_amount = detail.total_amount.unwrap_or_else(|| Decimal::from(0));
    let discount_amount = detail.discount_amount.unwrap_or_else(|| Decimal::from(0));
    let tax_amount = detail.tax_amount.unwrap_or_else(|| Decimal::from(0));

    let order_dto = OrderSaveDTO {
        order_no: Some(order_no),
        title: detail.title.clone(),
        order_type: Some(1),
        order_status: Some(0),
        customer_id: detail.customer_id,
        customer_name: detail.customer_name.clone(),
        contact_id: detail.contact_id,
        contact_name: detail.contact_name.clone(),
        opportunity_id: detail.opportunity_id,
        quotation_id: Some(quotation_id),
        contract_id: None,
        order_date: Some(chrono::Local::now().naive_local().date()),
        delivery_date: detail.delivery_date,
        currency: detail.currency,
        exchange_rate: Some(Decimal::from(1)),
        product_amount: Some(product_amount),
        discount_amount: Some(discount_amount),
        shipping_fee: Some(Decimal::from(0)),
        tax_amount: Some(tax_amount),
        other_fee: Some(Decimal::from(0)),
        total_amount: Some(grand_total),
        paid_amount: Some(Decimal::from(0)),
        unpaid_amount: Some(grand_total),
        pay_status: Some(0),
        payment_method: None,
        payment_due_date: None,
        shipping_method: None,
        tracking_no: None,
        shipping_time: None,
        complete_time: None,
        receiver_name: None,
        receiver_phone: None,
        shipping_address: None,
        billing_address: None,
        buyer_company_name: None,
        buyer_account_name: None,
        buyer_bank_name: None,
        buyer_account_number: None,
        seller_company_name: None,
        seller_bank_name: None,
        seller_account_name: None,
        seller_account_number: None,
        remark: detail.remark.clone(),
        owner_user_id: detail.owner_user_id,
        dept_id: detail.dept_id,
        approval_status: Some(0),
        instance_id: None,
        create_by: Some(created_by_i64),
        update_by: None,
    };

    let order_id = OrderModel::insert(&txn, &order_dto).await?;

    let order_items: Vec<OrderItemSaveDTO> = items.iter().map(|item| {
        OrderItemSaveDTO {
            product_id: item.product_id,
            product_name: item.product_name.clone(),
            product_code: item.product_code.clone(),
            sku: None,
            spec: item.spec.clone(),
            unit: item.unit.clone(),
            unit_id: None,
            quantity: item.quantity,
            unit_price: item.unit_price,
            discount_rate: item.discount_rate.map(|r| Decimal::from(100) - r),
            discount_amount: item.discount_amount,
            tax_rate: item.tax_rate,
            tax_amount: item.tax_amount,
            amount: item.subtotal,
            total_amount: item.subtotal,
            delivery_date: None,
            product_type: None,
            delivered_quantity: None,
            remark: item.remark.clone(),
            sort: item.sort,
        }
    }).collect();

    OrderItemModel::insert_batch(&txn, order_id, &order_items).await?;

    QuotationModel::update_status_and_approval(&txn, quotation_id, Some(8), None).await?;

    txn.commit().await?;
    Ok(order_id)
}
