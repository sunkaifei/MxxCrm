use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_approval_flow_edge")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub flow_id: Option<i64>,
    pub source_node_key: Option<String>,
    pub target_node_key: Option<String>,
    pub condition_expr: Option<String>,
    pub label: Option<String>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::approval_flow::Entity",
        from = "Column::FlowId",
        to = "super::approval_flow::Column::Id"
    )]
    ApprovalFlow,
}

impl Related<super::approval_flow::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApprovalFlow.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
