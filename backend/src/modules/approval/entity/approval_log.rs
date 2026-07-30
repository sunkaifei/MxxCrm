use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_approval_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub instance_id: Option<i64>,
    pub node_key: Option<String>,
    pub node_name: Option<String>,
    pub approver_id: Option<i64>,
    pub approver_name: Option<String>,
    pub action: Option<i32>,
    pub comment: Option<String>,
    pub create_time: Option<DateTime>,
    /// 目标用户ID（转办/委派/加签的目标用户）
    pub target_user_id: Option<i64>,
    /// 目标用户姓名
    pub target_user_name: Option<String>,
    /// 目标节点key（退回的目标节点）
    pub target_node_key: Option<String>,
    /// 目标节点名称
    pub target_node_name: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::approval_instance::Entity",
        from = "Column::InstanceId",
        to = "super::approval_instance::Column::Id"
    )]
    ApprovalInstance,
}

impl Related<super::approval_instance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApprovalInstance.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
