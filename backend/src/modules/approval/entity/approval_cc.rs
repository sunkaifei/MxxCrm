use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_approval_cc")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub instance_id: Option<i64>,
    pub user_id: Option<i64>,
    pub user_name: Option<String>,
    /// 抄送发起人ID（谁添加的抄送）
    pub cc_from_id: Option<i64>,
    pub cc_from_name: Option<String>,
    pub cc_reason: Option<String>,
    /// 是否已读：0=未读,1=已读
    pub is_read: Option<i32>,
    pub read_time: Option<DateTime>,
    pub create_time: Option<DateTime>,
    pub deleted: Option<i32>,
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
