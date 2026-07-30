use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::approval::entity::approval_flow_edge;
use crate::modules::approval::entity::approval_flow_node;
use crate::modules::approval::entity::approval_instance::{Column as InstanceColumn, Entity as InstanceEntity};
use crate::modules::approval::model::approval::*;
use crate::modules::crm::model::work_log::WorkLogCreateDTO;
use crate::modules::crm::service::work_log_service;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

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

    pub async fn delete_flow(db: &DatabaseConnection, id: i64) -> Result<()> {
        ApprovalModel::delete_flow(db, id).await
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

        // 解析发起人部门
        let submitter_dept_id = ApprovalModel::find_user_dept_id(db, req.submitter_id).await?;

        // 解析候选审批人列表（支持多审批人场景：角色/岗位下所有用户）
        let raw_candidates = ApprovalModel::resolve_approvers(
            db,
            first_node.approver_type,
            first_node.approver_id,
            req.submitter_id,
            submitter_dept_id,
        ).await?;

        // 自审回避：从候选列表中过滤掉发起人自己
        let candidates = ApprovalModel::filter_self_approvers(raw_candidates, req.submitter_id);

        let first_node_key = first_node.node_key.clone().unwrap_or_default();
        let first_node_name = first_node.node_name.clone().unwrap_or_default();
        let first_node_approver_type = first_node.approver_type;

        // 直属上级节点且候选为空（已到组织架构顶层）：自动通过当前节点并流转到下一节点
        if candidates.is_empty() {
            if ApprovalModel::is_direct_manager_node(first_node_approver_type) {
                // 创建实例（占位，current_approver_id=0 表示系统自动通过），随后立即自动流转
                let instance_id = ApprovalModel::create_instance(
                    db, req, &first_node_key, 0, &[], 
                ).await?;
                // 写入自动通过日志
                Self::insert_auto_pass_log(db, instance_id, &first_node_key, &first_node_name).await?;
                // 流转到下一节点
                let extra_data = req.extra_data.clone().unwrap_or_else(|| serde_json::json!({}));
                Self::advance_to_next_node(db, instance_id, &first_node_key, &nodes, &edges, req.submitter_id, &extra_data).await?;
                let _ = flow;
                return Ok(instance_id);
            } else {
                return Err(Error::from("审批节点未解析到候选审批人"));
            }
        }

        // 当前审批人取候选列表的第一个（或签/会签模式下所有人可见，依次审批按顺序）
        let primary_approver = candidates[0];

        let instance_id = ApprovalModel::create_instance(
            db,
            req,
            &first_node_key,
            primary_approver,
            &candidates,
        ).await?;

        let _ = flow; // flow 已使用
        Ok(instance_id)
    }

    /// 处理审批
    pub async fn process(db: &DatabaseConnection, req: &ApprovalProcessRequest) -> Result<()> {
        let instance = ApprovalModel::find_instance_by_id(db, req.instance_id).await?
            .ok_or_else(|| Error::from("审批实例不存在"))?;

        if instance.status != 1 && instance.status != 2 {
            return Err(Error::from("该审批实例已处理完成"));
        }

        // 权限校验：审批人必须在候选审批人池中
        if !instance.candidate_approvers.contains(&req.approver_id) {
            return Err(Error::from("您不是当前节点的审批人"));
        }

        let approve_mode = instance.approve_mode;

        // 依次审批：必须轮到当前审批人
        if approve_mode == 3 && instance.current_approver_id != Some(req.approver_id) {
            return Err(Error::from("当前还未轮到您审批，请等待前序审批人处理"));
        }

        // 会签/依次审批：不允许重复审批
        if (approve_mode == 2 || approve_mode == 3) && instance.processed_approvers.contains(&req.approver_id) {
            return Err(Error::from("您已审批过该节点"));
        }

        let flow_data = ApprovalModel::find_flow_by_code(db, &instance.flow_code).await?;
        let (_flow, nodes, edges) = flow_data.ok_or_else(|| Error::from("审批流模板不存在"))?;

        let current_node_key = instance.current_node_key.as_ref()
            .ok_or_else(|| Error::from("当前节点为空"))?;

        let current_node = nodes.iter().find(|n| n.node_key.as_deref() == Some(current_node_key))
            .ok_or_else(|| Error::from("当前节点不存在"))?;

        let node_name = current_node.node_name.clone().unwrap_or_default();

        ApprovalModel::insert_log(db, req.instance_id, current_node_key, &node_name, req).await?;

        let submitter_id = instance.submitter_id;
        let extra_data = instance.extra_data.clone().unwrap_or_else(|| serde_json::json!({}));

        match req.action {
            1 => {
                // 通过
                match approve_mode {
                    1 => {
                        // 或签：任一通过即流转到下一节点
                        Self::advance_to_next_node(db, req.instance_id, current_node_key, &nodes, &edges, submitter_id, &extra_data).await?;
                    }
                    2 => {
                        // 会签：全部通过才流转
                        let processed = ApprovalModel::append_processed_approver(db, req.instance_id, req.approver_id).await?;
                        if processed.len() >= instance.candidate_approvers.len() {
                            // 所有候选审批人均已通过，流转到下一节点
                            Self::advance_to_next_node(db, req.instance_id, current_node_key, &nodes, &edges, submitter_id, &extra_data).await?;
                        }
                        // 否则等待其他审批人处理
                    }
                    3 => {
                        // 依次审批：按候选池顺序逐个审批
                        let processed = ApprovalModel::append_processed_approver(db, req.instance_id, req.approver_id).await?;
                        if processed.len() >= instance.candidate_approvers.len() {
                            // 全部审批完成，流转到下一节点
                            Self::advance_to_next_node(db, req.instance_id, current_node_key, &nodes, &edges, submitter_id, &extra_data).await?;
                        } else {
                            // 更新当前审批人为候选池中下一个未处理的人
                            let next_approver = instance.candidate_approvers[processed.len()];
                            ApprovalModel::update_current_approver(db, req.instance_id, next_approver).await?;
                        }
                    }
                    _ => {
                        // 默认按或签处理
                        Self::advance_to_next_node(db, req.instance_id, current_node_key, &nodes, &edges, submitter_id, &extra_data).await?;
                    }
                }
            }
            2 => {
                // 驳回：直接结束实例（无论何种审批模式）
                ApprovalModel::finish_instance(db, req.instance_id, 4).await?;
            }
            _ => return Err(Error::from("无效的操作类型")),
        }

        // 工作日志埋点（审批通过/驳回），不影响主业务
        let action_name = if req.action == 1 { "审批通过" } else { "驳回审批" };
        let result_val = if req.action == 1 { 1 } else { 2 };
        let log_dto = WorkLogCreateDTO {
            user_id: req.approver_id,
            user_name: req.approver_name.clone(),
            action_type: Some(1),
            action_name: Some(action_name.to_string()),
            business_type: Some(instance.business_type.clone()),
            business_id: Some(instance.business_id),
            business_title: instance.business_title.clone(),
            description: req.comment.clone(),
            result: Some(result_val),
            work_date: Some(chrono::Local::now().naive_local().date()),
        };
        let _ = work_log_service::insert(db, &log_dto).await;

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

    /// 从当前节点查找下一节点并推进实例（处理条件分支）
    async fn advance_to_next_node(
        db: &DatabaseConnection,
        instance_id: i64,
        current_node_key: &str,
        nodes: &[approval_flow_node::Model],
        edges: &[approval_flow_edge::Model],
        submitter_id: i64,
        extra_data: &serde_json::Value,
    ) -> Result<()> {
        let out_edges: Vec<&approval_flow_edge::Model> = edges.iter()
            .filter(|e| e.source_node_key.as_deref() == Some(current_node_key))
            .collect();

        if out_edges.is_empty() {
            // 没有出边，直接完成实例
            ApprovalModel::finish_instance(db, instance_id, 3).await?;
            return Ok(());
        }

        let mut next_node_key: Option<String> = None;
        for edge in &out_edges {
            if let Some(cond) = &edge.condition_expr {
                if !cond.is_empty() {
                    if Self::eval_condition(cond, extra_data) {
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
            ApprovalModel::finish_instance(db, instance_id, 3).await?;
            return Ok(());
        }

        let next_key = next_node_key.unwrap();
        Self::move_to_next_node(db, instance_id, &next_key, nodes, edges, submitter_id, extra_data).await
    }

    async fn move_to_next_node(
        db: &DatabaseConnection,
        instance_id: i64,
        next_key: &str,
        nodes: &[approval_flow_node::Model],
        edges: &[approval_flow_edge::Model],
        submitter_id: i64,
        extra_data: &serde_json::Value,
    ) -> Result<()> {
        let next_node = nodes.iter().find(|n| n.node_key.as_deref() == Some(next_key))
            .ok_or_else(|| Error::from("下一节点不存在"))?;

        match next_node.node_type {
            Some(4) => {
                // 结束节点
                ApprovalModel::finish_instance(db, instance_id, 3).await?;
            }
            Some(2) => {
                // 审批节点 - 解析候选审批人列表
                let submitter_dept_id = ApprovalModel::find_user_dept_id(db, submitter_id).await?;
                let raw_candidates = ApprovalModel::resolve_approvers(
                    db,
                    next_node.approver_type,
                    next_node.approver_id,
                    submitter_id,
                    submitter_dept_id,
                ).await?;
                // 自审回避
                let candidates = ApprovalModel::filter_self_approvers(raw_candidates, submitter_id);

                let next_node_name = next_node.node_name.clone().unwrap_or_default();
                let next_node_approver_type = next_node.approver_type;

                if candidates.is_empty() {
                    if ApprovalModel::is_direct_manager_node(next_node_approver_type) {
                        // 直属上级场景：候选为空（已到顶层），自动通过并继续流转
                        ApprovalModel::update_instance_node(db, instance_id, next_key, 0, &[]).await?;
                        Self::insert_auto_pass_log(db, instance_id, next_key, &next_node_name).await?;
                        return Box::pin(Self::advance_to_next_node(db, instance_id, next_key, nodes, edges, submitter_id, extra_data)).await;
                    } else {
                        return Err(Error::from("审批节点未解析到候选审批人"));
                    }
                }

                let primary_approver = candidates[0];
                ApprovalModel::update_instance_node(db, instance_id, next_key, primary_approver, &candidates).await?;
            }
            Some(3) => {
                // 条件分支，继续遍历
                Box::pin(Self::traverse_condition(db, instance_id, next_key, nodes, edges, submitter_id, extra_data)).await?;
            }
            _ => {
                ApprovalModel::finish_instance(db, instance_id, 3).await?;
            }
        }
        Ok(())
    }

    /// 写入"系统自动通过"审批日志（用于直属上级节点候选为空时自动流转）
    /// approver_id=0 表示系统，action=1 表示通过
    async fn insert_auto_pass_log(
        db: &DatabaseConnection,
        instance_id: i64,
        node_key: &str,
        node_name: &str,
    ) -> Result<()> {
        let auto_req = ApprovalProcessRequest {
            instance_id,
            action: 1,
            approver_id: 0,
            approver_name: Some("系统自动通过".to_string()),
            comment: Some("直属上级节点已到组织架构顶层，系统自动通过".to_string()),
        };
        ApprovalModel::insert_log(db, instance_id, node_key, node_name, &auto_req).await
    }

    fn validate_flow(req: &FlowSaveRequest) -> Result<()> {
        let start_count = req.nodes.iter().filter(|n| n.node_type == 1).count();
        if start_count != 1 {
            return Err(Error::from("必须有且仅有一个开始节点"));
        }
        let end_count = req.nodes.iter().filter(|n| n.node_type == 4).count();
        if end_count == 0 {
            return Err(Error::from("必须至少有一个结束节点"));
        }
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

    /// 条件表达式求值
    ///
    /// 支持的语法：
    /// - 单一条件：`amount>10000`、`price>=500`、`count==10`
    /// - AND 组合：`amount>10000 && type==1`（所有子条件都需满足）
    /// - OR 组合：`amount>10000 || priority==1`（任一子条件满足即可）
    /// - 混合组合：`amount>10000 && type==1 || priority==2`（AND 优先级高于 OR）
    ///
    /// 支持的比较运算符：`<=`、`>=`、`==`、`!=`、`>`、`<`
    /// 字段值通过 `data.get(field).as_f64()` 取出，缺失或非数字时默认 0.0
    fn eval_condition(expr: &str, data: &serde_json::Value) -> bool {
        let expr = expr.trim();
        if expr.is_empty() {
            return true;
        }

        // 支持 OR（||）：按 || 分割，任一子条件满足即可
        // 注意：先处理 OR，再处理 AND，确保 AND 优先级更高
        let or_parts: Vec<&str> = expr.split("||").map(|s| s.trim()).collect();
        if or_parts.len() > 1 {
            return or_parts.iter().any(|part| Self::eval_condition(part, data));
        }

        // 支持 AND（&&）：按 && 分割，所有子条件都需满足
        let and_parts: Vec<&str> = expr.split("&&").map(|s| s.trim()).collect();
        if and_parts.len() > 1 {
            return and_parts.iter().all(|part| Self::eval_condition(part, data));
        }

        // 单一条件求值
        Self::eval_single_condition(expr, data)
    }

    /// 单一条件求值（不支持 AND/OR 组合）
    fn eval_single_condition(expr: &str, data: &serde_json::Value) -> bool {
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
        submitter_id: i64,
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
                        let submitter_dept_id = ApprovalModel::find_user_dept_id(db, submitter_id).await?;
                        let raw_candidates = ApprovalModel::resolve_approvers(
                            db,
                            target.approver_type,
                            target.approver_id,
                            submitter_id,
                            submitter_dept_id,
                        ).await?;
                        // 自审回避
                        let candidates = ApprovalModel::filter_self_approvers(raw_candidates, submitter_id);

                        let target_node_name = target.node_name.clone().unwrap_or_default();
                        let target_approver_type = target.approver_type;

                        if candidates.is_empty() {
                            if ApprovalModel::is_direct_manager_node(target_approver_type) {
                                // 直属上级场景：候选为空，自动通过并继续流转
                                ApprovalModel::update_instance_node(db, instance_id, target_key, 0, &[]).await?;
                                Self::insert_auto_pass_log(db, instance_id, target_key, &target_node_name).await?;
                                return Box::pin(Self::advance_to_next_node(db, instance_id, target_key, nodes, edges, submitter_id, extra_data)).await;
                            } else {
                                return Err(Error::from("审批节点未解析到候选审批人"));
                            }
                        }

                        let primary_approver = candidates[0];
                        ApprovalModel::update_instance_node(db, instance_id, target_key, primary_approver, &candidates).await?;
                        return Ok(());
                    }
                    Some(3) => {
                        return Box::pin(Self::traverse_condition(db, instance_id, target_key, nodes, edges, submitter_id, extra_data)).await;
                    }
                    _ => {}
                }
            }
        }
        ApprovalModel::finish_instance(db, instance_id, 3).await?;
        Ok(())
    }

    // ==================== 审批增强功能 ====================

    /// 取消（撤回）审批实例 - 仅发起人可操作
    /// action=7, 实例状态置为 5=已撤回
    pub async fn cancel_instance(
        db: &DatabaseConnection,
        req: &ApprovalCancelRequest,
        operator_id: i64,
        operator_name: &str,
    ) -> Result<()> {
        let instance = ApprovalModel::find_instance_by_id(db, req.instance_id).await?
            .ok_or_else(|| Error::from("审批实例不存在"))?;

        // 权限校验：仅发起人可撤回
        if instance.submitter_id != operator_id {
            return Err(Error::from("仅发起人可撤回审批"));
        }

        // 状态校验：仅进行中(1/2)可撤回
        if instance.status != 1 && instance.status != 2 {
            return Err(Error::from("当前审批状态不允许撤回"));
        }

        let cancel_reason = req.cancel_reason.clone().unwrap_or_default();
        let current_node_key = instance.current_node_key.clone().unwrap_or_default();
        let current_node_name = instance.flow_nodes.iter()
            .find(|n| n.node_key == current_node_key)
            .map(|n| n.node_name.clone())
            .unwrap_or_default();

        // 写入取消日志
        ApprovalModel::insert_log_with_target(
            db,
            req.instance_id,
            &current_node_key,
            &current_node_name,
            operator_id,
            Some(operator_name.to_string()),
            7, // action=7 取消
            Some(cancel_reason.clone()),
            None, None, None, None,
        ).await?;

        // 更新取消原因
        ApprovalModel::update_cancel_reason(db, req.instance_id, &cancel_reason).await?;

        // 结束实例，状态=5 已撤回
        ApprovalModel::finish_instance(db, req.instance_id, 5).await?;

        Ok(())
    }

    /// 退回 - 退回到发起人或指定节点
    /// action=6, 退回到发起人时 needs_resubmit=1
    pub async fn reject_to(
        db: &DatabaseConnection,
        req: &ApprovalRejectToRequest,
        operator_id: i64,
        operator_name: &str,
    ) -> Result<()> {
        let instance = ApprovalModel::find_instance_by_id(db, req.instance_id).await?
            .ok_or_else(|| Error::from("审批实例不存在"))?;

        // 权限校验：审批人必须在候选池中
        if !instance.candidate_approvers.contains(&operator_id) {
            return Err(Error::from("您不是当前节点的审批人"));
        }

        // 状态校验
        if instance.status != 1 && instance.status != 2 {
            return Err(Error::from("当前审批状态不允许退回"));
        }

        let current_node_key = instance.current_node_key.clone().unwrap_or_default();
        let current_node_name = instance.flow_nodes.iter()
            .find(|n| n.node_key == current_node_key)
            .map(|n| n.node_name.clone())
            .unwrap_or_default();

        match &req.reject_to_node_key {
            None => {
                // 退回到发起人：标记需要重新提交，状态置为 6=待修改
                ApprovalModel::insert_log_with_target(
                    db,
                    req.instance_id,
                    &current_node_key,
                    &current_node_name,
                    operator_id,
                    Some(operator_name.to_string()),
                    6, // action=6 退回
                    req.comment.clone(),
                    Some(instance.submitter_id),
                    instance.submitter_name.clone(),
                    None, None,
                ).await?;

                // 更新实例：current_approver 设为发起人，needs_resubmit=1，状态=6
                ApprovalModel::update_instance_node_with_extras(
                    db,
                    req.instance_id,
                    "start",
                    instance.submitter_id,
                    &[instance.submitter_id],
                    None, None, None,
                    Some(1), // needs_resubmit=1
                ).await?;

                // 状态改为 6=待修改（退回到发起人）
                let now = chrono::Utc::now().naive_utc();
                InstanceEntity::update_many()
                    .col_expr(InstanceColumn::Status, sea_orm::sea_query::Expr::value(6))
                    .col_expr(InstanceColumn::UpdateTime, sea_orm::sea_query::Expr::value(now))
                    .filter(InstanceColumn::Id.eq(req.instance_id))
                    .exec(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
            }
            Some(target_key) => {
                // 退回到指定节点
                let target_node = instance.flow_nodes.iter()
                    .find(|n| n.node_key == *target_key)
                    .ok_or_else(|| Error::from(format!("目标节点 {} 不存在", target_key)))?;

                // 查询目标节点的实际配置（从原始 flow_node 表获取 approver_type 等）
                let flow_data = ApprovalModel::find_flow_by_code(db, &instance.flow_code).await?;
                let (_flow, nodes, _edges) = flow_data.ok_or_else(|| Error::from("审批流模板不存在"))?;
                let target_node_cfg = nodes.iter()
                    .find(|n| n.node_key.as_deref() == Some(target_key))
                    .ok_or_else(|| Error::from("目标节点配置不存在"))?;

                let submitter_dept_id = ApprovalModel::find_user_dept_id(db, instance.submitter_id).await?;
                let raw_candidates = ApprovalModel::resolve_approvers(
                    db,
                    target_node_cfg.approver_type,
                    target_node_cfg.approver_id,
                    instance.submitter_id,
                    submitter_dept_id,
                ).await?;
                let candidates = ApprovalModel::filter_self_approvers(raw_candidates, instance.submitter_id);

                if candidates.is_empty() {
                    return Err(Error::from("目标节点未解析到候选审批人"));
                }

                let primary_approver = candidates[0];
                let target_node_name = target_node.node_name.clone();

                ApprovalModel::insert_log_with_target(
                    db,
                    req.instance_id,
                    &current_node_key,
                    &current_node_name,
                    operator_id,
                    Some(operator_name.to_string()),
                    6, // action=6 退回
                    req.comment.clone(),
                    Some(primary_approver),
                    None,
                    Some(target_key.clone()),
                    Some(target_node_name),
                ).await?;

                ApprovalModel::update_instance_node_with_extras(
                    db,
                    req.instance_id,
                    target_key,
                    primary_approver,
                    &candidates,
                    None, None, None,
                    Some(0),
                ).await?;
            }
        }

        Ok(())
    }

    /// 转办 - 当前审批人转给他人，责任转移
    /// action=3
    pub async fn transfer(
        db: &DatabaseConnection,
        req: &ApprovalTransferRequest,
        operator_id: i64,
        operator_name: &str,
    ) -> Result<()> {
        let instance = ApprovalModel::find_instance_by_id(db, req.instance_id).await?
            .ok_or_else(|| Error::from("审批实例不存在"))?;

        // 权限校验：审批人必须在候选池中
        if !instance.candidate_approvers.contains(&operator_id) {
            return Err(Error::from("您不是当前节点的审批人"));
        }

        // 状态校验
        if instance.status != 1 && instance.status != 2 {
            return Err(Error::from("当前审批状态不允许转办"));
        }

        // 不能转办给自己
        if req.target_user_id == operator_id {
            return Err(Error::from("不能转办给自己"));
        }

        let current_node_key = instance.current_node_key.clone().unwrap_or_default();
        let current_node_name = instance.flow_nodes.iter()
            .find(|n| n.node_key == current_node_key)
            .map(|n| n.node_name.clone())
            .unwrap_or_default();

        // 写入转办日志
        ApprovalModel::insert_log_with_target(
            db,
            req.instance_id,
            &current_node_key,
            &current_node_name,
            operator_id,
            Some(operator_name.to_string()),
            3, // action=3 转办
            req.comment.clone(),
            Some(req.target_user_id),
            req.target_user_name.clone(),
            None, None,
        ).await?;

        // 更新实例：当前审批人改为目标用户，候选池替换为目标用户，记录转办来源
        ApprovalModel::update_instance_node_with_extras(
            db,
            req.instance_id,
            &current_node_key,
            req.target_user_id,
            &[req.target_user_id],
            Some(operator_id), // transfer_from_id
            None, None, None,
        ).await?;

        Ok(())
    }

    /// 委派 - 委托他人处理，责任仍归原审批人
    /// action=4, 被委派人处理后结果记在原审批人名下
    pub async fn delegate(
        db: &DatabaseConnection,
        req: &ApprovalDelegateRequest,
        operator_id: i64,
        operator_name: &str,
    ) -> Result<()> {
        let instance = ApprovalModel::find_instance_by_id(db, req.instance_id).await?
            .ok_or_else(|| Error::from("审批实例不存在"))?;

        // 权限校验：审批人必须在候选池中
        if !instance.candidate_approvers.contains(&operator_id) {
            return Err(Error::from("您不是当前节点的审批人"));
        }

        // 状态校验
        if instance.status != 1 && instance.status != 2 {
            return Err(Error::from("当前审批状态不允许委派"));
        }

        if req.target_user_id == operator_id {
            return Err(Error::from("不能委派给自己"));
        }

        let current_node_key = instance.current_node_key.clone().unwrap_or_default();
        let current_node_name = instance.flow_nodes.iter()
            .find(|n| n.node_key == current_node_key)
            .map(|n| n.node_name.clone())
            .unwrap_or_default();

        // 写入委派日志
        ApprovalModel::insert_log_with_target(
            db,
            req.instance_id,
            &current_node_key,
            &current_node_name,
            operator_id,
            Some(operator_name.to_string()),
            4, // action=4 委派
            req.comment.clone(),
            Some(req.target_user_id),
            req.target_user_name.clone(),
            None, None,
        ).await?;

        // 更新实例：当前审批人改为被委派人，候选池增加被委派人，记录委派来源
        let mut new_candidates = instance.candidate_approvers.clone();
        if !new_candidates.contains(&req.target_user_id) {
            new_candidates.push(req.target_user_id);
        }

        ApprovalModel::update_instance_node_with_extras(
            db,
            req.instance_id,
            &current_node_key,
            req.target_user_id,
            &new_candidates,
            None,
            Some(operator_id), // delegate_from_id
            None, None,
        ).await?;

        Ok(())
    }

    /// 加签 - 前加签/后加签/并加签
    /// action=5
    /// 前加签(1)：新审批人在当前审批人之前审批，全部通过后当前审批人继续
    /// 后加签(2)：当前审批人通过后，新审批人继续审批
    /// 并加签(3)：新审批人与当前审批人并行审批
    pub async fn add_sign(
        db: &DatabaseConnection,
        req: &ApprovalAddSignRequest,
        operator_id: i64,
        operator_name: &str,
    ) -> Result<()> {
        if req.target_user_ids.is_empty() {
            return Err(Error::from("加签用户不能为空"));
        }

        let instance = ApprovalModel::find_instance_by_id(db, req.instance_id).await?
            .ok_or_else(|| Error::from("审批实例不存在"))?;

        // 权限校验：审批人必须在候选池中
        if !instance.candidate_approvers.contains(&operator_id) {
            return Err(Error::from("您不是当前节点的审批人"));
        }

        // 状态校验
        if instance.status != 1 && instance.status != 2 {
            return Err(Error::from("当前审批状态不允许加签"));
        }

        let current_node_key = instance.current_node_key.clone().unwrap_or_default();
        let current_node_name = instance.flow_nodes.iter()
            .find(|n| n.node_key == current_node_key)
            .map(|n| n.node_name.clone())
            .unwrap_or_default();

        // 过滤掉已在候选池中的用户（避免重复）
        let new_users: Vec<i64> = req.target_user_ids.iter()
            .filter(|uid| !instance.candidate_approvers.contains(uid))
            .copied()
            .collect();
        if new_users.is_empty() {
            return Err(Error::from("所选用户已在审批人中，无需加签"));
        }

        let target_names_str = new_users.iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // 写入加签日志
        ApprovalModel::insert_log_with_target(
            db,
            req.instance_id,
            &current_node_key,
            &current_node_name,
            operator_id,
            Some(operator_name.to_string()),
            5, // action=5 加签
            req.comment.clone().or(Some(format!("加签用户: {}", target_names_str))),
            None, None, None, None,
        ).await?;

        // 更新候选池：将新加签用户加入候选池
        let mut new_candidates = instance.candidate_approvers.clone();
        for uid in &new_users {
            if !new_candidates.contains(uid) {
                new_candidates.push(*uid);
            }
        }

        match req.add_sign_type {
            1 => {
                // 前加签：新审批人先审批，当前审批人暂不处理
                // 当前审批人改为第一个新加签用户
                ApprovalModel::update_instance_node_with_extras(
                    db,
                    req.instance_id,
                    &current_node_key,
                    new_users[0],
                    &new_candidates,
                    None, None,
                    Some(1), // add_sign_type=前加签
                    None,
                ).await?;
            }
            2 => {
                // 后加签：当前审批人通过后新审批人继续，仅更新候选池
                // 当前审批人保持不变，仅扩展候选池
                let now = chrono::Utc::now().naive_utc();
                let candidates_json: serde_json::Value =
                    serde_json::Value::Array(new_candidates.iter().map(|id| serde_json::json!(id)).collect());
                InstanceEntity::update_many()
                    .col_expr(InstanceColumn::CandidateApprovers, sea_orm::sea_query::Expr::value(candidates_json))
                    .col_expr(InstanceColumn::AddSignType, sea_orm::sea_query::Expr::value(2))
                    .col_expr(InstanceColumn::UpdateTime, sea_orm::sea_query::Expr::value(now))
                    .filter(InstanceColumn::Id.eq(req.instance_id))
                    .exec(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
            }
            3 => {
                // 并加签：新审批人与当前审批人并行，加入候选池即可
                let now = chrono::Utc::now().naive_utc();
                let candidates_json: serde_json::Value =
                    serde_json::Value::Array(new_candidates.iter().map(|id| serde_json::json!(id)).collect());
                InstanceEntity::update_many()
                    .col_expr(InstanceColumn::CandidateApprovers, sea_orm::sea_query::Expr::value(candidates_json))
                    .col_expr(InstanceColumn::AddSignType, sea_orm::sea_query::Expr::value(3))
                    .col_expr(InstanceColumn::UpdateTime, sea_orm::sea_query::Expr::value(now))
                    .filter(InstanceColumn::Id.eq(req.instance_id))
                    .exec(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?;
            }
            _ => return Err(Error::from("无效的加签类型，1=前加签,2=后加签,3=并加签")),
        }

        Ok(())
    }

    /// 抄送 - 知会相关人员，不参与审批
    /// action=8
    pub async fn add_cc(
        db: &DatabaseConnection,
        req: &ApprovalCcRequest,
        operator_id: i64,
        operator_name: &str,
    ) -> Result<()> {
        if req.user_ids.is_empty() {
            return Err(Error::from("抄送用户不能为空"));
        }

        let instance = ApprovalModel::find_instance_by_id(db, req.instance_id).await?
            .ok_or_else(|| Error::from("审批实例不存在"))?;

        // 权限校验：发起人或当前审批人可抄送
        let can_cc = instance.submitter_id == operator_id
            || instance.candidate_approvers.contains(&operator_id);
        if !can_cc {
            return Err(Error::from("仅发起人或当前审批人可添加抄送"));
        }

        // 写入抄送日志
        ApprovalModel::insert_log_with_target(
            db,
            req.instance_id,
            instance.current_node_key.as_deref().unwrap_or(""),
            instance.flow_nodes.iter()
                .find(|n| Some(n.node_key.clone()) == instance.current_node_key)
                .map(|n| n.node_name.clone())
                .unwrap_or_default().as_str(),
            operator_id,
            Some(operator_name.to_string()),
            8, // action=8 抄送
            req.cc_reason.clone(),
            None, None, None, None,
        ).await?;

        // 插入抄送记录
        ApprovalModel::insert_cc_records(
            db,
            req.instance_id,
            &req.user_ids,
            Some(operator_id),
            Some(operator_name.to_string()),
            req.cc_reason.clone(),
        ).await?;

        Ok(())
    }

    /// 查询抄送列表（分页）
    pub async fn find_cc_list(
        db: &DatabaseConnection,
        user_id: i64,
        is_read: Option<i32>,
        page_num: u64,
        page_size: u64,
    ) -> Result<ResultPage<Vec<ApprovalCcVO>>> {
        ApprovalModel::find_cc_list_for_user(db, user_id, is_read, page_num, page_size).await
    }

    /// 标记抄送为已读
    pub async fn mark_cc_read(db: &DatabaseConnection, cc_id: i64, user_id: i64) -> Result<()> {
        ApprovalModel::mark_cc_read(db, cc_id, user_id).await
    }
}
