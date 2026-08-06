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
use crate::modules::inventory::model::inbound::{InboundItemRequest, InboundSaveRequest};
use crate::modules::purchase::model::purchase_order::PurchaseOrderModel;
use crate::modules::purchase::model::purchase_order_item::PurchaseOrderItemModel;
use crate::modules::purchase::model::purchase_receipt::{
    generate_receipt_no, ReceiptDetailVO, ReceiptItemModel, ReceiptItemVO, ReceiptListQuery, ReceiptListVO,
    ReceiptModel, ReceiptSaveDTO, ReceiptSaveRequest,
};
use sea_orm::{DbConn, TransactionTrait};

/// 创建收货单
pub async fn insert(db: &DbConn, req: &ReceiptSaveRequest, operator: i64) -> Result<i64> {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let _prefix = format!("SH{}", today);
    let receipt_no = generate_receipt_no(1);

    let dto = ReceiptSaveDTO {
        id: None,
        receipt_no: Some(receipt_no),
        po_id: req.po_id,
        po_no: req.po_no.clone(),
        supplier_id: req.supplier_id,
        warehouse_id: req.warehouse_id,
        status: Some(0), // 草稿
        total_quantity: req.total_quantity,
        remark: req.remark.clone(),
        inbound_id: None,
        created_by: Some(operator),
        updated_by: Some(operator),
    };

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    let receipt_id = ReceiptModel::insert(&txn, &dto)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if !req.items.is_empty() {
        ReceiptItemModel::batch_insert(&txn, receipt_id, &req.items)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
    }

    // 累加采购订单明细的已收数量，并更新PO状态
    if let Some(po_id) = req.po_id {
        for item in &req.items {
            if let Some(po_item_id) = item.po_item_id {
                if let Some(quantity) = item.quantity {
                    let po_item = PurchaseOrderItemModel::find_by_id(&txn, po_item_id)
                        .await
                        .map_err(|e| Error::from(e.to_string()))?;
                    if let Some(po_item) = po_item {
                        let current = po_item.received_quantity.unwrap_or_default();
                        let new_received = current + quantity;
                        PurchaseOrderItemModel::update_received_quantity(&txn, po_item_id, new_received)
                            .await
                            .map_err(|e| Error::from(e.to_string()))?;
                    }
                }
            }
        }

        // 根据已收数量更新PO状态为 partial_received 或 received
        let po_items = PurchaseOrderItemModel::find_by_po_id(&txn, po_id)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        if !po_items.is_empty() {
            let all_fully_received = po_items.iter().all(|it| {
                let ordered = it.quantity.unwrap_or_default();
                let received = it.received_quantity.unwrap_or_default();
                received >= ordered
            });
            let any_received = po_items.iter().any(|it| {
                it.received_quantity.unwrap_or_default() > rust_decimal::Decimal::ZERO
            });

            let new_status = if all_fully_received {
                "received"
            } else if any_received {
                "partial_received"
            } else {
                ""
            };

            if !new_status.is_empty() {
                PurchaseOrderModel::update_status(&txn, po_id, new_status, operator)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
            }
        }
    }

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(receipt_id)
}

/// 批量删除
pub async fn batch_delete(db: &DbConn, ids: &[i64]) -> Result<i64> {
    ReceiptModel::batch_delete(db, ids)
        .await
        .map_err(|e| Error::from(e.to_string()))
        .map(|v| v as i64)
}

/// 获取详情
pub async fn get_info(db: &DbConn, id: i64) -> Result<ReceiptDetailVO> {
    let model = ReceiptModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("收货单不存在"))?;

    let mut vo: ReceiptDetailVO = model.into();

    let items = ReceiptItemModel::find_by_receipt_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    vo.items = items.into_iter().map(|m| ReceiptItemVO::from(m)).collect();
    Ok(vo)
}

/// 获取列表
pub async fn get_list(db: &DbConn, query: &ReceiptListQuery) -> Result<(Vec<ReceiptListVO>, u64)> {
    let (list, total) = ReceiptModel::find_list(db, query)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let vo_list: Vec<ReceiptListVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((vo_list, total))
}

/// 转为入库单
pub async fn to_inbound(db: &DbConn, receipt_id: i64, warehouse_id: i64, operator: i64) -> Result<i64> {
    let receipt = ReceiptModel::find_by_id(db, receipt_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("收货单不存在"))?;

    let status = receipt.status.unwrap_or(0);
    if status != 0 {
        return Err(Error::from("仅草稿状态的收货单可转为入库单"));
    }

    let items = ReceiptItemModel::find_by_receipt_id(db, receipt_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if items.is_empty() {
        return Err(Error::from("收货单明细为空，无法转为入库单"));
    }

    // 构建入库单请求
    let inbound_items: Vec<InboundItemRequest> = items
        .iter()
        .map(|item| InboundItemRequest {
            product_id: item.product_id.unwrap_or_default(),
            product_sku: None,
            quantity: item.quantity.unwrap_or_default(),
            unit_price: None,
            amount: None,
            batch_no: None,
            remark: item.remark.clone(),
        })
        .collect();

    let inbound_req = InboundSaveRequest {
        inbound_type: "purchase_receipt".to_string(),
        warehouse_id,
        source_order_id: Some(receipt_id),
        source_order_no: receipt.receipt_no.clone(),
        total_quantity: receipt.total_quantity,
        total_amount: None,
        remark: receipt.remark.clone(),
        items: inbound_items,
    };

    // 调用入库服务创建入库单
    let inbound_id = crate::modules::inventory::service::inbound_service::create(db, &inbound_req, operator)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 将入库单状态从草稿(0)更新为待审核(1)，以便自动审核
    crate::modules::inventory::model::inbound::update_status(db, inbound_id, 1, operator)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 自动审核入库单以增加库存
    crate::modules::inventory::service::inbound_service::audit(db, inbound_id, operator)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 更新收货单状态和入库单ID
    ReceiptModel::update_status(db, receipt_id, 1, operator)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    ReceiptModel::update_inbound_id(db, receipt_id, inbound_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 当PO全部收齐时，将PO状态改为 completed
    if let Some(po_id) = receipt.po_id {
        let po_items = PurchaseOrderItemModel::find_by_po_id(db, po_id)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        if !po_items.is_empty() {
            let all_fully_received = po_items.iter().all(|it| {
                let ordered = it.quantity.unwrap_or_default();
                let received = it.received_quantity.unwrap_or_default();
                received >= ordered
            });

            if all_fully_received {
                PurchaseOrderModel::update_status(db, po_id, "completed", operator)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
            }
        }
    }

    Ok(inbound_id)
}