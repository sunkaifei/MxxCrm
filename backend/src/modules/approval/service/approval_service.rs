use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::approval::entity::approval_flow_edge;
use crate::modules::approval::entity::approval_flow_node;
use crate::modules::approval::model::approval::*;

use sea_orm::DatabaseConnection;

pub struct ApprovalService;

impl ApprovalService {
    /// 保存审批流模板（含节点和连线）
    pub async fn save_flow(db: &DatabaseConnection, req: &FlowSaveRequest, operator: &str) -> Result<i64> {
        Self::validate_flow(req)?;
        ApprovalModel::save_flow(db, req, operator).await
    }

    /// 查询审批流详情
    pub async fn find_flow_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<FlowDetailVO>> {
        ApprovalModel::find_flow_by_id(db, id).await
    }

    /// 审批流模板列表
    pub async fn find_flow_list(db: &DatabaseConnection, query: &FlowListQuery) -> Result<ResultPage<Vec<FlowListVO>>> {
        ApprovalModel::find_flow_list(db, query).await
    }

    /// 启用/禁用审批流
    pub async fn toggle_flow(db: &DatabaseConnection, id: i64) -> Result<()> {
        ApprovalModel::toggle_flow(db, id).await
    }

    /// 提交审批
    pub async fn submit(db: &DatabaseConnection, req: &ApprovalSubmitRequest) -> Result<i64> {
        let flow_data = ApprovalModel::find_flow_by_code(db, &req.flow_code).await?;
        let (flow, nodes, edges) = flow_data.ok_or_else(|| Error::from("审批流模板不存在或未启用"))?;

        // 查找开始节点 (node_type=1)
        let start_node = nodes.iter().find(|n| n.node_type == Some(1))
            .ok_or_else(|| Error::from("审批流缺少开始节点"))?;

        // 从开始节点的出边找到第一个审批节点
        let first_edge = edges.iter().find(|e| e.source_node_key == start_node.node_key)
            .ok_or_else(|| Error::from("开始节点没有连线"))?;

        let first_node = nodes.iter().find(|n| n.node_key == first_edge.target_node_key)
            .ok_or_else(|| Error::from("开始节点的目标节点不存在"))?;

        if first_node.node_type != Some(2) {
            return Err(Error::from("第一个节点必须是审批节点"));
        }

        let approver_id = first_node.approver_id
            .ok_or_else(|| Error::from("审批节点未配置审批人"))?;

        let instance_id = ApprovalModel::create_instance(db, req, &first_node.node_key.clone().unwrap_or_default(), approver_id).await?;

        Ok(instance_id)
    }

    /// 处理审批
    pub async fn process(db: &DatabaseConnection, req: &ApprovalProcessRequest) -> Result<()> {
        let instance = ApprovalModel::find_instance_by_id(db, req.instance_id).await?
            .ok_or_else(|| Error::from("审批实例不存在"))?;

        if instance.status != 1 && instance.status != 2 {
            return Err(Error::from("该审批实例已处理完成"));
        }

        let flow_data = ApprovalModel::find_flow_by_code(db, &instance.flow_code).await?;
        let (_flow, nodes, edges) = flow_data.ok_or_else(|| Error::from("审批流模板不存在"))?;

        let current_node_key = instance.current_node_key.as_ref()
            .ok_or_else(|| Error::from("当前节点为空"))?;

        let current_node = nodes.iter().find(|n| n.node_key.as_deref() == Some(current_node_key))
            .ok_or_else(|| Error::from("当前节点不存在"))?;

        let node_name = current_node.node_name.clone().unwrap_or_default();

        ApprovalModel::insert_log(db, req.instance_id, current_node_key, &node_name, req).await?;

        match req.action {
            1 => {
                // 通过，查找下一节点
                let out_edges: Vec<&approval_flow_edge::Model> = edges.iter()
                    .filter(|e| e.source_node_key.as_deref() == Some(current_node_key))
                    .collect();

                if out_edges.is_empty() {
                    // 没有出边，直接完成
                    ApprovalModel::finish_instance(db, req.instance_id, 3).await?;
                } else {
                    // 评估条件，选择满足的边
                    let extra_data: serde_json::Value = instance.extra_data.clone().unwrap_or_else(|| serde_json::json!({}));
                    let mut next_node_key: Option<String> = None;

                    for edge in &out_edges {
                        if let Some(cond) = &edge.condition_expr {
                            if !cond.is_empty() {
                                if Self::eval_condition(cond, &extra_data) {
                                    next_node_key = edge.target_node_key.clone();
                                    break;
                                }
                            } else {
                                next_node_key = edge.target_node_key.clone();
                            }
                        } else {
                            next_node_key = edge.target_node_key.clone();
                        }
                    }

                    if next_node_key.is_none() {
                        ApprovalModel::finish_instance(db, req.instance_id, 3).await?;
                        return Ok(());
                    }

                    let next_key = next_node_key.unwrap();
                    let next_node = nodes.iter().find(|n| n.node_key.as_deref() == Some(next_key.as_str()))
                        .ok_or_else(|| Error::from("下一节点不存在"))?;

                    match next_node.node_type {
                        Some(4) => {
                            // 结束节点
                            ApprovalModel::finish_instance(db, req.instance_id, 3).await?;
                        }
                        Some(2) => {
                            // 审批节点
                            let approver_id = next_node.approver_id
                                .ok_or_else(|| Error::from("下一审批节点未配置审批人"))?;
                            ApprovalModel::update_instance_node(db, req.instance_id, &next_key, approver_id).await?;
                        }
                        Some(3) => {
                            // 条件分支，继续遍历
                            Box::pin(Self::traverse_condition(db, req.instance_id, &next_key, &nodes, &edges, &extra_data)).await?;
                        }
                        _ => {
                            ApprovalModel::finish_instance(db, req.instance_id, 3).await?;
                        }
                    }
                }
            }
            2 => {
                // 驳回
                ApprovalModel::finish_instance(db, req.instance_id, 4).await?;
            }
            _ => return Err(Error::from("无效的操作类型")),
        }

        Ok(())
    }

    /// 审批待办列表
    pub async fn find_instance_list(db: &DatabaseConnection, approver_id: i64, page_num: u64, page_size: u64) -> Result<ResultPage<Vec<ApprovalInstanceVO>>> {
        ApprovalModel::find_instance_list(db, approver_id, page_num, page_size).await
    }

    /// 审批实例详情
    pub async fn find_instance_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<ApprovalInstanceVO>> {
        ApprovalModel::find_instance_by_id(db, id).await
    }

    // ============ Private helpers ============

    fn validate_flow(req: &FlowSaveRequest) -> Result<()> {
        let start_count = req.nodes.iter().filter(|n| n.node_type == 1).count();
        if start_count != 1 {
            return Err(Error::from("必须有且仅有一个开始节点"));
        }
        let end_count = req.nodes.iter().filter(|n| n.node_type == 4).count();
        if end_count == 0 {
            return Err(Error::from("必须至少有一个结束节点"));
        }
        // 条件分支节点的出边必须配置条件
        for node in &req.nodes {
            if node.node_type == 3 {
                for edge in &req.edges {
                    if edge.source == node.node_key {
                        if edge.condition_expr.is_none() || edge.condition_expr.as_ref().map_or(true, |c| c.is_empty()) {
                            return Err(Error::from(format!("条件分支节点[{}]的出边必须配置条件表达式", node.node_name)));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn eval_condition(expr: &str, data: &serde_json::Value) -> bool {
        // 简单条件评估，支持 "field > value", "field <= value" 等
        // 实际项目中可使用更完善的表达式引擎
        let expr = expr.trim();
        for op in &["<=", ">=", "==", "!=", ">", "<"] {
            if let Some(pos) = expr.find(op) {
                let field = expr[..pos].trim();
                let value = expr[pos + op.len()..].trim();
                let actual = data.get(field).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let expected: f64 = value.parse().unwrap_or(0.0);
                return match *op {
                    "<=" => actual <= expected,
                    ">=" => actual >= expected,
                    "==" => (actual - expected).abs() < 0.001,
                    "!=" => (actual - expected).abs() >= 0.001,
                    ">" => actual > expected,
                    "<" => actual < expected,
                    _ => false,
                };
            }
        }
        false
    }

    async fn traverse_condition(
        db: &DatabaseConnection,
        instance_id: i64,
        node_key: &str,
        nodes: &[approval_flow_node::Model],
        edges: &[approval_flow_edge::Model],
        extra_data: &serde_json::Value,
    ) -> Result<()> {
        let out_edges: Vec<&approval_flow_edge::Model> = edges.iter()
            .filter(|e| e.source_node_key.as_deref() == Some(node_key))
            .collect();

        for edge in &out_edges {
            let cond = edge.condition_expr.as_deref().unwrap_or("");
            if !cond.is_empty() && !Self::eval_condition(cond, extra_data) {
                continue;
            }
            let target_key = edge.target_node_key.as_deref().unwrap_or("");
            let target_node = nodes.iter().find(|n| n.node_key.as_deref() == Some(target_key));
            if let Some(target) = target_node {
                match target.node_type {
                    Some(4) => {
                        ApprovalModel::finish_instance(db, instance_id, 3).await?;
                        return Ok(());
                    }
                    Some(2) => {
                        let approver_id = target.approver_id
                            .ok_or_else(|| Error::from("审批节点未配置审批人"))?;
                        ApprovalModel::update_instance_node(db, instance_id, target_key, approver_id).await?;
                        return Ok(());
                    }
                    Some(3) => {
                        // 继续遍历条件分支
                        return Box::pin(Self::traverse_condition(db, instance_id, target_key, nodes, edges, extra_data)).await;
                    }
                    _ => {}
                }
            }
        }
        ApprovalModel::finish_instance(db, instance_id, 3).await?;
        Ok(())
    }
}
