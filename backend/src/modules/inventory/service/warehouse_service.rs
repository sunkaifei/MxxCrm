use crate::core::errors::error::Result;
use crate::modules::inventory::entity::warehouse::{Entity as WarehouseEntity, ActiveModel as WarehouseActiveModel, Column as WarehouseColumn};
use crate::modules::inventory::model::warehouse::{WarehouseDetailVO, WarehouseListQuery, WarehouseListVO, WarehouseSaveRequest, WarehouseUpdateRequest};
use crate::modules::inventory::model::warehouse;
use sea_orm::*;

pub async fn get_list(db: &DatabaseConnection, query: &WarehouseListQuery) -> Result<WarehouseListVO> {
    let (models, total) = warehouse::select_page(db, query).await?;
    let items = models.into_iter().map(|m| m.into()).collect();
    Ok(WarehouseListVO { total: total as i64, items })
}

pub async fn get_detail(db: &DatabaseConnection, id: i64) -> Result<WarehouseDetailVO> {
    let model = warehouse::find_by_id(db, id).await?
        .ok_or_else(|| format!("仓库不存在，ID: {}", id))?;
    Ok(model.into())
}

pub async fn insert(db: &DatabaseConnection, data: &WarehouseSaveRequest, created_by: i64) -> Result<i64> {
    let id = warehouse::insert(db, data, created_by).await?;
    Ok(id)
}

pub async fn update(db: &DatabaseConnection, data: &WarehouseUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = data.id.ok_or_else(|| "仓库ID不能为空".to_string())?;
    let rows = warehouse::update_by_id(db, id, data, updated_by).await?;
    Ok(rows)
}

pub async fn batch_delete(db: &DatabaseConnection, ids: &[i64]) -> Result<i64> {
    let res = WarehouseEntity::update_many()
        .set(WarehouseActiveModel {
            deleted: Set(Some(1)),
            ..Default::default()
        })
        .filter(WarehouseColumn::Id.is_in(ids.to_vec()))
        .exec(db)
        .await?;
    Ok(res.rows_affected as i64)
}
