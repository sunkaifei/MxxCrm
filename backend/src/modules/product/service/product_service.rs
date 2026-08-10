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
use crate::modules::product::entity::brand;
use crate::modules::product::model::product::{ProductDetailVO, ProductListQuery, ProductListVO, ProductSaveDTO, ProductSaveRequest, ProductUpdateRequest};
use crate::modules::product::model::product::ProductModel;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::Decimal;
use sea_orm::{ColumnTrait, ConnectionTrait, DbConn, EntityTrait, QueryFilter, TransactionTrait, Value};
use std::collections::HashMap;

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

pub async fn get_detail_with_specs(db: &DbConn, id: i64) -> Result<(ProductDetailVO, crate::modules::product::model::spec::SpecGroupVO)> {
    let result = ProductModel::find_by_id(db, id).await?;
    match result {
        Some(item) => {
            let mut vo: ProductDetailVO = item.into();
            let skus = ProductModel::find_skus_by_product_id(db, id).await?;
            vo.skus = Some(skus.into_iter().map(|s| s.into()).collect());
            let specs = crate::modules::product::service::spec_service::get_specs(db, id).await?;
            Ok((vo, specs))
        },
        None => Err(crate::core::errors::error::Error::from("产品不存在")),
    }
}

pub async fn get_list(db: &DbConn, query: &ProductListQuery) -> Result<(Vec<ProductListVO>, i64, i64)> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);
    let (models, total_pages) = ProductModel::select_in_page(
        db,
        page_num,
        page_size,
        query.keywords.clone(),
        query.category_id,
        query.brand_id,
        query.is_active,
    ).await?;

    // 批量查询库存（按仓库过滤或全仓库汇总）
    let product_ids: Vec<i64> = models.iter().map(|m| m.id).collect();
    let stock_map: HashMap<i64, i64> = if product_ids.is_empty() {
        HashMap::new()
    } else {
        let placeholders: Vec<String> = (1..=product_ids.len()).map(|i| format!("${}", i)).collect();
        let mut values: Vec<Value> = product_ids.iter().map(|&id| id.into()).collect();
        let mut sql = format!(
            "SELECT product_id, COALESCE(SUM(quantity), 0) AS total FROM mxx_inventory_stock WHERE deleted = 0 AND product_id IN ({}) GROUP BY product_id",
            placeholders.join(", ")
        );
        if let Some(wid) = query.warehouse_id {
            sql.push_str(&format!(" AND warehouse_id = ${}", product_ids.len() + 1));
            values.push(wid.into());
        }
        let stmt = sea_orm::Statement::from_sql_and_values(db.get_database_backend(), &sql, values);
        let rows = db.query_all_raw(stmt).await?;
        let mut map = HashMap::new();
        for row in rows {
            let pid: i64 = row.try_get("", "product_id")?;
            let total: Decimal = row.try_get("", "total")?;
            map.insert(pid, total.to_i64().unwrap_or(0));
        }
        map
    };

    // 批量查询品牌名称
    let brand_ids: Vec<i64> = models.iter().filter_map(|m| m.brand_id).collect();
    let brand_map: HashMap<i64, String> = if brand_ids.is_empty() {
        HashMap::new()
    } else {
        brand::Entity::find()
            .filter(brand::Column::Id.is_in(brand_ids))
            .filter(brand::Column::Deleted.eq(0))
            .all(db)
            .await?
            .into_iter()
            .filter_map(|b| b.name.map(|n| (b.id, n)))
            .collect()
    };

    let list: Vec<ProductListVO> = models.into_iter().map(|m| {
        let pid = m.id;
        let bid = m.brand_id;
        let mut vo: ProductListVO = m.into();
        if let Some(bid) = bid {
            vo.brand_name = brand_map.get(&bid).cloned();
        }
        vo.total_stock = stock_map.get(&pid).cloned();
        vo
    }).collect();

    let total = ProductModel::select_count(
        db,
        query.keywords.clone(),
        query.category_id,
        query.brand_id,
        query.is_active,
    ).await?;

    Ok((list, total, total_pages))
}
