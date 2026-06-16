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
use crate::modules::sale::model::order::{OrderDetailVO, OrderListQuery, OrderListVO, OrderSaveDTO, OrderSaveRequest, OrderUpdateRequest};
use crate::modules::sale::model::order::OrderModel;
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &OrderSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: OrderSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    dto.updated_by = Some(created_by);
    let result = OrderModel::insert(db, &dto).await?;
    Ok(result)
}

pub async fn batch_delete(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let result = OrderModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &OrderUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: OrderSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = OrderModel::update_by_id(db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn get_detail(db: &DbConn, id: i64) -> Result<OrderDetailVO> {
    let result = OrderModel::find_by_id(db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(crate::core::errors::error::Error::from("订单不存在")),
    }
}

pub async fn get_list(db: &DbConn, query: &OrderListQuery) -> Result<(Vec<OrderListVO>, i64, i64)> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);
    let (list, total_pages) = OrderModel::select_in_page(
        db,
        page_num,
        page_size,
        query.keywords.clone(),
        query.status.clone(),
        query.customer_id,
        query.order_type.clone(),
    ).await?;
    
    let total = OrderModel::select_count(
        db,
        query.keywords.clone(),
        query.status.clone(),
        query.customer_id,
        query.order_type.clone(),
    ).await?;
    
    let list: Vec<OrderListVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((list, total, total_pages))
}