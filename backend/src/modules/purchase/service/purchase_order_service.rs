//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::modules::purchase::model::purchase_order::{PurchaseOrderDetailVO, PurchaseOrderListQuery, PurchaseOrderListVO, PurchaseOrderSaveDTO, PurchaseOrderSaveRequest, PurchaseOrderUpdateRequest};
use crate::modules::purchase::model::purchase_order::PurchaseOrderModel;
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &PurchaseOrderSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: PurchaseOrderSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    dto.updated_by = Some(created_by);
    let result = PurchaseOrderModel::insert(db, &dto).await?;
    Ok(result)
}

pub async fn batch_delete(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let result = PurchaseOrderModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &PurchaseOrderUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: PurchaseOrderSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = PurchaseOrderModel::update_by_id(db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn get_detail(db: &DbConn, id: i64) -> Result<PurchaseOrderDetailVO> {
    let result = PurchaseOrderModel::find_by_id(db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(crate::core::errors::error::Error::from("閲囪喘鍗曚笉瀛樺湪".to_string())),
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
        query.order_type.clone(),
    ).await?;
    
    let total = PurchaseOrderModel::select_count(
        db,
        query.keywords.clone(),
        query.status.clone(),
        query.supplier_id,
        query.order_type.clone(),
    ).await?;
    
    let list: Vec<PurchaseOrderListVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((list, total, total_pages))
}