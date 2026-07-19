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
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DbConn, TransactionTrait, EntityTrait, ColumnTrait, QueryFilter, Set};
use std::collections::{HashMap, HashSet};

fn calculate_product_amount(items: &Vec<OrderItemSaveDTO>) -> Decimal {
    let hundred = Decimal::from(100);
    items.iter().map(|item| {
        let qty = item.quantity.unwrap_or(Decimal::from(1));
        let price = item.unit_price.unwrap_or(Decimal::from(0));
        let disc_rate = item.discount_rate.unwrap_or(Decimal::from(100));
        let tax_rate = item.tax_rate.unwrap_or(Decimal::from(0));

        let gross = price * qty;
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

    if let (Some(customer_id), Some(title)) = (form_data.customer_id, form_data.title.as_ref()) {
        let existing = OrderModel::find_by_customer_and_title(db, customer_id, title, None).await?;
        if existing.is_some() {
            return Err(Error::from("该客户下已存在相同标题的订单"));
        }
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
    dto.approval_status = Some(0);
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

    let existing_order = existing.unwrap();
    let customer_id = form_data.customer_id.or(existing_order.customer_id);
    let title = form_data.title.as_ref().or(existing_order.title.as_ref());

    if let (Some(cid), Some(t)) = (customer_id, title) {
        let existing_title = OrderModel::find_by_customer_and_title(db, cid, t, form_data.id).await?;
        if existing_title.is_some() {
            return Err(Error::from("该客户下已存在相同标题的订单"));
        }
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
            let mut vo: OrderDetailVO = (&o, items, shipments).into();

            // 实时查询客户名称（订单表中 customer_name 是历史快照，可能已过期）
            if let Some(cid) = vo.customer_id {
                if let Some(c) = Customer::find_by_id(cid).one(db).await? {
                    let fresh_name = c.company_name.or(c.short_name).unwrap_or_default();
                    if !fresh_name.is_empty() {
                        vo.customer_name = Some(fresh_name);
                    }
                }
            }

            Ok(vo)
        }
        None => Err(Error::from("订单不存在")),
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

pub async fn get_list(db: &DbConn, query: &OrderListQuery, current_user_id: i64) -> Result<ResultPage<Vec<OrderListVO>>> {
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
        OrderModel::select_in_page(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.order_status,
            query.payment_status,
            query.customer_id,
            Some(current_user_id),
            query.start_date.clone(),
            query.end_date.clone(),
        ).await?
    } else {
        OrderModel::select_in_page_by_owner_user_ids(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.order_status,
            query.payment_status,
            query.customer_id,
            query.start_date.clone(),
            query.end_date.clone(),
            owner_user_ids_opt,
        ).await?
    };

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

    // 查询我方签署人姓名（订单创建人）
    let our_signer_name = if let Some(creator_id) = order.create_by {
        crate::modules::system::entity::admin::Entity::find_by_id(creator_id)
            .one(db)
            .await
            .ok()
            .flatten()
            .and_then(|a| a.nick_name.or(a.user_name))
    } else {
        None
    };

    let txn = db.begin().await?;

    // 创建合同（在事务内）
    // 我方签署人默认为订单创建人（业务员），对方签署人默认为订单联系人
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
        our_signer_id: order.create_by,
        our_signer_name: our_signer_name.clone(),
        their_signer_name: order.contact_name.clone(),
        their_signer_phone: None,
        order_id: Some(order_id),
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
