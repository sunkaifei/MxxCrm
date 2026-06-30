use chrono::Utc;
use sea_orm::sea_query::Expr;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::system::model::admin::AdminModel;
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
    pub status: i32,
    pub submitted_at: Option<String>,
    pub finished_at: Option<String>,
    #[serde(skip_serializing)]
    pub extra_data: Option<serde_json::Value>,
    pub flow_nodes: Vec<ApprovalFlowNodeVO>,
    pub flow_edges: Vec<ApprovalFlowEdgeVO>,
    pub logs: Vec<ApprovalLogVO>,
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
}

// ============ Data Access ============

pub struct ApprovalModel;

impl ApprovalModel {
    pub async fn save_flow(db: &DatabaseConnection, req: &FlowSaveRequest, operator: &str) -> Result<i64> {
        let now = Utc::now().naive_utc();

        let flow_id = if let Some(id) = req.flow_id {
            // Update existing flow
            let mut active: FlowActiveModel = FlowEntity::find_by_id(id)
                .one(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?
                .ok_or_else(|| Error::from("审批流不存在"))?
                .into();
            active.flow_code = Set(Some(req.flow_code.clone()));
            active.flow_name = Set(Some(req.flow_name.clone()));
            active.business_type = Set(Some(req.business_type.clone()));
            active.description = Set(req.description.clone());
            active.update_by = Set(Some(operator.to_string()));
            active.update_time = Set(Some(now));
            active.update(db).await.map_err(|e| Error::from(e.to_string()))?;
            id
        } else {
            // Insert new flow
            let active = FlowActiveModel {
                flow_code: Set(Some(req.flow_code.clone())),
                flow_name: Set(Some(req.flow_name.clone())),
                business_type: Set(Some(req.business_type.clone())),
                description: Set(req.description.clone()),
                enabled: Set(Some(1)),
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
            nodes: nodes
                .into_iter()
                .map(|n| NodeVO {
                    node_key: n.node_key.unwrap_or_default(),
                    node_type: n.node_type.unwrap_or(2),
                    node_name: n.node_name.unwrap_or_default(),
                    node_order: n.node_order.unwrap_or(0),
                    approver_type: n.approver_type,
                    approver_id: n.approver_id,
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
    ) -> Result<i64> {
        let now = Utc::now().naive_utc();
        let active = InstanceActiveModel {
            flow_code: Set(Some(req.flow_code.clone())),
            business_type: Set(Some(req.business_type.clone())),
            business_id: Set(Some(req.business_id)),
            business_title: Set(req.business_title.clone()),
            submitter_id: Set(Some(req.submitter_id)),
            submitter_name: Set(req.submitter_name.clone()),
            current_node_key: Set(Some(first_node_key.to_string())),
            current_approver_id: Set(Some(approver_id)),
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
                }
            })
            .collect();

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
            status: inst.status.unwrap_or(1),
            submitted_at: inst.submitted_at.map(|t| t.to_string()),
            finished_at: inst.finished_at.map(|t| t.to_string()),
            extra_data: inst.extra_data,
            flow_nodes,
            flow_edges,
            logs: log_vos,
        }))
    }

    pub async fn find_instance_list(
        db: &DatabaseConnection,
        approver_id: i64,
        page_num: u64,
        page_size: u64,
    ) -> Result<ResultPage<Vec<ApprovalInstanceVO>>> {
        let paginator = InstanceEntity::find()
            .filter(InstanceColumn::CurrentApproverId.eq(approver_id))
            .filter(InstanceColumn::Status.is_in(vec![1, 2]))
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
            .map(|inst| ApprovalInstanceVO {
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
                status: inst.status.unwrap_or(1),
                submitted_at: inst.submitted_at.map(|t| t.to_string()),
                finished_at: inst.finished_at.map(|t| t.to_string()),
                extra_data: inst.extra_data,
                flow_nodes: vec![],
                flow_edges: vec![],
                logs: vec![],
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
    ) -> Result<()> {
        let now = Utc::now().naive_utc();
        InstanceEntity::update_many()
            .col_expr(InstanceColumn::CurrentNodeKey, Expr::value(node_key.to_string()))
            .col_expr(InstanceColumn::CurrentApproverId, Expr::value(approver_id))
            .col_expr(InstanceColumn::Status, Expr::value(2))
            .col_expr(InstanceColumn::UpdateTime, Expr::value(now))
            .filter(InstanceColumn::Id.eq(instance_id))
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(())
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
    /// 根据节点配置的 approver_type/approver_id 解析出实际审批人ID
    /// approver_type: 1=指定用户, 2=指定角色, 3=部门主管, 4=发起人自己
    pub async fn resolve_approver(
        db: &DatabaseConnection,
        approver_type: Option<i32>,
        approver_id: Option<i64>,
        submitter_id: i64,
        submitter_dept_id: Option<i64>,
    ) -> Result<i64> {
        match approver_type.unwrap_or(1) {
            1 => {
                // 指定用户
                approver_id.ok_or_else(|| Error::from("审批节点未配置审批人"))
            }
            2 => {
                // 指定角色：找到该角色下任意一个启用的用户
                let role_id = approver_id.ok_or_else(|| Error::from("审批节点未配置角色"))?;
                let merge = RoleMergeEntity::find()
                    .filter(RoleMergeColumn::RoleId.eq(role_id))
                    .one(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
                let admin_id = merge
                    .map(|m| m.admin_id.unwrap_or_default())
                    .filter(|&id| id > 0)
                    .ok_or_else(|| Error::from("该角色下未找到审批人"))?;
                Ok(admin_id)
            }
            3 => {
                // 部门主管：若节点配置了 dept_id 则用该部门，否则用发起人所在部门
                let dept_id = approver_id.or(submitter_dept_id)
                    .ok_or_else(|| Error::from("无法确定审批部门"))?;
                let dept = DeptEntity::find_by_id(dept_id)
                    .one(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?
                    .ok_or_else(|| Error::from("部门不存在"))?;
                dept.leader_id
                    .filter(|&id| id > 0)
                    .ok_or_else(|| Error::from(format!("部门[{}]未配置负责人", dept.dept_name.unwrap_or_default())))
            }
            4 => {
                // 发起人自己
                Ok(submitter_id)
            }
            5 => {
                // 指定岗位：找到该岗位下任意一个启用的用户
                let post_id = approver_id.ok_or_else(|| Error::from("审批节点未配置岗位"))?;
                let merge = PostMergeEntity::find()
                    .filter(PostMergeColumn::PostId.eq(post_id))
                    .one(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
                let admin_id = merge
                    .map(|m| m.admin_id.unwrap_or_default())
                    .filter(|&id| id > 0)
                    .ok_or_else(|| Error::from("该岗位下未找到审批人"))?;
                Ok(admin_id)
            }
            other => Err(Error::from(format!("不支持的审批人类型: {}", other))),
        }
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
