use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::approval::entity::approval_flow_edge;
use crate::modules::approval::entity::approval_flow_node;
use crate::modules::approval::entity::approval_instance::{Column as InstanceColumn, Entity as InstanceEntity, Model as InstanceModel};
use crate::modules::approval::model::approval::*;
use crate::modules::crm::entity::contract;
use crate::modules::crm::model::work_log::WorkLogCreateDTO;
use crate::modules::crm::service::work_log_service;
use crate::modules::message::service::notification_service::NotificationService;
use crate::modules::sale::entity::invoice;
use crate::modules::sale::entity::order;
use crate::modules::system::entity::admin::{Column as AdminColumn, Entity as AdminEntity};
use crate::modules::system::entity::admin_role_merge::{Column as RoleMergeColumn, Entity as RoleMergeEntity};
use crate::modules::system::entity::role::{Column as RoleColumn, Entity as RoleEntity};
use crate::modules::system::service::profile_service;

use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, TransactionTrait};

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

    /// 按流程编码查询启用的审批流详情（供业务模块提交审批前预览流程，无需 system:approval:list 权限）
    pub async fn find_flow_vo_by_code(
        db: &DatabaseConnection,
        code: &str,
    ) -> Result<Option<FlowDetailVO>> {
        ApprovalModel::find_flow_vo_by_code(db, code).await
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
        let mut req = (*req).clone();
        // 用户审核（business_type=user）：按被审核用户角色动态注入 userLevel（供条件分支分流），
        // 并携带被审核用户部门 userDeptId（供部门负责人链 type=7 节点基于被审核用户解析）
        if req.business_type == "user" {
            if let Some(extra) = Self::build_user_audit_extra(db, &req).await? {
                req.extra_data = Some(extra);
            }
            // 档案完善度前置校验：员工须先完善 个人信息/个人简历/财务信息/紧急联系人 才能提交入职审批
            let missing = profile_service::profile_completeness(db, req.business_id).await?;
            if !missing.is_empty() {
                return Err(Error::from(format!(
                    "档案信息不完整，请先在个人中心完善：{}",
                    missing.join("、")
                )));
            }
            // 合同信息随提交写入员工档案（前端在 extra_data 中携带 contractType/contractMonths，
            // 同时自动随审批实例留痕供审批历史追溯；contract_type=2 无固定期限时清空合同期限），
            // 写入失败视为提交失败，保证档案与审批内容一致；未携带合同信息的提交入口不触发写入
            if let Some(extra) = req.extra_data.as_ref() {
                if let Some(ct) = extra.get("contractType").and_then(|v| v.as_i64()).map(|v| v as i16) {
                    let cm: Option<i32> = extra.get("contractMonths").and_then(|v| v.as_i64()).map(|v| v as i32);
                    // 无固定期限合同依法不约定期限，清空
                    let (ct_val, cm_val): (Option<i16>, Option<i32>) = if ct == 2 { (Some(2), None) } else { (Some(ct), cm) };
                    AdminEntity::update_many()
                        .col_expr(AdminColumn::ContractType, sea_orm::sea_query::Expr::value(ct_val))
                        .col_expr(AdminColumn::ContractMonths, sea_orm::sea_query::Expr::value(cm_val))
                        .filter(AdminColumn::Id.eq(req.business_id))
                        .exec(db)
                        .await?;
                }
            }
        }
        // 幂等防护（应用层快速失败）：同一业务单据同时只允许一个进行中的审批实例，
        // 防止双击/网络重试/多入口重复发起导致审批人多条待办；
        // 数据库层兜底由部分唯一索引 uq_approval_instance_active_business 承担（见 sql/d14），
        // 在 ApprovalModel::create_instance 中统一转译冲突错误，覆盖并发竞态窗口；
        // 已终结实例(3通过/4驳回/5撤回/6待修改)不受影响，重新发起新实例属合法流程
        let active = InstanceEntity::find()
            .filter(InstanceColumn::BusinessType.eq(req.business_type.clone()))
            .filter(InstanceColumn::BusinessId.eq(req.business_id))
            .filter(InstanceColumn::Status.is_in(vec![1, 2]))
            .order_by_desc(InstanceColumn::SubmittedAt)
            .one(db)
            .await?;
        if let Some(instance) = active {
            return Err(Self::dup_active_error(&instance));
        }
        let flow_data = ApprovalModel::find_flow_by_code(db, &req.flow_code).await?;
        let (flow, nodes, edges) = flow_data.ok_or_else(|| Error::from("审批流模板不存在或未启用"))?;

        // 查找开始节点 (node_type=1)
        let start_node = nodes.iter().find(|n| n.node_type == Some(1))
            .ok_or_else(|| Error::from("审批流缺少开始节点"))?;

        // 创建流程模板快照（防止模板修改影响在途实例）
        let flow_snapshot = serde_json::json!({
            "nodes": nodes.iter().map(|n| serde_json::to_value(n).unwrap_or_default()).collect::<Vec<_>>(),
            "edges": edges.iter().map(|e| serde_json::to_value(e).unwrap_or_default()).collect::<Vec<_>>(),
            "version": 1,
        });

        // 从开始节点的出边找到第一个节点
        let first_edge = edges.iter().find(|e| e.source_node_key == start_node.node_key)
            .ok_or_else(|| Error::from("开始节点没有连线"))?;

        let mut first_node = nodes.iter().find(|n| n.node_key == first_edge.target_node_key)
            .ok_or_else(|| Error::from("开始节点的目标节点不存在"))?;

        let extra_data = req.extra_data.clone().unwrap_or_else(|| serde_json::json!({}));

        // 如果第一个节点是条件分支(type=3)，根据条件解析到真正的审批节点(type=2)
        if first_node.node_type == Some(3) {
            let mut current_key = first_node.node_key.clone().unwrap_or_default();
            loop {
                // 查找满足条件的出边
                let cond_edges: Vec<&approval_flow_edge::Model> = edges.iter()
                    .filter(|e| e.source_node_key.as_deref() == Some(&current_key))
                    .collect();
                
                let mut next_node = None;
                for edge in &cond_edges {
                    let cond = edge.condition_expr.as_deref().unwrap_or("");
                    if cond.is_empty() || Self::eval_condition(cond, &extra_data) {
                        let target_key = edge.target_node_key.as_deref().unwrap_or("");
                        next_node = nodes.iter().find(|n| n.node_key.as_deref() == Some(target_key));
                        break;
                    }
                }

                match next_node {
                    Some(node) if node.node_type == Some(2) => {
                        first_node = node;
                        break;
                    }
                    Some(node) if node.node_type == Some(3) => {
                        // 嵌套条件分支，继续解析
                        current_key = node.node_key.clone().unwrap_or_default();
                        continue;
                    }
                    Some(node) if node.node_type == Some(4) => {
                        // 直接到结束节点，说明不需要审批（如条件不满足直接通过）
                        // 创建实例并直接完成（事务保证原子性）
                        let nk = node.node_key.clone().unwrap_or_default();
                        let req = req.clone();
                        let instance_id = db.transaction::<_, i64, Error>(|txn| {
                            Box::pin(async move {
                                let instance_id = ApprovalModel::create_instance(
                                    txn, &req, &nk, 0, &[],
                                    Some(flow_snapshot),
                                ).await?;
                                // 提交时抄送（无审批直接通过场景同样记录）
                                Self::insert_submit_cc(txn, instance_id, &nk, "", &req).await?;
                                ApprovalModel::finish_instance(txn, instance_id, 3).await?;
                                Ok(instance_id)
                            })
                        })
                        .await
                        .map_err(|e| Error::from(e.to_string()))?;
                        let _ = flow;
                        // B10：提交后站内通知（best-effort，失败不影响主流程）
                        Self::notify_after_submit(db, instance_id).await;
                        return Ok(instance_id);
                    }
                    _ => {
                        return Err(Error::from("条件分支未匹配到任何审批节点"));
                    }
                }
            }
        }

        if first_node.node_type != Some(2) {
            return Err(Error::from("第一个节点必须是审批节点或条件分支"));
        }

        // 解析发起人部门（用户审核场景优先取被审核用户部门）
        let submitter_dept_id = Self::resolve_flow_dept_id(db, req.submitter_id, &extra_data).await?;

        // 解析候选审批人列表（支持多审批人场景：角色/岗位下所有用户）
        let raw_candidates = ApprovalModel::resolve_approvers(
            db,
            first_node.approver_type,
            first_node.approver_id,
            req.submitter_id,
            submitter_dept_id,
        ).await?;

        // 自审回避：从候选列表中过滤掉发起人自己
        let mut candidates = ApprovalModel::filter_self_approvers(raw_candidates.clone(), req.submitter_id);

        let first_node_key = first_node.node_key.clone().unwrap_or_default();
        let first_node_name = first_node.node_name.clone().unwrap_or_default();
        let first_node_approver_type = first_node.approver_type;

        // 直属上级节点且候选为空（已到组织架构顶层）：自动通过当前节点并流转到下一节点
        if candidates.is_empty() {
            if ApprovalModel::is_direct_manager_node(first_node_approver_type) {
                // 入职/离职审核（user/resign）：部门负责人链（type=7）解析为空时不允许自动通过，
                // 明确报错（审批必须有人审，部门负责人节点不可静默跳过）
                if (req.business_type == "user" || req.business_type == "resign")
                    && first_node_approver_type == Some(7)
                {
                    return Err(Error::from(
                        "该员工部门未配置负责人或负责人不可用，请联系管理员完善部门负责人设置后再提交",
                    ));
                }
                // 创建实例（占位，current_approver_id=0 表示系统自动通过），随后立即自动流转
                let ed = req.extra_data.clone().unwrap_or_else(|| serde_json::json!({}));
                let sid = req.submitter_id;
                let req = req.clone();
                let instance_id = db.transaction::<_, i64, Error>(|txn| {
                    Box::pin(async move {
                        let instance_id = ApprovalModel::create_instance(
                            txn, &req, &first_node_key, 0, &[],
                            Some(flow_snapshot),
                        ).await?;
                        // 写入自动通过日志
                        Self::insert_auto_pass_log(txn, instance_id, &first_node_key, &first_node_name).await?;
                        // 提交时抄送
                        Self::insert_submit_cc(txn, instance_id, &first_node_key, &first_node_name, &req).await?;
                        // 流转到下一节点
                        Self::advance_to_next_node(txn, instance_id, &first_node_key, &nodes, &edges, sid, &ed).await?;
                        Ok(instance_id)
                    })
                })
                .await
                .map_err(|e| Error::from(e.to_string()))?;
                let _ = flow;
                // 用户审核：直属上级自动通过完成时启用用户
                Self::finish_user_audit_if_approved(db, instance_id).await;
                // B10：提交后站内通知（best-effort，失败不影响主流程）
                Self::notify_after_submit(db, instance_id).await;
                return Ok(instance_id);
            } else if !raw_candidates.is_empty() {
                // 自审回避后候选为空但原候选非空（如指定角色下仅有提交人自己）：
                // 退化为允许提交人自己审批，避免流程无法流转
                candidates = raw_candidates;
            } else {
                return Err(Error::from("审批节点未解析到候选审批人"));
            }
        }

        // 当前审批人取候选列表的第一个（或签/会签模式下所有人可见，依次审批按顺序）
        let primary_approver = candidates[0];

        let req = req.clone();
        let instance_id = db.transaction::<_, i64, Error>(|txn| {
            Box::pin(async move {
                let instance_id = ApprovalModel::create_instance(
                    txn,
                    &req,
                    &first_node_key,
                    primary_approver,
                    &candidates,
                    Some(flow_snapshot),
                ).await?;
                // 提交时抄送
                Self::insert_submit_cc(txn, instance_id, &first_node_key, &first_node_name, &req).await?;
                Ok(instance_id)
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

        let _ = flow; // flow 已使用
        // B10：提交后站内通知首节点审批人（best-effort，失败不影响主流程）
        Self::notify_after_submit(db, instance_id).await;
        Ok(instance_id)
    }

    /// 在途重复拦截的统一文案：携带流程编号与最近发起时间上下文（编号可直接对照数据库核查），
    /// 并引导先撤回当前流程再重新提交
    fn dup_active_error(instance: &InstanceModel) -> Error {
        let time = instance.submitted_at.as_ref()
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "时间未知".to_string());
        Error::from(format!(
            "该记录已存在进行中的审批流程（流程编号 #{}，最近发起时间：{}），请勿重复提交；如需重新发起，请先撤回当前流程后再提交",
            instance.id, time
        ))
    }

    /// 提交审批时插入抄送记录 + 抄送日志（在实例创建的事务内调用，保证原子性）
    /// 过滤提交人本人并去重，未指定抄送人时直接跳过
    async fn insert_submit_cc(
        txn: &impl ConnectionTrait,
        instance_id: i64,
        node_key: &str,
        node_name: &str,
        req: &ApprovalSubmitRequest,
    ) -> Result<()> {
        let Some(cc_ids) = &req.cc_user_ids else { return Ok(()) };
        let mut ids: Vec<i64> = Vec::new();
        for id in cc_ids {
            if *id != req.submitter_id && !ids.contains(id) {
                ids.push(*id);
            }
        }
        if ids.is_empty() {
            return Ok(());
        }
        ApprovalModel::insert_cc_records(
            txn,
            instance_id,
            &ids,
            Some(req.submitter_id),
            req.submitter_name.clone(),
            req.cc_reason.clone(),
        ).await?;
        // 写入抄送日志（action=8）
        ApprovalModel::insert_log_with_target(
            txn,
            instance_id,
            node_key,
            node_name,
            req.submitter_id,
            req.submitter_name.clone(),
            8,
            req.cc_reason.clone(),
            None, None, None, None,
        ).await?;
        Ok(())
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

        let flow_data = ApprovalModel::find_instance_by_id_raw(db, req.instance_id).await?
            .ok_or_else(|| Error::from("审批实例不存在"))?;
        let (nodes, edges) = ApprovalModel::get_instance_flow_data(db, &flow_data).await?;

        // 驳回时必须填写理由（后端兜底校验，不能只依赖前端）
        if req.action == 2 && req.comment.as_deref().map(str::trim).unwrap_or("").is_empty() {
            return Err(Error::from("驳回时必须填写理由"));
        }

        let current_node_key = instance.current_node_key.as_ref()
            .ok_or_else(|| Error::from("当前节点为空"))?;

        let current_node = nodes.iter().find(|n| n.node_key.as_deref() == Some(current_node_key))
            .ok_or_else(|| Error::from("当前节点不存在"))?;

        let node_name = current_node.node_name.clone().unwrap_or_default();

        let current_node_key = current_node_key.clone();
        let candidate_approvers = instance.candidate_approvers.clone();
        let submitter_id = instance.submitter_id;
        let extra_data = instance.extra_data.clone().unwrap_or_else(|| serde_json::json!({}));
        let wl_approver_id = req.approver_id;
        let wl_approver_name = req.approver_name.clone();
        let wl_action = req.action;
        let wl_comment = req.comment.clone();
        let wl_instance_id = req.instance_id;
        let req = (*req).clone();
        let flow_code = flow_data.flow_code.clone().unwrap_or_default();

        // 审批日志写入 + 实例状态变更：事务包裹，保证原子性
        db.transaction::<_, (), Error>(|txn| {
            Box::pin(async move {
                ApprovalModel::insert_log(txn, req.instance_id, &current_node_key, &node_name, &req).await?;

                match req.action {
                    1 => {
                        // 入职定薪：当前节点通过时保存该环节填写的定薪数据（幂等覆盖，仅 hire_approval 生效）
                        Self::save_hire_salary_stage_if_needed(
                            txn,
                            req.instance_id,
                            &current_node_key,
                            &flow_code,
                            &req,
                        )
                        .await?;
                        // 通过
                        match approve_mode {
                            1 => {
                                // 或签：任一通过即流转到下一节点
                                Self::advance_to_next_node(txn, req.instance_id, &current_node_key, &nodes, &edges, submitter_id, &extra_data).await?;
                            }
                            2 => {
                                // 会签：全部通过才流转
                                let processed = ApprovalModel::append_processed_approver(txn, req.instance_id, req.approver_id).await?;
                                if processed.len() >= candidate_approvers.len() {
                                    // 所有候选审批人均已通过，流转到下一节点
                                    Self::advance_to_next_node(txn, req.instance_id, &current_node_key, &nodes, &edges, submitter_id, &extra_data).await?;
                                }
                                // 否则等待其他审批人处理
                            }
                            3 => {
                                // 依次审批：按候选池顺序逐个审批
                                let processed = ApprovalModel::append_processed_approver(txn, req.instance_id, req.approver_id).await?;
                                if processed.len() >= candidate_approvers.len() {
                                    // 全部审批完成，流转到下一节点
                                    Self::advance_to_next_node(txn, req.instance_id, &current_node_key, &nodes, &edges, submitter_id, &extra_data).await?;
                                } else {
                                    // 更新当前审批人为候选池中下一个未处理的人
                                    let next_approver = candidate_approvers[processed.len()];
                                    ApprovalModel::update_current_approver(txn, req.instance_id, next_approver).await?;
                                }
                            }
                            _ => {
                                // 默认按或签处理
                                Self::advance_to_next_node(txn, req.instance_id, &current_node_key, &nodes, &edges, submitter_id, &extra_data).await?;
                            }
                        }
                    }
                    2 => {
                        // 驳回：直接结束实例（无论何种审批模式）
                        ApprovalModel::finish_instance(txn, req.instance_id, 4).await?;
                    }
                    _ => return Err(Error::from("无效的操作类型")),
                }

                // B5：离职审批流转完成（实例 status=3）后，事务内创建交接单；失败随事务整体回滚
                if let Some(inst_now) = ApprovalModel::find_instance_by_id_raw(txn, req.instance_id).await? {
                    if inst_now.status == Some(3) && inst_now.business_type.as_deref() == Some("resign") {
                        crate::modules::system::service::resign_service::create_handover_after_approval(
                            txn,
                            req.instance_id,
                        )
                        .await?;
                    }
                    // 入职定薪：流程完成（status=3）后，事务内按财务环节数据生成员工薪资档案
                    if inst_now.status == Some(3) && inst_now.flow_code.as_deref() == Some("hire_approval") {
                        crate::modules::system::service::hire_salary_service::create_employee_salary_from_approval(
                            txn,
                            req.instance_id,
                        )
                        .await?;
                    }
                }
                Ok(())
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

        // 用户审核：审批通过（实例完成）后自动启用用户
        Self::finish_user_audit_if_approved(db, wl_instance_id).await;

        // B10：审批结果站内通知（best-effort，失败不影响主流程）
        // 通知发起人审批结果 + 当前节点审批人新待办 + 离职审批通过时通知交接确认人
        Self::notify_after_process(db, wl_instance_id).await;

        // 工作日志埋点（审批通过/驳回），不影响主业务
        let action_name = if wl_action == 1 { "审批通过" } else { "驳回审批" };
        let result_val = if wl_action == 1 { 1 } else { 2 };
        let log_dto = WorkLogCreateDTO {
            user_id: wl_approver_id,
            user_name: wl_approver_name,
            action_type: Some(1),
            action_name: Some(action_name.to_string()),
            business_type: Some(instance.business_type.clone()),
            business_id: Some(instance.business_id),
            business_title: instance.business_title.clone(),
            description: wl_comment,
            result: Some(result_val),
            work_date: Some(chrono::Utc::now().naive_utc().date()),
        };
        let _ = work_log_service::insert(db, &log_dto).await;

        Ok(())
    }

    /// 审批待办列表
    pub async fn find_instance_list(db: &DatabaseConnection, approver_id: i64, page_num: u64, page_size: u64) -> Result<ResultPage<Vec<ApprovalInstanceVO>>> {
        ApprovalModel::find_instance_list(db, approver_id, page_num, page_size).await
    }

    /// 待办列表（支持 business_type/status/business_title 筛选）
    pub async fn find_instance_list_filtered(
        db: &DatabaseConnection,
        approver_id: i64,
        business_type: Option<&str>,
        status: Option<i32>,
        business_title: Option<&str>,
        page_num: u64,
        page_size: u64,
    ) -> Result<ResultPage<Vec<ApprovalInstanceVO>>> {
        ApprovalModel::find_instance_list_filtered(
            db,
            approver_id,
            business_type,
            status,
            business_title,
            page_num,
            page_size,
        )
        .await
    }

    /// 已办列表：我处理过的全部实例（含已结束），支持 business_type/status/business_title 筛选
    pub async fn find_done_instance_list(
        db: &DatabaseConnection,
        approver_id: i64,
        business_type: Option<&str>,
        status: Option<i32>,
        business_title: Option<&str>,
        page_num: u64,
        page_size: u64,
    ) -> Result<ResultPage<Vec<ApprovalInstanceVO>>> {
        ApprovalModel::find_done_instance_list(
            db,
            approver_id,
            business_type,
            status,
            business_title,
            page_num,
            page_size,
        )
        .await
    }

    /// 审批实例详情
    pub async fn find_instance_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<ApprovalInstanceVO>> {
        ApprovalModel::find_instance_by_id(db, id).await
    }

    /// 某业务单据的全部审批实例（历史），按提交时间正序
    /// 用于发票"流转记录"按单据维度聚合展示：历次提交（含已驳回/已撤回的旧实例）全部保留、可完整追溯。
    pub async fn find_instance_history(
        db: &DatabaseConnection,
        business_type: &str,
        business_id: i64,
    ) -> Result<Vec<ApprovalInstanceVO>> {
        ApprovalModel::find_instance_history(db, business_type, business_id).await
    }

    // ============ Private helpers ============

    /// 构建用户审核（business_type=user）的 extra_data：
    /// - userLevel：按被审核用户角色名分级（含「总监」→director；含「经理/管理员」→manager；其余→employee），
    ///   供审批流条件分支（如 userLevel=="director"）分流
    /// - userDeptId：被审核用户所在部门，供部门负责人链(type=7)节点基于被审核用户解析
    /// 被审核用户不存在时返回 None（不注入，走默认 employee 链）
    async fn build_user_audit_extra(
        db: &DatabaseConnection,
        req: &ApprovalSubmitRequest,
    ) -> Result<Option<serde_json::Value>> {
        let admin = AdminEntity::find_by_id(req.business_id)
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        if admin.is_none() {
            return Ok(None);
        }

        let role_names = Self::find_user_role_names(db, req.business_id).await?;
        let level = if role_names.iter().any(|r| r.contains("总监")) {
            "director"
        } else if role_names.iter().any(|r| r.contains("经理") || r.contains("管理员")) {
            "manager"
        } else {
            "employee"
        };

        let mut extra = req.extra_data.clone().unwrap_or_else(|| serde_json::json!({}));
        if let serde_json::Value::Object(map) = &mut extra {
            map.insert("userLevel".to_string(), serde_json::json!(level));
            if let Ok(Some(dept_id)) = ApprovalModel::find_user_dept_id(db, req.business_id).await {
                map.insert("userDeptId".to_string(), serde_json::json!(dept_id));
            }
        }
        Ok(Some(extra))
    }

    /// 查询用户的所有角色名称（未删除角色），用于角色分级
    async fn find_user_role_names(db: &impl ConnectionTrait, user_id: i64) -> Result<Vec<String>> {
        let merges = RoleMergeEntity::find()
            .filter(RoleMergeColumn::AdminId.eq(user_id))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        let role_ids: Vec<i64> = merges.into_iter().filter_map(|m| m.role_id).collect();
        if role_ids.is_empty() {
            return Ok(Vec::new());
        }
        let roles = RoleEntity::find()
            .filter(RoleColumn::Id.is_in(role_ids))
            .filter(RoleColumn::Deleted.eq(0))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(roles.into_iter().filter_map(|r| r.role_name).collect())
    }

    /// 解析审批流推进时"部门负责人链(type=7)"节点的起始部门ID
    /// 用户审核场景（extra_data 含 userDeptId）取被审核用户所在部门，
    /// 其他场景回退到发起人所在部门
    async fn resolve_flow_dept_id(
        db: &impl ConnectionTrait,
        submitter_id: i64,
        extra_data: &serde_json::Value,
    ) -> Result<Option<i64>> {
        if let Some(dept_id) = extra_data.get("userDeptId").and_then(|v| v.as_i64()) {
            return Ok(Some(dept_id));
        }
        ApprovalModel::find_user_dept_id(db, submitter_id).await
    }

    /// 用户审核审批：business_type="user" 的实例审批通过（status=3）后自动启用用户
    /// （audit_status=1 + status=1），best-effort，失败不影响审批主流程
    async fn finish_user_audit_if_approved(db: &DatabaseConnection, instance_id: i64) {
        if let Ok(Some(inst)) = ApprovalModel::find_instance_by_id_raw(db, instance_id).await {
            if inst.business_type.as_deref() == Some("user") && inst.status == Some(3) {
                let _ = crate::modules::system::service::admin_service::update_audit_status(
                    db,
                    inst.business_id.unwrap_or_default(),
                    1,
                )
                .await;
            }
        }
    }

    /// B10：站内通知配置（仅人事相关流程 user/resign 发通知，其他业务类型保持原行为不打扰）
    /// 返回 (通知类型, 落地页链接)；返回 None 表示不发通知
    fn notify_config(business_type: &str) -> Option<(i32, &'static str)> {
        match business_type {
            "user" | "resign" => Some((9, "/system/user")),
            _ => None,
        }
    }

    /// 当前节点应被通知的审批人ID：
    /// 依次审批(approve_mode=3)=仅当前审批人；或签/会签(1/2)=全部候选审批人
    fn notify_approver_ids(inst: &ApprovalInstanceVO) -> Vec<i64> {
        if inst.approve_mode == 3 {
            inst.current_approver_id
                .filter(|&id| id > 0)
                .into_iter()
                .collect()
        } else {
            inst.candidate_approvers
                .iter()
                .copied()
                .filter(|&id| id > 0)
                .collect()
        }
    }

    /// B10：提交审批后站内通知（best-effort，失败不影响主流程）
    /// - 实例待审（1/2）→ 通知当前节点候选审批人"有新的待办"
    /// - 实例自动完成（3）→ 通知发起人"已通过"
    async fn notify_after_submit(db: &DatabaseConnection, instance_id: i64) {
        let Ok(Some(inst)) = ApprovalModel::find_instance_by_id(db, instance_id).await else {
            return;
        };
        let Some((ntype, link)) = Self::notify_config(&inst.business_type) else {
            return;
        };
        let biz_title = inst.business_title.clone().unwrap_or_else(|| "审批申请".to_string());
        if inst.status == 1 || inst.status == 2 {
            let approvers: Vec<i64> = Self::notify_approver_ids(&inst)
                .into_iter()
                .filter(|&id| id != inst.submitter_id)
                .collect();
            if approvers.is_empty() {
                return;
            }
            let content = format!(
                "{} 提交了审批申请，请您及时处理。",
                inst.submitter_name.clone().unwrap_or_else(|| "申请人".to_string())
            );
            for aid in approvers {
                let _ = NotificationService::send_system_notification(
                    db,
                    aid,
                    format!("【待审批】{}", biz_title),
                    content.clone(),
                    ntype,
                    Some(link.to_string()),
                )
                .await;
            }
        } else if inst.status == 3 {
            let _ = NotificationService::send_system_notification(
                db,
                inst.submitter_id,
                format!("【审批通过】{}", biz_title),
                "您的审批申请已通过（系统自动通过）。".to_string(),
                ntype,
                Some(link.to_string()),
            )
            .await;
        }
    }

    /// B10：审批结果站内通知（best-effort，失败不影响主流程）
    /// - 终态 3/4 → 通知发起人审批结果
    /// - 流转中（1/2）→ 通知当前节点审批人"有新待办"
    /// - 离职审批通过（resign + status=3）→ 通知交接确认人
    async fn notify_after_process(db: &DatabaseConnection, instance_id: i64) {
        let Ok(Some(inst)) = ApprovalModel::find_instance_by_id(db, instance_id).await else {
            return;
        };
        let Some((ntype, link)) = Self::notify_config(&inst.business_type) else {
            return;
        };
        let biz_title = inst.business_title.clone().unwrap_or_else(|| "审批申请".to_string());
        match inst.status {
            3 => {
                let _ = NotificationService::send_system_notification(
                    db,
                    inst.submitter_id,
                    format!("【审批通过】{}", biz_title),
                    "您的审批申请已通过。".to_string(),
                    ntype,
                    Some(link.to_string()),
                )
                .await;
                // 离职审批通过：通知交接确认人
                if inst.business_type == "resign" {
                    let _ =
                        crate::modules::system::service::resign_service::notify_handover_assignees(
                            db,
                            inst.business_id,
                        )
                        .await;
                }
            }
            4 => {
                let _ = NotificationService::send_system_notification(
                    db,
                    inst.submitter_id,
                    format!("【审批驳回】{}", biz_title),
                    "您的审批申请已被驳回，请查看驳回理由。".to_string(),
                    ntype,
                    Some(link.to_string()),
                )
                .await;
            }
            _ => {
                let approvers: Vec<i64> = Self::notify_approver_ids(&inst)
                    .into_iter()
                    .filter(|&id| id != inst.submitter_id)
                    .collect();
                if approvers.is_empty() {
                    return;
                }
                let content = format!(
                    "{} 的审批流转到您，请及时处理。",
                    inst.submitter_name.clone().unwrap_or_else(|| "申请人".to_string())
                );
                for aid in approvers {
                    let _ = NotificationService::send_system_notification(
                        db,
                        aid,
                        format!("【待审批】{}", biz_title),
                        content.clone(),
                        ntype,
                        Some(link.to_string()),
                    )
                    .await;
                }
            }
        }
    }

    /// 审批撤销/退回后回写业务侧审批状态（合同/订单/发票/出入库单）。
    /// 失败时忽略错误（best-effort），不影响审批主流程。
    async fn sync_business_approval_status(
        db: &impl ConnectionTrait,
        business_type: &str,
        business_id: i64,
        approval_status: i32,
    ) {
        let value = sea_orm::sea_query::Expr::value(approval_status);
        match business_type {
            "contract" => {
                let _ = contract::Entity::update_many()
                    .col_expr(contract::Column::ApprovalStatus, value.clone())
                    .filter(contract::Column::Id.eq(business_id))
                    .exec(db)
                    .await;
            }
            "order" => {
                let _ = order::Entity::update_many()
                    .col_expr(order::Column::ApprovalStatus, value)
                    .filter(order::Column::Id.eq(business_id))
                    .exec(db)
                    .await;
            }
            // 发票：撤回/退回后回写审批状态，保证发票可重新提交（否则会卡在"待审批"无法重提）
            "invoice" => {
                // 引擎状态 6=退回发起人 → 已驳回(4)，其余（撤回等）→ 草稿(0)
                let biz_status = if approval_status == 6 { 4 } else { 0 };
                let mut update = invoice::Entity::update_many()
                    .col_expr(
                        invoice::Column::ApprovalStatus,
                        sea_orm::sea_query::Expr::value(biz_status),
                    )
                    .filter(invoice::Column::Id.eq(business_id));
                // 撤回（biz_status=0）：instance_id 清空，实例保留为历史（参考规则 2.3 / 验收 C8、H2）
                if biz_status == 0 {
                    update = update.col_expr(
                        invoice::Column::InstanceId,
                        sea_orm::sea_query::Expr::value(None::<i64>),
                    );
                }
                let _ = update.exec(db).await;
            }
            // 入库单：审批引擎状态 0=撤回→草稿(0)，6=退回发起人→已驳回(4)
            // （审批通过/驳回由入库单自身 audit/reject 流程处理，不经此处回写）
            "inbound" => {
                let biz_status = if approval_status == 6 { 4 } else { 0 };
                let _ = crate::modules::inventory::entity::inbound::Entity::update_many()
                    .col_expr(
                        crate::modules::inventory::entity::inbound::Column::Status,
                        sea_orm::sea_query::Expr::value(biz_status),
                    )
                    .filter(crate::modules::inventory::entity::inbound::Column::Id.eq(business_id))
                    .exec(db)
                    .await;
            }
            // 出库单：审批引擎状态 0=撤回→草稿(0)，6=退回发起人→已驳回(4)
            "outbound" => {
                let biz_status = if approval_status == 6 { 4 } else { 0 };
                let _ = crate::modules::inventory::entity::outbound::Entity::update_many()
                    .col_expr(
                        crate::modules::inventory::entity::outbound::Column::Status,
                        sea_orm::sea_query::Expr::value(biz_status),
                    )
                    .filter(crate::modules::inventory::entity::outbound::Column::Id.eq(business_id))
                    .exec(db)
                    .await;
            }
            _ => {}
        }
    }

    /// 节点审批通过后自动抄送（读取节点的 cc_user_ids 配置）
    async fn auto_cc_on_node_pass(
        db: &impl ConnectionTrait,
        instance_id: i64,
        node: &approval_flow_node::Model,
    ) {
        if let Some(cc_ids) = &node.cc_user_ids {
            let user_ids: Vec<i64> = cc_ids.as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
                .unwrap_or_default();
            if !user_ids.is_empty() {
                let _ = ApprovalModel::insert_cc_records(
                    db,
                    instance_id,
                    &user_ids,
                    Some(0),
                    Some("系统".to_string()),
                    Some("节点审批通过自动抄送".to_string()),
                ).await;
            }
        }
    }

    /// 从当前节点查找下一节点并推进实例（处理条件分支）
    async fn advance_to_next_node(
        db: &impl ConnectionTrait,
        instance_id: i64,
        current_node_key: &str,
        nodes: &[approval_flow_node::Model],
        edges: &[approval_flow_edge::Model],
        submitter_id: i64,
        extra_data: &serde_json::Value,
    ) -> Result<()> {
        // 节点审批通过后自动抄送（cc_user_ids 不为空时触发）
        if let Some(current_node) = nodes.iter().find(|n| n.node_key.as_deref() == Some(current_node_key)) {
            Self::auto_cc_on_node_pass(db, instance_id, current_node).await;
        }

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
        db: &impl ConnectionTrait,
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
                let submitter_dept_id = Self::resolve_flow_dept_id(db, submitter_id, extra_data).await?;
                let raw_candidates = ApprovalModel::resolve_approvers(
                    db,
                    next_node.approver_type,
                    next_node.approver_id,
                    submitter_id,
                    submitter_dept_id,
                ).await?;
                // 自审回避
                let mut candidates = ApprovalModel::filter_self_approvers(raw_candidates.clone(), submitter_id);

                let next_node_name = next_node.node_name.clone().unwrap_or_default();
                let next_node_approver_type = next_node.approver_type;

                if candidates.is_empty() {
                    if ApprovalModel::is_direct_manager_node(next_node_approver_type) {
                        // 直属上级场景：候选为空（已到顶层），自动通过并继续流转
                        ApprovalModel::update_instance_node(db, instance_id, next_key, 0, &[]).await?;
                        Self::insert_auto_pass_log(db, instance_id, next_key, &next_node_name).await?;
                        return Box::pin(Self::advance_to_next_node(db, instance_id, next_key, nodes, edges, submitter_id, extra_data)).await;
                    } else if !raw_candidates.is_empty() {
                        // 自审回避后候选为空但原候选非空（如指定角色下仅有提交人自己）：
                        // 退化为允许提交人自己审批，避免流程无法流转
                        candidates = raw_candidates;
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

    /// 入职定薪：按当前节点保存该环节填写的定薪数据（仅 hire_approval 流程生效，幂等覆盖）
    /// 节点 key → 环节：部门经理审批(n_1787687341591_1)=1、人事经理审批(hr_manager)=2、
    /// CEO终审(ceo_approval)=3、财务定薪录入(finance_manager)=4
    async fn save_hire_salary_stage_if_needed(
        txn: &sea_orm::DatabaseTransaction,
        instance_id: i64,
        node_key: &str,
        flow_code: &str,
        req: &ApprovalProcessRequest,
    ) -> Result<()> {
        if flow_code != "hire_approval" {
            return Ok(());
        }
        let stage = match node_key {
            "n_1787687341591_1" => crate::modules::system::service::hire_salary_service::STAGE_DEPT_MANAGER,
            "hr_manager" => crate::modules::system::service::hire_salary_service::STAGE_HR,
            "ceo_approval" => crate::modules::system::service::hire_salary_service::STAGE_CEO,
            "finance_manager" => crate::modules::system::service::hire_salary_service::STAGE_FINANCE,
            _ => return Ok(()),
        };
        crate::modules::system::service::hire_salary_service::save_stage(
            txn,
            instance_id,
            node_key,
            stage,
            req,
        )
        .await
    }

    /// 写入"系统自动通过"审批日志（用于直属上级节点候选为空时自动流转）
    /// approver_id=0 表示系统，action=1 表示通过
    async fn insert_auto_pass_log(
        db: &impl ConnectionTrait,
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
            ..Default::default()
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
        // 校验连线端点必须存在于节点列表中，防止保存引用丢失节点的脏数据
        let node_keys: std::collections::HashSet<&str> =
            req.nodes.iter().map(|n| n.node_key.as_str()).collect();
        for edge in &req.edges {
            if !node_keys.contains(edge.source.as_str()) {
                return Err(Error::from(format!(
                    "连线起点节点[{}]不存在于节点列表中",
                    edge.source
                )));
            }
            if !node_keys.contains(edge.target.as_str()) {
                return Err(Error::from(format!(
                    "连线终点节点[{}]不存在于节点列表中",
                    edge.target
                )));
            }
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
                let value = expr[pos + op.len()..].trim().trim_matches('"').trim_matches('\'');
                // 优先尝试字符串比较（支持 customerType=="VIP" 等场景）
                if let Some(actual_str) = data.get(field).and_then(|v| v.as_str()) {
                    return match *op {
                        "==" => actual_str == value,
                        "!=" => actual_str != value,
                        _ => false, // 字符串不支持大小比较
                    };
                }
                // 回退到数值比较
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
        db: &impl ConnectionTrait,
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
                        let submitter_dept_id = Self::resolve_flow_dept_id(db, submitter_id, extra_data).await?;
                        let raw_candidates = ApprovalModel::resolve_approvers(
                            db,
                            target.approver_type,
                            target.approver_id,
                            submitter_id,
                            submitter_dept_id,
                        ).await?;
                        // 自审回避
                        let mut candidates = ApprovalModel::filter_self_approvers(raw_candidates.clone(), submitter_id);

                        let target_node_name = target.node_name.clone().unwrap_or_default();
                        let target_approver_type = target.approver_type;

                        if candidates.is_empty() {
                            if ApprovalModel::is_direct_manager_node(target_approver_type) {
                                // 直属上级场景：候选为空，自动通过并继续流转
                                ApprovalModel::update_instance_node(db, instance_id, target_key, 0, &[]).await?;
                                Self::insert_auto_pass_log(db, instance_id, target_key, &target_node_name).await?;
                                return Box::pin(Self::advance_to_next_node(db, instance_id, target_key, nodes, edges, submitter_id, extra_data)).await;
                            } else if !raw_candidates.is_empty() {
                                // 自审回避后候选为空但原候选非空：退化为允许提交人自己审批
                                candidates = raw_candidates;
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

    /// 公共校验：加载审批实例 + 状态校验 + 取当前节点信息
    /// 6 个增强功能的共同前置逻辑，消除重复样板
    async fn load_active_instance(
        db: &DatabaseConnection,
        instance_id: i64,
        action_name: &str,
    ) -> Result<(ApprovalInstanceVO, String, String)> {
        let instance = ApprovalModel::find_instance_by_id(db, instance_id)
            .await?
            .ok_or_else(|| Error::from("审批实例不存在"))?;

        if instance.status != 1 && instance.status != 2 {
            return Err(Error::from(format!("当前审批状态不允许{}", action_name)));
        }

        let current_node_key = instance.current_node_key.clone().unwrap_or_default();
        let current_node_name = instance
            .flow_nodes
            .iter()
            .find(|n| n.node_key == current_node_key)
            .map(|n| n.node_name.clone())
            .unwrap_or_default();

        Ok((instance, current_node_key, current_node_name))
    }

    /// 取消（撤回）审批实例 - 仅发起人可操作
    /// action=7, 实例状态置为 5=已撤回
    pub async fn cancel_instance(
        db: &DatabaseConnection,
        req: &ApprovalCancelRequest,
        operator_id: i64,
        operator_name: &str,
    ) -> Result<()> {
        let (instance, current_node_key, current_node_name) =
            Self::load_active_instance(db, req.instance_id, "撤回").await?;

        // 权限校验：仅发起人可撤回
        if instance.submitter_id != operator_id {
            return Err(Error::from("仅发起人可撤回审批"));
        }

        // 撤回时必须填写理由（后端兜底校验，不能只依赖前端）
        let cancel_reason = req.cancel_reason.clone().unwrap_or_default();
        if cancel_reason.trim().is_empty() {
            return Err(Error::from("撤回时必须填写理由"));
        }
        let instance_id = req.instance_id;
        let operator_name = operator_name.to_string();

        // 审批日志写入 + 实例状态变更：事务包裹，保证原子性
        let business_type = instance.business_type.clone();
        let business_id = instance.business_id;
        db.transaction::<_, (), Error>(|txn| {
            Box::pin(async move {
                // 写入取消日志
                ApprovalModel::insert_log_with_target(
                    txn,
                    instance_id,
                    &current_node_key,
                    &current_node_name,
                    operator_id,
                    Some(operator_name.clone()),
                    7, // action=7 取消
                    Some(cancel_reason.clone()),
                    None, None, None, None,
                ).await?;

                // 更新取消原因
                ApprovalModel::update_cancel_reason(txn, instance_id, &cancel_reason).await?;

                // 结束实例，状态=5 已撤回
                ApprovalModel::finish_instance(txn, instance_id, 5).await?;

                // 撤销后回写业务侧状态：合同/订单 approval_status 回到 0（草稿），便于发起人重新编辑提交
                Self::sync_business_approval_status(txn, &business_type, business_id, 0).await;

                Ok(())
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

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
        let (instance, current_node_key, current_node_name) =
            Self::load_active_instance(db, req.instance_id, "退回").await?;

        // 权限校验：审批人必须在候选池中
        if !instance.candidate_approvers.contains(&operator_id) {
            return Err(Error::from("您不是当前节点的审批人"));
        }

        // 退回到发起人时必须填写理由（后端兜底校验，不能只依赖前端）
        if req.reject_to_node_key.is_none()
            && req.comment.as_deref().map(str::trim).unwrap_or("").is_empty()
        {
            return Err(Error::from("退回发起人时必须填写理由"));
        }

        let inst_submitter_id = instance.submitter_id;
        let inst_submitter_name = instance.submitter_name.clone();
        let inst_business_type = instance.business_type.clone();
        let inst_business_id = instance.business_id;
        let inst_extra_data = instance.extra_data.clone().unwrap_or_else(|| serde_json::json!({}));
        let inst_flow_nodes = instance.flow_nodes.clone();
        let req = (*req).clone();
        let operator_name = operator_name.to_string();

        // 审批日志写入 + 实例状态变更：事务包裹，保证原子性
        db.transaction::<_, (), Error>(|txn| {
            Box::pin(async move {
                match &req.reject_to_node_key {
                    None => {
                        // 退回到发起人：标记需要重新提交，状态置为 6=待修改
                        ApprovalModel::insert_log_with_target(
                            txn,
                            req.instance_id,
                            &current_node_key,
                            &current_node_name,
                            operator_id,
                            Some(operator_name.to_string()),
                            6, // action=6 退回
                            req.comment.clone(),
                            Some(inst_submitter_id),
                            inst_submitter_name.clone(),
                            None, None,
                        ).await?;

                        // 更新实例：current_approver 设为发起人，needs_resubmit=1，状态=6
                        ApprovalModel::update_instance_node_with_extras(
                            txn,
                            req.instance_id,
                            "start",
                            inst_submitter_id,
                            &[inst_submitter_id],
                            None, None, None,
                            Some(1), // needs_resubmit=1
                        ).await?;

                        // 状态改为 6=待修改（退回到发起人）
                        let now = chrono::Utc::now().naive_utc();
                        InstanceEntity::update_many()
                            .col_expr(InstanceColumn::Status, sea_orm::sea_query::Expr::value(6))
                            .col_expr(InstanceColumn::UpdateTime, sea_orm::sea_query::Expr::value(now))
                            .filter(InstanceColumn::Id.eq(req.instance_id))
                            .exec(txn)
                            .await
                            .map_err(|e| Error::from(e.to_string()))?;

                        // 退回到发起人后回写业务侧状态：合同/订单 approval_status 置为 6（待修改），便于发起人修改后重新提交
                        Self::sync_business_approval_status(txn, &inst_business_type, inst_business_id, 6).await;
                    }
                    Some(target_key) => {
                        // 退回到指定节点
                        let target_node_name = inst_flow_nodes.iter()
                            .find(|n| n.node_key == *target_key)
                            .map(|n| n.node_name.clone())
                            .ok_or_else(|| Error::from(format!("目标节点 {} 不存在", target_key)))?;

                        // 读取目标节点配置：优先从快照读取
                        let inst_raw = ApprovalModel::find_instance_by_id_raw(txn, req.instance_id).await?
                            .ok_or_else(|| Error::from("审批实例不存在"))?;
                        let (nodes, _edges) = ApprovalModel::get_instance_flow_data(txn, &inst_raw).await?;
                        let target_node_cfg = nodes.iter()
                            .find(|n| n.node_key.as_deref() == Some(target_key))
                            .ok_or_else(|| Error::from("目标节点配置不存在"))?;

                        let submitter_dept_id = Self::resolve_flow_dept_id(txn, inst_submitter_id, &inst_extra_data).await?;
                        let raw_candidates = ApprovalModel::resolve_approvers(
                            txn,
                            target_node_cfg.approver_type,
                            target_node_cfg.approver_id,
                            inst_submitter_id,
                            submitter_dept_id,
                        ).await?;
                        let mut candidates = ApprovalModel::filter_self_approvers(raw_candidates.clone(), inst_submitter_id);

                        if candidates.is_empty() {
                            // 动态节点（type=6/7）候选为空时自动通过，而非报错
                            if ApprovalModel::is_direct_manager_node(target_node_cfg.approver_type) {
                                let (nodes2, edges2) = ApprovalModel::get_instance_flow_data(txn, &inst_raw).await?;
                                ApprovalModel::update_instance_node(txn, req.instance_id, target_key, 0, &[]).await?;
                                Self::insert_auto_pass_log(txn, req.instance_id, target_key, &target_node_name).await?;
                                Self::advance_to_next_node(txn, req.instance_id, target_key, &nodes2, &edges2, inst_submitter_id, &inst_extra_data).await?;
                                return Ok(());
                            } else if !raw_candidates.is_empty() {
                                // 自审回避后候选为空但原候选非空：退化为允许发起人自己审批
                                candidates = raw_candidates;
                            } else {
                                return Err(Error::from("目标节点未解析到候选审批人"));
                            }
                        }

                        let primary_approver = candidates[0];

                        ApprovalModel::insert_log_with_target(
                            txn,
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
                            txn,
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
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

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

        let req = (*req).clone();
        let operator_name = operator_name.to_string();

        // 审批日志写入 + 实例状态变更：事务包裹，保证原子性
        db.transaction::<_, (), Error>(|txn| {
            Box::pin(async move {
                // 写入转办日志
                ApprovalModel::insert_log_with_target(
                    txn,
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
                    txn,
                    req.instance_id,
                    &current_node_key,
                    req.target_user_id,
                    &[req.target_user_id],
                    Some(operator_id), // transfer_from_id
                    None, None, None,
                ).await?;

                Ok(())
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

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

        // 更新候选池：当前审批人改为被委派人，候选池增加被委派人，记录委派来源
        let new_candidates = {
            let mut nc = instance.candidate_approvers.clone();
            if !nc.contains(&req.target_user_id) {
                nc.push(req.target_user_id);
            }
            nc
        };
        let req = (*req).clone();
        let operator_name = operator_name.to_string();

        // 审批日志写入 + 实例状态变更：事务包裹，保证原子性
        db.transaction::<_, (), Error>(|txn| {
            Box::pin(async move {
                // 写入委派日志
                ApprovalModel::insert_log_with_target(
                    txn,
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

                ApprovalModel::update_instance_node_with_extras(
                    txn,
                    req.instance_id,
                    &current_node_key,
                    req.target_user_id,
                    &new_candidates,
                    None,
                    Some(operator_id), // delegate_from_id
                    None, None,
                ).await?;

                Ok(())
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

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

        let (instance, current_node_key, current_node_name) =
            Self::load_active_instance(db, req.instance_id, "加签").await?;

        // 权限校验：审批人必须在候选池中
        if !instance.candidate_approvers.contains(&operator_id) {
            return Err(Error::from("您不是当前节点的审批人"));
        }

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

        // 更新候选池：将新加签用户加入候选池
        let new_candidates = {
            let mut nc = instance.candidate_approvers.clone();
            for uid in &new_users {
                if !nc.contains(uid) {
                    nc.push(*uid);
                }
            }
            nc
        };
        let req = (*req).clone();
        let operator_name = operator_name.to_string();

        // 审批日志写入 + 实例状态变更：事务包裹，保证原子性
        db.transaction::<_, (), Error>(|txn| {
            Box::pin(async move {
                // 写入加签日志
                ApprovalModel::insert_log_with_target(
                    txn,
                    req.instance_id,
                    &current_node_key,
                    &current_node_name,
                    operator_id,
                    Some(operator_name.to_string()),
                    5, // action=5 加签
                    req.comment.clone().or(Some(format!("加签用户: {}", target_names_str))),
                    None, None, None, None,
                ).await?;

                match req.add_sign_type {
                    1 => {
                        // 前加签：新审批人先审批，当前审批人暂不处理
                        // 当前审批人改为第一个新加签用户
                        ApprovalModel::update_instance_node_with_extras(
                            txn,
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
                            .exec(txn)
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
                            .exec(txn)
                            .await
                            .map_err(|e| Error::from(e.to_string()))?;
                    }
                    _ => return Err(Error::from("无效的加签类型，1=前加签,2=后加签,3=并加签")),
                }
                Ok(())
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

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

        // 提取当前节点信息（事务前准备）
        let cc_node_key = instance.current_node_key.clone().unwrap_or_default();
        let cc_node_name = instance.flow_nodes.iter()
            .find(|n| Some(n.node_key.clone()) == instance.current_node_key)
            .map(|n| n.node_name.clone())
            .unwrap_or_default();

        let req = (*req).clone();
        let operator_name = operator_name.to_string();

        // 审批日志写入 + 抄送记录写入：事务包裹，保证原子性
        db.transaction::<_, (), Error>(|txn| {
            Box::pin(async move {
                // 写入抄送日志
                ApprovalModel::insert_log_with_target(
                    txn,
                    req.instance_id,
                    &cc_node_key,
                    &cc_node_name,
                    operator_id,
                    Some(operator_name.to_string()),
                    8, // action=8 抄送
                    req.cc_reason.clone(),
                    None, None, None, None,
                ).await?;

                // 插入抄送记录
                ApprovalModel::insert_cc_records(
                    txn,
                    req.instance_id,
                    &req.user_ids,
                    Some(operator_id),
                    Some(operator_name.to_string()),
                    req.cc_reason.clone(),
                ).await?;

                Ok(())
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

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
