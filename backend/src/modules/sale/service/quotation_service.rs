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
use crate::modules::crm::entity::opportunity::{self as opp_entity, Entity as Opportunity};
use crate::modules::sale::entity::quotation::{self as quo_entity, Entity as Quotation};
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

    // 数据完整性校验：客户必填、标题必填
    let customer_id = form_data.customer_id.ok_or_else(|| Error::from("客户不能为空".to_string()))?;
    let title = form_data.title.as_deref().ok_or_else(|| Error::from("报价单标题不能为空".to_string()))?;

    let txn = db.begin().await?;

    // 同公司标题唯一性校验
    let existing = QuotationModel::find_by_customer_and_title(&txn, customer_id, title, None).await?;
    if existing.is_some() {
        txn.rollback().await?;
        return Err(Error::from("该客户下已存在相同标题的报价单"));
    }

    // 商机存在性校验：若传入 opportunity_id，必须查询到未删除的商机
    if let Some(opp_id) = form_data.opportunity_id {
        let opp = Opportunity::find_by_id(opp_id)
            .filter(opp_entity::Column::Deleted.eq(0))
            .one(&txn)
            .await
            .map_err(|e| Error::from(format!("查询商机失败: {}", e)))?;
        if opp.is_none() {
            txn.rollback().await?;
            return Err(Error::from(format!("关联的商机(id={})不存在或已删除", opp_id)));
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

    // 负责人未指定时自动绑定为当前登录用户（创建人）
    if dto.owner_user_id.is_none() {
        if let Ok(uid) = created_by.parse::<i64>() {
            if uid > 0 {
                dto.owner_user_id = Some(uid);
            }
        }
    }

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

    // 数据完整性校验：客户必填、标题必填
    let customer_id = form_data.customer_id.ok_or_else(|| Error::from("客户不能为空".to_string()))?;
    let title = form_data.title.as_deref().ok_or_else(|| Error::from("报价单标题不能为空".to_string()))?;

    let existing = QuotationModel::find_by_id(db, id).await?;
    let existing = existing.ok_or_else(|| Error::from("报价单不存在"))?;

    // 审批中(approval_status=2)不允许修改
    if existing.approval_status == Some(2) {
        return Err(Error::from("报价单审批中，不允许修改"));
    }

    let txn = db.begin().await?;

    // 同公司标题唯一性校验（排除自身 ID）
    let duplicate = QuotationModel::find_by_customer_and_title(&txn, customer_id, title, form_data.id).await?;
    if duplicate.is_some() {
        txn.rollback().await?;
        return Err(Error::from("该客户下已存在相同标题的报价单"));
    }

    // 商机存在性校验：若传入 opportunity_id，必须查询到未删除的商机
    if let Some(opp_id) = form_data.opportunity_id {
        let opp = Opportunity::find_by_id(opp_id)
            .filter(opp_entity::Column::Deleted.eq(0))
            .one(&txn)
            .await
            .map_err(|e| Error::from(format!("查询商机失败: {}", e)))?;
        if opp.is_none() {
            txn.rollback().await?;
            return Err(Error::from(format!("关联的商机(id={})不存在或已删除", opp_id)));
        }
    }

    let mut dto: QuotationSaveDTO = form_data.clone().into();
    dto.update_by = Some(updated_by.clone());

    // 编辑时保留原有负责人：前端表单已移除负责人字段，不覆盖已有值
    dto.owner_user_id = existing.owner_user_id;

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

    // 删除前释放已审批通过报价单的冻结库存（best-effort）
    for &id in ids_vec {
        let _ = unfreeze_stock_for_quotation(db, id, 0).await;
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



pub async fn list(db: &DbConn, query: &QuotationListQuery, current_user_id: i64) -> Result<ResultPage<Vec<QuotationListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("all");

    let owner_user_ids_opt: Option<Vec<i64>> = match list_type {
        "my" => {
            Some(vec![current_user_id])
        }
        "subordinate" => {
            // 下属报价单：获取数据权限范围内的其他用户（排除自己）
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

    let (list, total) = QuotationModel::select_in_page_by_owner_user_ids(
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
    ).await?;

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

    // 批量查询负责人名称（ID -> 名称）
    let owner_user_ids: Vec<i64> = list.iter()
        .filter_map(|c| c.owner_user_id)
        .collect();
    let owner_name_map = crate::modules::system::service::admin_service::build_admin_name_map(db, owner_user_ids).await;

    let data: Vec<QuotationListVO> = list.into_iter().map(|item| {
        let cid = item.customer_id;
        let oid = item.owner_user_id;
        let mut vo: QuotationListVO = item.into();
        if let Some(c) = cid {
            vo.customer_name = customer_map.get(&c).cloned();
        }
        if let Some(o) = oid {
            if let Some(name) = owner_name_map.get(&o) {
                vo.owner_user_name = Some(name.clone());
            }
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

    // 仅报价单负责人本人可提交审批，其他人无权提交
    if quotation.owner_user_id.unwrap_or(0) != operator_id {
        return Err(Error::from("只能提交自己负责的报价单进行审批".to_string()));
    }

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

    // 审批通过后自动冻结库存（best-effort，不阻断审批流程）
    if new_approval_status == 3 {
        if let Err(e) = freeze_stock_for_quotation(db, id, operator_id).await {
            log::warn!("[quotation::approve] 报价单 {} 库存冻结失败: {}", id, e);
        }
        // 审批通过后自动生成 PDF（best-effort，不阻断审批流程）
        crate::modules::system::service::pdf_generator_service::generate_for_quotation_approval(db, id, Some(operator_id));
    }

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

    // 转单后释放报价单冻结的库存（best-effort，不阻断转单）
    let _ = unfreeze_stock_for_quotation(db, quotation_id, created_by_i64).await;

    Ok(order_id)
}

/// 根据报价单明细冻结库存（审批通过时调用）
async fn freeze_stock_for_quotation(db: &DbConn, quotation_id: i64, operator_id: i64) -> Result<()> {
    use crate::modules::inventory::service::freeze_service;
    use crate::modules::inventory::entity::stock as stock_entity;

    let items = QuotationItemModel::find_by_quotation_id(db, quotation_id).await?;
    let quotation = QuotationModel::find_by_id(db, quotation_id).await?
        .ok_or_else(|| Error::from("报价单不存在"))?;
    let quotation_no = quotation.quotation_no.as_deref().unwrap_or("");

    for item in &items {
        let product_id = item.product_id.unwrap_or(0);
        let quantity = item.quantity.unwrap_or(Decimal::from(0));
        if product_id <= 0 || quantity <= Decimal::from(0) {
            continue;
        }
        // 查找该产品有库存的仓库，选择库存最充足的仓库冻结
        let stock_record = stock_entity::Entity::find()
            .filter(stock_entity::Column::ProductId.eq(product_id))
            .filter(stock_entity::Column::AvailableQuantity.gte(quantity))
            .filter(stock_entity::Column::Deleted.eq(0))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
            .into_iter()
            .max_by_key(|s| s.available_quantity.unwrap_or(Decimal::from(0)));

        if let Some(s) = stock_record {
            let warehouse_id = s.warehouse_id.unwrap_or(0);
            let reason = format!("报价单 {} 审批通过，预留库存", quotation_no);
            freeze_service::freeze_stock(db, product_id, warehouse_id, quantity, Some(reason), operator_id).await?;
        }
    }
    Ok(())
}

/// 根据报价单明细释放冻结库存（转单/删除时调用）
async fn unfreeze_stock_for_quotation(db: &DbConn, quotation_id: i64, operator_id: i64) -> Result<()> {
    use crate::modules::inventory::service::freeze_service;
    use crate::modules::inventory::entity::stock_freeze;

    let quotation = match QuotationModel::find_by_id(db, quotation_id).await? {
        Some(q) => q,
        None => return Ok(()),
    };

    // 仅审批通过(status=3)的报价单可能冻结了库存
    if quotation.approval_status != Some(3) && quotation.status != Some(3) {
        return Ok(());
    }

    let quotation_no = quotation.quotation_no.as_deref().unwrap_or("");
    let items = QuotationItemModel::find_by_quotation_id(db, quotation_id).await?;

    for item in &items {
        let product_id = item.product_id.unwrap_or(0);
        let quantity = item.quantity.unwrap_or(Decimal::from(0));
        if product_id <= 0 || quantity <= Decimal::from(0) {
            continue;
        }
        // 查找该产品所有冻结记录并解冻
        let freeze_records = stock_freeze::Entity::find()
            .filter(stock_freeze::Column::ProductId.eq(product_id))
            .filter(stock_freeze::Column::Status.eq(0))
            .filter(stock_freeze::Column::Deleted.eq(0))
            .filter(stock_freeze::Column::Remark.contains(format!("报价单 {}", quotation_no).as_str()))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        for record in freeze_records {
            let fq = record.freeze_quantity.unwrap_or(Decimal::from(0));
            if fq > Decimal::from(0) {
                let wid = record.warehouse_id.unwrap_or(0);
                let _ = freeze_service::unfreeze_stock(db, product_id, wid, fq.min(quantity), operator_id).await;
            }
        }
    }
    Ok(())
}
