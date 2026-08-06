//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::model::warehouse_area;
use crate::modules::inventory::model::warehouse_area::{WarehouseAreaDetailVO, WarehouseAreaListQuery, WarehouseAreaListVO, WarehouseAreaSaveRequest, WarehouseAreaUpdateRequest, WarehouseAreaVO};

pub async fn get_list(db: &DatabaseConnection, query: &WarehouseAreaListQuery) -> Result<WarehouseAreaListVO> {
    let (models, total) = warehouse_area::select_page(db, query).await?;
    let items: Vec<WarehouseAreaVO> = models.into_iter().map(|m| m.into()).collect();
    Ok(WarehouseAreaListVO { total: total as i64, items })
}

pub async fn get_detail(db: &DatabaseConnection, id: i64) -> Result<WarehouseAreaDetailVO> {
    let model = warehouse_area::find_by_id(db, id).await?
        .ok_or_else(|| format!("库位不存在，ID: {}", id))?;
    Ok(model.into())
}

pub async fn list_all(db: &DatabaseConnection) -> Result<Vec<WarehouseAreaVO>> {
    use crate::modules::inventory::entity::warehouse_area as warehouse_area_entity;
    let models = warehouse_area_entity::Entity::find()
        .filter(warehouse_area_entity::Column::Deleted.eq(0))
        .order_by_asc(warehouse_area_entity::Column::SortOrder)
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let items: Vec<WarehouseAreaVO> = models.into_iter().map(|m| m.into()).collect();
    Ok(items)
}

pub async fn list_by_warehouse(db: &DatabaseConnection, warehouse_id: i64) -> Result<Vec<WarehouseAreaVO>> {
    let models = warehouse_area::find_by_warehouse(db, warehouse_id).await?;
    let items: Vec<WarehouseAreaVO> = models.into_iter().map(|m| m.into()).collect();
    Ok(items)
}

pub async fn insert(db: &DatabaseConnection, data: &WarehouseAreaSaveRequest, created_by: i64) -> Result<i64> {
    let data = data.clone();
    let id = db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            warehouse_area::insert(txn, &data, created_by).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

pub async fn update(db: &DatabaseConnection, data: &WarehouseAreaUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = data.id;
    let data = data.clone();
    let rows = db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            warehouse_area::update_by_id(txn, id, &data, updated_by).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(rows)
}

pub async fn batch_delete(db: &DatabaseConnection, ids: &[i64]) -> Result<i64> {
    let ids_vec = ids.to_vec();
    let res = db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            warehouse_area::batch_delete(txn, &ids_vec).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(res)
}
