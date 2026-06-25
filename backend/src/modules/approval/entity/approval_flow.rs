use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_approval_flow")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub flow_code: Option<String>,
    pub flow_name: Option<String>,
    pub business_type: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<i32>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::approval_flow_node::Entity")]
    ApprovalFlowNode,
    #[sea_orm(has_many = "super::approval_flow_edge::Entity")]
    ApprovalFlowEdge,
    #[sea_orm(has_many = "super::approval_instance::Entity")]
    ApprovalInstance,
}

impl Related<super::approval_flow_node::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApprovalFlowNode.def()
    }
}

impl Related<super::approval_flow_edge::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApprovalFlowEdge.def()
    }
}

impl Related<super::approval_instance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApprovalInstance.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
