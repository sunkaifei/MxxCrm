use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_approval_instance")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub flow_id: Option<i64>,
    pub flow_code: Option<String>,
    pub business_type: Option<String>,
    pub business_id: Option<i64>,
    pub business_title: Option<String>,
    pub submitter_id: Option<i64>,
    pub submitter_name: Option<String>,
    pub current_node_key: Option<String>,
    pub current_approver_id: Option<i64>,
    pub status: Option<i32>,
    pub submitted_at: Option<DateTime>,
    pub finished_at: Option<DateTime>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub extra_data: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::approval_flow::Entity",
        from = "Column::FlowId",
        to = "super::approval_flow::Column::Id"
    )]
    ApprovalFlow,
    #[sea_orm(has_many = "super::approval_log::Entity")]
    ApprovalLog,
}

impl Related<super::approval_flow::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApprovalFlow.def()
    }
}

impl Related<super::approval_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApprovalLog.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
