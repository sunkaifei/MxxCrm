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
use crate::modules::crm::model::contract::{ContractSaveDTO, ContractModel};
use crate::modules::sale::entity::order::{self as order_entity, Entity as SaleOrder};
use crate::modules::sale::model::order::{OrderApprovalDetailVO, OrderDetailVO, OrderItemModel, OrderItemSaveDTO, OrderListQuery, OrderListVO, OrderModel, OrderSaveDTO, OrderSaveRequest, OrderStatusUpdateRequest, OrderUpdateRequest};
use crate::modules::sale::model::shipment::ShipmentModel;
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DbConn, TransactionTrait, EntityTrait, ColumnTrait, QueryFilter, Set};
use std::collections::{HashMap, HashSet};

fn calculate_product_amount(items: &Vec<OrderItemSaveDTO>) -> Decimal {
    let hundred = Decimal::from(100);
    items.iter().map(|item| {
        let qty = item.quantity.unwrap_or(1);
        let price = item.unit_price.unwrap_or(Decimal::from(0));
        let disc_rate = item.discount_rate.unwrap_or(Decimal::from(100));
        let tax_rate = item.tax_rate.unwrap_or(Decimal::from(0));

        let gross = price * Decimal::from(qty);
        let disc_amt = gross * (hundred - disc_rate) / hundred;
        let tax_amt = (gross - disc_amt) * tax_rate / hundred;
        gross - disc_amt + tax_amt
    }).fold(Decimal::from(0), |acc, x| acc + x)
}

pub async fn insert(db: &DbConn, form_data: &OrderSaveRequest, created_by: i64) -> Result<i64> {
    let items = form_data.items.clone().unwrap_or_default();
    if items.is_empty() {
        return Err(Error::from("订单明细不能为空"));
    }

    let txn = db.begin().await?;

    let date_prefix = format!("SO{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = OrderModel::get_max_order_no_today(&txn, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let order_no = format!("{}{:04}", date_prefix, seq);

    let product_amount = calculate_product_amount(&items);
    let discount_amount = form_data.discount_amount.unwrap_or(Decimal::from(0));
    let shipping_fee = form_data.shipping_fee.unwrap_or(Decimal::from(0));
    let tax_amount = form_data.tax_amount.unwrap_or(Decimal::from(0));
    let other_fee = form_data.other_fee.unwrap_or(Decimal::from(0));
    let total_amount = product_amount - discount_amount + shipping_fee + tax_amount + other_fee;

    let mut dto: OrderSaveDTO = form_data.clone().into();
    dto.order_no = Some(order_no);
    dto.order_status = Some(1);
    dto.product_amount = Some(product_amount);
    dto.total_amount = Some(total_amount);
    dto.paid_amount = Some(Decimal::from(0));
    dto.unpaid_amount = Some(total_amount);
    dto.pay_status = Some(1);
    dto.create_by = Some(created_by);

    let order_id = OrderModel::insert(&txn, &dto).await?;
    OrderItemModel::insert_batch(&txn, order_id, &items).await?;

    txn.commit().await?;

    Ok(order_id)
}

pub async fn update(db: &DbConn, form_data: &OrderUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("订单ID不能为空"));
    }
    let items = form_data.items.clone().unwrap_or_default();
    if items.is_empty() {
        return Err(Error::from("订单明细不能为空"));
    }

    let existing = OrderModel::find_by_id(db, id).await?;
    if existing.is_none() {
        return Err(Error::from("订单不存在"));
    }

    // 审批状态校验：仅草稿(0)或已驳回(4)允许编辑
    let approval_status = existing.as_ref().unwrap().approval_status.unwrap_or(0);
    if approval_status != 0 && approval_status != 4 {
        return Err(Error::from("当前订单审批状态不允许编辑"));
    }

    let txn = db.begin().await?;

    let product_amount = calculate_product_amount(&items);
    let discount_amount = form_data.discount_amount.unwrap_or(Decimal::from(0));
    let shipping_fee = form_data.shipping_fee.unwrap_or(Decimal::from(0));
    let tax_amount = form_data.tax_amount.unwrap_or(Decimal::from(0));
    let other_fee = form_data.other_fee.unwrap_or(Decimal::from(0));
    let total_amount = product_amount - discount_amount + shipping_fee + tax_amount + other_fee;

    let mut dto: OrderSaveDTO = form_data.clone().into();
    dto.product_amount = Some(product_amount);
    dto.total_amount = Some(total_amount);
    dto.update_by = Some(updated_by);

    OrderModel::update_by_id(&txn, id, &dto).await?;
    OrderItemModel::delete_by_order_id(&txn, id).await?;
    OrderItemModel::insert_batch(&txn, id, &items).await?;

    txn.commit().await?;

    Ok(id)
}

pub async fn update_status(db: &DbConn, form_data: &OrderStatusUpdateRequest) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("订单ID不能为空"));
    }
    let order_status = form_data.order_status.unwrap_or(1);

    let existing = OrderModel::find_by_id(db, id).await?;
    if existing.is_none() {
        return Err(Error::from("订单不存在"));
    }

    let result = OrderModel::update_status(db, id, order_status, form_data.tracking_no.clone()).await?;
    Ok(result)
}

pub async fn batch_delete(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    // 审批状态校验：仅草稿(0)或已驳回(4)允许删除
    for &id in ids_vec {
        let existing = OrderModel::find_by_id(db, id).await?;
        if let Some(order) = existing {
            let approval_status = order.approval_status.unwrap_or(0);
            if approval_status != 0 && approval_status != 4 {
                return Err(Error::from(format!(
                    "订单[{}]当前审批状态不允许删除", order.order_no.unwrap_or_default()
                )));
            }
        }
    }
    let result = OrderModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

pub async fn get_detail(db: &DbConn, id: i64) -> Result<OrderDetailVO> {
    let order = OrderModel::find_by_id(db, id).await?;
    match order {
        Some(o) => {
            let items = OrderItemModel::find_by_order_id(db, id).await?;
            let shipments = ShipmentModel::find_by_order_id(db, id).await?;
            Ok((&o, items, shipments).into())
        }
        None => Err(Error::from("订单不存在")),
    }
}

pub async fn get_list(db: &DbConn, query: &OrderListQuery) -> Result<ResultPage<Vec<OrderListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (list, total) = OrderModel::select_in_page(
        db,
        page,
        page_size,
        query.keywords.clone(),
        query.order_status,
        query.payment_status,
        query.customer_id,
        query.owner_user_id,
        query.start_date.clone(),
        query.end_date.clone(),
    ).await?;

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
            .map(|c| (c.id, c.company_name.or(c.short_name).unwrap_or_default()))
            .collect()
    } else {
        HashMap::new()
    };

    let data: Vec<OrderListVO> = list.iter().map(|item| {
        let mut vo: OrderListVO = item.into();
        if let Some(cid) = vo.customer_id {
            if let Some(name) = customer_name_map.get(&cid) {
                vo.customer_name = Some(name.clone());
            }
        }
        vo
    }).collect();
    Ok(ResultPage { items: data, total, current_page: page, page_size, total_pages: 0 })
}

/// 提交订单审批
pub async fn submit_order(db: &DbConn, order_id: i64, operator_id: i64, operator_name: &str) -> Result<OrderDetailVO> {
    let order = OrderModel::find_by_id(db, order_id).await?
        .ok_or_else(|| Error::from("订单不存在"))?;

    if order.approval_status != Some(0) && order.approval_status != Some(4) {
        return Err(Error::from("当前状态不允许提交，仅草稿或已驳回状态可提交"));
    }

    let title = order.title.clone().unwrap_or_default();
    let total_amount = order.total_amount.unwrap_or(Decimal::from(0));

    // 调用审批引擎提交
    let submit_req = ApprovalSubmitRequest {
        flow_code: "order_approval".to_string(),
        business_type: "order".to_string(),
        business_id: order_id,
        business_title: Some(title),
        submitter_id: operator_id,
        submitter_name: Some(operator_name.to_string()),
        extra_data: Some(serde_json::json!({ "amount": total_amount })),
    };
    let instance_id = ApprovalService::submit(db, &submit_req).await?;

    // 事务更新订单表
    let txn = db.begin().await?;
    use sea_orm::IntoActiveModel;
    let mut active: order_entity::ActiveModel = order.clone().into_active_model();
    active.approval_status = Set(Some(1));
    active.instance_id = Set(Some(instance_id));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    active.update(&txn).await?;
    txn.commit().await?;

    get_detail(db, order_id).await
}

/// 审批通过订单
pub async fn approve_order(db: &DbConn, order_id: i64, operator_id: i64, operator_name: &str, reason: Option<String>) -> Result<OrderDetailVO> {
    let order = OrderModel::find_by_id(db, order_id).await?
        .ok_or_else(|| Error::from("订单不存在"))?;

    if order.approval_status != Some(1) && order.approval_status != Some(2) {
        return Err(Error::from("仅待审批或审批中状态可进行审批操作"));
    }

    let instance_id = order.instance_id
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

    // 事务更新订单表
    let txn = db.begin().await?;
    use sea_orm::IntoActiveModel;
    let mut active: order_entity::ActiveModel = order.into_active_model();
    active.approval_status = Set(Some(new_status));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    active.update(&txn).await?;
    txn.commit().await?;

    get_detail(db, order_id).await
}

/// 驳回订单
pub async fn reject_order(db: &DbConn, order_id: i64, operator_id: i64, operator_name: &str, reason: Option<String>) -> Result<OrderDetailVO> {
    let order = OrderModel::find_by_id(db, order_id).await?
        .ok_or_else(|| Error::from("订单不存在"))?;

    if order.approval_status != Some(1) && order.approval_status != Some(2) {
        return Err(Error::from("仅待审批或审批中状态可进行驳回操作"));
    }

    let instance_id = order.instance_id
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

    // 事务更新订单表
    let txn = db.begin().await?;
    use sea_orm::IntoActiveModel;
    let mut active: order_entity::ActiveModel = order.into_active_model();
    active.approval_status = Set(Some(4));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    active.update(&txn).await?;
    txn.commit().await?;

    get_detail(db, order_id).await
}

/// 获取订单审批详情
pub async fn get_approval_detail(db: &DbConn, order_id: i64) -> Result<OrderApprovalDetailVO> {
    let order = OrderModel::find_by_id(db, order_id).await?
        .ok_or_else(|| Error::from("订单不存在"))?;

    let customer_name = if let Some(cid) = order.customer_id {
        Customer::find_by_id(cid)
            .one(db)
            .await?
            .and_then(|c| c.company_name.or(c.short_name))
    } else {
        None
    };

    let instance = if let Some(iid) = order.instance_id {
        ApprovalService::find_instance_by_id(db, iid).await?
    } else {
        None
    };

    Ok(OrderApprovalDetailVO {
        order_id: Some(order.id),
        order_no: order.order_no,
        title: order.title,
        customer_name,
        total_amount: order.total_amount,
        approval_status: order.approval_status,
        instance,
    })
}

/// 从订单创建合同
pub async fn create_contract_from_order(db: &DbConn, order_id: i64, operator_id: i64) -> Result<i64> {
    let order = OrderModel::find_by_id(db, order_id).await?
        .ok_or_else(|| Error::from("订单不存在"))?;

    // 只有审批通过的订单才能创建合同
    if order.approval_status != Some(3) {
        return Err(Error::from("仅审批通过的订单可创建合同"));
    }
    // 已经关联合同的不能重复创建
    if order.contract_id.is_some() {
        return Err(Error::from("该订单已关联合同，不能重复创建"));
    }

    let txn = db.begin().await?;

    // 创建合同（在事务内）
    let contract_dto = ContractSaveDTO {
        id: None,
        contract_no: None,
        customer_id: order.customer_id,
        opportunity_id: order.opportunity_id,
        title: order.title.clone(),
        contract_type: None,
        amount: order.total_amount,
        currency: order.currency.map(|c| CurrencyCode::from_i32(c).unwrap_or(CurrencyCode::CNY)),
        tax_amount: order.tax_amount,
        total_amount: order.total_amount,
        status: None,
        start_date: order.order_date,
        end_date: order.delivery_date,
        sign_date: None,
        payment_terms: None,
        delivery_terms: None,
        payment_method_type: None,
        assigned_to: order.owner_user_id,
        contract_file: None,
        contract_images: None,
        approval_status: Some(0),
        current_approval_stage: None,
        next_approver_id: None,
        approval_amount_limit: None,
        instance_id: None,
        remark: order.remark.clone(),
        commission_rule_id: None,
        commission_mode: None,
        deleted: Some(0),
        created_by: Some(operator_id),
        create_time: None,
        updated_by: None,
        update_time: None,
    };
    let contract_id = ContractModel::insert(&txn, &contract_dto).await?;

    // 更新订单的合同关联（在事务内）
    use sea_orm::IntoActiveModel;
    let mut active: order_entity::ActiveModel = order.into_active_model();
    active.contract_id = Set(Some(contract_id));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    active.update(&txn).await?;

    txn.commit().await?;
    Ok(contract_id)
}
