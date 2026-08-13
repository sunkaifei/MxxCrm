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
use crate::modules::crm::entity::opportunity::{self as opp_entity, Entity as Opportunity};
use crate::modules::crm::model::contract::{ContractSaveDTO, ContractModel};
use crate::modules::sale::entity::order::{self as order_entity, Entity as SaleOrder};
use crate::modules::sale::entity::quotation::{self as quo_entity, Entity as Quotation};
use crate::modules::sale::model::order::{OrderApprovalDetailVO, OrderDetailVO, OrderItemModel, OrderItemSaveDTO, OrderListQuery, OrderListVO, OrderModel, OrderSaveDTO, OrderSaveRequest, OrderStatusUpdateRequest, OrderUpdateRequest};
use crate::modules::sale::model::shipment::ShipmentModel;
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
use crate::modules::system::service::sales_flow_config_service;
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
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

    // 数据完整性校验：客户必填、标题必填
    let customer_id = form_data.customer_id.ok_or_else(|| Error::from("客户不能为空".to_string()))?;
    let title = form_data.title.as_ref().ok_or_else(|| Error::from("订单标题不能为空".to_string()))?;

    // 销售流程模式校验：未关联报价单时，校验企业配置允许跳过报价单
    if form_data.quotation_id.is_none() {
        let mode = sales_flow_config_service::get_mode(db).await;
        if !sales_flow_config_service::can_skip_quotation(&mode) {
            return Err(Error::from("当前企业销售流程模式要求订单必须关联报价单，请先创建报价单后再转订单"));
        }
        // 跳过报价单时，强制要求关联商机
        if form_data.opportunity_id.is_none() {
            return Err(Error::from("跳过报价单创建订单时必须关联商机"));
        }
    }

    let txn = db.begin().await?;

    // 同公司标题唯一性校验
    let existing = OrderModel::find_by_customer_and_title(&txn, customer_id, title, None).await?;
    if existing.is_some() {
        txn.rollback().await?;
        return Err(Error::from("该客户下已存在相同标题的订单"));
    }

    // 报价单存在性校验：若传入 quotation_id，必须查询到未删除的报价单
    if let Some(qid) = form_data.quotation_id {
        let quo = Quotation::find_by_id(qid)
            .filter(quo_entity::Column::Deleted.eq(0))
            .one(&txn)
            .await
            .map_err(|e| Error::from(format!("查询报价单失败: {}", e)))?;
        if quo.is_none() {
            txn.rollback().await?;
            return Err(Error::from(format!("关联的报价单(id={})不存在或已删除", qid)));
        }
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
    // 自动绑定创建者为订单负责人（若前端未指定负责人）
    if dto.owner_user_id.is_none() {
        dto.owner_user_id = Some(created_by);
    }

    // === 虚拟商品与服务订单支持：order_type / fulfillment_type 自动推导 ===
    use crate::modules::sale::model::order::{
        derive_order_type, default_fulfillment_type, PRODUCT_TYPE_PHYSICAL,
    };
    // 推导每条明细的 fulfillment_type（如未传则按 product_type 默认）
    let enriched_items: Vec<OrderItemSaveDTO> = items.iter().map(|it| {
        let pt = it.product_type.unwrap_or(PRODUCT_TYPE_PHYSICAL);
        let ft = it.fulfillment_type.unwrap_or_else(|| default_fulfillment_type(pt));
        let mut cloned = it.clone();
        cloned.fulfillment_type = Some(ft);
        cloned
    }).collect();
    // 推导订单 order_type
    let item_types: Vec<i32> = enriched_items.iter()
        .map(|it| it.product_type.unwrap_or(PRODUCT_TYPE_PHYSICAL))
        .collect();
    dto.order_type = Some(derive_order_type(&item_types));
    dto.fulfillment_type = enriched_items.first()
        .and_then(|it| it.fulfillment_type);
    // 服务/订阅订单：聚合服务期
    if let Some(max_duration) = enriched_items.iter()
        .filter_map(|it| it.service_duration)
        .max() {
        dto.service_duration = Some(max_duration);
        let earliest_start = enriched_items.iter()
            .filter_map(|it| it.service_start_date)
            .min();
        dto.service_start_date = earliest_start.or(Some(chrono::Local::now().date_naive()));
        if let Some(start) = dto.service_start_date {
            dto.service_end_date = Some(add_months_naive(start, max_duration));
        }
    }

    let order_id = OrderModel::insert(&txn, &dto).await?;
    OrderItemModel::insert_batch(&txn, order_id, &enriched_items).await?;

    txn.commit().await?;

    Ok(order_id)
}

/// 增加 N 月（处理跨年/月末）
fn add_months_naive(date: chrono::NaiveDate, months: i32) -> chrono::NaiveDate {
    use chrono::Datelike;
    let total = date.year() * 12 + (date.month() as i32 - 1) + months;
    let new_year = total.div_euclid(12);
    let new_month = total.rem_euclid(12) + 1;
    let last_day = chrono::NaiveDate::from_ymd_opt(
        if new_month == 12 { new_year + 1 } else { new_year },
        if new_month == 12 { 1u32 } else { (new_month + 1) as u32 },
        1
    ).map(|d| d.pred_opt().unwrap_or(d).day()).unwrap_or(28);
    let day = date.day().min(last_day);
    chrono::NaiveDate::from_ymd_opt(new_year, new_month as u32, day).unwrap_or(date)
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

    // 数据完整性校验：客户必填、标题必填
    let customer_id = form_data.customer_id.ok_or_else(|| Error::from("客户不能为空".to_string()))?;
    let title = form_data.title.as_ref().ok_or_else(|| Error::from("订单标题不能为空".to_string()))?;

    let existing = OrderModel::find_by_id(db, id).await?;
    if existing.is_none() {
        return Err(Error::from("订单不存在"));
    }

    // 审批状态校验：仅草稿(0)或已驳回(4)允许编辑
    let approval_status = existing.as_ref().unwrap().approval_status.unwrap_or(0);
    if approval_status != 0 && approval_status != 4 {
        return Err(Error::from("当前订单审批状态不允许编辑"));
    }

    let _existing_order = existing.unwrap();

    let txn = db.begin().await?;

    // 同公司标题唯一性校验（排除自身 ID）
    let existing_title = OrderModel::find_by_customer_and_title(&txn, customer_id, title, form_data.id).await?;
    if existing_title.is_some() {
        txn.rollback().await?;
        return Err(Error::from("该客户下已存在相同标题的订单"));
    }

    // 报价单存在性校验：若传入 quotation_id，必须查询到未删除的报价单
    if let Some(qid) = form_data.quotation_id {
        let quo = Quotation::find_by_id(qid)
            .filter(quo_entity::Column::Deleted.eq(0))
            .one(&txn)
            .await
            .map_err(|e| Error::from(format!("查询报价单失败: {}", e)))?;
        if quo.is_none() {
            txn.rollback().await?;
            return Err(Error::from(format!("关联的报价单(id={})不存在或已删除", qid)));
        }
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

    // === 虚拟商品与服务订单支持：order_type / fulfillment_type 自动推导（编辑场景） ===
    use crate::modules::sale::model::order as order_model_mod;
    let enriched_items: Vec<OrderItemSaveDTO> = items.iter().map(|it| {
        let pt = it.product_type.unwrap_or(order_model_mod::PRODUCT_TYPE_PHYSICAL);
        let ft = it.fulfillment_type.unwrap_or_else(|| order_model_mod::default_fulfillment_type(pt));
        let mut cloned = it.clone();
        cloned.fulfillment_type = Some(ft);
        cloned
    }).collect();
    let item_types: Vec<i32> = enriched_items.iter()
        .map(|it| it.product_type.unwrap_or(order_model_mod::PRODUCT_TYPE_PHYSICAL))
        .collect();
    dto.order_type = Some(order_model_mod::derive_order_type(&item_types));
    dto.fulfillment_type = enriched_items.first().and_then(|it| it.fulfillment_type);

    OrderModel::update_by_id(&txn, id, &dto).await?;
    OrderItemModel::delete_by_order_id(&txn, id).await?;
    OrderItemModel::insert_batch(&txn, id, &enriched_items).await?;

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

    // 订单作废校验：审批通过(3)且未签合同才能作废
    if order_status == 11 {
        let order = existing.as_ref().unwrap();
        let approval_status = order.approval_status.unwrap_or(0);
        if approval_status != 3 {
            return Err(Error::from("只有审批通过的订单才能作废"));
        }
        if order.contract_id.is_some() {
            return Err(Error::from("已签合同的订单不能作废"));
        }
    }

    // === 虚拟商品与服务订单支持：状态机前置校验 ===
    use crate::modules::sale::model::order as order_const;
    let order = existing.as_ref().unwrap();
    let current_status = order.order_status.unwrap_or(0);
    let order_type = order.order_type.unwrap_or(order_const::ORDER_TYPE_PHYSICAL);

    // 校验1：实物订单未发货完成不允许置"已完成"（除非全部明细 fulfillment_type=5 无需交付）
    if order_status == 10 && order_type == order_const::ORDER_TYPE_PHYSICAL
        && current_status != 9 && current_status != 6 {
        // 允许：已发货(6)/已签收(9) → 已完成(10)；其他拒绝
        if current_status < 6 {
            return Err(Error::from("实物订单未发货完成，不允许置为已完成"));
        }
    }

    // 校验2：虚拟订单未全部交付不允许置"已完成"
    if order_status == 10 && matches!(order_type,
        order_const::ORDER_TYPE_VIRTUAL | order_const::ORDER_TYPE_MIXED) {
        let items = OrderItemModel::find_by_order_id(db, id).await?;
        use crate::modules::sale::model::order_delivery::DeliveryModel;
        let mut all_delivered = true;
        for item in items.iter() {
            let pt = item.product_type.unwrap_or(order_const::PRODUCT_TYPE_PHYSICAL);
            if !matches!(pt, order_const::PRODUCT_TYPE_VIRTUAL | order_const::PRODUCT_TYPE_SUBSCRIPTION) {
                continue;
            }
            let item_id = item.id;
            let delivered = DeliveryModel::count_by_item(db, item_id).await
                .map_err(|e| Error::from(e.to_string()))?;
            let needed = item.quantity.unwrap_or(rust_decimal::Decimal::ZERO)
                .to_i64().unwrap_or(0);
            if delivered < needed {
                all_delivered = false;
                break;
            }
        }
        if !all_delivered {
            return Err(Error::from("虚拟订单未全部交付，不允许置为已完成"));
        }
    }

    // 校验3：服务/订阅订单置"服务中"必须有 entitlement
    if order_status == order_const::ORDER_STATUS_IN_SERVICE
        && matches!(order_type, order_const::ORDER_TYPE_SERVICE | order_const::ORDER_TYPE_SUBSCRIPTION) {
        use crate::modules::sale::model::entitlement::EntitlementModel;
        let ents = EntitlementModel::find_by_order(db, id).await
            .map_err(|e| Error::from(e.to_string()))?;
        if ents.is_empty() {
            return Err(Error::from("服务订单缺少权益记录，不允许置为服务中"));
        }
    }

    let result = OrderModel::update_status(db, id, order_status, form_data.tracking_no.clone(), form_data.remark.clone()).await?;

    // 订单作废时，同步更新关联备货计划状态
    if order_status == 11 {
        use crate::modules::purchase::service::purchase_stock_plan_service;
        let _ = purchase_stock_plan_service::sync_order_change(db, id, "cancelled").await;
    }

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

            // 实时查询关联商机名称（用于编辑时回显）
            if let Some(opp_id) = vo.opportunity_id {
                if let Some(opp) = Opportunity::find_by_id(opp_id)
                    .filter(opp_entity::Column::Deleted.eq(0))
                    .one(db)
                    .await?
                {
                    vo.opportunity_name = opp.title.clone();
                }
            }

            // 实时查询关联报价单标题与编号（用于编辑时回显）
            if let Some(qid) = vo.quotation_id {
                if let Some(q) = Quotation::find_by_id(qid)
                    .filter(quo_entity::Column::Deleted.eq(0))
                    .one(db)
                    .await?
                {
                    vo.quotation_title = q.title.clone();
                    vo.quotation_no = q.quotation_no.clone();
                }
            }

            // 实时查询关联合同标题与编号（用于编辑时回显）
            if let Some(ctid) = vo.contract_id {
                if let Ok(Some(ct)) = ContractModel::find_by_id(db, ctid).await {
                    vo.contract_title = ct.title.clone();
                    vo.contract_no = ct.contract_no.clone();
                }
            }

            Ok(vo)
        }
        None => Err(Error::from("订单不存在")),
    }
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
            // 下属订单：获取数据权限范围内的其他用户（排除自己）
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

    // 批量查询负责人名称（ID -> 名称）
    let owner_user_ids: Vec<i64> = list.iter()
        .filter_map(|c| c.owner_user_id)
        .collect();
    let owner_name_map = crate::modules::system::service::admin_service::build_admin_name_map(db, owner_user_ids).await;

    let data: Vec<OrderListVO> = list.iter().map(|item| {
        let mut vo: OrderListVO = item.into();
        if let Some(cid) = vo.customer_id {
            if let Some(name) = customer_name_map.get(&cid) {
                vo.customer_name = Some(name.clone());
            }
        }
        if let Some(oid) = vo.owner_user_id {
            if let Some(name) = owner_name_map.get(&oid) {
                vo.owner_user_name = Some(name.clone());
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

    if order.approval_status != Some(0) && order.approval_status != Some(4)
        && order.approval_status != Some(5) && order.approval_status != Some(6) {
        return Err(Error::from("当前状态不允许提交，仅草稿、已驳回、已撤回或待修改状态可提交"));
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
        cc_user_ids: None,
        cc_reason: None,
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

    // 审批通过后自动生成 PDF（best-effort，不阻断审批流程）
    if new_status == 3 {
        crate::modules::system::service::pdf_generator_service::generate_for_order_approval(db, order_id, Some(operator_id));

        // === 07-虚拟商品：审批通过后，服务/订阅订单自动创建权益 ===
        // best-effort，失败不阻断审批流程
        if let Ok(Some(order_check)) = OrderModel::find_by_id(db, order_id).await {
            let order_type = order_check.order_type.unwrap_or(1);
            if matches!(order_type, 3 | 4 | 5) { // 服务/订阅/混合
                let _ = crate::modules::sale::service::entitlement_service::create_for_order(db, order_id).await;
            }
        }
    }

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
