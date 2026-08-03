//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 销售退货单业务逻辑层
//!

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::approval::model::approval::ApprovalSubmitRequest;
use crate::modules::approval::service::approval_service::ApprovalService;
use crate::modules::crm::entity::customer::{Entity as Customer, Column as CustomerColumn};
use crate::modules::sale::entity::invoice::{self as invoice_entity, Entity as SaleInvoice};
use crate::modules::sale::entity::order::{self as order_entity, Entity as SaleOrder};
use crate::modules::sale::entity::order_item::{self as order_item_entity, Entity as SaleOrderItem};
use crate::modules::sale::entity::payment::{self as payment_entity, Entity as SalePayment};
use crate::modules::sale::entity::refund::{self as refund_entity, Entity as SaleRefund};
use crate::modules::sale::entity::refund_item::{self as refund_item_entity, Entity as SaleRefundItem};
use crate::modules::sale::model::refund::{
    RefundApprovalReq, RefundItemModel, RefundItemSaveDTO, RefundListQuery, RefundListVO,
    RefundModel, RefundPaymentModel, RefundPaymentRequest, RefundQualityCheckReq,
    RefundReceiveReq, RefundSaveDTO, RefundSaveRequest, RefundUpdateRequest, RefundDetailVO,
    RefundItemVO, RefundPaymentVO,
};
use crate::modules::system::entity::admin::{self, Entity as Admin};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, ConnectionTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait};
use std::collections::{HashMap, HashSet};

/// 计算退货明细金额合计
fn calculate_refund_amount(items: &Vec<RefundItemSaveDTO>) -> Decimal {
    items.iter().map(|item| {
        let qty = item.refund_qty.unwrap_or(Decimal::from(0));
        let price = item.unit_price.unwrap_or(Decimal::from(0));
        item.refund_amount.unwrap_or(qty * price)
    }).fold(Decimal::from(0), |acc, x| acc + x)
}

/// 创建退货单
pub async fn insert(db: &DbConn, form_data: &RefundSaveRequest, created_by: i64) -> Result<i64> {
    let items = form_data.items.clone().unwrap_or_default();
    if items.is_empty() {
        return Err(Error::from("退货明细不能为空"));
    }

    // 数据完整性校验
    let order_id = form_data.order_id.ok_or_else(|| Error::from("关联订单不能为空".to_string()))?;
    let customer_id = form_data.customer_id.ok_or_else(|| Error::from("客户不能为空".to_string()))?;
    let title = form_data.title.as_ref().ok_or_else(|| Error::from("退货标题不能为空".to_string()))?;

    // 校验关联订单存在且状态为已发货(5)/已签收(9)/已完成(10)
    let order = SaleOrder::find_by_id(order_id)
        .filter(order_entity::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(format!("查询订单失败: {}", e)))?
        .ok_or_else(|| Error::from("关联的销售订单不存在"))?;

    let order_status = order.order_status.unwrap_or(0);
    if order_status != 5 && order_status != 9 && order_status != 10 {
        return Err(Error::from(format!(
            "仅已发货(5)、已签收(9)或已完成(10)的订单可发起退货，当前订单状态为 {}",
            order_status
        )));
    }

    // 校验退货数量不超过订单明细的可退数量（已发货 - 已退货）
    let existing_refunded_map = get_existing_refunded_qty_by_order(db, order_id).await?;
    let order_items = SaleOrderItem::find()
        .filter(order_item_entity::Column::OrderId.eq(order_id))
        .filter(order_item_entity::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(format!("查询订单明细失败: {}", e)))?;

    let mut order_item_map: HashMap<i64, &order_item_entity::Model> = HashMap::new();
    for oi in &order_items {
        order_item_map.insert(oi.id, oi);
    }

    for item in &items {
        if let Some(oi_id) = item.order_item_id {
            let oi = order_item_map.get(&oi_id)
                .ok_or_else(|| Error::from(format!("订单明细 id={} 不存在", oi_id)))?;
            let delivered = oi.delivered_quantity.unwrap_or(oi.quantity.unwrap_or(Decimal::from(0)));
            let already_refunded = existing_refunded_map.get(&oi_id).copied().unwrap_or(Decimal::from(0));
            let refundable = delivered - already_refunded;
            let want = item.refund_qty.unwrap_or(Decimal::from(0));
            if want > refundable {
                return Err(Error::from(format!(
                    "产品 [{}] 退货数量 {} 超过可退数量 {}（已发货 {}，已退货 {}）",
                    item.product_name.clone().unwrap_or_default(),
                    want, refundable, delivered, already_refunded
                )));
            }
        }
    }

    // 生成退货单号 RF{YYYYMMDD}{4位序号}
    let date_prefix = format!("RF{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = RefundModel::get_max_refund_no_today(db, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let refund_no = format!("{}{:04}", date_prefix, seq);

    // 计算金额
    let total_amount = calculate_refund_amount(&items);
    let restocking_fee = form_data.restocking_fee.unwrap_or(Decimal::from(0));
    let refund_amount = total_amount - restocking_fee;

    let txn = db.begin().await?;

    // 同客户同标题排重校验
    let dup_customer = RefundModel::find_by_customer_and_title(&txn, customer_id, title, None).await?;
    if dup_customer.is_some() {
        txn.rollback().await?;
        return Err(Error::from("该客户下已存在相同标题的退货单"));
    }

    // 同订单同标题排重校验（针对同一订单不允许重复退货标题）
    let dup_order = RefundModel::find_by_order_and_title(&txn, order_id, title, None).await?;
    if dup_order.is_some() {
        txn.rollback().await?;
        return Err(Error::from("该订单下已存在相同标题的退货单"));
    }

    let mut dto: RefundSaveDTO = form_data.clone().into();
    dto.refund_no = Some(refund_no);
    dto.refund_status = Some(1); // 草稿
    dto.approval_status = Some(0);
    dto.total_amount = Some(total_amount);
    dto.refund_amount = Some(refund_amount);
    dto.refunded_amount = Some(Decimal::from(0));
    dto.create_by = Some(created_by);
    // 自动绑定创建者为负责人（若前端未指定）
    if dto.owner_user_id.is_none() {
        dto.owner_user_id = Some(created_by);
    }

    let refund_id = RefundModel::insert(&txn, &dto).await?;
    RefundItemModel::insert_batch(&txn, refund_id, &items).await?;

    txn.commit().await?;

    Ok(refund_id)
}

/// 更新退货单（仅草稿/已驳回状态允许编辑）
pub async fn update(db: &DbConn, form_data: &RefundUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("退货单ID不能为空"));
    }
    let items = form_data.items.clone().unwrap_or_default();
    if items.is_empty() {
        return Err(Error::from("退货明细不能为空"));
    }

    // 数据完整性校验
    let order_id = form_data.order_id.ok_or_else(|| Error::from("关联订单不能为空".to_string()))?;
    let customer_id = form_data.customer_id.ok_or_else(|| Error::from("客户不能为空".to_string()))?;
    let title = form_data.title.as_ref().ok_or_else(|| Error::from("退货标题不能为空".to_string()))?;

    let existing = RefundModel::find_by_id(db, id).await?;
    if existing.is_none() {
        return Err(Error::from("退货单不存在"));
    }
    let existing_refund = existing.unwrap();

    // 状态校验：仅草稿(1)或已驳回(8)或已取消(9)允许编辑
    let refund_status = existing_refund.refund_status.unwrap_or(1);
    if refund_status != 1 && refund_status != 8 && refund_status != 9 {
        return Err(Error::from(format!("当前退货单状态({})不允许编辑", refund_status)));
    }

    // 审批状态校验：仅草稿(0)或已驳回(4)允许编辑
    let approval_status = existing_refund.approval_status.unwrap_or(0);
    if approval_status != 0 && approval_status != 4 {
        return Err(Error::from("当前退货单审批状态不允许编辑"));
    }

    // 校验关联订单存在且状态为已发货/已签收/已完成
    let order = SaleOrder::find_by_id(order_id)
        .filter(order_entity::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(format!("查询订单失败: {}", e)))?
        .ok_or_else(|| Error::from("关联的销售订单不存在"))?;

    let order_status = order.order_status.unwrap_or(0);
    if order_status != 5 && order_status != 9 && order_status != 10 {
        return Err(Error::from(format!(
            "仅已发货(5)、已签收(9)或已完成(10)的订单可发起退货，当前订单状态为 {}",
            order_status
        )));
    }

    // 退货数量校验（排除自身的已退数量）
    let mut existing_refunded_map = get_existing_refunded_qty_by_order(db, order_id).await?;
    // 排除当前退货单自身的明细
    let self_items = RefundItemModel::find_by_refund_id(db, id).await?;
    for si in &self_items {
        if let Some(oi_id) = si.order_item_id {
            if let Some(qty) = existing_refunded_map.get_mut(&oi_id) {
                if let Some(refund_qty) = si.refund_qty {
                    *qty -= refund_qty;
                }
            }
        }
    }

    let order_items = SaleOrderItem::find()
        .filter(order_item_entity::Column::OrderId.eq(order_id))
        .filter(order_item_entity::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(format!("查询订单明细失败: {}", e)))?;

    let mut order_item_map: HashMap<i64, &order_item_entity::Model> = HashMap::new();
    for oi in &order_items {
        order_item_map.insert(oi.id, oi);
    }

    for item in &items {
        if let Some(oi_id) = item.order_item_id {
            let oi = order_item_map.get(&oi_id)
                .ok_or_else(|| Error::from(format!("订单明细 id={} 不存在", oi_id)))?;
            let delivered = oi.delivered_quantity.unwrap_or(oi.quantity.unwrap_or(Decimal::from(0)));
            let already_refunded = existing_refunded_map.get(&oi_id).copied().unwrap_or(Decimal::from(0));
            let refundable = delivered - already_refunded;
            let want = item.refund_qty.unwrap_or(Decimal::from(0));
            if want > refundable {
                return Err(Error::from(format!(
                    "产品 [{}] 退货数量 {} 超过可退数量 {}（已发货 {}，已退货 {}）",
                    item.product_name.clone().unwrap_or_default(),
                    want, refundable, delivered, already_refunded
                )));
            }
        }
    }

    // 计算金额
    let total_amount = calculate_refund_amount(&items);
    let restocking_fee = form_data.restocking_fee.unwrap_or(Decimal::from(0));
    let refund_amount = total_amount - restocking_fee;

    let txn = db.begin().await?;

    // 同客户同标题排重校验（排除自身）
    let dup_customer = RefundModel::find_by_customer_and_title(&txn, customer_id, title, Some(id)).await?;
    if dup_customer.is_some() {
        txn.rollback().await?;
        return Err(Error::from("该客户下已存在相同标题的退货单"));
    }

    // 同订单同标题排重校验（排除自身）
    let dup_order = RefundModel::find_by_order_and_title(&txn, order_id, title, Some(id)).await?;
    if dup_order.is_some() {
        txn.rollback().await?;
        return Err(Error::from("该订单下已存在相同标题的退货单"));
    }

    let mut dto: RefundSaveDTO = form_data.clone().into();
    dto.total_amount = Some(total_amount);
    dto.refund_amount = Some(refund_amount);
    dto.update_by = Some(updated_by);

    RefundModel::update_by_id(&txn, id, &dto).await?;
    RefundItemModel::delete_by_refund_id(&txn, id).await?;
    RefundItemModel::insert_batch(&txn, id, &items).await?;

    txn.commit().await?;

    Ok(id)
}

/// 批量删除退货单（仅草稿状态允许删除）
pub async fn batch_delete(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    // 状态校验：仅草稿(1)/已驳回(8)/已取消(9)允许删除
    for &id in ids_vec {
        let existing = RefundModel::find_by_id(db, id).await?;
        if let Some(refund) = existing {
            let refund_status = refund.refund_status.unwrap_or(1);
            if refund_status != 1 && refund_status != 8 && refund_status != 9 {
                return Err(Error::from(format!(
                    "退货单[{}]当前状态不允许删除",
                    refund.refund_no.unwrap_or_default()
                )));
            }
        }
    }
    let result = RefundModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

/// 获取退货单详情
pub async fn get_detail(db: &DbConn, id: i64) -> Result<RefundDetailVO> {
    let refund = RefundModel::find_by_id(db, id).await?
        .ok_or_else(|| Error::from("退货单不存在"))?;

    let items = RefundItemModel::find_by_refund_id(db, id).await?;
    let payments = RefundPaymentModel::find_by_refund_id(db, id).await?;

    let mut vo = RefundDetailVO {
        id: refund.id.into(),
        refund_no: refund.refund_no.clone(),
        title: refund.title.clone(),
        order_id: refund.order_id,
        order_no: None,
        customer_id: refund.customer_id,
        customer_name: refund.customer_name.clone(),
        refund_type: refund.refund_type,
        refund_reason: refund.refund_reason.clone(),
        refund_status: refund.refund_status,
        approval_status: refund.approval_status,
        instance_id: refund.instance_id,
        total_amount: refund.total_amount,
        restocking_fee: refund.restocking_fee,
        refund_amount: refund.refund_amount,
        refunded_amount: refund.refunded_amount,
        warehouse_id: refund.warehouse_id,
        receiver: refund.receiver.clone(),
        receiver_phone: refund.receiver_phone.clone(),
        receiver_address: refund.receiver_address.clone(),
        logistics_no: refund.logistics_no.clone(),
        logistics_company: refund.logistics_company.clone(),
        quality_check_result: refund.quality_check_result,
        quality_check_remark: refund.quality_check_remark.clone(),
        owner_user_id: refund.owner_user_id,
        owner_user_name: None,
        dept_id: refund.dept_id,
        remark: refund.remark.clone(),
        create_by: refund.create_by,
        create_time: refund.create_time,
        update_by: refund.update_by,
        update_time: refund.update_time,
        items: items.iter().map(|i| i.into()).collect(),
        payments: payments.iter().map(|p| p.into()).collect(),
        warning: None,
    };

    // 实时查询客户名称
    if let Some(cid) = vo.customer_id {
        if let Some(c) = Customer::find_by_id(cid).one(db).await? {
            let fresh_name = c.company_name.or(c.short_name).unwrap_or_default();
            if !fresh_name.is_empty() {
                vo.customer_name = Some(fresh_name);
            }
        }
    }

    // 实时查询订单号
    if let Some(oid) = vo.order_id {
        if let Some(o) = SaleOrder::find_by_id(oid).one(db).await? {
            vo.order_no = o.order_no;
        }
    }

    // 实时查询负责人名称
    if let Some(uid) = vo.owner_user_id {
        if let Some(a) = Admin::find_by_id(uid).one(db).await? {
            vo.owner_user_name = a.nick_name.or(a.user_name);
        }
    }

    Ok(vo)
}

/// 查询某订单下各明细已退货数量（不含未保存的草稿）
pub async fn get_existing_refunded_qty_by_order(db: &DbConn, order_id: i64) -> Result<HashMap<i64, Decimal>> {
    // 查询该订单关联的所有未删除退货单
    let refunds = SaleRefund::find()
        .filter(refund_entity::Column::OrderId.eq(order_id))
        .filter(refund_entity::Column::Deleted.eq(0))
        // 排除已取消(9)状态的退货单
        .filter(refund_entity::Column::RefundStatus.ne(9))
        .all(db)
        .await?;

    let refund_ids: Vec<i64> = refunds.iter().map(|r| r.id).collect();
    if refund_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let items = SaleRefundItem::find()
        .filter(refund_item_entity::Column::RefundId.is_in(refund_ids))
        .all(db)
        .await?;

    let mut map: HashMap<i64, Decimal> = HashMap::new();
    for item in items {
        if let Some(oi_id) = item.order_item_id {
            let qty = item.refund_qty.unwrap_or(Decimal::from(0));
            let entry = map.entry(oi_id).or_insert(Decimal::from(0));
            *entry += qty;
        }
    }
    Ok(map)
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



/// 退货单列表（支持 全部/我的/下属）
pub async fn get_list(db: &DbConn, query: &RefundListQuery, current_user_id: i64) -> Result<ResultPage<Vec<RefundListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("all");

    let owner_user_ids_opt: Option<Vec<i64>> = match list_type {
        "my" => Some(vec![current_user_id]),
        "subordinate" => {
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter()
                .filter_map(|r| r.data_scope)
                .min();

            match data_scope {
                Some(5) => Some(Vec::new()),
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
        RefundModel::select_in_page(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.refund_status,
            query.approval_status,
            query.customer_id,
            query.order_id,
            Some(current_user_id),
            query.start_date.clone(),
            query.end_date.clone(),
        ).await?
    } else {
        RefundModel::select_in_page_by_owner_user_ids(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.refund_status,
            query.approval_status,
            query.customer_id,
            query.order_id,
            query.start_date.clone(),
            query.end_date.clone(),
            owner_user_ids_opt,
        ).await?
    };

    // 批量查询客户名称
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

    // 批量查询订单号
    let order_ids: Vec<i64> = list.iter()
        .filter_map(|c| c.order_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let order_no_map: HashMap<i64, String> = if !order_ids.is_empty() {
        SaleOrder::find()
            .filter(order_entity::Column::Id.is_in(order_ids.clone()))
            .all(db)
            .await?
            .into_iter()
            .filter_map(|o| o.order_no.map(|n| (o.id, n)))
            .collect()
    } else {
        HashMap::new()
    };

    // 批量查询负责人名称
    let owner_user_ids: Vec<i64> = list.iter()
        .filter_map(|c| c.owner_user_id)
        .collect();

    let owner_name_map = crate::modules::system::service::admin_service::build_admin_name_map(db, owner_user_ids).await;

    let data: Vec<RefundListVO> = list.iter().map(|item| {
        let mut vo: RefundListVO = item.into();
        if let Some(cid) = vo.customer_id {
            if let Some(name) = customer_name_map.get(&cid) {
                vo.customer_name = Some(name.clone());
            }
        }
        if let Some(oid) = vo.order_id {
            if let Some(no) = order_no_map.get(&oid) {
                vo.order_no = Some(no.clone());
            }
        }
        if let Some(uid) = vo.owner_user_id {
            if let Some(name) = owner_name_map.get(&uid) {
                vo.owner_user_name = Some(name.clone());
            }
        }
        vo
    }).collect();

    Ok(ResultPage { items: data, total, current_page: page, page_size, total_pages: 0 })
}

/// 提交审批
/// 提交审批（接入审批引擎）
pub async fn submit_refund(db: &DbConn, refund_id: i64, operator_id: i64, operator_name: &str) -> Result<RefundDetailVO> {
    let refund = RefundModel::find_by_id(db, refund_id).await?
        .ok_or_else(|| Error::from("退货单不存在"))?;

    let approval_status = refund.approval_status.unwrap_or(0);
    if approval_status != 0 && approval_status != 4 {
        return Err(Error::from("仅草稿或已驳回状态可提交审批"));
    }

    // 已开票订单退货提示（不阻止提交，仅提示）
    let mut warning: Option<String> = None;
    if let Some(order_id) = refund.order_id {
        let invoiced = SaleInvoice::find()
            .filter(invoice_entity::Column::OrderId.eq(order_id))
            .filter(invoice_entity::Column::Status.eq(2)) // 已开票
            .filter(invoice_entity::Column::Deleted.eq(0))
            .one(db)
            .await?;
        if invoiced.is_some() {
            warning = Some("该订单已开票，退货后需进行红冲处理".to_string());
        }
    }

    // 接入审批引擎：提交退货审批流
    let submit_req = ApprovalSubmitRequest {
        flow_code: "refund_approval".to_string(),
        business_type: "refund".to_string(),
        business_id: refund_id,
        business_title: refund.title.clone(),
        submitter_id: operator_id,
        submitter_name: Some(operator_name.to_string()),
        extra_data: Some(serde_json::json!({
            "amount": refund.total_amount.unwrap_or(Decimal::from(0)),
        })),
    };
    let instance_id = ApprovalService::submit(db, &submit_req).await?;

    // 更新退货单状态为审批中，记录审批实例ID
    let txn = db.begin().await?;
    RefundModel::update_approval(&txn, refund_id, 2, Some(instance_id)).await?;
    RefundModel::update_status(&txn, refund_id, 2).await?; // 待审批
    txn.commit().await?;

    let mut vo = get_detail(db, refund_id).await?;
    vo.warning = warning;
    Ok(vo)
}

/// 审批通过（审批引擎处理，此函数由 controller 在审批引擎回调后调用）
pub async fn approve_refund(db: &DbConn, refund_id: i64, operator_id: i64, _reason: Option<String>) -> Result<RefundDetailVO> {
    let refund = RefundModel::find_by_id(db, refund_id).await?
        .ok_or_else(|| Error::from("退货单不存在"))?;

    let approval_status = refund.approval_status.unwrap_or(0);
    if approval_status != 1 && approval_status != 2 {
        return Err(Error::from("仅待审批或审批中状态可进行审批操作"));
    }

    let txn = db.begin().await?;
    RefundModel::update_approval(&txn, refund_id, 3, None).await?;
    RefundModel::update_status(&txn, refund_id, 3).await?; // 审批通过
    txn.commit().await?;

    let _ = operator_id;
    get_detail(db, refund_id).await
}

/// 审批驳回（审批引擎处理，此函数由 controller 在审批引擎回调后调用）
pub async fn reject_refund(db: &DbConn, refund_id: i64, operator_id: i64, _reason: Option<String>) -> Result<RefundDetailVO> {
    let refund = RefundModel::find_by_id(db, refund_id).await?
        .ok_or_else(|| Error::from("退货单不存在"))?;

    let approval_status = refund.approval_status.unwrap_or(0);
    if approval_status != 1 && approval_status != 2 {
        return Err(Error::from("仅待审批或审批中状态可进行驳回操作"));
    }

    let txn = db.begin().await?;
    RefundModel::update_approval(&txn, refund_id, 4, None).await?;
    RefundModel::update_status(&txn, refund_id, 8).await?; // 已驳回
    txn.commit().await?;

    let _ = operator_id;
    get_detail(db, refund_id).await
}

/// 仓库收货
pub async fn receive_refund(db: &DbConn, req: &RefundReceiveReq, operator_id: i64) -> Result<RefundDetailVO> {
    let refund = RefundModel::find_by_id(db, req.refund_id).await?
        .ok_or_else(|| Error::from("退货单不存在"))?;

    let refund_status = refund.refund_status.unwrap_or(0);
    if refund_status != 3 && refund_status != 4 {
        return Err(Error::from("仅审批通过或待收货状态可进行收货操作"));
    }

    let txn = db.begin().await?;
    // 更新退货物流单号
    RefundModel::update_logistics(&txn, req.refund_id, req.logistics_no.clone(), req.logistics_company.clone()).await?;
    // 更新状态为已收货(5)
    RefundModel::update_status(&txn, req.refund_id, 5).await?;
    txn.commit().await?;

    let _ = operator_id;
    get_detail(db, req.refund_id).await
}

/// 质检完成
pub async fn quality_check(db: &DbConn, req: &RefundQualityCheckReq, operator_id: i64) -> Result<RefundDetailVO> {
    let refund = RefundModel::find_by_id(db, req.refund_id).await?
        .ok_or_else(|| Error::from("退货单不存在"))?;

    let refund_status = refund.refund_status.unwrap_or(0);
    if refund_status != 5 && refund_status != 6 {
        return Err(Error::from("仅已收货或质检中状态可进行质检操作"));
    }

    if req.quality_check_result != 1 && req.quality_check_result != 2 {
        return Err(Error::from("质检结果只能为 1=合格 或 2=不合格"));
    }

    let txn = db.begin().await?;
    RefundModel::update_quality_check(&txn, req.refund_id, req.quality_check_result, req.quality_check_remark.clone()).await?;
    // 若合格，状态更新为已完成(7)；若不合格，状态更新为质检中(6)，等待处理
    let new_status = if req.quality_check_result == 1 { 7 } else { 6 };
    RefundModel::update_status(&txn, req.refund_id, new_status).await?;
    txn.commit().await?;

    let _ = operator_id;
    get_detail(db, req.refund_id).await
}

/// 取消退货单
pub async fn cancel_refund(db: &DbConn, refund_id: i64, operator_id: i64) -> Result<RefundDetailVO> {
    let refund = RefundModel::find_by_id(db, refund_id).await?
        .ok_or_else(|| Error::from("退货单不存在"))?;

    let refund_status = refund.refund_status.unwrap_or(0);
    // 仅草稿(1)、待审批(2)、已驳回(8)可取消
    if refund_status != 1 && refund_status != 2 && refund_status != 8 {
        return Err(Error::from("当前退货单状态不允许取消"));
    }

    let txn = db.begin().await?;
    RefundModel::update_status(&txn, refund_id, 9).await?; // 已取消
    txn.commit().await?;

    let _ = operator_id;
    get_detail(db, refund_id).await
}

/// 发起退款
pub async fn create_payment(db: &DbConn, req: &RefundPaymentRequest, operator_id: i64) -> Result<i64> {
    let refund = RefundModel::find_by_id(db, req.refund_id).await?
        .ok_or_else(|| Error::from("退货单不存在"))?;

    let refund_status = refund.refund_status.unwrap_or(0);
    // 仅审批通过(3)/已完成(7)/已收货(5)状态可发起退款
    if refund_status != 3 && refund_status != 5 && refund_status != 7 {
        return Err(Error::from("当前退货单状态不允许发起退款"));
    }

    let payment_amount = req.payment_amount.unwrap_or(Decimal::from(0));
    if payment_amount <= Decimal::from(0) {
        return Err(Error::from("退款金额必须大于0"));
    }

    let refund_amount = refund.refund_amount.unwrap_or(Decimal::from(0));
    let refunded_amount = refund.refunded_amount.unwrap_or(Decimal::from(0));
    if refunded_amount + payment_amount > refund_amount {
        return Err(Error::from(format!(
            "退款金额超出应退金额：已退 {} + 本次 {} > 应退 {}",
            refunded_amount, payment_amount, refund_amount
        )));
    }

    // 生成退款单号 RFP{YYYYMMDD}{4位序号}
    let date_prefix = format!("RFP{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = RefundPaymentModel::get_max_payment_no_today(db, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let payment_no = format!("{}{:04}", date_prefix, seq);

    let txn = db.begin().await?;
    let payment_id = RefundPaymentModel::insert(&txn, req.refund_id, payment_no, req, operator_id).await?;

    // 累加已退款金额
    let new_refunded = refunded_amount + payment_amount;
    RefundModel::update_payment_amount(&txn, req.refund_id, new_refunded).await?;

    // 若已退款=应退款，更新退货状态为已完成(7)
    if new_refunded >= refund_amount {
        RefundModel::update_status(&txn, req.refund_id, 7).await?;
    }

    // ===== 对冲逻辑开始 =====
    let now = chrono::Local::now().naive_local().to_owned();

    // 1. 冲减回款已核销额（applied_amount）
    if let Some(order_id) = refund.order_id {
        let mut remaining_deduct = payment_amount;

        // 查询关联订单的回款记录（有已核销额的），按 id 升序依次冲减
        let payments = SalePayment::find()
            .filter(payment_entity::Column::OrderId.eq(order_id))
            .filter(payment_entity::Column::Deleted.eq(0))
            .filter(payment_entity::Column::AppliedAmount.gt(Decimal::from(0)))
            .order_by_asc(payment_entity::Column::Id)
            .all(&txn)
            .await
            .map_err(|e| Error::from(format!("查询回款记录失败: {}", e)))?;

        for pm in &payments {
            if remaining_deduct <= Decimal::from(0) {
                break;
            }

            let old_applied = pm.applied_amount.unwrap_or(Decimal::from(0));
            let old_unapplied = pm.unapplied_amount.unwrap_or(Decimal::from(0));
            let deduct = if old_applied >= remaining_deduct {
                remaining_deduct
            } else {
                old_applied
            };
            let new_applied = old_applied - deduct;
            let new_unapplied = old_unapplied + deduct;
            // applied_amount 减到 0 时，回款状态回到已确认(2)
            let new_status = if new_applied <= Decimal::from(0) {
                2
            } else {
                pm.status.unwrap_or(2)
            };

            SalePayment::update_many()
                .set(payment_entity::ActiveModel {
                    applied_amount: Set(Some(new_applied)),
                    unapplied_amount: Set(Some(new_unapplied)),
                    status: Set(Some(new_status)),
                    update_time: Set(Some(now)),
                    ..Default::default()
                })
                .filter(payment_entity::Column::Id.eq(pm.id))
                .filter(payment_entity::Column::Deleted.eq(0))
                .exec(&txn)
                .await
                .map_err(|e| Error::from(format!("更新回款已核销额失败: {}", e)))?;

            remaining_deduct -= deduct;
        }

        // 2. 更新订单支付状态
        let order = SaleOrder::find_by_id(order_id)
            .filter(order_entity::Column::Deleted.eq(0))
            .one(&txn)
            .await
            .map_err(|e| Error::from(format!("查询订单失败: {}", e)))?;

        if let Some(o) = order {
            let old_paid = o.paid_amount.unwrap_or(Decimal::from(0));
            let total = o.total_amount.unwrap_or(Decimal::from(0));
            let mut new_paid = old_paid - payment_amount;
            if new_paid < Decimal::from(0) {
                new_paid = Decimal::from(0);
            }
            let new_unpaid = if new_paid >= total {
                Decimal::from(0)
            } else {
                total - new_paid
            };
            // pay_status: 1=未支付, 2=部分支付, 3=已支付, 4=已退款
            let new_pay_status = if new_paid <= Decimal::from(0) {
                4
            } else if new_paid < total {
                2
            } else {
                3
            };

            SaleOrder::update_many()
                .set(order_entity::ActiveModel {
                    paid_amount: Set(Some(new_paid)),
                    unpaid_amount: Set(Some(new_unpaid)),
                    pay_status: Set(Some(new_pay_status)),
                    update_time: Set(Some(now)),
                    ..Default::default()
                })
                .filter(order_entity::Column::Id.eq(order_id))
                .filter(order_entity::Column::Deleted.eq(0))
                .exec(&txn)
                .await
                .map_err(|e| Error::from(format!("更新订单支付状态失败: {}", e)))?;
        }
    }
    // ===== 对冲逻辑结束 =====

    txn.commit().await?;

    Ok(payment_id)
}
