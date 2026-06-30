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
use crate::modules::sale::model::order::{OrderItemModel, OrderItemSaveDTO, OrderModel, OrderSaveDTO};
use crate::modules::sale::model::quotation::{
    QuotationApprovalModel, QuotationDetailVO, QuotationItemModel, QuotationListQuery,
    QuotationListVO, QuotationModel, QuotationSaveDTO, QuotationSaveRequest,
    QuotationUpdateRequest, recalculate_amounts,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::{DbConn, Set, TransactionTrait, ActiveModelTrait, IntoActiveModel};

pub async fn insert(db: &DbConn, form_data: &QuotationSaveRequest, created_by: String) -> Result<i64> {
    let items = form_data.items.clone().unwrap_or_default();
    if items.is_empty() {
        return Err(Error::from("报价单明细不能为空"));
    }

    let txn = db.begin().await?;

    let date_prefix = format!("QT{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = QuotationModel::get_max_quotation_no_today(&txn, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let quotation_no = format!("{}{:04}", date_prefix, seq);

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
    if existing.is_none() {
        return Err(Error::from("报价单不存在"));
    }

    let txn = db.begin().await?;

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

pub async fn list(db: &DbConn, query: &QuotationListQuery) -> Result<ResultPage<Vec<QuotationListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (list, total) = QuotationModel::select_in_page(
        db,
        page,
        page_size,
        query.keywords.clone(),
        query.customer_id,
        query.status,
        query.approval_status,
        query.start_date.clone(),
        query.end_date.clone(),
    ).await?;

    let data: Vec<QuotationListVO> = list.into_iter().map(|item| item.into()).collect();
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
        remark: detail.remark.clone(),
        owner_user_id: detail.owner_user_id,
        dept_id: detail.dept_id,
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
            quantity: item.quantity.and_then(|q| q.to_i32()),
            unit_price: item.unit_price,
            discount_rate: item.discount_rate.map(|r| Decimal::from(100) - r),
            discount_amount: item.discount_amount,
            tax_rate: item.tax_rate,
            tax_amount: item.tax_amount,
            amount: item.subtotal,
            total_amount: item.subtotal,
            delivery_date: None,
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
