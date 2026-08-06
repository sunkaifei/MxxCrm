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
use crate::modules::purchase::model::purchase_return::{
    generate_return_no, PurchaseReturnDetailVO, PurchaseReturnItemModel, PurchaseReturnListQuery,
    PurchaseReturnListVO, PurchaseReturnModel, PurchaseReturnSaveDTO, PurchaseReturnSaveRequest,
    ReturnItemVO,
};
use sea_orm::DbConn;
use sea_orm::TransactionTrait;

/// 创建退货单
pub async fn insert(db: &DbConn, req: &PurchaseReturnSaveRequest, operator: i64) -> Result<i64> {
    let return_no = generate_return_no(1);

    let dto = PurchaseReturnSaveDTO {
        id: None,
        return_no: Some(return_no),
        receipt_id: req.receipt_id,
        po_id: req.po_id,
        supplier_id: req.supplier_id,
        return_date: req.return_date,
        total_amount: req.total_amount,
        reason: req.reason.clone(),
        status: Some(0), // 草稿
        remark: req.remark.clone(),
        created_by: Some(operator),
        updated_by: Some(operator),
    };

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    let return_id = PurchaseReturnModel::insert(&txn, &dto)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if !req.items.is_empty() {
        PurchaseReturnItemModel::batch_insert(&txn, return_id, &req.items)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
    }

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(return_id)
}

/// 更新退货单
pub async fn update(db: &DbConn, req: &PurchaseReturnSaveRequest, operator: i64) -> Result<i64> {
    let id = req.id.ok_or_else(|| Error::from("ID不能为空"))?;

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    // 先删除旧明细
    PurchaseReturnItemModel::delete_by_return_id(&txn, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let dto = PurchaseReturnSaveDTO {
        id: Some(id),
        return_no: None,
        receipt_id: req.receipt_id,
        po_id: req.po_id,
        supplier_id: req.supplier_id,
        return_date: req.return_date,
        total_amount: req.total_amount,
        reason: req.reason.clone(),
        status: None,
        remark: req.remark.clone(),
        created_by: None,
        updated_by: Some(operator),
    };

    PurchaseReturnModel::update(&txn, &dto)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if !req.items.is_empty() {
        PurchaseReturnItemModel::batch_insert(&txn, id, &req.items)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
    }

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 批量删除
pub async fn batch_delete(db: &DbConn, ids: &[i64]) -> Result<i64> {
    PurchaseReturnModel::batch_delete(db, ids)
        .await
        .map_err(|e| Error::from(e.to_string()))
        .map(|v| v as i64)
}

/// 获取详情
pub async fn get_info(db: &DbConn, id: i64) -> Result<PurchaseReturnDetailVO> {
    let model = PurchaseReturnModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("退货单不存在"))?;

    let mut vo: PurchaseReturnDetailVO = model.into();

    let items = PurchaseReturnItemModel::find_by_return_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    vo.items = items.into_iter().map(|m| ReturnItemVO::from(m)).collect();
    Ok(vo)
}

/// 获取列表
pub async fn get_list(db: &DbConn, query: &PurchaseReturnListQuery) -> Result<(Vec<PurchaseReturnListVO>, u64)> {
    let (list, total) = PurchaseReturnModel::find_list(db, query)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let vo_list: Vec<PurchaseReturnListVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((vo_list, total))
}