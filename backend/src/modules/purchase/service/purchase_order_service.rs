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
use crate::core::r#enum::purchase_status_enum::PurchaseStatus;
use crate::modules::purchase::model::purchase_order::{
    PurchaseOrderDetailVO, PurchaseOrderListQuery, PurchaseOrderListVO, PurchaseOrderModel, PurchaseOrderSaveDTO, PurchaseOrderSaveRequest, PurchaseOrderUpdateRequest,
};
use crate::modules::purchase::model::purchase_order_item::{PurchaseOrderItemModel, PoItemVO};
use sea_orm::DbConn;
use sea_orm::TransactionTrait;

pub async fn insert(db: &DbConn, form_data: &PurchaseOrderSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: PurchaseOrderSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    dto.updated_by = Some(created_by);

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    let po_id = PurchaseOrderModel::insert(&txn, &dto)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if !dto.items.is_empty() {
        PurchaseOrderItemModel::batch_insert(&txn, po_id, &dto.items)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
    }

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(po_id)
}

pub async fn batch_delete(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let result = PurchaseOrderModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &PurchaseOrderUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = form_data.id.ok_or_else(|| Error::from("采购单ID不能为空"))?;

    let existing = PurchaseOrderModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("采购单不存在"))?;

    if existing.status != Some(PurchaseStatus::Draft) {
        return Err(Error::from("仅草稿状态的采购单可编辑"));
    }

    let mut dto: PurchaseOrderSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    PurchaseOrderModel::update_by_id(&txn, &form_data.id, &dto)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 删除旧明细，插入新明细
    PurchaseOrderItemModel::delete_by_po_id(&txn, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if !dto.items.is_empty() {
        PurchaseOrderItemModel::batch_insert(&txn, id, &dto.items)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
    }

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

pub async fn get_detail(db: &DbConn, id: i64) -> Result<PurchaseOrderDetailVO> {
    let result = PurchaseOrderModel::find_by_id(db, id).await?;
    match result {
        Some(item) => {
            let mut vo: PurchaseOrderDetailVO = item.into();
            // 加载明细
            let items = PurchaseOrderItemModel::find_by_po_id(db, id)
                .await
                .map_err(|e| Error::from(e.to_string()))?;
            vo.items = items.into_iter().map(|m| PoItemVO::from(m)).collect();
            Ok(vo)
        }
        None => Err(Error::from("采购单不存在".to_string())),
    }
}

pub async fn get_list(db: &DbConn, query: &PurchaseOrderListQuery) -> Result<(Vec<PurchaseOrderListVO>, i64, i64)> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);
    let (list, total_pages) = PurchaseOrderModel::select_in_page(
        db,
        page_num,
        page_size,
        query.keywords.clone(),
        query.status.clone(),
        query.supplier_id,
        query.brand_id,
    ).await?;
    
    let total = PurchaseOrderModel::select_count(
        db,
        query.keywords.clone(),
        query.status.clone(),
        query.supplier_id,
        query.brand_id,
    ).await?;
    
    let list: Vec<PurchaseOrderListVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((list, total, total_pages))
}

/// 审核采购单：草稿→待审核，待审核→已审核
pub async fn audit_po(db: &DbConn, po_id: i64, operator: i64) -> Result<()> {
    let existing = PurchaseOrderModel::find_by_id(db, po_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("采购单不存在"))?;

    let status = existing.status.unwrap_or(PurchaseStatus::Draft);
    let new_status = match status {
        PurchaseStatus::Draft => "pending_audit",
        PurchaseStatus::PendingAudit => "audited",
        _ => return Err(Error::from("当前状态不允许审核操作")),
    };

    PurchaseOrderModel::update_status(db, po_id, new_status, operator)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(())
}

/// 关闭采购单：将状态改为已取消
pub async fn close_po(db: &DbConn, po_id: i64, operator: i64) -> Result<()> {
    let existing = PurchaseOrderModel::find_by_id(db, po_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("采购单不存在"))?;

    let status = existing.status.unwrap_or(PurchaseStatus::Draft);
    if status == PurchaseStatus::Cancelled {
        return Err(Error::from("采购单已关闭"));
    }

    PurchaseOrderModel::update_status(db, po_id, "cancelled", operator)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(())
}

/// 驳回采购单：将状态改为已驳回
pub async fn reject_po(db: &DbConn, po_id: i64, operator: i64, _comment: Option<String>) -> Result<()> {
    let existing = PurchaseOrderModel::find_by_id(db, po_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("采购单不存在"))?;

    let status = existing.status.unwrap_or(PurchaseStatus::Draft);
    if status != PurchaseStatus::PendingAudit {
        return Err(Error::from("仅待审核状态的采购单可驳回"));
    }

    PurchaseOrderModel::update_status(db, po_id, "rejected", operator)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(())
}