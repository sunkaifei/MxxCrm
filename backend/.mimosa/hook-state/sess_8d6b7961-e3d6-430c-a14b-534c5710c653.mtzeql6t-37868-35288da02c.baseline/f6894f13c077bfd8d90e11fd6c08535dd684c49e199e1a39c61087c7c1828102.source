//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::collections::HashSet;

use sea_orm::{
    ColumnTrait, ConnectionTrait, DbConn, EntityTrait, QueryFilter, QueryOrder, TransactionTrait,
};

use crate::core::errors::error::{Error, Result};
use crate::modules::approval::entity::{approval_flow_node, approval_instance, approval_log};
use crate::modules::system::entity::admin;
use crate::modules::system::model::onboarding::{
    OnboardingAdminStepVO, OnboardingAdminVO, OnboardingApprovalVO, OnboardingConfigResponse,
    OnboardingConfigVO, OnboardingProgressVO, OnboardingSaveRequest, OnboardingStepSaveDTO,
    OnboardingStepVO, OnboardingConfigModel, OnboardingStepModel,
};
use crate::modules::system::service::profile_service;

/// 内置步骤编码（d29 种子，与代码判定逻辑一一对应）
pub const STEP_PROFILE: &str = "profile_fullfill";
pub const STEP_SUBMIT: &str = "submit_approval";
pub const STEP_CONTRACT: &str = "contract";
pub const STEP_BANK_CARD: &str = "bank_card";

/// 引导卡五态（契约 9.3，前端只渲染不判定）
pub const STATE_PASSED: &str = "passed";
pub const STATE_IN_APPROVAL: &str = "in_approval";
pub const STATE_REJECTED: &str = "rejected";
pub const STATE_PENDING_PROFILE: &str = "pending_profile";
pub const STATE_READY_SUBMIT: &str = "ready_submit";

/// 用户审核流程业务类型
const BUSINESS_TYPE_USER: &str = "user";

/// 保存结果（VersionConflict 由 controller 转换为 409 业务码）
pub enum SaveOutcome {
    Ok(i64),
    VersionConflict,
}

/// 确保配置单行存在（无则插入默认行）
pub async fn ensure_config_row(db: &DbConn) -> Result<crate::modules::system::entity::onboarding_config::Model> {
    match OnboardingConfigModel::find_first(db).await? {
        Some(m) => Ok(m),
        None => {
            let id = OnboardingConfigModel::insert_default(db).await?;
            crate::modules::system::entity::onboarding_config::Entity::find_by_id(id)
                .one(db)
                .await?
                .ok_or_else(|| Error::from("初始化入职引导配置失败"))
        }
    }
}

/// GET /onboarding/config：登录用户返回个人进度视图，管理员态（is_admin）追加配置页全量数据
pub async fn get_config_for_user(
    db: &DbConn,
    user_id: i64,
    is_admin: bool,
) -> Result<OnboardingConfigResponse> {
    let cfg = ensure_config_row(db).await?;
    let steps = OnboardingStepModel::find_all(db, !is_admin).await?;

    // 审核档案与实例数据（管理员态无需个人进度，但仍查询以保持代码路径简单）
    let admin_row = admin::Entity::find_by_id(user_id)
        .filter(admin::Column::Deleted.eq(0))
        .one(db)
        .await?;
    let audit_status = admin_row.as_ref().and_then(|a| a.audit_status);
    let bank_no = admin_row
        .as_ref()
        .and_then(|a| a.bank_card_no.clone())
        .unwrap_or_default();

    // 审批实例：进行中（status IN (1,2)）与最近一次驳回（status=4）
    let ongoing = approval_instance::Entity::find()
        .filter(approval_instance::Column::BusinessType.eq(BUSINESS_TYPE_USER))
        .filter(approval_instance::Column::BusinessId.eq(user_id))
        .filter(approval_instance::Column::Status.is_in([1, 2]))
        .order_by_desc(approval_instance::Column::CreateTime)
        .one(db)
        .await?;
    let rejected_latest = approval_instance::Entity::find()
        .filter(approval_instance::Column::BusinessType.eq(BUSINESS_TYPE_USER))
        .filter(approval_instance::Column::BusinessId.eq(user_id))
        .filter(approval_instance::Column::Status.eq(4))
        .order_by_desc(approval_instance::Column::CreateTime)
        .one(db)
        .await?;

    // 档案缺失项（checklist 有缺项 → pending_profile）
    let missing = if is_admin {
        Vec::new()
    } else {
        profile_service::profile_completeness(db, user_id)
            .await
            .unwrap_or_default()
    };

    // 五态判定（契约 9.3 顺序：passed → in_approval → rejected → pending_profile → ready_submit）
    let state = if audit_status == Some(1) {
        STATE_PASSED
    } else if ongoing.is_some() {
        STATE_IN_APPROVAL
    } else if rejected_latest.is_some() {
        STATE_REJECTED
    } else if !missing.is_empty() {
        STATE_PENDING_PROFILE
    } else {
        STATE_READY_SUBMIT
    };

    // 内置步骤 done 判定（自定义步骤 done 恒 null，不计入 progress）
    let done_profile = missing.is_empty();
    let done_submit = audit_status == Some(1) || ongoing.is_some() || rejected_latest.is_some();
    let done_contract = audit_status == Some(1);
    let done_bank_card = !bank_no.trim().is_empty();
    let done_of = |code: &str| -> bool {
        match code {
            STEP_PROFILE => done_profile,
            STEP_SUBMIT => done_submit,
            STEP_CONTRACT => done_contract,
            STEP_BANK_CARD => done_bank_card,
            _ => false,
        }
    };

    let mut step_vos: Vec<OnboardingStepVO> = Vec::with_capacity(steps.len());
    let mut builtin_total: i64 = 0;
    let mut builtin_done: i64 = 0;
    for s in &steps {
        let done = if is_admin || s.step_type != 1 {
            None
        } else {
            builtin_total += 1;
            let d = done_of(&s.step_code);
            if d {
                builtin_done += 1;
            }
            Some(d)
        };
        step_vos.push(OnboardingStepVO {
            step_code: s.step_code.clone(),
            step_name: s.step_name.clone(),
            step_type: s.step_type,
            link_url: s.link_url.clone(),
            done,
            sort_order: s.sort_order,
        });
    }

    // approval 节点：state=in_approval/rejected 时非空（异步预取节点名与驳回理由）
    let approval = match state {
        STATE_IN_APPROVAL => match &ongoing {
            Some(inst) => Some(OnboardingApprovalVO {
                current_node_name: current_node_name_async(db, inst).await,
                submitted_at: inst.submitted_at,
                reject_reason: None,
            }),
            None => None,
        },
        STATE_REJECTED => match &rejected_latest {
            Some(inst) => Some(OnboardingApprovalVO {
                current_node_name: None,
                submitted_at: inst.submitted_at,
                reject_reason: latest_reject_reason_async(db, inst.id).await,
            }),
            None => None,
        },
        _ => None,
    };

    // 管理员态额外字段：version + 全量步骤（含停用）
    let admin_vo = if is_admin {
        Some(OnboardingAdminVO {
            version: cfg.version,
            steps: steps
                .iter()
                .map(|s| OnboardingAdminStepVO {
                    id: s.id,
                    step_code: s.step_code.clone(),
                    step_name: s.step_name.clone(),
                    step_desc: s.step_desc.clone(),
                    step_type: s.step_type,
                    link_url: s.link_url.clone(),
                    sort_order: s.sort_order,
                    status: s.status,
                })
                .collect(),
        })
    } else {
        None
    };

    Ok(OnboardingConfigResponse {
        state: state.to_string(),
        steps: step_vos,
        progress: OnboardingProgressVO {
            done: builtin_done,
            total: builtin_total,
        },
        config: OnboardingConfigVO {
            announce_enabled: cfg.announce_enabled,
            todo_enabled: cfg.todo_enabled,
            quick_enabled: cfg.quick_enabled,
            quick_preset: cfg.quick_preset.clone(),
        },
        approval,
        admin: admin_vo,
    })
}

/// POST /onboarding/save：全量覆盖式保存（13.3-3 乐观锁 / 13.4-1 链接校验）
pub async fn save_config(
    db: &DbConn,
    req: OnboardingSaveRequest,
    operator: &str,
) -> Result<SaveOutcome> {
    let cfg = ensure_config_row(db).await?;
    let config_req = req
        .config
        .clone()
        .ok_or_else(|| Error::from("缺少 config 配置参数"))?;

    // 乐观锁：version 必传且与库中一致，不匹配返回 409
    let req_version = config_req
        .version
        .ok_or_else(|| Error::from("缺少 version 参数，请刷新页面后重试"))?;
    if req_version != cfg.version {
        return Ok(SaveOutcome::VersionConflict);
    }

    // 链接校验（步骤 link_url + 快捷入口 path，白名单字符集，拒绝协议相对地址与外部链接）
    if let Some(steps) = &req.steps {
        for s in steps {
            validate_link(s.link_url.as_deref())?;
        }
    }
    if let Some(preset) = &config_req.quick_preset {
        validate_quick_preset(preset)?;
    }

    // 步骤编码唯一性
    if let Some(steps) = &req.steps {
        let mut seen: HashSet<String> = HashSet::new();
        for s in steps {
            let code = s.step_code.clone().unwrap_or_default().trim().to_string();
            if code.is_empty() {
                return Err(Error::from("步骤编码不能为空"));
            }
            if !seen.insert(code) {
                return Err(Error::from(format!("步骤编码重复：{}", s.step_code.clone().unwrap_or_default().trim())));
            }
            if s.step_name.clone().unwrap_or_default().trim().is_empty() {
                return Err(Error::from("步骤名称不能为空"));
            }
        }
    }

    let new_version = cfg.version + 1;
    let row_id = cfg.id;
    let operator = operator.to_string();
    db.transaction::<_, _, Error>(|txn| {
        let req = req.clone();
        let operator = operator.clone();
        Box::pin(async move {
            if let Some(steps) = &req.steps {
                merge_steps(txn, steps, &operator).await?;
            }
            OnboardingConfigModel::update_config(txn, row_id, &config_req, &operator, new_version)
                .await?;
            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(SaveOutcome::Ok(new_version as i64))
}

/// 全量覆盖步骤：提交列表内的更新/插入，列表外的自定义步骤软删，内置步骤保留不动
async fn merge_steps<C: ConnectionTrait>(
    txn: &C,
    steps: &[OnboardingStepSaveDTO],
    operator: &str,
) -> Result<()> {
    let existing = OnboardingStepModel::find_all(txn, false).await?;
    let mut submitted_codes: HashSet<String> = HashSet::new();
    for s in steps {
        let code = s.step_code.clone().unwrap_or_default().trim().to_string();
        submitted_codes.insert(code.clone());
        match existing.iter().find(|e| e.step_code == code) {
            Some(e) => {
                OnboardingStepModel::update_by_id(txn, e.id, s)
                    .await
                    .map_err(|e| Error::from(format!("更新步骤失败: {}", e)))?;
            }
            None => {
                if s.id.is_some() {
                    return Err(Error::from(format!("步骤不存在：{}", code)));
                }
                OnboardingStepModel::insert(txn, s, operator)
                    .await
                    .map_err(|e| Error::from(format!("新增步骤失败: {}", e)))?;
            }
        }
    }
    // 内置步骤不在提交列表时保留不动（防误删）；自定义步骤软删
    for e in &existing {
        if e.step_type == 2 && !submitted_codes.contains(&e.step_code) {
            OnboardingStepModel::soft_delete_by_id(txn, e.id)
                .await
                .map_err(|err| Error::from(format!("删除步骤失败: {}", err)))?;
        }
    }
    Ok(())
}

/// 当前节点名称（current_node_key → flow node node_name）
async fn current_node_name_async(
    db: &DbConn,
    inst: &approval_instance::Model,
) -> Option<String> {
    let flow_id = inst.flow_id?;
    let node_key = inst.current_node_key.clone()?;
    approval_flow_node::Entity::find()
        .filter(approval_flow_node::Column::FlowId.eq(flow_id))
        .filter(approval_flow_node::Column::NodeKey.eq(node_key))
        .one(db)
        .await
        .ok()
        .flatten()
        .and_then(|n| n.node_name)
}

/// 最近一次驳回理由（最新 action=2 日志的 comment）
async fn latest_reject_reason_async(db: &DbConn, instance_id: i64) -> Option<String> {
    let log = approval_log::Entity::find()
        .filter(approval_log::Column::InstanceId.eq(instance_id))
        .filter(approval_log::Column::Action.eq(2))
        .order_by_desc(approval_log::Column::CreateTime)
        .one(db)
        .await
        .ok()
        .flatten()?;
    log.comment
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
}

/// 跳转链接校验（13.4-1）：必须以 / 开头、不以 // 开头、仅白名单字符（拒绝 javascript:、外部域名、控制字符）
pub fn is_valid_link(url: &str) -> bool {
    url.starts_with('/')
        && !url.starts_with("//")
        && url
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "/_?=&.-".contains(c))
}

fn validate_link(url: Option<&str>) -> Result<()> {
    let u = match url {
        Some(u) => u.trim(),
        None => return Ok(()),
    };
    if u.is_empty() {
        return Ok(());
    }
    if !is_valid_link(u) {
        return Err(Error::from(format!("跳转链接不合法：{}（仅允许站内路径，如 /profile）", u)));
    }
    Ok(())
}

/// 快捷入口预设校验：数组元素须含非空 code/label 与合法 path
fn validate_quick_preset(v: &serde_json::Value) -> Result<()> {
    let arr = v
        .as_array()
        .ok_or_else(|| Error::from("快捷入口预设必须为数组"))?;
    for item in arr {
        let code = item.get("code").and_then(|c| c.as_str()).unwrap_or("");
        let label = item.get("label").and_then(|l| l.as_str()).unwrap_or("");
        let path = item.get("path").and_then(|p| p.as_str()).unwrap_or("");
        if code.trim().is_empty() || label.trim().is_empty() {
            return Err(Error::from("快捷入口 code 与 label 不能为空"));
        }
        if !is_valid_link(path) {
            return Err(Error::from(format!("快捷入口跳转路径不合法：{}", path)));
        }
    }
    Ok(())
}
