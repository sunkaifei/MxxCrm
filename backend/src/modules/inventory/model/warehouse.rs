use rust_decimal::Decimal;
use crate::modules::inventory::entity::warehouse::{ActiveModel, Column, Entity, Model};
use sea_orm::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub warehouse_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WarehouseListVO {
    pub total: i64,
    pub items: Vec<WarehouseVO>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseVO {
    pub id: i64,
    pub warehouse_name: Option<String>,
    pub warehouse_name_show: Option<String>,
    pub code: Option<String>,
    pub warehouse_type: Option<i16>,
    pub warehouse_type_name: Option<String>,
    pub region: Option<String>,
    pub address: Option<String>,
    pub manager_id: Option<i64>,
    pub manager_name: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub is_active: Option<bool>,
    pub status: Option<i16>,
    pub create_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseDetailVO {
    pub id: i64,
    pub warehouse_name: Option<String>,
    pub code: Option<String>,
    pub warehouse_type: Option<i16>,
    pub region: Option<String>,
    pub address: Option<String>,
    pub area_sqm: Option<Decimal>,
    pub manager_id: Option<i64>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub backup_phone: Option<String>,
    pub logistics_types: Option<String>,
    pub is_active: Option<bool>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseSaveRequest {
    pub warehouse_name: Option<String>,
    pub code: Option<String>,
    pub warehouse_type: Option<i16>,
    pub region: Option<String>,
    pub address: Option<String>,
    pub area_sqm: Option<Decimal>,
    pub manager_id: Option<i64>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub backup_phone: Option<String>,
    pub logistics_types: Option<String>,
    pub is_active: Option<bool>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseUpdateRequest {
    pub id: Option<i64>,
    pub warehouse_name: Option<String>,
    pub code: Option<String>,
    pub warehouse_type: Option<i16>,
    pub region: Option<String>,
    pub address: Option<String>,
    pub area_sqm: Option<Decimal>,
    pub manager_id: Option<i64>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub backup_phone: Option<String>,
    pub logistics_types: Option<String>,
    pub is_active: Option<bool>,
    pub remark: Option<String>,
}

fn warehouse_type_name(t: Option<i16>) -> Option<String> {
    match t.unwrap_or(0) {
        1 => Some("原材料仓".to_string()),
        2 => Some("成品仓".to_string()),
        3 => Some("半成品仓".to_string()),
        4 => Some("退货仓".to_string()),
        5 => Some("中转仓".to_string()),
        _ => None,
    }
}

impl From<Model> for WarehouseVO {
    fn from(model: Model) -> Self {
        WarehouseVO {
            id: model.id,
            warehouse_name: model.name.clone(),
            warehouse_name_show: model.name,
            code: model.code,
            warehouse_type: model.warehouse_type,
            warehouse_type_name: warehouse_type_name(model.warehouse_type),
            region: model.region,
            address: model.address,
            manager_id: model.manager_id,
            manager_name: model.contact_person.clone(),
            contact_person: model.contact_person,
            contact_phone: model.contact_phone,
            is_active: model.is_active,
            status: if model.is_active.unwrap_or(true) { Some(1) } else { Some(0) },
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

impl From<Model> for WarehouseDetailVO {
    fn from(model: Model) -> Self {
        WarehouseDetailVO {
            id: model.id,
            warehouse_name: model.name,
            code: model.code,
            warehouse_type: model.warehouse_type,
            region: model.region,
            address: model.address,
            area_sqm: model.area_sqm,
            manager_id: model.manager_id,
            contact_person: model.contact_person,
            contact_phone: model.contact_phone,
            backup_phone: model.backup_phone,
            logistics_types: model.logistics_types,
            is_active: model.is_active,
            remark: model.remark,
        }
    }
}

pub async fn select_page(db: &DbConn, query: &WarehouseListQuery) -> Result<(Vec<Model>, u64), DbErr> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let mut q = Entity::find().filter(Column::Deleted.eq(0));

    if let Some(name) = &query.warehouse_name {
        if !name.is_empty() {
            q = q.filter(Column::Name.contains(name));
        }
    }

    let paginator = q.paginate(db, page_size as u64);
    let total = paginator.num_items().await?;
    let list = paginator.fetch_page((page_num - 1) as u64).await?;
    Ok((list, total))
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<Model>, DbErr> {
    Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await
}

pub async fn insert(db: &DbConn, req: &WarehouseSaveRequest, created_by: i64) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local().to_owned();
    let am = ActiveModel {
        name: Set(req.warehouse_name.clone()),
        code: Set(req.code.clone()),
        warehouse_type: Set(req.warehouse_type.or(Some(1))),
        region: Set(req.region.clone()),
        address: Set(req.address.clone()),
        area_sqm: Set(req.area_sqm),
        manager_id: Set(req.manager_id),
        contact_person: Set(req.contact_person.clone()),
        contact_phone: Set(req.contact_phone.clone()),
        backup_phone: Set(req.backup_phone.clone()),
        logistics_types: Set(req.logistics_types.clone()),
        is_active: Set(req.is_active.or(Some(true))),
        remark: Set(req.remark.clone()),
        deleted: Set(Some(0)),
        created_by: Set(Some(created_by)),
        updated_by: Set(Some(created_by)),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let res = Entity::insert(am).exec(db).await?;
    Ok(res.last_insert_id)
}

pub async fn update_by_id(db: &DbConn, id: i64, req: &WarehouseUpdateRequest, updated_by: i64) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local().to_owned();
    let am = ActiveModel {
        name: Set(req.warehouse_name.clone()),
        code: Set(req.code.clone()),
        warehouse_type: Set(req.warehouse_type),
        region: Set(req.region.clone()),
        address: Set(req.address.clone()),
        area_sqm: Set(req.area_sqm),
        manager_id: Set(req.manager_id),
        contact_person: Set(req.contact_person.clone()),
        contact_phone: Set(req.contact_phone.clone()),
        backup_phone: Set(req.backup_phone.clone()),
        logistics_types: Set(req.logistics_types.clone()),
        is_active: Set(req.is_active),
        remark: Set(req.remark.clone()),
        updated_by: Set(Some(updated_by)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let res = Entity::update_many()
        .set(am)
        .filter(Column::Id.eq(id))
        .filter(Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(res.rows_affected as i64)
}
