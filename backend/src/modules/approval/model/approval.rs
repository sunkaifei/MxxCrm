use chrono::Utc;
use sea_orm::sea_query::Expr;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::system::model::admin::AdminModel;
use crate::modules::system::entity::admin::{Entity as AdminEntity, Column as AdminColumn};
use crate::modules::system::entity::dept::{Column as DeptColumn, Entity as DeptEntity};
use crate::modules::system::entity::admin_role_merge::{Column as RoleMergeColumn, Entity as RoleMergeEntity};
use crate::modules::system::entity::admin_dept_merge::{Column as DeptMergeColumn, Entity as DeptMergeEntity};
use crate::modules::system::entity::admin_post_merge::{Column as PostMergeColumn, Entity as PostMergeEntity};
use crate::modules::approval::entity::approval_flow::{
    ActiveModel as FlowActiveModel, Column as FlowColumn, Entity as FlowEntity, Model as FlowModel,
};
use crate::modules::approval::entity::approval_flow_edge::{
    ActiveModel as EdgeActiveModel, Column as EdgeColumn, Entity as EdgeEntity,
    Model as EdgeModel,
};
use crate::modules::approval::entity::approval_flow_node::{
    ActiveModel as NodeActiveModel, Column as NodeColumn, Entity as NodeEntity,
    Model as NodeModel,
};
use crate::modules::approval::entity::approval_instance::{
    ActiveModel as InstanceActiveModel, Column as InstanceColumn, Entity as InstanceEntity,
    Model as InstanceModel,
};
use crate::modules::approval::entity::approval_log::{
    ActiveModel as LogActiveModel, Column as LogColumn, Entity as LogEntity,
};
use crate::modules::approval::entity::approval_cc::{
    ActiveModel as CcActiveModel, Column as CcColumn, Entity as CcEntity,
};

// ============ Flow Request/Response ============

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FlowSaveRequest {
    pub flow_id: Option<i64>,
    pub flow_code: String,
    pub flow_name: String,
    pub business_type: String,
    pub description: Option<String>,
    pub nodes: Vec<NodeDTO>,
    pub edges: Vec<EdgeDTO>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct NodeDTO {
    pub node_key: String,
    pub node_type: i32,
    pub node_name: String,
    pub node_order: i32,
    pub approver_type: Option<i32>,
    pub approver_id: Option<i64>,
    /// 审批模式：1=或签(任一通过)，2=会签(全通过)，3=依次审批
    pub approve_mode: Option<i32>,
    pub is_final: Option<i32>,
    pub position_x: Option<i32>,
    pub position_y: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EdgeDTO {
    pub source: String,
    pub target: String,
    pub condition_expr: Option<String>,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FlowDetailVO {
    pub id: i64,
    pub flow_code: String,
    pub flow_name: String,
    pub business_type: String,
    pub description: Option<String>,
    pub enabled: bool,
    /// 是否系统内置（1=系统内置不可删除，0=用户自定义可删除）
    pub is_system: Option<i32>,
    pub nodes: Vec<NodeVO>,
    pub edges: Vec<EdgeVO>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct NodeVO {
    pub node_key: String,
    pub node_type: i32,
    pub node_name: String,
    pub node_order: i32,
    pub approver_type: Option<i32>,
    pub approver_id: Option<i64>,
    /// 审批模式：1=或签(任一通过)，2=会签(全通过)，3=依次审批
    pub approve_mode: Option<i32>,
    pub is_final: Option<i32>,
    pub position_x: Option<i32>,
    pub position_y: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct EdgeVO {
    pub source: String,
    pub target: String,
    pub condition_expr: Option<String>,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FlowListVO {
    pub id: i64,
    pub flow_code: String,
    pub flow_name: String,
    pub business_type: String,
    pub description: Option<String>,
    pub enabled: bool,
    /// 是否系统内置（1=系统内置不可删除，0=用户自定义可删除）
    pub is_system: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowListQuery {
    pub page_num: u64,
    pub page_size: u64,
    pub flow_name: Option<String>,
    pub business_type: Option<String>,
}

// ============ Submit/Process Request ============

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ApprovalSubmitRequest {
    pub flow_code: String,
    pub business_type: String,
    pub business_id: i64,
    pub business_title: Option<String>,
    pub submitter_id: i64,
    pub submitter_name: Option<String>,
    pub extra_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ApprovalProcessRequest {
    pub instance_id: i64,
    pub action: i32,
    pub approver_id: i64,
    pub approver_name: Option<String>,
    pub comment: Option<String>,
}

// ============ 审批增强：取消/退回/转办/委派/加签/抄送 ============

/// 发起人取消（撤回）请求
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ApprovalCancelRequest {
    pub instance_id: i64,
    pub cancel_reason: Option<String>,
}

/// 退回请求（退回到发起人或指定节点）
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ApprovalRejectToRequest {
    pub instance_id: i64,
    /// 退回目标节点 key；为空表示退回到发起人重新修改后提交
    pub reject_to_node_key: Option<String>,
    pub comment: Option<String>,
}

/// 转办请求（当前审批人转给他人审批，责任转移）
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ApprovalTransferRequest {
    pub instance_id: i64,
    pub target_user_id: i64,
    pub target_user_name: Option<String>,
    pub comment: Option<String>,
}

/// 委派请求（委派他人处理，责任仍归原审批人）
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ApprovalDelegateRequest {
    pub instance_id: i64,
    pub target_user_id: i64,
    pub target_user_name: Option<String>,
    pub comment: Option<String>,
}

/// 加签请求
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ApprovalAddSignRequest {
    pub instance_id: i64,
    /// 加签类型：1=前加签,2=后加签,3=并加签
    pub add_sign_type: i32,
    /// 加签用户ID列表
    pub target_user_ids: Vec<i64>,
    pub comment: Option<String>,
}

/// 抄送请求
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ApprovalCcRequest {
    pub instance_id: i64,
    /// 抄送用户ID列表
    pub user_ids: Vec<i64>,
    pub cc_reason: Option<String>,
}

/// 抄送记录 VO
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ApprovalCcVO {
    pub id: i64,
    pub instance_id: i64,
    pub user_id: i64,
    pub user_name: Option<String>,
    pub cc_from_id: Option<i64>,
    pub cc_from_name: Option<String>,
    pub cc_reason: Option<String>,
    pub is_read: i32,
    pub read_time: Option<String>,
    pub create_time: Option<String>,
    /// 审批实例关联信息（列表页展示用）
    pub business_type: Option<String>,
    pub business_id: Option<i64>,
    pub business_title: Option<String>,
    pub submitter_name: Option<String>,
    pub instance_status: Option<i32>,
}

// ============ Instance VO ============

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ApprovalInstanceVO {
    pub id: i64,
    pub flow_code: String,
    pub business_type: String,
    pub business_id: i64,
    pub business_title: Option<String>,
    pub submitter_id: i64,
    pub submitter_name: Option<String>,
    pub current_node_key: Option<String>,
    pub current_approver_id: Option<i64>,
    pub current_approver_name: Option<String>,
    /// 当前节点候选审批人ID列表
    pub candidate_approvers: Vec<i64>,
    /// 候选审批人姓名列表（与 ID 一一对应）
    pub candidate_approver_names: Vec<String>,
    /// 当前节点已处理审批人ID列表
    pub processed_approvers: Vec<i64>,
    /// 当前节点审批模式：1=或签, 2=会签, 3=依次审批
    pub approve_mode: i32,
    pub status: i32,
    pub submitted_at: Option<String>,
    pub finished_at: Option<String>,
    #[serde(skip_serializing)]
    pub extra_data: Option<serde_json::Value>,
    pub flow_nodes: Vec<ApprovalFlowNodeVO>,
    pub flow_edges: Vec<ApprovalFlowEdgeVO>,
    pub logs: Vec<ApprovalLogVO>,
    /// 取消原因
    pub cancel_reason: Option<String>,
    /// 转办来源人ID
    pub transfer_from_id: Option<i64>,
    /// 委派来源人ID
    pub delegate_from_id: Option<i64>,
    /// 加签类型
    pub add_sign_type: Option<i32>,
    /// 是否需要重新提交（退回到发起人时置1）
    pub needs_resubmit: Option<i32>,
    /// 抄送人列表
    pub cc_users: Vec<ApprovalCcVO>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ApprovalFlowNodeVO {
    pub node_key: String,
    pub node_type: i32,
    pub node_name: String,
    pub node_order: i32,
    pub approver_id: Option<i64>,
    pub approver_name: Option<String>,
    /// 审批模式：1=或签, 2=会签, 3=依次审批
    pub approve_mode: i32,
    pub node_status: i32,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ApprovalFlowEdgeVO {
    pub source: String,
    pub target: String,
    pub condition_expr: Option<String>,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ApprovalLogVO {
    pub node_key: Option<String>,
    pub node_name: Option<String>,
    pub approver_id: i64,
    pub approver_name: Option<String>,
    pub action: i32,
    pub comment: Option<String>,
    pub create_time: Option<String>,
    pub duration: Option<String>,
    /// 目标用户ID（转办/委派/加签）
    pub target_user_id: Option<i64>,
    /// 目标用户姓名
    pub target_user_name: Option<String>,
    /// 目标节点key（退回）
    pub target_node_key: Option<String>,
    /// 目标节点名称
    pub target_node_name: Option<String>,
}

// ============ Data Access ============

pub struct ApprovalModel;

impl ApprovalModel {
    pub async fn save_flow(db: &DatabaseConnection, req: &FlowSaveRequest, operator: &str) -> Result<i64> {
        let now = Utc::now().naive_utc();

        let flow_id = if let Some(id) = req.flow_id {
            // Update existing flow
            let existing = FlowEntity::find_by_id(id)
                .one(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?
                .ok_or_else(|| Error::from("审批流不存在"))?;

            // 系统内置审批流不允许修改 flow_code 和 business_type
            if existing.is_system == Some(1) {
                if existing.flow_code.as_deref() != Some(&req.flow_code)
                    || existing.business_type.as_deref() != Some(&req.business_type)
                {
                    return Err(Error::from("系统内置审批流不允许修改流程编码和业务类型"));
                }
            }

            let mut active: FlowActiveModel = existing.into();
            active.flow_code = Set(Some(req.flow_code.clone()));
            active.flow_name = Set(Some(req.flow_name.clone()));
            active.business_type = Set(Some(req.business_type.clone()));
            active.description = Set(req.description.clone());
            active.update_by = Set(Some(operator.to_string()));
            active.update_time = Set(Some(now));
            active.update(db).await.map_err(|e| Error::from(e.to_string()))?;
            id
        } else {
            // Insert new flow（用户自定义，is_system = 0）
            let active = FlowActiveModel {
                flow_code: Set(Some(req.flow_code.clone())),
                flow_name: Set(Some(req.flow_name.clone())),
                business_type: Set(Some(req.business_type.clone())),
                description: Set(req.description.clone()),
                enabled: Set(Some(1)),
                is_system: Set(Some(0)),
                create_by: Set(Some(operator.to_string())),
                create_time: Set(Some(now)),
                update_by: Set(Some(operator.to_string())),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            let result = FlowEntity::insert(active)
                .exec(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?;
            result.last_insert_id
        };

        // Delete old nodes and edges
        NodeEntity::delete_many()
            .filter(NodeColumn::FlowId.eq(flow_id))
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        EdgeEntity::delete_many()
            .filter(EdgeColumn::FlowId.eq(flow_id))
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        // Insert new nodes
        for node in &req.nodes {
            let active = NodeActiveModel {
                flow_id: Set(Some(flow_id)),
                node_key: Set(Some(node.node_key.clone())),
                node_type: Set(Some(node.node_type)),
                node_order: Set(Some(node.node_order)),
                node_name: Set(Some(node.node_name.clone())),
                approver_type: Set(node.approver_type),
                approver_id: Set(node.approver_id),
                approve_mode: Set(node.approve_mode),
                is_final: Set(node.is_final),
                position_x: Set(node.position_x),
                position_y: Set(node.position_y),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            NodeEntity::insert(active)
                .exec(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?;
        }

        // Insert new edges
        for edge in &req.edges {
            let active = EdgeActiveModel {
                flow_id: Set(Some(flow_id)),
                source_node_key: Set(Some(edge.source.clone())),
                target_node_key: Set(Some(edge.target.clone())),
                condition_expr: Set(edge.condition_expr.clone()),
                label: Set(edge.label.clone()),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            EdgeEntity::insert(active)
                .exec(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?;
        }

        Ok(flow_id)
    }

    pub async fn find_flow_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<FlowDetailVO>> {
        let flow = FlowEntity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let flow = match flow {
            Some(f) => f,
            None => return Ok(None),
        };

        let nodes = NodeEntity::find()
            .filter(NodeColumn::FlowId.eq(id))
            .order_by_asc(NodeColumn::NodeOrder)
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let edges = EdgeEntity::find()
            .filter(EdgeColumn::FlowId.eq(id))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        Ok(Some(FlowDetailVO {
            id: flow.id,
            flow_code: flow.flow_code.unwrap_or_default(),
            flow_name: flow.flow_name.unwrap_or_default(),
            business_type: flow.business_type.unwrap_or_default(),
            description: flow.description,
            enabled: flow.enabled.unwrap_or(0) == 1,
            is_system: flow.is_system,
            nodes: nodes
                .into_iter()
                .map(|n| NodeVO {
                    node_key: n.node_key.unwrap_or_default(),
                    node_type: n.node_type.unwrap_or(2),
                    node_name: n.node_name.unwrap_or_default(),
                    node_order: n.node_order.unwrap_or(0),
                    approver_type: n.approver_type,
                    approver_id: n.approver_id,
                    approve_mode: n.approve_mode,
                    is_final: n.is_final,
                    position_x: n.position_x,
                    position_y: n.position_y,
                })
                .collect(),
            edges: edges
                .into_iter()
                .map(|e| EdgeVO {
                    source: e.source_node_key.unwrap_or_default(),
                    target: e.target_node_key.unwrap_or_default(),
                    condition_expr: e.condition_expr,
                    label: e.label,
                })
                .collect(),
        }))
    }

    pub async fn find_flow_list(db: &DatabaseConnection, query: &FlowListQuery) -> Result<ResultPage<Vec<FlowListVO>>> {
        let mut qb = FlowEntity::find();
        if let Some(name) = &query.flow_name {
            if !name.is_empty() {
                qb = qb.filter(FlowColumn::FlowName.contains(name));
            }
        }
        if let Some(bt) = &query.business_type {
            if !bt.is_empty() {
                qb = qb.filter(FlowColumn::BusinessType.eq(bt));
            }
        }

        let paginator = qb.paginate(db, query.page_size);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        let items = paginator
            .fetch_page(query.page_num - 1)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let list: Vec<FlowListVO> = items
            .into_iter()
            .map(|f| FlowListVO {
                id: f.id,
                flow_code: f.flow_code.unwrap_or_default(),
                flow_name: f.flow_name.unwrap_or_default(),
                business_type: f.business_type.unwrap_or_default(),
                description: f.description,
                enabled: f.enabled.unwrap_or(0) == 1,
                is_system: f.is_system,
                create_time: f.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            })
            .collect();

        Ok(ResultPage {
            items: list,
            total: total as i64,
            current_page: query.page_num as i64,
            page_size: query.page_size as i64,
            total_pages: ((total as f64) / (query.page_size as f64)).ceil() as i64,
        })
    }

    pub async fn toggle_flow(db: &DatabaseConnection, id: i64) -> Result<()> {
        let flow = FlowEntity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        if let Some(f) = flow {
            let new_enabled: i32 = if f.enabled.unwrap_or(0) == 1 { 0 } else { 1 };
            FlowEntity::update_many()
                .col_expr(FlowColumn::Enabled, Expr::value(new_enabled))
                .filter(FlowColumn::Id.eq(id))
                .exec(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn delete_flow(db: &DatabaseConnection, id: i64) -> Result<()> {
        let flow = FlowEntity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
            .ok_or_else(|| Error::from("审批流不存在"))?;

        // 系统内置审批流不允许删除
        if flow.is_system == Some(1) {
            return Err(Error::from("系统内置审批流不可删除，如需停用请使用启用/禁用功能"));
        }

        // 检查是否有审批实例引用了该流程
        let instance_count = InstanceEntity::find()
            .filter(InstanceColumn::FlowId.eq(id))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        if instance_count > 0 {
            return Err(Error::from(
                "该审批流已被使用，无法删除，请禁用",
            ));
        }

        // 无引用，级联删除：边 → 节点 → 流程
        EdgeEntity::delete_many()
            .filter(EdgeColumn::FlowId.eq(id))
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        NodeEntity::delete_many()
            .filter(NodeColumn::FlowId.eq(id))
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        FlowEntity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        Ok(())
    }

    pub async fn find_flow_by_code(
        db: &DatabaseConnection,
        code: &str,
    ) -> Result<Option<(FlowModel, Vec<NodeModel>, Vec<EdgeModel>)>> {
        let flow = FlowEntity::find()
            .filter(FlowColumn::FlowCode.eq(code))
            .filter(FlowColumn::Enabled.eq(1))
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let flow = match flow {
            Some(f) => f,
            None => return Ok(None),
        };
        let flow_id = flow.id;

        let nodes = NodeEntity::find()
            .filter(NodeColumn::FlowId.eq(flow_id))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let edges = EdgeEntity::find()
            .filter(EdgeColumn::FlowId.eq(flow_id))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        Ok(Some((flow, nodes, edges)))
    }

    pub async fn create_instance(
        db: &DatabaseConnection,
        req: &ApprovalSubmitRequest,
        first_node_key: &str,
        approver_id: i64,
        candidate_approvers: &[i64],
    ) -> Result<i64> {
        let now = Utc::now().naive_utc();
        // 候选审批人列表（去重，保留顺序）；若为空则退化为 [approver_id]
        let candidates: Vec<i64> = {
            let mut v: Vec<i64> = if candidate_approvers.is_empty() {
                vec![approver_id]
            } else {
                candidate_approvers.to_vec()
            };
            v.dedup();
            v
        };
        let candidates_json: serde_json::Value =
            serde_json::Value::Array(candidates.iter().map(|id| serde_json::json!(id)).collect());
        let active = InstanceActiveModel {
            flow_code: Set(Some(req.flow_code.clone())),
            business_type: Set(Some(req.business_type.clone())),
            business_id: Set(Some(req.business_id)),
            business_title: Set(req.business_title.clone()),
            submitter_id: Set(Some(req.submitter_id)),
            submitter_name: Set(req.submitter_name.clone()),
            current_node_key: Set(Some(first_node_key.to_string())),
            current_approver_id: Set(Some(approver_id)),
            candidate_approvers: Set(Some(candidates_json)),
            processed_approvers: Set(Some(serde_json::Value::Array(vec![]))),
            status: Set(Some(1)),
            submitted_at: Set(Some(now)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            extra_data: Set(req.extra_data.clone()),
            ..Default::default()
        };
        let result = InstanceEntity::insert(active)
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(result.last_insert_id)
    }

    pub async fn find_instance_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<ApprovalInstanceVO>> {
        let inst = InstanceEntity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let inst = match inst {
            Some(i) => i,
            None => return Ok(None),
        };

        let logs = LogEntity::find()
            .filter(LogColumn::InstanceId.eq(id))
            .order_by_asc(LogColumn::CreateTime)
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        // 查询当前审批人名字
        let current_approver_name = if let Some(approver_id) = inst.current_approver_id {
            AdminModel::find_by_id(db, &Some(approver_id))
                .await
                .ok()
                .flatten()
                .and_then(|a| a.nick_name.or(a.user_name))
        } else {
            None
        };

        // 查询提交人名字
        let submitter_name = if inst.submitter_name.is_some() {
            inst.submitter_name.clone()
        } else {
            AdminModel::find_by_id(db, &inst.submitter_id)
                .await
                .ok()
                .flatten()
                .and_then(|a| a.nick_name.or(a.user_name))
        };

        // 查询流程节点和边
        let flow_data = Self::find_flow_by_code(db, &inst.flow_code.clone().unwrap_or_default()).await?;
        let (flow_nodes, flow_edges) = if let Some((_flow, nodes, edges)) = flow_data {
            let instance_status = inst.status.unwrap_or(1);
            let current_node_key = inst.current_node_key.clone().unwrap_or_default();

            // Build a set of node keys that have been approved/rejected from logs
            let approved_node_keys: std::collections::HashSet<String> = logs.iter()
                .filter(|l| l.action == Some(1))
                .filter_map(|l| l.node_key.clone())
                .collect();
            let rejected_node_keys: std::collections::HashSet<String> = logs.iter()
                .filter(|l| l.action == Some(2))
                .filter_map(|l| l.node_key.clone())
                .collect();

            let mut node_vos: Vec<ApprovalFlowNodeVO> = Vec::new();
            for n in &nodes {
                let nkey = n.node_key.clone().unwrap_or_default();
                let approver_id = n.approver_id;
                let approver_name = if let Some(aid) = approver_id {
                    AdminModel::find_by_id(db, &Some(aid))
                        .await
                        .ok()
                        .flatten()
                        .and_then(|a| a.nick_name.or(a.user_name))
                } else {
                    None
                };

                let node_status = if rejected_node_keys.contains(&nkey) {
                    3
                } else if approved_node_keys.contains(&nkey) {
                    2
                } else if nkey == current_node_key && (instance_status == 1 || instance_status == 2) {
                    1
                } else if n.node_type == Some(4) && instance_status == 3 {
                    4
                } else if n.node_type == Some(1) {
                    2
                } else {
                    0
                };

                node_vos.push(ApprovalFlowNodeVO {
                    node_key: nkey,
                    node_type: n.node_type.unwrap_or(2),
                    node_name: n.node_name.clone().unwrap_or_default(),
                    node_order: n.node_order.unwrap_or(0),
                    approver_id,
                    approver_name,
                    approve_mode: n.approve_mode.unwrap_or(1),
                    node_status,
                    label: None,
                });
            }

            let edge_vos: Vec<ApprovalFlowEdgeVO> = edges.iter()
                .map(|e| ApprovalFlowEdgeVO {
                    source: e.source_node_key.clone().unwrap_or_default(),
                    target: e.target_node_key.clone().unwrap_or_default(),
                    condition_expr: e.condition_expr.clone(),
                    label: e.label.clone(),
                })
                .collect();

            (node_vos, edge_vos)
        } else {
            (vec![], vec![])
        };

        // 计算每条日志的耗时
        let submitted_time = inst.submitted_at;
        let log_vos: Vec<ApprovalLogVO> = logs.iter()
            .enumerate()
            .map(|(i, l)| {
                let duration = if i == 0 {
                    // 第一条日志与提交时间的差
                    if let (Some(submit_t), Some(log_t)) = (submitted_time, l.create_time) {
                        Some(format_duration(log_t.signed_duration_since(submit_t)))
                    } else {
                        None
                    }
                } else {
                    // 与上一条日志的时间差
                    if let (Some(prev_t), Some(curr_t)) = (logs[i-1].create_time, l.create_time) {
                        Some(format_duration(curr_t.signed_duration_since(prev_t)))
                    } else {
                        None
                    }
                };
                ApprovalLogVO {
                    node_key: l.node_key.clone(),
                    node_name: l.node_name.clone(),
                    approver_id: l.approver_id.unwrap_or_default(),
                    approver_name: l.approver_name.clone(),
                    action: l.action.unwrap_or(0),
                    comment: l.comment.clone(),
                    create_time: l.create_time.map(|t| t.to_string()),
                    duration,
                    target_user_id: l.target_user_id,
                    target_user_name: l.target_user_name.clone(),
                    target_node_key: l.target_node_key.clone(),
                    target_node_name: l.target_node_name.clone(),
                }
            })
            .collect();

        // 解析候选审批人ID列表
        let candidate_approvers: Vec<i64> = inst
            .candidate_approvers
            .as_ref()
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_i64())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // 解析已处理审批人ID列表
        let processed_approvers: Vec<i64> = inst
            .processed_approvers
            .as_ref()
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_i64())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // 查询候选审批人姓名（与ID一一对应）
        let mut candidate_approver_names: Vec<String> = Vec::with_capacity(candidate_approvers.len());
        for cid in &candidate_approvers {
            let name = AdminModel::find_by_id(db, &Some(*cid))
                .await
                .ok()
                .flatten()
                .and_then(|a| a.nick_name.or(a.user_name))
                .unwrap_or_default();
            candidate_approver_names.push(name);
        }

        // 从流程节点中获取当前节点的审批模式
        let current_node_key = inst.current_node_key.clone().unwrap_or_default();
        let approve_mode = flow_nodes
            .iter()
            .find(|n| n.node_key == current_node_key)
            .map(|n| n.approve_mode)
            .unwrap_or(1);

        // 查询抄送列表
        let cc_records = CcEntity::find()
            .filter(CcColumn::InstanceId.eq(id))
            .filter(CcColumn::Deleted.eq(0))
            .order_by_asc(CcColumn::CreateTime)
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        let cc_users: Vec<ApprovalCcVO> = cc_records.iter().map(|c| ApprovalCcVO {
            id: c.id,
            instance_id: c.instance_id.unwrap_or_default(),
            user_id: c.user_id.unwrap_or_default(),
            user_name: c.user_name.clone(),
            cc_from_id: c.cc_from_id,
            cc_from_name: c.cc_from_name.clone(),
            cc_reason: c.cc_reason.clone(),
            is_read: c.is_read.unwrap_or(0),
            read_time: c.read_time.map(|t| t.to_string()),
            create_time: c.create_time.map(|t| t.to_string()),
            business_type: None,
            business_id: None,
            business_title: None,
            submitter_name: None,
            instance_status: None,
        }).collect();

        Ok(Some(ApprovalInstanceVO {
            id: inst.id,
            flow_code: inst.flow_code.unwrap_or_default(),
            business_type: inst.business_type.unwrap_or_default(),
            business_id: inst.business_id.unwrap_or_default(),
            business_title: inst.business_title,
            submitter_id: inst.submitter_id.unwrap_or_default(),
            submitter_name,
            current_node_key: inst.current_node_key,
            current_approver_id: inst.current_approver_id,
            current_approver_name,
            candidate_approvers,
            candidate_approver_names,
            processed_approvers,
            approve_mode,
            status: inst.status.unwrap_or(1),
            submitted_at: inst.submitted_at.map(|t| t.to_string()),
            finished_at: inst.finished_at.map(|t| t.to_string()),
            extra_data: inst.extra_data,
            flow_nodes,
            flow_edges,
            logs: log_vos,
            cancel_reason: inst.cancel_reason,
            transfer_from_id: inst.transfer_from_id,
            delegate_from_id: inst.delegate_from_id,
            add_sign_type: inst.add_sign_type,
            needs_resubmit: inst.needs_resubmit,
            cc_users,
        }))
    }

    pub async fn find_instance_list(
        db: &DatabaseConnection,
        approver_id: i64,
        page_num: u64,
        page_size: u64,
    ) -> Result<ResultPage<Vec<ApprovalInstanceVO>>> {
        Self::find_instance_list_filtered(db, approver_id, None, None, None, page_num, page_size).await
    }

    pub async fn find_instance_list_filtered(
        db: &DatabaseConnection,
        approver_id: i64,
        business_type: Option<&str>,
        status: Option<i32>,
        business_title: Option<&str>,
        page_num: u64,
        page_size: u64,
    ) -> Result<ResultPage<Vec<ApprovalInstanceVO>>> {
        // 查询条件：当前审批人是指定用户 OR 候选审批人池包含该用户（支持或签/会签多审批人场景）
        let candidate_filter = Expr::cust(format!(
            r#""candidate_approvers" @> '[{}]'"#,
            approver_id
        ));
        let mut query = InstanceEntity::find()
            .filter(
                Condition::any()
                    .add(InstanceColumn::CurrentApproverId.eq(approver_id))
                    .add(candidate_filter),
            );

        // 按 status 过滤（默认只看待审批/审批中，传了 status 就按传的查）
        if let Some(s) = status {
            query = query.filter(InstanceColumn::Status.eq(s));
        } else {
            query = query.filter(InstanceColumn::Status.is_in(vec![1, 2]));
        }

        if let Some(bt) = business_type {
            query = query.filter(InstanceColumn::BusinessType.eq(bt));
        }

        if let Some(title) = business_title {
            query = query.filter(InstanceColumn::BusinessTitle.like(format!("%{}%", title)));
        }

        let paginator = query
            .order_by_desc(InstanceColumn::SubmittedAt)
            .paginate(db, page_size);

        let total = paginator
            .num_items()
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        let items = paginator
            .fetch_page(page_num - 1)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let list: Vec<ApprovalInstanceVO> = items
            .into_iter()
            .map(|inst| {
                let candidate_approvers: Vec<i64> = inst
                    .candidate_approvers
                    .as_ref()
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_i64())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let processed_approvers: Vec<i64> = inst
                    .processed_approvers
                    .as_ref()
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_i64())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                ApprovalInstanceVO {
                    id: inst.id,
                    flow_code: inst.flow_code.unwrap_or_default(),
                    business_type: inst.business_type.unwrap_or_default(),
                    business_id: inst.business_id.unwrap_or_default(),
                    business_title: inst.business_title,
                    submitter_id: inst.submitter_id.unwrap_or_default(),
                    submitter_name: inst.submitter_name,
                    current_node_key: inst.current_node_key,
                    current_approver_id: inst.current_approver_id,
                    current_approver_name: None,
                    candidate_approvers,
                    candidate_approver_names: vec![],
                    processed_approvers,
                    approve_mode: 1,
                    status: inst.status.unwrap_or(1),
                    submitted_at: inst.submitted_at.map(|t| t.to_string()),
                    finished_at: inst.finished_at.map(|t| t.to_string()),
                    extra_data: inst.extra_data,
                    flow_nodes: vec![],
                    flow_edges: vec![],
                    logs: vec![],
                    cancel_reason: inst.cancel_reason,
                    transfer_from_id: inst.transfer_from_id,
                    delegate_from_id: inst.delegate_from_id,
                    add_sign_type: inst.add_sign_type,
                    needs_resubmit: inst.needs_resubmit,
                    cc_users: vec![],
                }
            })
            .collect();

        Ok(ResultPage {
            items: list,
            total: total as i64,
            current_page: page_num as i64,
            page_size: page_size as i64,
            total_pages: ((total as f64) / (page_size as f64)).ceil() as i64,
        })
    }

    pub async fn update_instance_node(
        db: &DatabaseConnection,
        instance_id: i64,
        node_key: &str,
        approver_id: i64,
        candidate_approvers: &[i64],
    ) -> Result<()> {
        let now = Utc::now().naive_utc();
        // 候选审批人列表（去重；若为空则退化为 [approver_id]）
        let candidates: Vec<i64> = {
            let mut v: Vec<i64> = if candidate_approvers.is_empty() {
                vec![approver_id]
            } else {
                candidate_approvers.to_vec()
            };
            v.dedup();
            v
        };
        let candidates_json: serde_json::Value =
            serde_json::Value::Array(candidates.iter().map(|id| serde_json::json!(id)).collect());
        InstanceEntity::update_many()
            .col_expr(InstanceColumn::CurrentNodeKey, Expr::value(node_key.to_string()))
            .col_expr(InstanceColumn::CurrentApproverId, Expr::value(approver_id))
            .col_expr(InstanceColumn::CandidateApprovers, Expr::value(candidates_json.clone()))
            .col_expr(InstanceColumn::ProcessedApprovers, Expr::value(serde_json::Value::Array(vec![])))
            .col_expr(InstanceColumn::Status, Expr::value(2))
            .col_expr(InstanceColumn::UpdateTime, Expr::value(now))
            .filter(InstanceColumn::Id.eq(instance_id))
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(())
    }

    /// 追加已处理审批人到 processed_approvers JSON 数组（幂等：已存在则不重复添加）
    /// 返回追加后的已处理列表
    pub async fn append_processed_approver(
        db: &DatabaseConnection,
        instance_id: i64,
        approver_id: i64,
    ) -> Result<Vec<i64>> {
        // 读取当前 processed_approvers
        let inst = InstanceEntity::find_by_id(instance_id)
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
            .ok_or_else(|| Error::from("审批实例不存在"))?;

        let mut processed: Vec<i64> = inst
            .processed_approvers
            .as_ref()
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_i64())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if !processed.contains(&approver_id) {
            processed.push(approver_id);
        }

        let processed_json: serde_json::Value =
            serde_json::Value::Array(processed.iter().map(|id| serde_json::json!(id)).collect());
        let now = Utc::now().naive_utc();
        InstanceEntity::update_many()
            .col_expr(InstanceColumn::ProcessedApprovers, Expr::value(processed_json))
            .col_expr(InstanceColumn::UpdateTime, Expr::value(now))
            .filter(InstanceColumn::Id.eq(instance_id))
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(processed)
    }

    pub async fn finish_instance(db: &DatabaseConnection, instance_id: i64, status: i32) -> Result<()> {
        let now = Utc::now().naive_utc();
        InstanceEntity::update_many()
            .col_expr(InstanceColumn::Status, Expr::value(status))
            .col_expr(InstanceColumn::FinishedAt, Expr::value(now))
            .col_expr(InstanceColumn::UpdateTime, Expr::value(now))
            .filter(InstanceColumn::Id.eq(instance_id))
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(())
    }

    /// 仅更新当前审批人（用于依次审批模式下同节点内流转，不重置候选池/已处理池）
    pub async fn update_current_approver(
        db: &DatabaseConnection,
        instance_id: i64,
        approver_id: i64,
    ) -> Result<()> {
        let now = Utc::now().naive_utc();
        InstanceEntity::update_many()
            .col_expr(InstanceColumn::CurrentApproverId, Expr::value(approver_id))
            .col_expr(InstanceColumn::Status, Expr::value(2))
            .col_expr(InstanceColumn::UpdateTime, Expr::value(now))
            .filter(InstanceColumn::Id.eq(instance_id))
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(())
    }

    pub async fn insert_log(
        db: &DatabaseConnection,
        instance_id: i64,
        node_key: &str,
        node_name: &str,
        req: &ApprovalProcessRequest,
    ) -> Result<()> {
        let now = Utc::now().naive_utc();
        let active = LogActiveModel {
            instance_id: Set(Some(instance_id)),
            node_key: Set(Some(node_key.to_string())),
            node_name: Set(Some(node_name.to_string())),
            approver_id: Set(Some(req.approver_id)),
            approver_name: Set(req.approver_name.clone()),
            action: Set(Some(req.action)),
            comment: Set(req.comment.clone()),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        LogEntity::insert(active)
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(())
    }

    /// 插入带目标用户/节点的审批日志（用于转办/委派/加签/退回/取消）
    pub async fn insert_log_with_target(
        db: &DatabaseConnection,
        instance_id: i64,
        node_key: &str,
        node_name: &str,
        approver_id: i64,
        approver_name: Option<String>,
        action: i32,
        comment: Option<String>,
        target_user_id: Option<i64>,
        target_user_name: Option<String>,
        target_node_key: Option<String>,
        target_node_name: Option<String>,
    ) -> Result<()> {
        let now = Utc::now().naive_utc();
        let active = LogActiveModel {
            instance_id: Set(Some(instance_id)),
            node_key: Set(Some(node_key.to_string())),
            node_name: Set(Some(node_name.to_string())),
            approver_id: Set(Some(approver_id)),
            approver_name: Set(approver_name),
            action: Set(Some(action)),
            comment: Set(comment),
            create_time: Set(Some(now)),
            target_user_id: Set(target_user_id),
            target_user_name: Set(target_user_name),
            target_node_key: Set(target_node_key),
            target_node_name: Set(target_node_name),
            ..Default::default()
        };
        LogEntity::insert(active)
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(())
    }

    /// 更新实例的当前节点和审批人（用于退回/转办/委派/加签）
    pub async fn update_instance_node_with_extras(
        db: &DatabaseConnection,
        instance_id: i64,
        node_key: &str,
        approver_id: i64,
        candidates: &[i64],
        transfer_from_id: Option<i64>,
        delegate_from_id: Option<i64>,
        add_sign_type: Option<i32>,
        needs_resubmit: Option<i32>,
    ) -> Result<()> {
        let now = Utc::now().naive_utc();
        let candidates_json: serde_json::Value =
            serde_json::Value::Array(candidates.iter().map(|id| serde_json::json!(id)).collect());
        let mut query = InstanceEntity::update_many()
            .col_expr(InstanceColumn::CurrentNodeKey, Expr::value(node_key.to_string()))
            .col_expr(InstanceColumn::CurrentApproverId, Expr::value(approver_id))
            .col_expr(InstanceColumn::CandidateApprovers, Expr::value(candidates_json.clone()))
            .col_expr(InstanceColumn::ProcessedApprovers, Expr::value(serde_json::Value::Array(vec![])))
            .col_expr(InstanceColumn::Status, Expr::value(2))
            .col_expr(InstanceColumn::UpdateTime, Expr::value(now));
        if let Some(tf) = transfer_from_id {
            query = query.col_expr(InstanceColumn::TransferFromId, Expr::value(tf));
        }
        if let Some(df) = delegate_from_id {
            query = query.col_expr(InstanceColumn::DelegateFromId, Expr::value(df));
        }
        if let Some(ast) = add_sign_type {
            query = query.col_expr(InstanceColumn::AddSignType, Expr::value(ast));
        }
        if let Some(nr) = needs_resubmit {
            query = query.col_expr(InstanceColumn::NeedsResubmit, Expr::value(nr));
        }
        query
            .filter(InstanceColumn::Id.eq(instance_id))
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(())
    }

    /// 更新实例的取消原因
    pub async fn update_cancel_reason(
        db: &DatabaseConnection,
        instance_id: i64,
        cancel_reason: &str,
    ) -> Result<()> {
        let now = Utc::now().naive_utc();
        InstanceEntity::update_many()
            .col_expr(InstanceColumn::CancelReason, Expr::value(cancel_reason.to_string()))
            .col_expr(InstanceColumn::UpdateTime, Expr::value(now))
            .filter(InstanceColumn::Id.eq(instance_id))
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(())
    }

    /// 批量插入抄送记录
    pub async fn insert_cc_records(
        db: &DatabaseConnection,
        instance_id: i64,
        user_ids: &[i64],
        cc_from_id: Option<i64>,
        cc_from_name: Option<String>,
        cc_reason: Option<String>,
    ) -> Result<()> {
        let now = Utc::now().naive_utc();
        // 查询用户姓名
        let mut user_names: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
        for uid in user_ids {
            if let Ok(Some(admin)) = AdminModel::find_by_id(db, &Some(*uid)).await {
                let name = admin.nick_name.or(admin.user_name).unwrap_or_default();
                user_names.insert(*uid, name);
            }
        }
        let records: Vec<CcActiveModel> = user_ids.iter().map(|uid| {
            let name = user_names.get(uid).cloned().unwrap_or_default();
            CcActiveModel {
                instance_id: Set(Some(instance_id)),
                user_id: Set(Some(*uid)),
                user_name: Set(Some(name)),
                cc_from_id: Set(cc_from_id),
                cc_from_name: Set(cc_from_name.clone()),
                cc_reason: Set(cc_reason.clone()),
                is_read: Set(Some(0)),
                create_time: Set(Some(now)),
                deleted: Set(Some(0)),
                ..Default::default()
            }
        }).collect();
        if records.is_empty() {
            return Ok(());
        }
        CcEntity::insert_many(records)
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(())
    }

    /// 查询用户被抄送的列表（分页）
    pub async fn find_cc_list_for_user(
        db: &DatabaseConnection,
        user_id: i64,
        is_read: Option<i32>,
        page_num: u64,
        page_size: u64,
    ) -> Result<ResultPage<Vec<ApprovalCcVO>>> {
        let mut query = CcEntity::find()
            .filter(CcColumn::UserId.eq(user_id))
            .filter(CcColumn::Deleted.eq(0));
        if let Some(r) = is_read {
            query = query.filter(CcColumn::IsRead.eq(r));
        }
        let total = query.clone()
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))? as i64;
        let records = query
            .order_by_desc(CcColumn::CreateTime)
            .offset((page_num - 1) * page_size)
            .limit(page_size)
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        // 查询关联实例信息
        let instance_ids: Vec<i64> = records.iter()
            .filter_map(|c| c.instance_id)
            .collect();
        let instances = if instance_ids.is_empty() {
            vec![]
        } else {
            InstanceEntity::find()
                .filter(InstanceColumn::Id.is_in(instance_ids))
                .all(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?
        };
        let inst_map: std::collections::HashMap<i64, &InstanceModel> = instances.iter()
            .map(|i| (i.id, i))
            .collect();

        let items: Vec<ApprovalCcVO> = records.iter().map(|c| {
            let inst = c.instance_id.and_then(|iid| inst_map.get(&iid));
            ApprovalCcVO {
                id: c.id,
                instance_id: c.instance_id.unwrap_or_default(),
                user_id: c.user_id.unwrap_or_default(),
                user_name: c.user_name.clone(),
                cc_from_id: c.cc_from_id,
                cc_from_name: c.cc_from_name.clone(),
                cc_reason: c.cc_reason.clone(),
                is_read: c.is_read.unwrap_or(0),
                read_time: c.read_time.map(|t| t.to_string()),
                create_time: c.create_time.map(|t| t.to_string()),
                business_type: inst.and_then(|i| i.business_type.clone()),
                business_id: inst.and_then(|i| i.business_id),
                business_title: inst.and_then(|i| i.business_title.clone()),
                submitter_name: inst.and_then(|i| i.submitter_name.clone()),
                instance_status: inst.and_then(|i| i.status),
            }
        }).collect();

        Ok(ResultPage {
            items,
            total,
            current_page: page_num as i64,
            page_size: page_size as i64,
            total_pages: ((total as f64) / (page_size as f64)).ceil() as i64,
        })
    }

    /// 标记抄送为已读
    pub async fn mark_cc_read(db: &DatabaseConnection, cc_id: i64, user_id: i64) -> Result<()> {
        let now = Utc::now().naive_utc();
        CcEntity::update_many()
            .col_expr(CcColumn::IsRead, Expr::value(1))
            .col_expr(CcColumn::ReadTime, Expr::value(now))
            .filter(CcColumn::Id.eq(cc_id))
            .filter(CcColumn::UserId.eq(user_id))
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(())
    }
    /// 根据节点配置的 approver_type/approver_id 解析出实际审批人ID列表
    /// approver_type: 1=指定用户, 2=指定角色, 3=部门主管, 4=发起人自己, 5=指定岗位, 6=直属上级
    /// type=6 时 approver_id 表示向上查找的层级（默认1=直属上级，2=上级的上级，依此类推）
    /// 返回候选审批人列表（或签/会签模式下均为全部候选；依次审批时按返回顺序处理）
    /// 注意：本函数不过滤发起人自审，调用方需在拿到候选列表后调用 filter_self_approvers 进行回避
    /// 对于 type=6，若到达组织架构顶层（无更高级别上级），返回空列表（调用方应作为自动通过信号处理）
    pub async fn resolve_approvers(
        db: &DatabaseConnection,
        approver_type: Option<i32>,
        approver_id: Option<i64>,
        submitter_id: i64,
        submitter_dept_id: Option<i64>,
    ) -> Result<Vec<i64>> {
        match approver_type.unwrap_or(1) {
            1 => {
                // 指定用户：校验用户存在且启用
                let uid = approver_id
                    .ok_or_else(|| Error::from("审批节点未配置审批人"))?;
                let admin = AdminEntity::find_by_id(uid)
                    .one(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?
                    .ok_or_else(|| Error::from("审批人用户不存在"))?;
                if admin.status.unwrap_or(0) != 1 {
                    return Err(Error::from("审批人用户已停用，请联系管理员"));
                }
                Ok(vec![uid])
            }
            2 => {
                // 指定角色：返回该角色下所有启用用户（过滤 status=1 且 deleted=0）
                let role_id = approver_id
                    .ok_or_else(|| Error::from("审批节点未配置角色"))?;
                let merges = RoleMergeEntity::find()
                    .filter(RoleMergeColumn::RoleId.eq(role_id))
                    .all(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
                let admin_ids: Vec<i64> = merges
                    .into_iter()
                    .filter_map(|m| m.admin_id)
                    .filter(|&id| id > 0)
                    .collect();
                if admin_ids.is_empty() {
                    return Err(Error::from("该角色下未找到审批人"));
                }
                // 批量查询用户状态，仅保留启用的用户
                let active_admins: std::collections::HashSet<i64> = AdminEntity::find()
                    .filter(AdminColumn::Id.is_in(admin_ids.clone()))
                    .filter(AdminColumn::Status.eq(1))
                    .filter(AdminColumn::Deleted.eq(0))
                    .all(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?
                    .into_iter()
                    .map(|a| a.id)
                    .collect();
                let filtered: Vec<i64> = admin_ids.into_iter()
                    .filter(|id| active_admins.contains(id))
                    .collect();
                if filtered.is_empty() {
                    return Err(Error::from("该角色下未找到启用的审批人"));
                }
                Ok(filtered)
            }
            3 => {
                // 部门主管：单个人（部门负责人），校验用户状态
                let dept_id = approver_id.or(submitter_dept_id)
                    .ok_or_else(|| Error::from("无法确定审批部门"))?;
                let dept = DeptEntity::find_by_id(dept_id)
                    .one(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?
                    .ok_or_else(|| Error::from("部门不存在"))?;
                let dept_name = dept.dept_name.clone().unwrap_or_default();
                let leader_id = dept.leader_id
                    .filter(|&id| id > 0)
                    .ok_or_else(|| Error::from(format!("部门[{}]未配置负责人", dept_name)))?;
                // 校验部门负责人是否启用
                let leader = AdminEntity::find_by_id(leader_id)
                    .one(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?
                    .ok_or_else(|| Error::from("部门负责人用户不存在"))?;
                if leader.status.unwrap_or(0) != 1 {
                    return Err(Error::from(format!("部门[{}]负责人已停用，请联系管理员", dept_name)));
                }
                Ok(vec![leader_id])
            }
            4 => {
                // 发起人自己
                Ok(vec![submitter_id])
            }
            5 => {
                // 指定岗位：返回该岗位下所有启用用户（过滤 status=1 且 deleted=0）
                let post_id = approver_id
                    .ok_or_else(|| Error::from("审批节点未配置岗位"))?;
                let merges = PostMergeEntity::find()
                    .filter(PostMergeColumn::PostId.eq(post_id))
                    .all(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
                let admin_ids: Vec<i64> = merges
                    .into_iter()
                    .filter_map(|m| m.admin_id)
                    .filter(|&id| id > 0)
                    .collect();
                if admin_ids.is_empty() {
                    return Err(Error::from("该岗位下未找到审批人"));
                }
                // 批量查询用户状态，仅保留启用的用户
                let active_admins: std::collections::HashSet<i64> = AdminEntity::find()
                    .filter(AdminColumn::Id.is_in(admin_ids.clone()))
                    .filter(AdminColumn::Status.eq(1))
                    .filter(AdminColumn::Deleted.eq(0))
                    .all(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?
                    .into_iter()
                    .map(|a| a.id)
                    .collect();
                let filtered: Vec<i64> = admin_ids.into_iter()
                    .filter(|id| active_admins.contains(id))
                    .collect();
                if filtered.is_empty() {
                    return Err(Error::from("该岗位下未找到启用的审批人"));
                }
                Ok(filtered)
            }
            6 => {
                // 直属上级：根据 approver_id 作为层级（默认1），沿 direct_manager_id 链向上查找
                // 返回空 Vec 表示已到组织架构顶层，调用方应作为"自动通过"信号处理
                let level = approver_id.filter(|&l| l > 0).unwrap_or(1) as usize;
                let mut current_id: Option<i64> = Some(submitter_id);
                let mut visited: std::collections::HashSet<i64> = std::collections::HashSet::new();
                visited.insert(submitter_id);
                for _ in 0..level {
                    let cur = match current_id {
                        Some(id) => id,
                        None => return Ok(Vec::new()),
                    };
                    let admin = AdminEntity::find_by_id(cur)
                        .one(db)
                        .await
                        .map_err(|e| Error::from(e.to_string()))?;
                    let admin = match admin {
                        Some(a) => a,
                        None => return Ok(Vec::new()),
                    };
                    let next = admin.direct_manager_id.filter(|&id| id > 0);
                    match next {
                        Some(mid) => {
                            // 防止循环引用（A→B→A）
                            if !visited.insert(mid) {
                                log::warn!("检测到 direct_manager_id 循环引用: {} -> {}", cur, mid);
                                return Ok(Vec::new());
                            }
                            current_id = Some(mid);
                        }
                        None => {
                            // 当前节点无上级，已到顶层
                            return Ok(Vec::new());
                        }
                    }
                }
                // 最终 current_id 即为指定层级的上级，校验其状态
                match current_id {
                    Some(mid) if mid != submitter_id => {
                        // 校验上级用户是否启用
                        let manager = AdminEntity::find_by_id(mid)
                            .one(db)
                            .await
                            .map_err(|e| Error::from(e.to_string()))?;
                        match manager {
                            Some(m) if m.status.unwrap_or(0) == 1 => Ok(vec![mid]),
                            Some(_) => {
                                // 上级已停用，视为无可用上级，返回空触发自动通过
                                log::warn!("直属上级(id={})已停用，审批节点将自动通过", mid);
                                Ok(Vec::new())
                            }
                            None => Ok(Vec::new()),
                        }
                    }
                    _ => Ok(Vec::new()),
                }
            }
            other => Err(Error::from(format!("不支持的审批人类型: {}", other))),
        }
    }

    /// 自审回避过滤：从候选审批人列表中移除发起人自己
    /// 返回过滤后的列表（不会报错，仅移除匹配项）
    pub fn filter_self_approvers(candidates: Vec<i64>, submitter_id: i64) -> Vec<i64> {
        candidates.into_iter().filter(|&id| id != submitter_id).collect()
    }

    /// 判断节点是否为"直属上级"类型（type=6），用于空候选列表时的自动通过决策
    pub fn is_direct_manager_node(approver_type: Option<i32>) -> bool {
        approver_type == Some(6)
    }

    /// 兼容旧调用：解析单个审批人（返回候选列表的第一个）
    /// 新代码应直接使用 resolve_approvers
    pub async fn resolve_approver(
        db: &DatabaseConnection,
        approver_type: Option<i32>,
        approver_id: Option<i64>,
        submitter_id: i64,
        submitter_dept_id: Option<i64>,
    ) -> Result<i64> {
        let mut list = Self::resolve_approvers(
            db,
            approver_type,
            approver_id,
            submitter_id,
            submitter_dept_id,
        )
        .await?;
        if list.is_empty() {
            return Err(Error::from("未解析到审批人"));
        }
        Ok(list.remove(0))
    }

    /// 查询用户的部门ID
    pub async fn find_user_dept_id(db: &DatabaseConnection, user_id: i64) -> Result<Option<i64>> {
        let merge = DeptMergeEntity::find()
            .filter(DeptMergeColumn::AdminId.eq(user_id))
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(merge.and_then(|m| m.dept_id))
    }
}

fn format_duration(duration: chrono::Duration) -> String {
    let total_secs = duration.num_seconds();
    if total_secs < 0 {
        return "0秒".to_string();
    }
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}天", days));
    }
    if hours > 0 {
        parts.push(format!("{}小时", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}分钟", minutes));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{}秒", seconds));
    }
    parts.join("")
}
