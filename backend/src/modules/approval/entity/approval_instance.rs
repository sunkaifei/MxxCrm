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
    /// 当前节点候选审批人ID列表（JSON数组，如 [1,2,3]）
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub candidate_approvers: Option<serde_json::Value>,
    /// 当前节点已处理审批人ID列表（JSON数组）
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub processed_approvers: Option<serde_json::Value>,
    pub status: Option<i32>,
    pub submitted_at: Option<DateTime>,
    pub finished_at: Option<DateTime>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub extra_data: Option<serde_json::Value>,
    /// 取消原因（发起人撤回时填写）
    pub cancel_reason: Option<String>,
    /// 转办来源人ID（记录转办链）
    pub transfer_from_id: Option<i64>,
    /// 委派来源人ID（委派时记录原审批人）
    pub delegate_from_id: Option<i64>,
    /// 加签类型：1=前加签,2=后加签,3=并加签
    pub add_sign_type: Option<i32>,
    /// 加签用户ID列表（JSON数组）
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub add_sign_user_ids: Option<serde_json::Value>,
    /// 是否需要重新提交：0=否,1=是（退回到发起人时置1）
    pub needs_resubmit: Option<i32>,
    /// 流程模板快照（创建实例时拷贝的 nodes+edges JSON，防止模板修改影响在途实例）
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub flow_snapshot: Option<serde_json::Value>,
    /// 流程模板版本号
    pub flow_version: Option<i32>,
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
