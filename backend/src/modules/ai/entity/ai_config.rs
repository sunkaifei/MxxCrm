use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_ai_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub config_key: Option<String>,
    pub config_name: Option<String>,
    pub config_value: Option<String>,
    pub config_type: Option<String>,
    pub remark: Option<String>,
    pub sort: Option<i32>,
    pub created_by: Option<String>,
    pub created_at: Option<DateTime>,
    pub updated_by: Option<String>,
    pub updated_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
