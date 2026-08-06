//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{DbConn, TransactionTrait};

use crate::core::errors::error::{Error, Result};
use crate::modules::product::model::unit_conversion::{
    batch_delete, find_by_id, find_by_product, insert, select_page, update_by_id,
    UnitConversionListQuery, UnitConversionListVO, UnitConversionSaveRequest,
};

/// 保存（新增）
pub async fn save(db: &DbConn, req: &UnitConversionSaveRequest, created_by: i64) -> Result<i64> {
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let id = insert(&txn, req, created_by)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    txn.commit()
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 更新
pub async fn update(
    db: &DbConn,
    id: i64,
    req: &UnitConversionSaveRequest,
    updated_by: i64,
) -> Result<i64> {
    let existing = find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("单位换算记录不存在".to_string()))?;
    let _ = existing;

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    update_by_id(&txn, id, req, updated_by)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    txn.commit()
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 批量删除
pub async fn batch_delete_ids(db: &DbConn, ids: &[i64]) -> Result<i64> {
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let result = batch_delete(&txn, ids)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    txn.commit()
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 列表查询
pub async fn get_list(db: &DbConn, query: &UnitConversionListQuery) -> Result<UnitConversionListVO> {
    let (list, total) = select_page(db, query)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(UnitConversionListVO { list, total })
}

/// 按产品查询
pub async fn list_by_product(
    db: &DbConn,
    product_id: i64,
) -> Result<Vec<crate::modules::product::entity::unit_conversion::Model>> {
    find_by_product(db, product_id)
        .await
        .map_err(|e| Error::from(e.to_string()))
}
