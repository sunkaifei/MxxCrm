use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_company_background_check")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub company_name: Option<String>,
    pub company_id: Option<i64>,
    pub lead_id: Option<i64>,
    pub risk_score: Option<i32>,
    pub risk_level: Option<String>,
    pub report_data: Option<serde_json::Value>,
    pub ai_model: Option<String>,
    pub prompt_version: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
