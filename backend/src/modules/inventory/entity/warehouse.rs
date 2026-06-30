use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_inventory_warehouse")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub name: Option<String>,
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
    pub deleted: Option<i16>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
