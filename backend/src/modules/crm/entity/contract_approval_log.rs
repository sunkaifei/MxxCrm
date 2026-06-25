use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_contract_approval_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub contract_id: i64,
    pub action: i32,
    pub operator_id: i64,
    pub operator_name: Option<String>,
    pub reason: Option<String>,
    pub previous_status: Option<i32>,
    pub new_status: Option<i32>,
    pub current_stage: Option<i32>,
    pub next_stage: Option<i32>,
    pub create_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
