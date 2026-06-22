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
use crate::modules::product::model::product::{ProductDetailVO, ProductListQuery, ProductListVO, ProductSaveDTO, ProductSaveRequest, ProductUpdateRequest};
use crate::modules::product::model::product::ProductModel;
use sea_orm::{DbConn, TransactionTrait};

pub async fn insert(db: &DbConn, form_data: &ProductSaveRequest, created_by: i64) -> Result<i64> {
    let name = form_data.name.as_ref().ok_or("产品名称不能为空")?;
    
    if ProductModel::exists_by_name(db, name, None).await? {
        return Err(crate::core::errors::error::Error::from("已存在同名产品"));
    }

    let txn = db.begin().await?;
    
    let mut dto: ProductSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    dto.updated_by = Some(created_by);
    let product_id = ProductModel::insert(&txn, &dto).await?;

    if let Some(skus) = &form_data.skus {
        if !skus.is_empty() {
            ProductModel::batch_save_skus(&txn, product_id, skus).await?;
        }
    }

    txn.commit().await?;
    Ok(product_id)
}

pub async fn batch_delete(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let result = ProductModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &ProductUpdateRequest, updated_by: i64) -> Result<i64> {
    let name = form_data.name.as_ref().ok_or("产品名称不能为空")?;
    
    if ProductModel::exists_by_name(db, name, form_data.id).await? {
        return Err(crate::core::errors::error::Error::from("已存在同名产品"));
    }

    let txn = db.begin().await?;
    
    let mut dto: ProductSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = ProductModel::update_by_id(&txn, &form_data.id, &dto).await?;

    if let (Some(id), Some(skus)) = (&form_data.id, &form_data.skus) {
        ProductModel::batch_save_skus(&txn, *id, skus).await?;
    }

    txn.commit().await?;
    Ok(result)
}

pub async fn get_detail(db: &DbConn, id: i64) -> Result<ProductDetailVO> {
    let result = ProductModel::find_by_id(db, id).await?;
    match result {
        Some(item) => {
            let mut vo: ProductDetailVO = item.into();
            let skus = ProductModel::find_skus_by_product_id(db, id).await?;
            vo.skus = Some(skus.into_iter().map(|s| s.into()).collect());
            Ok(vo)
        },
        None => Err(crate::core::errors::error::Error::from("产品不存在")),
    }
}

pub async fn get_list(db: &DbConn, query: &ProductListQuery) -> Result<(Vec<ProductListVO>, i64, i64)> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);
    let (list, total_pages) = ProductModel::select_in_page(
        db,
        page_num,
        page_size,
        query.keywords.clone(),
        query.category_id,
        query.is_active,
    ).await?;

    let list: Vec<ProductListVO> = list.into_iter().map(|m| m.into()).collect();

    let total = ProductModel::select_count(
        db,
        query.keywords.clone(),
        query.category_id,
        query.is_active,
    ).await?;

    Ok((list, total, total_pages))
}
