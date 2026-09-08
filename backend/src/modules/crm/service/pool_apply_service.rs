//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use std::collections::HashMap;

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::entity::{lead, lead_pool, lead_pool_apply, lead_pool_member};
use crate::modules::crm::model::lead_pool::{LeadPoolApplyPageQuery, LeadPoolApplyVO, PoolOpResultVO};
use crate::modules::crm::service::{lead_pool_ops_service, pool_config_service, pool_service};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set};

/// 申请状态：1=待审批 2=通过 3=拒绝
const STATUS_PENDING: i16 = 1;
const STATUS_APPROVED: i16 = 2;
const STATUS_REJECTED: i16 = 3;

fn map_db_err(e: DbErr) -> Error {
    Error::from(e.to_string())
}

fn err<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::from(msg.into()))
}

fn now() -> chrono::NaiveDateTime {
    chrono::Local::now().naive_local()
}

/// 查询公海池管理员ID列表（member_type=1）
async fn pool_admin_ids(db: &DbConn, pool_id: i64) -> Result<Vec<i64>> {
    let rows = lead_pool_member::Entity::find()
        .filter(lead_pool_member::Column::PoolId.eq(pool_id))
        .filter(lead_pool_member::Column::MemberType.eq(1i16))
        .all(db)
        .await
        .map_err(map_db_err)?;
    Ok(rows.iter().filter_map(|m| m.user_id).collect())
}

/// 批量查线索名称（company_name 优先 → contact_name → 线索#id）
async fn load_lead_title_map(db: &DbConn, lead_ids: &[i64]) -> Result<HashMap<i64, String>> {
    if lead_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = lead::Entity::find()
        .filter(lead::Column::Id.is_in(lead_ids.to_vec()))
        .all(db)
        .await
        .map_err(map_db_err)?;
    Ok(rows.into_iter().map(|l| (l.id, lead_pool_ops_service::lead_title(&l))).collect())
}

/// 批量查公海池名称
async fn load_pool_name_map(db: &DbConn, pool_ids: &[i64]) -> Result<HashMap<i64, String>> {
    if pool_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = lead_pool::Entity::find()
        .filter(lead_pool::Column::Id.is_in(pool_ids.to_vec()))
        .all(db)
        .await
        .map_err(map_db_err)?;
    Ok(rows.into_iter().map(|p| (p.id, p.name.unwrap_or_else(|| format!("公海池#{}", p.id)))).collect())
}

/// 单条申领校验与落库：线索在池且无人占用 → 所在池 claim_mode=2 → 无待审重复申请 → 插入申请
///
/// 返回 (池ID, 线索名称) 供通知分组使用。
async fn apply_one(db: &DbConn, operator_id: i64, lead_id: i64, reason: &str) -> Result<(i64, String)> {
    let lead_model = lead::Entity::find_by_id(lead_id)
        .filter(lead::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("线索不存在".to_string()))?;
    if lead_model.converted_to_customer_id.is_some() {
        return err("该线索已转客户，无法申领");
    }
    if lead_model.assigned_to.is_some() {
        return err("该线索已被他人领取，无需申领");
    }
    let pool_id = lead_model.pool_id.ok_or_else(|| Error::from("该线索不在公海池中，无法申领".to_string()))?;

    let pool = pool_service::find_by_id(db, pool_id).await?
        .ok_or_else(|| Error::from("公海池不存在".to_string()))?;
    if pool.status != Some(1) {
        return err("该线索所在公海池已停用，无法申领");
    }
    let config = pool_config_service::get_raw(db, pool_id).await?
        .ok_or_else(|| Error::from("公海池配置缺失，请联系管理员".to_string()))?;
    if config.claim_mode.unwrap_or(1) != 2 {
        return err("该池无需申领，可直接领取");
    }

    let pending = lead_pool_apply::Entity::find()
        .filter(lead_pool_apply::Column::LeadId.eq(lead_id))
        .filter(lead_pool_apply::Column::Status.eq(STATUS_PENDING))
        .count(db)
        .await
        .map_err(map_db_err)?;
    if pending > 0 {
        return err("该线索已有待审批的申领申请，请耐心等待");
    }

    let active = lead_pool_apply::ActiveModel {
        pool_id: Set(Some(pool_id)),
        lead_id: Set(Some(lead_id)),
        user_id: Set(Some(operator_id)),
        status: Set(Some(STATUS_PENDING)),
        reason: Set(Some(reason.to_string())),
        create_time: Set(Some(now())),
        ..Default::default()
    };
    lead_pool_apply::Entity::insert(active).exec(db).await.map_err(map_db_err)?;
    Ok((pool_id, lead_pool_ops_service::lead_title(&lead_model)))
}

/// 批量提交申领（claim_mode=2 的池，§5.4 申领校验链）
///
/// 逐条校验并插入申请（部分成功部分失败，逐条返回明细）；
/// 提交后按池分组向池管理员发送待审批通知（§5.8 #2）。
pub async fn apply(db: &DbConn, operator_id: i64, lead_ids: &[i64], reason: Option<String>) -> Result<Vec<PoolOpResultVO>> {
    let reason_text = reason.map(|r| r.trim().to_string()).unwrap_or_default();
    if reason_text.is_empty() {
        return err("请填写申领理由");
    }
    if lead_ids.is_empty() {
        return err("请选择要申领的线索");
    }

    let mut results: Vec<PoolOpResultVO> = Vec::with_capacity(lead_ids.len());
    let mut applied_by_pool: Vec<(i64, Vec<String>)> = Vec::new();

    for lead_id in lead_ids {
        match apply_one(db, operator_id, *lead_id, &reason_text).await {
            Ok((pool_id, title)) => {
                results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: true, message: Some("申领申请已提交".to_string()) });
                match applied_by_pool.iter_mut().find(|(pid, _)| *pid == pool_id) {
                    Some((_, titles)) => titles.push(title),
                    None => applied_by_pool.push((pool_id, vec![title])),
                }
            },
            Err(e) => {
                results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some(e.to_string()) });
            },
        }
    }

    if !applied_by_pool.is_empty() {
        let name_map = crate::modules::system::service::admin_service::build_admin_name_map(db, vec![operator_id]).await;
        let applicant_name = name_map.get(&operator_id).cloned().unwrap_or_else(|| format!("用户{}", operator_id));
        for (pool_id, titles) in &applied_by_pool {
            let admin_ids = pool_admin_ids(db, *pool_id).await.unwrap_or_default();
            let preview: Vec<String> = titles.iter().take(5).cloned().collect();
            let mut content = format!(
                "<p><strong>{}</strong> 申请领取您管理的公海池中的 <strong>{}</strong> 条线索：</p><ul>{}</ul><p>申领理由：{}</p><p>请及时前往【公海申领审批】处理。</p>",
                applicant_name,
                titles.len(),
                preview.iter().map(|t| format!("<li>{}</li>", t)).collect::<String>(),
                reason_text
            );
            if titles.len() > 5 {
                content.push_str(&format!("<p>…等共 {} 条线索</p>", titles.len()));
            }
            for admin_id in admin_ids {
                if admin_id != operator_id {
                    let _ = lead_pool_ops_service::send_pool_notice(db, admin_id, "新的线索申领待审批", &content, operator_id).await;
                }
            }
        }
    }

    Ok(results)
}

/// 申领待办/已办分页（按池/状态/申请人过滤，创建时间倒序）
pub async fn page(db: &DbConn, query: &LeadPoolApplyPageQuery) -> Result<ResultPage<Vec<LeadPoolApplyVO>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    let mut select = lead_pool_apply::Entity::find();
    if let Some(pool_id) = query.pool_id {
        select = select.filter(lead_pool_apply::Column::PoolId.eq(pool_id));
    }
    if let Some(status) = query.status {
        select = select.filter(lead_pool_apply::Column::Status.eq(status));
    }
    if let Some(user_id) = query.user_id {
        select = select.filter(lead_pool_apply::Column::UserId.eq(user_id));
    }

    let paginator = select
        .order_by_desc(lead_pool_apply::Column::CreateTime)
        .order_by_desc(lead_pool_apply::Column::Id)
        .paginate(db, page_size as u64);
    let total = paginator.num_items().await? as i64;
    let rows = paginator.fetch_page((page - 1) as u64).await?;

    let lead_ids: Vec<i64> = rows.iter().filter_map(|r| r.lead_id).collect();
    let lead_map = load_lead_title_map(db, &lead_ids).await?;
    let pool_ids: Vec<i64> = rows.iter().filter_map(|r| r.pool_id).collect();
    let pool_name_map = load_pool_name_map(db, &pool_ids).await?;

    let mut user_ids: Vec<i64> = rows.iter().filter_map(|r| r.user_id).collect();
    user_ids.extend(rows.iter().filter_map(|r| r.audit_by));
    let name_map = crate::modules::system::service::admin_service::build_admin_name_map(db, user_ids).await;

    let items: Vec<LeadPoolApplyVO> = rows
        .into_iter()
        .map(|r| {
            let uid = r.user_id.unwrap_or_default();
            LeadPoolApplyVO {
                id: Some(r.id),
                pool_id: r.pool_id,
                pool_name: r.pool_id.and_then(|pid| pool_name_map.get(&pid).cloned()),
                lead_id: r.lead_id,
                lead_name: r.lead_id.and_then(|lid| lead_map.get(&lid).cloned()),
                user_id: r.user_id,
                user_name: name_map.get(&uid).cloned(),
                status: r.status,
                reason: r.reason,
                audit_by: r.audit_by,
                audit_by_name: r.audit_by.and_then(|aid| name_map.get(&aid).cloned()),
                audit_time: r.audit_time,
                audit_remark: r.audit_remark,
                create_time: r.create_time,
            }
        })
        .collect();

    Ok(ResultPage::new(items, total, page, page_size))
}

/// 申领审批（§5.10 防双审批：条件 UPDATE status 1→2/3）
///
/// 通过=二次校验线索仍在池且无人占用 → 复用领取链执行领取（历史记为"申领通过"）→ 通知申请人；
/// 领取失败时回退申请为待审并提示审批人重新处理。拒绝=留痕 + 通知申请人（§5.8 #3）。
pub async fn audit(db: &DbConn, auditor_id: i64, apply_id: i64, approved: bool, remark: Option<String>) -> Result<()> {
    let apply_model = lead_pool_apply::Entity::find_by_id(apply_id)
        .one(db)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("申领申请不存在".to_string()))?;
    if apply_model.status != Some(STATUS_PENDING) {
        return err("该申请已被处理，请勿重复审批");
    }
    let lead_id = apply_model.lead_id.ok_or_else(|| Error::from("申请数据异常：缺少线索ID".to_string()))?;
    let applicant_id = apply_model.user_id.unwrap_or_default();

    if approved {
        // 二次校验：线索仍在池且无人占用（§5.2）
        let lead_model = lead::Entity::find_by_id(lead_id)
            .filter(lead::Column::Deleted.eq(0))
            .one(db)
            .await
            .map_err(map_db_err)?
            .ok_or_else(|| Error::from("该线索已不存在，无法通过审批".to_string()))?;
        if lead_model.assigned_to.is_some() || lead_model.pool_id.is_none() {
            return err("该线索已不在公海或已被他人领取，无法通过审批");
        }
    }

    let ts = now();
    let new_status = if approved { STATUS_APPROVED } else { STATUS_REJECTED };
    let upd = lead_pool_apply::ActiveModel {
        status: Set(Some(new_status)),
        audit_by: Set(Some(auditor_id)),
        audit_time: Set(Some(ts)),
        audit_remark: Set(remark.clone()),
        ..Default::default()
    };
    let res = lead_pool_apply::Entity::update_many()
        .set(upd)
        .filter(lead_pool_apply::Column::Id.eq(apply_id))
        .filter(lead_pool_apply::Column::Status.eq(STATUS_PENDING))
        .exec(db)
        .await
        .map_err(map_db_err)?;
    if res.rows_affected == 0 {
        return err("该申请已被处理，请勿重复审批");
    }

    let title = lead::Entity::find_by_id(lead_id)
        .one(db)
        .await
        .map_err(map_db_err)?
        .map(|l| lead_pool_ops_service::lead_title(&l))
        .unwrap_or_else(|| format!("线索#{}", lead_id));

    if !approved {
        let mut content = format!("<p>您提交的线索【{}】申领未通过。</p>", title);
        if let Some(r) = remark.as_ref().filter(|s| !s.trim().is_empty()) {
            content.push_str(&format!("<p>审批备注：{}</p>", r));
        }
        let _ = lead_pool_ops_service::send_pool_notice(db, applicant_id, "申领审批结果通知", &content, auditor_id).await;
        return Ok(());
    }

    // 通过：复用领取链执行领取（bypass claim_mode 拦截，归属历史记为"申领通过"）
    if let Err(e) = lead_pool_ops_service::claim_for_apply_audit(db, lead_id, applicant_id).await {
        // 领取失败：回退申请为待审，交由审批人重新处理
        let rollback = lead_pool_apply::ActiveModel {
            status: Set(Some(STATUS_PENDING)),
            audit_by: Set(None),
            audit_time: Set(None),
            audit_remark: Set(None),
            ..Default::default()
        };
        let _ = lead_pool_apply::Entity::update_many()
            .set(rollback)
            .filter(lead_pool_apply::Column::Id.eq(apply_id))
            .filter(lead_pool_apply::Column::Status.eq(STATUS_APPROVED))
            .exec(db)
            .await;
        return err(format!("审批通过但领取失败，已退回待审：{}", e));
    }

    let content = format!("<p>您提交的线索【{}】申领已通过，线索已分配到您的私海，请及时跟进。</p>", title);
    let _ = lead_pool_ops_service::send_pool_notice(db, applicant_id, "申领审批通过通知", &content, auditor_id).await;
    Ok(())
}
