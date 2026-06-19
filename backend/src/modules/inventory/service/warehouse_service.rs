use crate::core::errors::error::Result;
use crate::modules::inventory::entity::warehouse::{ActiveModel, Column, Entity, Model};
use crate::modules::inventory::model::warehouse::{WarehouseDetailVO, WarehouseListQuery, WarehouseListVO, WarehouseSaveRequest, WarehouseUpdateRequest};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Set};
use sea_orm::ActiveValue::NotSet;

pub async fn get_list(db: &DatabaseConnection, query: &WarehouseListQuery) -> Result<WarehouseListVO> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);
    
    let mut condition = Entity::find();
    
    if let Some(name) = &query.warehouse_name {
        if !name.is_empty() {
            condition = condition.filter(Column::Name.contains(name));
        }
    }
    
    let paginator = condition.paginate(db, page_size as u64);
    let total = paginator.num_items().await?;
    let models = paginator.fetch_page((page_num - 1) as u64).await?;
    
    let list = models.into_iter().map(|m| m.into()).collect();
    
    Ok(WarehouseListVO { total: total as i64, list })
}

pub async fn get_detail(db: &DatabaseConnection, id: i64) -> Result<WarehouseDetailVO> {
    let model = Entity::find_by_id(id).one(db).await?
        .ok_or_else(|| format!("仓库不存在，ID: {}", id))?;
    
    Ok(model.into())
}

pub async fn insert(db: &DatabaseConnection, data: &WarehouseSaveRequest, created_by: i64) -> Result<i64> {
    let active_model = ActiveModel {
        id: NotSet,
        name: Set(data.name.clone()),
        code: Set(data.code.clone()),
        address: Set(data.address.clone()),
        contact_person: Set(data.contact_person.clone()),
        contact_phone: Set(data.contact_phone.clone()),
        is_active: Set(data.is_active.or(Some(true))),
        ..Default::default()
    };
    
    let result = Entity::insert(active_model).exec(db).await?;
    Ok(result.last_insert_id)
}

pub async fn update(db: &DatabaseConnection, data: &WarehouseUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = data.id.ok_or_else(|| "仓库ID不能为空".to_string())?;
    
    let mut active_model: ActiveModel = Entity::find_by_id(id).one(db).await?
        .ok_or_else(|| format!("仓库不存在，ID: {}", id))?
        .into();
    
    if let Some(name) = &data.name {
        active_model.name = Set(Some(name.clone()));
    }
    if let Some(code) = &data.code {
        active_model.code = Set(Some(code.clone()));
    }
    if let Some(address) = &data.address {
        active_model.address = Set(Some(address.clone()));
    }
    if let Some(contact_person) = &data.contact_person {
        active_model.contact_person = Set(Some(contact_person.clone()));
    }
    if let Some(contact_phone) = &data.contact_phone {
        active_model.contact_phone = Set(Some(contact_phone.clone()));
    }
    if data.is_active.is_some() {
        active_model.is_active = Set(data.is_active);
    }
    
    let result = active_model.update(db).await?;
    Ok(result.id)
}

pub async fn batch_delete(db: &DatabaseConnection, ids: &[i64]) -> Result<i64> {
    let result = Entity::delete_many()
        .filter(Column::Id.is_in(ids.to_vec()))
        .exec(db)
        .await?;
    
    Ok(result.rows_affected as i64)
}