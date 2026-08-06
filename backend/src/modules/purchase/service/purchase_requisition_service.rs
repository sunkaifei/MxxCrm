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
use crate::modules::purchase::model::purchase_approval_record::{ApprovalRecordDTO, ApprovalRecordModel};
use crate::modules::purchase::model::purchase_order::PurchaseOrderModel;
use crate::modules::purchase::model::purchase_order::PurchaseOrderSaveDTO;
use crate::modules::purchase::model::purchase_order_item::PoItemDTO;
use crate::modules::purchase::model::purchase_requisition::{
    requisition_status, RequisitionItemModel, RequisitionListQuery, RequisitionListVO,
    RequisitionModel, RequisitionSaveDTO, RequisitionSaveRequest, RequisitionDetailVO, RequisitionItemVO,
};
use sea_orm::DbConn;
use sea_orm::TransactionTrait;

/// 创建采购申请单
pub async fn insert(db: &DbConn, req: &RequisitionSaveRequest, operator: i64) -> Result<i64> {
    // 生成单号：查询当天最大流水号并+1
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let prefix = format!("PR{}", today);
    let max_no = RequisitionModel::find_max_pr_no_today(db, &prefix).await
        .map_err(|e| Error::from(e.to_string()))?;

    let seq = match max_no {
        Some(no) => {
            // 从 PR{yyyyMMdd}{0001} 中提取流水号
            if no.len() >= 14 {
                no[12..].parse::<i32>().unwrap_or(0) + 1
            } else {
                1
            }
        }
        None => 1,
    };
    let pr_no = format!("{}{:04}", prefix, seq);

    let dto = RequisitionSaveDTO {
        id: None,
        pr_no: Some(pr_no),
        pr_type: req.pr_type.clone(),
        title: req.title.clone(),
        department_id: req.department_id,
        requester_id: req.requester_id,
        expected_date: req.expected_date,
        urgency: req.urgency.clone(),
        total_amount: req.total_amount,
        currency: req.currency.clone(),
        status: Some(requisition_status::DRAFT),
        source_type: None,
        source_id: None,
        source_no: None,
        reason: req.reason.clone(),
        remark: req.remark.clone(),
        created_by: Some(operator),
        updated_by: Some(operator),
    };

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    let pr_id = RequisitionModel::insert(&txn, &dto)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if !req.items.is_empty() {
        RequisitionItemModel::batch_insert(&txn, pr_id, &req.items)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
    }

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(pr_id)
}

/// 更新采购申请单
pub async fn update(db: &DbConn, req: &RequisitionSaveRequest, operator: i64) -> Result<()> {
    let id = req.id.ok_or_else(|| Error::from("ID不能为空"))?;

    let existing = RequisitionModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("采购申请单不存在"))?;

    if existing.status.unwrap_or(0) != requisition_status::DRAFT {
        return Err(Error::from("仅草稿状态的申请单可编辑"));
    }

    let update_dto = crate::modules::purchase::model::purchase_requisition::RequisitionUpdateDTO {
        id,
        pr_type: req.pr_type.clone(),
        title: req.title.clone(),
        department_id: req.department_id,
        requester_id: req.requester_id,
        expected_date: req.expected_date,
        urgency: req.urgency.clone(),
        total_amount: req.total_amount,
        currency: req.currency.clone(),
        reason: req.reason.clone(),
        remark: req.remark.clone(),
        items: req.items.clone(),
        updated_by: Some(operator),
    };

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    RequisitionModel::update(&txn, &update_dto)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 删除旧明细，插入新明细
    RequisitionItemModel::delete_by_pr_id(&txn, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if !req.items.is_empty() {
        RequisitionItemModel::batch_insert(&txn, id, &req.items)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
    }

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 提交审批
pub async fn submit_approval(db: &DbConn, pr_id: i64, operator: i64) -> Result<i64> {
    let existing = RequisitionModel::find_by_id(db, pr_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("采购申请单不存在"))?;

    let status = existing.status.unwrap_or(0);
    if status != requisition_status::DRAFT {
        return Err(Error::from("仅草稿状态的申请单可提交审批"));
    }

    // 更新状态为待审批
    RequisitionModel::update_status(db, pr_id, requisition_status::PENDING, operator)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(pr_id)
}

/// 审批通过
pub async fn approve(db: &DbConn, pr_id: i64, approver_id: i64, comment: Option<String>) -> Result<i64> {
    let existing = RequisitionModel::find_by_id(db, pr_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("采购申请单不存在"))?;

    let status = existing.status.unwrap_or(0);
    if status != requisition_status::PENDING && status != requisition_status::APPROVING {
        return Err(Error::from("当前状态不允许审批"));
    }

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    // 记录审批记录
    let record = ApprovalRecordDTO {
        biz_type: Some("requisition".to_string()),
        biz_id: Some(pr_id),
        approval_level: Some(1),
        approver_id: Some(approver_id),
        action: Some("approved".to_string()),
        comment,
    };
    ApprovalRecordModel::insert(&txn, &record, approver_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 更新状态为已通过
    RequisitionModel::update_status(&txn, pr_id, requisition_status::APPROVED, approver_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(pr_id)
}

/// 审批驳回
pub async fn reject(db: &DbConn, pr_id: i64, approver_id: i64, comment: Option<String>) -> Result<i64> {
    let existing = RequisitionModel::find_by_id(db, pr_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("采购申请单不存在"))?;

    let status = existing.status.unwrap_or(0);
    if status != requisition_status::PENDING && status != requisition_status::APPROVING {
        return Err(Error::from("当前状态不允许驳回"));
    }

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    // 记录驳回记录
    let record = ApprovalRecordDTO {
        biz_type: Some("requisition".to_string()),
        biz_id: Some(pr_id),
        approval_level: Some(1),
        approver_id: Some(approver_id),
        action: Some("rejected".to_string()),
        comment,
    };
    ApprovalRecordModel::insert(&txn, &record, approver_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 更新状态为已驳回
    RequisitionModel::update_status(&txn, pr_id, requisition_status::REJECTED, approver_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(pr_id)
}

/// 撤回审批
pub async fn withdraw(db: &DbConn, pr_id: i64, operator: i64) -> Result<i64> {
    let existing = RequisitionModel::find_by_id(db, pr_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("采购申请单不存在"))?;

    let status = existing.status.unwrap_or(0);
    if status != requisition_status::PENDING {
        return Err(Error::from("仅待审批状态的申请单可撤回"));
    }

    RequisitionModel::update_status(db, pr_id, requisition_status::DRAFT, operator)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(pr_id)
}

/// 批量删除
pub async fn batch_delete(db: &DbConn, ids: &[i64]) -> Result<i64> {
    RequisitionModel::batch_delete(db, ids)
        .await
        .map_err(|e| Error::from(e.to_string()))
        .map(|v| v as i64)
}

/// 获取详情
pub async fn get_info(db: &DbConn, id: i64) -> Result<RequisitionDetailVO> {
    let model = RequisitionModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("采购申请单不存在"))?;

    let mut vo: RequisitionDetailVO = model.into();

    // 加载明细
    let items = RequisitionItemModel::find_by_pr_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    vo.items = items.into_iter().map(|m| RequisitionItemVO::from(m)).collect();

    // 加载审批记录
    let records = ApprovalRecordModel::find_by_biz(db, "requisition", id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    vo.approval_records = records.into_iter()
        .map(|m| crate::modules::purchase::model::purchase_approval_record::ApprovalRecordVO::from(m))
        .collect();

    Ok(vo)
}

/// 获取列表
pub async fn get_list(db: &DbConn, query: &RequisitionListQuery) -> Result<(Vec<RequisitionListVO>, u64)> {
    let (list, total) = RequisitionModel::find_list(db, query)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let vo_list: Vec<RequisitionListVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((vo_list, total))
}

/// 转采购单
pub async fn convert_to_po(db: &DbConn, pr_ids: Vec<i64>, supplier_id: i64, operator: i64) -> Result<i64> {
    if pr_ids.is_empty() {
        return Err(Error::from("请选择采购申请单"));
    }

    // 获取第一个申请单作为主申请单
    let pr_id = pr_ids[0];
    let model = RequisitionModel::find_by_id(db, pr_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("采购申请单不存在"))?;

    let status = model.status.unwrap_or(0);
    if status != requisition_status::APPROVED {
        return Err(Error::from("仅已审批通过的申请单可转为采购单"));
    }

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    // 更新申请单状态为已转采购单
    for pid in &pr_ids {
        RequisitionModel::update_status(&txn, *pid, requisition_status::CONVERTED, operator)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
    }

    // 收集所有申请单的明细并合并到采购订单
    let mut po_items: Vec<PoItemDTO> = Vec::new();
    for pid in &pr_ids {
        let pr_items = RequisitionItemModel::find_by_pr_id(&txn, *pid)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        for item in pr_items {
            po_items.push(PoItemDTO {
                po_id: None,
                pr_item_id: Some(item.id),
                product_id: item.product_id,
                product_name: item.product_name,
                product_sku: item.product_sku,
                spec: item.spec,
                unit: item.unit,
                quantity: item.quantity,
                received_quantity: None,
                unit_price: item.estimated_price,
                amount: item.estimated_amount,
                tax_rate: None,
                tax_amount: None,
                expected_date: model.expected_date,
                remark: item.remark,
            });
        }
    }

    // 创建采购单
    let po_dto = PurchaseOrderSaveDTO {
        id: None,
        purchase_no: None,
        supplier_id: Some(supplier_id),
        purchase_date: Some(chrono::Local::now().date_naive()),
        expected_date: model.expected_date,
        amount: model.total_amount,
        currency: model.currency.and_then(|s| {
            s.parse::<i32>().ok().and_then(|v| {
                crate::core::r#enum::currency_code_enum::CurrencyCode::from_i32(v)
            })
        }),
        status: Some(crate::core::r#enum::purchase_status_enum::PurchaseStatus::Draft),
        payment_status: None,
        notes: Some(format!("由采购申请单[{}]生成", model.pr_no.clone().unwrap_or_default())),
        created_by: Some(operator),
        updated_by: Some(operator),
        pr_id: Some(pr_id),
        pr_no: model.pr_no.clone(),
        department_id: model.department_id,
        buyer_id: None,
        total_quantity: None,
        tax_total: None,
        discount_amount: None,
        freight_amount: None,
        delivery_address: None,
        delivery_terms: None,
        payment_terms: None,
        items: po_items,
    };

    let po_id = PurchaseOrderModel::insert(&txn, &po_dto)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 插入采购订单明细
    if !po_dto.items.is_empty() {
        crate::modules::purchase::model::purchase_order_item::PurchaseOrderItemModel::batch_insert(&txn, po_id, &po_dto.items)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
    }

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(po_id)
}

/// 获取我的审批列表
pub async fn get_my_approval_list(db: &DbConn, user_id: i64, page_num: i64, page_size: i64) -> Result<(Vec<RequisitionListVO>, u64)> {
    let (list, total) = RequisitionModel::find_approval_pending_list(db, user_id, page_num as u64, page_size as u64)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let vo_list: Vec<RequisitionListVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((vo_list, total))
}