use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_approval_flow_node")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub flow_id: Option<i64>,
    pub node_key: Option<String>,
    pub node_type: Option<i32>,
    pub node_order: Option<i32>,
    pub node_name: Option<String>,
    pub approver_type: Option<i32>,
    pub approver_id: Option<i64>,
    pub is_final: Option<i32>,
    pub position_x: Option<i32>,
    pub position_y: Option<i32>,
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
