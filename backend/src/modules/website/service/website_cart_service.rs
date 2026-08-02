//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::{DbConn, DbErr, TransactionTrait};
use crate::core::errors::error::{Error, Result};
use crate::modules::website::entity::website_cart;
use crate::modules::website::model::website_cart::{
    CartAddRequest, CartBatchDeleteRequest, CartSummaryVO, CartUpdateRequest, CartVO, WebsiteCartModel,
};

/// 添加购物车
pub async fn add(db: &DbConn, user_id: i64, req: CartAddRequest) -> Result<i64> {
    if req.quantity <= 0 {
        return Err(Error::from("数量必须大于0"));
    }
    let req_clone = req.clone();
    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            WebsiteCartModel::upsert(txn, user_id, &req_clone).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(0)
}

/// 获取购物车列表
pub async fn list(db: &DbConn, user_id: i64) -> Result<CartSummaryVO> {
    let items = WebsiteCartModel::find_by_user(db, user_id).await?;
    let total_count = items.len() as i64;
    let selected_items: Vec<&website_cart::Model> = items.iter().filter(|i| i.selected.unwrap_or(0) == 1).collect();
    let selected_count = selected_items.len() as i64;
    let selected_amount: sea_orm::prelude::Decimal = selected_items
        .iter()
        .map(|i| i.price * sea_orm::prelude::Decimal::from(i.quantity))
        .sum();
    let items_vo: Vec<CartVO> = items.into_iter().map(|m| m.into()).collect();
    Ok(CartSummaryVO {
        items: items_vo,
        total_count,
        selected_count,
        selected_amount,
    })
}

/// 更新购物车项
pub async fn update(db: &DbConn, user_id: i64, id: i64, req: CartUpdateRequest) -> Result<i64> {
    if let Some(q) = req.quantity {
        if q <= 0 {
            return Err(Error::from("数量必须大于0"));
        }
    }
    let req_clone = req.clone();
    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            WebsiteCartModel::update(txn, id, user_id, &req_clone).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 删除购物车项
pub async fn delete(db: &DbConn, user_id: i64, id: i64) -> Result<i64> {
    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            WebsiteCartModel::delete(txn, id, user_id).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 批量删除购物车
pub async fn batch_delete(db: &DbConn, user_id: i64, req: CartBatchDeleteRequest) -> Result<i64> {
    if req.ids.is_empty() {
        return Err(Error::from("请选择要删除的项"));
    }
    let ids_clone = req.ids.clone();
    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            WebsiteCartModel::batch_delete(txn, ids_clone, user_id).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}
