use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sea_orm::sea_query::Expr;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
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
use crate::modules::system::entity::post::{Column as PostColumn, Entity as PostEntity};
use crate::modules::system::entity::salary_band::{Column as BandColumn, Entity as BandEntity};
use crate::modules::system::entity::hire_salary_data::{Column as HsdColumn, Entity as HsdEntity};
use crate::utils::string_utils::{
    deserialize_string_or_number_to_i64, deserialize_string_to_i32,
    deserialize_string_vec_to_u64_vec,
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
    pub cc_user_ids: Option<serde_json::Value>,
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
    pub cc_user_ids: Option<serde_json::Value>,
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
    /// 业务ID：兼容前端字符串 "6" 与数字 6 两种格式（如员工列表 row.id 为字符串）
    #[serde(deserialize_with = "deserialize_string_or_number_to_i64")]
    pub business_id: i64,
    pub business_title: Option<String>,
    /// 提交人ID：前端无需传，controller 从 JWT 提取覆盖（防伪造发起人）
    #[serde(default)]
    pub submitter_id: i64,
    pub submitter_name: Option<String>,
    pub extra_data: Option<serde_json::Value>,
    /// 提交时指定抄送人（可选，为空则不抄送）
    #[serde(deserialize_with = "deserialize_string_vec_to_u64_vec", default)]
    pub cc_user_ids: Option<Vec<i64>>,
    /// 抄送说明（可选）
    pub cc_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ApprovalProcessRequest {
    pub instance_id: i64,
    pub action: i32,
    pub approver_id: i64,
    pub approver_name: Option<String>,
    pub comment: Option<String>,

    // ===== 入职定薪专用（hire_approval 各环节通过时按节点填写，其他流程忽略）=====
    /// 建议工资（部门经理填）
    pub suggested_salary: Option<Decimal>,
    /// 试用期月数（部门经理填）
    #[serde(default, deserialize_with = "deserialize_string_to_i32")]
    pub probation_months: Option<i32>,
    /// 工作能力评估（部门经理填，仅审批人可见）
    pub ability_assessment: Option<String>,
    /// 带宽评估：1带宽内 2超带宽（人事填）
    #[serde(default, deserialize_with = "deserialize_string_to_i32")]
    pub band_status: Option<i32>,
    /// 超带宽原因（人事填）
    pub band_reason: Option<String>,
    /// 谈定工资（人事填，与候选人协商确定的月工资金额）
    pub negotiated_salary: Option<Decimal>,
    /// 试用期工资比例（人事填）
    pub probation_ratio: Option<Decimal>,
    /// CEO终审意见/特批说明（CEO填）
    pub ceo_opinion: Option<String>,
    /// 最终定薪（财务填）
    pub final_salary: Option<Decimal>,
    /// 生效日期（财务填）
    pub effective_date: Option<NaiveDate>,
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
    /// 岗位薪资带宽参照（入职定薪专用）
    pub salary_band: Option<SalaryBandInfoVO>,
    /// 各环节定薪填写数据（入职定薪专用，按环节正序）
    pub hire_salary_stages: Vec<HireSalaryStageVO>,
    /// 已办视角：当前查询人在这张单上的最新处理动作（1通过/2驳回/3转办/4委派/5加签/6退回/7撤回；其余视角为空）
    pub my_action: Option<i32>,
}

/// 岗位薪资带宽参照（入职定薪详情展示：岗位带宽区间）
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SalaryBandInfoVO {
    pub post_id: Option<i64>,
    pub post_name: Option<String>,
    pub min_salary: Option<Decimal>,
    pub max_salary: Option<Decimal>,
}

/// 入职定薪环节数据（按 instance_id + 节点记录各环节填写的结构化字段）
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct HireSalaryStageVO {
    pub node_key: Option<String>,
    /// 1部门经理 2人事 3CEO 4财务
    pub stage: i32,
    /// 建议工资（部门经理）
    pub suggested_salary: Option<Decimal>,
    /// 试用期月数（部门经理）
    pub probation_months: Option<i32>,
    /// 工作能力评估（部门经理，仅审批人可见）
    pub ability_assessment: Option<String>,
    /// 带宽评估：1带宽内 2超带宽（人事）
    pub band_status: Option<i32>,
    /// 超带宽原因（人事）
    pub band_reason: Option<String>,
    /// 谈定工资（人事）
    pub negotiated_salary: Option<Decimal>,
    /// 试用期工资比例（人事）
    pub probation_ratio: Option<Decimal>,
    /// CEO终审意见/特批说明（CEO）
    pub ceo_opinion: Option<String>,
    /// 最终定薪（财务）
    pub final_salary: Option<Decimal>,
    /// 生效日期（财务）
    pub effective_date: Option<String>,
    /// 填写人ID
    pub approver_id: Option<i64>,
    /// 填写人姓名
    pub approver_name: Option<String>,
    /// 审批意见
    pub comment: Option<String>,
    /// 填写时间
    pub create_time: Option<String>,
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
        use sea_orm::TransactionTrait;
        let now = Utc::now().naive_utc();
        let req = (*req).clone();
        let operator = operator.to_string();

        db.transaction::<_, i64, Error>(|txn| {
            Box::pin(async move {
        let flow_id = if let Some(id) = req.flow_id {
            // Update existing flow
            let existing = FlowEntity::find_by_id(id)
                .one(txn)
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
            active.update(txn).await.map_err(|e| Error::from(e.to_string()))?;
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
                .exec(txn)
                .await
                .map_err(|e| Error::from(e.to_string()))?;
            result.last_insert_id
        };

        // Delete old nodes and edges
        NodeEntity::delete_many()
            .filter(NodeColumn::FlowId.eq(flow_id))
            .exec(txn)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        EdgeEntity::delete_many()
            .filter(EdgeColumn::FlowId.eq(flow_id))
            .exec(txn)
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
                cc_user_ids: Set(node.cc_user_ids.clone()),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            NodeEntity::insert(active)
                .exec(txn)
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
                .exec(txn)
                .await
                .map_err(|e| Error::from(e.to_string()))?;
        }

        Ok(flow_id)
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))
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

        Ok(Some(Self::build_flow_detail_vo(flow, nodes, edges)))
    }

    /// 按流程编码查询启用的审批流详情（供业务模块提交审批前预览流程，无需 system:approval:list 权限）
    pub async fn find_flow_vo_by_code(
        db: &impl ConnectionTrait,
        code: &str,
    ) -> Result<Option<FlowDetailVO>> {
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
            .order_by_asc(NodeColumn::NodeOrder)
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let edges = EdgeEntity::find()
            .filter(EdgeColumn::FlowId.eq(flow_id))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        Ok(Some(Self::build_flow_detail_vo(flow, nodes, edges)))
    }

    fn build_flow_detail_vo(
        flow: FlowModel,
        nodes: Vec<NodeModel>,
        edges: Vec<EdgeModel>,
    ) -> FlowDetailVO {
        FlowDetailVO {
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
                    cc_user_ids: n.cc_user_ids,
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
        }
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
        db: &impl ConnectionTrait,
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
        db: &impl ConnectionTrait,
        req: &ApprovalSubmitRequest,
        first_node_key: &str,
        approver_id: i64,
        candidate_approvers: &[i64],
        flow_snapshot: Option<serde_json::Value>,
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
            flow_snapshot: Set(flow_snapshot),
            flow_version: Set(Some(1)),
            ..Default::default()
        };
        // 数据库层幂等兜底：并发双击/重试窗口内命中部分唯一索引 uq_approval_instance_active_business（见 sql/d14），
        // 统一在此转译为业务友好提示，覆盖 submit() 全部三个创建实例出口及未来新增调用方
        let result = InstanceEntity::insert(active)
            .exec(db)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("duplicate key value violates unique constraint")
                    && msg.contains("uq_approval_instance_active_business")
                {
                    Error::from(ApprovalModel::active_dup_hint())
                } else {
                    Error::from(msg)
                }
            })?;
        Ok(result.last_insert_id)
    }

    /// 在途实例唯一冲突的兜底文案（数据库部分唯一索引触发时的通用提示）
    pub fn active_dup_hint() -> String {
        "该记录已存在进行中的审批流程，请勿重复提交；如需重新发起，请先撤回当前流程后再提交"
            .to_string()
    }

    /// 从快照 JSON 中解析出 flow nodes 和 edges
    /// 用于在途审批实例优先读取快照（而非实时查模板表），防止模板修改影响在途实例
    pub fn parse_flow_snapshot(
        snapshot: &serde_json::Value,
    ) -> Result<(Vec<NodeModel>, Vec<EdgeModel>)> {
        let nodes_val = snapshot.get("nodes")
            .ok_or_else(|| Error::from("快照缺少 nodes 字段"))?;
        let edges_val = snapshot.get("edges")
            .ok_or_else(|| Error::from("快照缺少 edges 字段"))?;

        let nodes: Vec<NodeModel> = serde_json::from_value(nodes_val.clone())
            .map_err(|e| Error::from(format!("快照 nodes 反序列化失败: {}", e)))?;
        let edges: Vec<EdgeModel> = serde_json::from_value(edges_val.clone())
            .map_err(|e| Error::from(format!("快照 edges 反序列化失败: {}", e)))?;
        Ok((nodes, edges))
    }

    /// 获取实例的流程数据：优先从快照读取，无快照时回退到实时查询模板表（兼容旧实例）
    pub async fn get_instance_flow_data(
        db: &impl ConnectionTrait,
        instance: &InstanceModel,
    ) -> Result<(Vec<NodeModel>, Vec<EdgeModel>)> {
        if let Some(ref snapshot) = instance.flow_snapshot {
            return Self::parse_flow_snapshot(snapshot);
        }
        // 兼容旧实例（无快照）：实时查询模板表
        let flow_data = Self::find_flow_by_code(
            db,
            &instance.flow_code.clone().unwrap_or_default(),
        ).await?;
        match flow_data {
            Some((_flow, nodes, edges)) => Ok((nodes, edges)),
            None => Err(Error::from("审批流模板不存在")),
        }
    }

    /// 查询实例原始数据（InstanceModel），用于 process 等需要读取快照的场景
    pub async fn find_instance_by_id_raw(db: &impl ConnectionTrait, id: i64) -> Result<Option<InstanceModel>> {
        InstanceEntity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))
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

        // 查询提交人姓名（始终从 admin 表取真实姓名，优先 nick_name）
        let submitter_name = AdminModel::find_by_id(db, &inst.submitter_id)
            .await
            .ok()
            .flatten()
            .and_then(|a| a.nick_name.or(a.user_name));

        // 查询流程节点和边：优先从快照读取，无快照时回退到实时查询模板表
        let (nodes, edges) = Self::get_instance_flow_data(db, &inst).await?;
        let (flow_nodes, flow_edges) = {
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

                // 从审批日志中找当前节点的实际审批人（已审批/驳回的节点）
                let log_approver: Option<i64> = logs.iter()
                    .filter(|l| l.node_key.as_deref() == Some(&nkey))
                    .filter(|l| l.action == Some(1) || l.action == Some(2))
                    .last()
                    .and_then(|l| l.approver_id);

                // 当前进行中的节点：用实例的 current_approver_id
                let is_current_node = nkey == current_node_key && (instance_status == 1 || instance_status == 2);
                let instance_approver_id = if is_current_node { inst.current_approver_id } else { None };

                // 提前计算节点状态（approver_name 解析依赖此值）
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

                // 优先用日志中的实际审批人，其次用实例当前审批人
                // 注意：type=6/7 的 n.approver_id 是层级数(1/2)，不是 userId，不能直接用于查用户名
                let approver_type = n.approver_type.unwrap_or(1);
                let is_dynamic_type = approver_type == 6 || approver_type == 7;
                
                // 静态类型可以直接用配置的 approver_id
                let approver_id = if is_dynamic_type {
                    log_approver.or(instance_approver_id)
                } else {
                    log_approver.or(instance_approver_id).or(n.approver_id)
                };
                
                // 如果动态类型且没有已知的审批人（未流转节点），提前解析
                let approver_id = if approver_id.is_none() && is_dynamic_type && node_status == 0 {
                    // 用提交人信息预解析 type=6/7 节点的审批人
                    let submitter_id = inst.submitter_id.unwrap_or(0);
                    if submitter_id > 0 {
                        match Self::resolve_approvers(db, n.approver_type, n.approver_id, submitter_id, None).await {
                            Ok(candidates) if !candidates.is_empty() => Some(candidates[0]),
                            Ok(_) => Some(0), // 空列表 = 到顶自动通过
                            Err(_) => None,
                        }
                    } else {
                        None
                    }
                } else {
                    approver_id
                };
                
                let approver_name = if let Some(aid) = approver_id {
                    if aid > 0 {
                        AdminModel::find_by_id(db, &Some(aid))
                            .await
                            .ok()
                            .flatten()
                            .and_then(|a| a.nick_name.or(a.user_name))
                    } else {
                        Some("系统自动通过".to_string())
                    }
                } else {
                    None
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
        };

        // 计算每条日志的耗时
        let submitted_time = inst.submitted_at;

        // 批量查询审批人姓名（始终从 admin 表取真实姓名）
        let log_approver_ids: Vec<i64> = logs.iter().filter_map(|l| l.approver_id).collect::<std::collections::HashSet<_>>().into_iter().collect();
        let approver_name_map: std::collections::HashMap<i64, String> = if !log_approver_ids.is_empty() {
            AdminEntity::find()
                .filter(AdminColumn::Id.is_in(log_approver_ids))
                .all(db)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|a| (a.id, a.nick_name.or(a.user_name).unwrap_or_default()))
                .collect()
        } else {
            std::collections::HashMap::new()
        };

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
                    approver_name: l.approver_id.and_then(|aid| approver_name_map.get(&aid).cloned()).or_else(|| l.approver_name.clone()),
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

        // 入职定薪：带宽参照 + 各环节定薪数据（仅 hire_approval）
        let (salary_band, hire_salary_stages) =
            Self::load_hire_salary_context(db, &inst).await.unwrap_or((None, vec![]));

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
            salary_band,
            hire_salary_stages,
            my_action: None,
        }))
    }

    /// 加载入职定薪上下文：岗位薪资带宽参照 + 各环节定薪数据（仅 hire_approval 流程）
    async fn load_hire_salary_context(
        db: &impl ConnectionTrait,
        inst: &InstanceModel,
    ) -> Result<(Option<SalaryBandInfoVO>, Vec<HireSalaryStageVO>)> {
        if inst.flow_code.as_deref() != Some("hire_approval") {
            return Ok((None, vec![]));
        }

        // 1. 带宽参照：员工岗位 -> 启用的薪资带宽
        let employee_id = inst.business_id.unwrap_or_default();
        let salary_band = if employee_id > 0 {
            let post_id = PostMergeEntity::find()
                .filter(PostMergeColumn::AdminId.eq(employee_id))
                .one(db)
                .await
                .ok()
                .flatten()
                .and_then(|m| m.post_id);
            if let Some(pid) = post_id {
                let band = BandEntity::find()
                    .filter(BandColumn::PostId.eq(pid))
                    .filter(BandColumn::Status.eq(1))
                    .filter(BandColumn::Deleted.eq(0))
                    .one(db)
                    .await
                    .ok()
                    .flatten();
                if let Some(b) = band {
                    let post_name = PostEntity::find()
                        .filter(PostColumn::Id.eq(pid))
                        .one(db)
                        .await
                        .ok()
                        .flatten()
                        .and_then(|p| p.post_name);
                    Some(SalaryBandInfoVO {
                        post_id: Some(pid),
                        post_name,
                        min_salary: Some(b.min_salary),
                        max_salary: Some(b.max_salary),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // 2. 各环节定薪数据（按环节正序）
        let stages = HsdEntity::find()
            .filter(HsdColumn::InstanceId.eq(inst.id))
            .order_by_asc(HsdColumn::Stage)
            .all(db)
            .await
            .map_err(|e| Error::from(format!("查询定薪环节数据失败: {}", e)))?;

        // 批量查询填写人姓名
        let approver_ids: Vec<i64> = stages
            .iter()
            .filter_map(|s| s.approver_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let approver_name_map: std::collections::HashMap<i64, String> = if approver_ids.is_empty() {
            std::collections::HashMap::new()
        } else {
            AdminEntity::find()
                .filter(AdminColumn::Id.is_in(approver_ids))
                .all(db)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|a| (a.id, a.nick_name.or(a.user_name).unwrap_or_default()))
                .collect()
        };

        let stage_vos: Vec<HireSalaryStageVO> = stages
            .into_iter()
            .map(|s| HireSalaryStageVO {
                node_key: s.node_key,
                stage: s.stage,
                suggested_salary: s.suggested_salary,
                probation_months: s.probation_months,
                ability_assessment: s.ability_assessment,
                band_status: s.band_status,
                band_reason: s.band_reason,
                negotiated_salary: s.negotiated_salary,
                probation_ratio: s.probation_ratio,
                ceo_opinion: s.ceo_opinion,
                final_salary: s.final_salary,
                effective_date: s.effective_date.map(|d| d.format("%Y-%m-%d").to_string()),
                approver_id: s.approver_id,
                approver_name: s
                    .approver_id
                    .and_then(|aid| approver_name_map.get(&aid).cloned()),
                comment: s.comment,
                create_time: s
                    .create_time
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            })
            .collect();

        Ok((salary_band, stage_vos))
    }

    pub async fn find_instance_list(
        db: &DatabaseConnection,
        approver_id: i64,
        page_num: u64,
        page_size: u64,
    ) -> Result<ResultPage<Vec<ApprovalInstanceVO>>> {
        Self::find_instance_list_filtered(db, approver_id, None, None, None, page_num, page_size).await
    }

    /// 查询某业务单据的全部审批实例（历史），按提交时间正序
    /// 用于发票"流转记录"按单据维度聚合展示：历次提交（含已驳回/已撤回的旧实例）全部保留、可完整追溯。
    pub async fn find_instance_history(
        db: &DatabaseConnection,
        business_type: &str,
        business_id: i64,
    ) -> Result<Vec<ApprovalInstanceVO>> {
        let ids: Vec<i64> = InstanceEntity::find()
            .filter(InstanceColumn::BusinessType.eq(business_type))
            .filter(InstanceColumn::BusinessId.eq(business_id))
            .order_by_asc(InstanceColumn::SubmittedAt)
            .order_by_asc(InstanceColumn::Id)
            .all(db)
            .await
            .map_err(|e| Error::from(format!("查询审批实例历史失败: {}", e)))?
            .into_iter()
            .map(|i| i.id)
            .collect();

        let mut instances = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(vo) = Self::find_instance_by_id(db, id).await? {
                instances.push(vo);
            }
        }
        Ok(instances)
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
                    salary_band: None,
                    hire_salary_stages: vec![],
                    my_action: None,
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

    /// 已办列表：我处理过的全部实例（审批日志中存在我的操作记录），任意状态
    /// 支持 business_type/status/business_title 筛选，VO 用 find_instance_by_id 补全详情所需字段
    pub async fn find_done_instance_list(
        db: &DatabaseConnection,
        approver_id: i64,
        business_type: Option<&str>,
        status: Option<i32>,
        business_title: Option<&str>,
        page_num: u64,
        page_size: u64,
    ) -> Result<ResultPage<Vec<ApprovalInstanceVO>>> {
        // 我处理过的实例ID集合（去重），并记录我在每张单上的最新动作（日志按主键递增，后写覆盖）
        let mut handled_ids: Vec<i64> = vec![];
        let mut my_actions: HashMap<i64, i32> = HashMap::new();
        let my_logs = LogEntity::find()
            .filter(LogColumn::ApproverId.eq(approver_id))
            .order_by_asc(LogColumn::Id)
            .all(db)
            .await
            .map_err(|e| Error::from(format!("查询已办审批实例失败: {}", e)))?;
        for l in my_logs {
            if let Some(iid) = l.instance_id {
                if !handled_ids.contains(&iid) {
                    handled_ids.push(iid);
                }
                if let Some(act) = l.action {
                    my_actions.insert(iid, act);
                }
            }
        }

        if handled_ids.is_empty() {
            return Ok(ResultPage {
                items: vec![],
                total: 0,
                current_page: page_num as i64,
                page_size: page_size as i64,
                total_pages: 0,
            });
        }

        let mut query = InstanceEntity::find().filter(InstanceColumn::Id.is_in(handled_ids));
        if let Some(s) = status {
            query = query.filter(InstanceColumn::Status.eq(s));
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

        let mut list = Vec::with_capacity(items.len());
        for inst in items {
            if let Some(mut vo) = Self::find_instance_by_id(db, inst.id).await? {
                // 已办视角附加：当前查询人在这张单上的最新处理动作
                vo.my_action = my_actions.get(&inst.id).copied();
                list.push(vo);
            }
        }
        Ok(ResultPage {
            items: list,
            total: total as i64,
            current_page: page_num as i64,
            page_size: page_size as i64,
            total_pages: ((total as f64) / (page_size as f64)).ceil() as i64,
        })
    }

    pub async fn update_instance_node(
        db: &impl ConnectionTrait,
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
        db: &impl ConnectionTrait,
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

    pub async fn finish_instance(db: &impl ConnectionTrait, instance_id: i64, status: i32) -> Result<()> {
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
        db: &impl ConnectionTrait,
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
        db: &impl ConnectionTrait,
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
        db: &impl ConnectionTrait,
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
        db: &impl ConnectionTrait,
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
        db: &impl ConnectionTrait,
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
        db: &impl ConnectionTrait,
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
            if let Ok(Some(admin)) = AdminEntity::find_by_id(*uid).one(db).await {
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
    /// approver_type: 1=指定用户, 2=指定角色, 3=部门主管, 4=发起人自己, 5=指定岗位, 6=直属上级, 7=部门主管链
    /// type=6 时 approver_id 表示向上查找的层级（默认1=直属上级，2=上级的上级，依此类推；0=连续逐级直到顶层）
    /// type=7 时 approver_id 表示部门树向上层级（语义同 type=6，但沿 dept.parent_id 链查找部门负责人）
    /// 超管(user_type=1)和已停用用户自动跳过；负责人空缺时向上跳一级（空缺容错）
    /// 返回候选审批人列表；空列表表示已到顶层，调用方应作为"自动通过"信号处理
    pub async fn resolve_approvers(
        db: &impl ConnectionTrait,
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
                    // 角色下无启用用户：返回空 Vec 触发跳过（而非报错）
                    log::warn!("角色(id={})下无启用的审批人，审批节点将跳过", role_id);
                    return Ok(Vec::new());
                }
                Ok(filtered)
            }
            3 => {
                // 部门主管：单个人（部门负责人），校验用户状态
                // 空缺容错：负责人未配置/已停用/超管 → 返回空 Vec 触发跳过（而非报错）
                let dept_id = approver_id.or(submitter_dept_id)
                    .ok_or_else(|| Error::from("无法确定审批部门"))?;
                let dept = DeptEntity::find_by_id(dept_id)
                    .one(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?
                    .ok_or_else(|| Error::from("部门不存在"))?;
                let dept_name = dept.dept_name.clone().unwrap_or_default();
                let leader_id = dept.leader_id.filter(|&id| id > 0);
                match leader_id {
                    Some(lid) => {
                        let leader = AdminEntity::find_by_id(lid)
                            .one(db)
                            .await
                            .map_err(|e| Error::from(e.to_string()))?;
                        match leader {
                            Some(a) if a.status.unwrap_or(0) == 1 && a.user_type.unwrap_or(0) != 1 => Ok(vec![lid]),
                            _ => {
                                log::warn!("部门[{}]负责人不可用或为超管，审批节点将跳过", dept_name);
                                Ok(Vec::new())
                            }
                        }
                    }
                    None => {
                        log::warn!("部门[{}]未配置负责人，审批节点将跳过", dept_name);
                        Ok(Vec::new())
                    }
                }
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
                    // 岗位下无启用用户：返回空 Vec 触发跳过（而非报错）
                    log::warn!("岗位(id={})下无启用的审批人，审批节点将跳过", post_id);
                    return Ok(Vec::new());
                }
                Ok(filtered)
            }
            6 => {
                // 直属上级：沿 direct_manager_id 链向上查找
                // approver_id = 层级数（默认1=直属上级，2=上级的上级）
                // level=0 表示沿链向上查找首个有效上级（带容错），到顶层仍无则返回空
                // 超管(user_type=1)和已停用用户自动跳过
                // 返回空 Vec 表示已到顶层无可用审批人，调用方作为"自动通过"信号处理
                let level_raw = approver_id.unwrap_or(1);
                let target_level: usize = if level_raw <= 0 { 1 } else { level_raw as usize };

                let mut current_id: Option<i64> = Some(submitter_id);
                let mut visited: std::collections::HashSet<i64> = std::collections::HashSet::new();
                visited.insert(submitter_id);

                // Step 1: 沿链向上走 target_level 步（跳过超管/停用）
                for _ in 0..target_level {
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
                            if !visited.insert(mid) {
                                log::warn!("检测到 direct_manager_id 循环引用: {} -> {}", cur, mid);
                                return Ok(Vec::new());
                            }
                            current_id = Some(mid);
                        }
                        None => return Ok(Vec::new()), // 未到目标层级已到顶
                    }
                }

                // Step 2: 从目标层级开始，向上查找首个有效审批人
                // 不参与业务（超管/显式关闭）/停用/是发起人自己 → 向上跳一级继续查找
                let safety_max = 40usize;
                for _ in 0..safety_max {
                    let cur = match current_id {
                        Some(id) => id,
                        None => return Ok(Vec::new()),
                    };
                    if cur == submitter_id {
                        // 回到发起人，到顶
                        return Ok(Vec::new());
                    }
                    let manager = AdminEntity::find_by_id(cur)
                        .one(db)
                        .await
                        .map_err(|e| Error::from(e.to_string()))?;
                    match manager {
                        Some(m) if m.status.unwrap_or(0) == 1 && m.is_biz_participant() => {
                            return Ok(vec![cur]);
                        }
                        Some(m) if !m.is_biz_participant() => {
                            log::info!("直属上级(id={})不参与业务，自动跳过继续向上", cur);
                        }
                        Some(_) => {
                            log::info!("直属上级(id={})已停用，自动跳过继续向上", cur);
                        }
                        None => return Ok(Vec::new()),
                    }
                    // 向上一级
                    let admin = AdminEntity::find_by_id(cur)
                        .one(db)
                        .await
                        .map_err(|e| Error::from(e.to_string()))?;
                    match admin.and_then(|a| a.direct_manager_id).filter(|&id| id > 0 && !visited.contains(&id)) {
                        Some(mid) => {
                            visited.insert(mid);
                            current_id = Some(mid);
                        }
                        None => return Ok(Vec::new()),
                    }
                }
                Ok(Vec::new())
            }
            7 => {
                // 部门主管链：沿 dept.parent_id 链向上，取部门负责人
                // approver_id = 层级数（默认1=直属部门负责人，2=上级部门负责人）
                // level=0 表示从直属部门开始，沿链向上查找首个有效负责人
                // 空缺容错：负责人未配置/已停用/超管/是发起人自己 → 自动向上跳一级继续查找
                // 返回空 Vec 表示已到顶层无可用审批人，调用方作为"自动通过"信号处理
                let level_raw = approver_id.unwrap_or(1);
                // level=0 等价于 level=1（找直属部门负责人，配合空缺容错向上递进）
                let target_level: usize = if level_raw <= 0 { 1 } else { level_raw as usize };

                let start_dept_id = submitter_dept_id
                    .ok_or_else(|| Error::from("无法确定发起人所属部门，请先为用户分配部门"))?;

                // Step 1: 沿 parent_id 链向上走 target_level 步，定位到目标部门
                let mut current_dept_id = start_dept_id;
                let mut visited = std::collections::HashSet::new();
                visited.insert(start_dept_id);

                for _ in 0..target_level {
                    let dept = DeptEntity::find_by_id(current_dept_id)
                        .one(db)
                        .await
                        .map_err(|e| Error::from(e.to_string()))?
                        .ok_or_else(|| Error::from("部门不存在"))?;
                    match dept.parent_id.filter(|&id| id > 0 && !visited.contains(&id)) {
                        Some(pid) => {
                            visited.insert(pid);
                            current_dept_id = pid;
                        }
                        None => return Ok(Vec::new()), // 已经到顶，没找到目标层级
                    }
                }

                // Step 2: 从目标部门开始，向上查找首个有效负责人
                // 处理空缺容错：未配置/不参与业务（超管/显式关闭）/停用/是发起人自己 → 向上跳一级
                let mut search_dept_id = current_dept_id;
                let mut search_visited = std::collections::HashSet::new();
                search_visited.insert(current_dept_id);
                let safety_max = 40usize; // 安全上限防死循环

                for _ in 0..safety_max {
                    let dept = DeptEntity::find_by_id(search_dept_id)
                        .one(db)
                        .await
                        .map_err(|e| Error::from(e.to_string()))?
                        .ok_or_else(|| Error::from("部门不存在"))?;

                    let dept_name = dept.dept_name.clone().unwrap_or_default();
                    let leader_id = dept.leader_id.filter(|&id| id > 0);

                    // 检查当前部门负责人是否可用
                    let mut found_valid = false;
                    let mut result_id: Option<i64> = None;
                    if let Some(lid) = leader_id {
                        if lid == submitter_id {
                            log::info!("部门[{}]负责人是发起人自己，向上跳一级", dept_name);
                        } else {
                            let leader = AdminEntity::find_by_id(lid)
                                .one(db)
                                .await
                                .map_err(|e| Error::from(e.to_string()))?;
                            match leader {
                                Some(a) if a.status.unwrap_or(0) == 1 && a.is_biz_participant() => {
                                    found_valid = true;
                                    result_id = Some(lid);
                                }
                                Some(a) if !a.is_biz_participant() => {
                                    log::info!("部门[{}]负责人不参与业务，自动跳过继续向上", dept_name);
                                }
                                _ => {
                                    log::warn!("部门[{}]负责人不可用，向上跳一级", dept_name);
                                }
                            }
                        }
                    } else {
                        log::warn!("部门[{}]未配置负责人，向上跳一级", dept_name);
                    }

                    if found_valid {
                        return Ok(vec![result_id.unwrap()]);
                    }

                    // 向上一级
                    match dept.parent_id.filter(|&id| id > 0 && !search_visited.contains(&id)) {
                        Some(pid) => {
                            search_visited.insert(pid);
                            search_dept_id = pid;
                        }
                        None => return Ok(Vec::new()), // 到顶，无可用审批人
                    }
                }
                Ok(Vec::new())
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
        approver_type == Some(6) || approver_type == Some(7)
    }

    /// 兼容旧调用：解析单个审批人（返回候选列表的第一个）
    /// 新代码应直接使用 resolve_approvers
    pub async fn resolve_approver(
        db: &impl ConnectionTrait,
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
    pub async fn find_user_dept_id(db: &impl ConnectionTrait, user_id: i64) -> Result<Option<i64>> {
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
