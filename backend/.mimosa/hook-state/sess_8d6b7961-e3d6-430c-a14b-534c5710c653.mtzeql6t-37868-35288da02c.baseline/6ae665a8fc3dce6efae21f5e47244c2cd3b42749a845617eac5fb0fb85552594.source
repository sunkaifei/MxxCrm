//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::core::errors::error::{Error, Result};
use crate::modules::crm::entity::{lead, lead_assign_history, lead_pool, lead_pool_config};
use crate::modules::crm::model::lead_pool::{LeadAssignHistoryVO, PoolOpResultVO};
use crate::modules::crm::service::{delete_guard_service, pool_config_service, pool_member_service, pool_service};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseTransaction, DbConn, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait};

/// 归属历史操作类型：1=领取 2=退回 3=管理员分配 4=转移 5=自动回收 6=管理员收回 7=申领通过
pub const ACTION_CLAIM: i16 = 1;
pub const ACTION_RELEASE: i16 = 2;
pub const ACTION_ASSIGN: i16 = 3;
pub const ACTION_TRANSFER: i16 = 4;
pub const ACTION_AUTO_RECYCLE: i16 = 5;
pub const ACTION_RECYCLE: i16 = 6;
pub const ACTION_APPLY_PASS: i16 = 7;

fn map_db_err(e: DbErr) -> Error {
    Error::from(e.to_string())
}

fn err<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::from(msg.into()))
}

fn now() -> chrono::NaiveDateTime {
    chrono::Local::now().naive_local()
}

pub fn lead_title(lead: &lead::Model) -> String {
    lead.company_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| lead.contact_name.clone().filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| format!("线索#{}", lead.id))
}

fn action_label(action_type: i16) -> &'static str {
    match action_type {
        ACTION_CLAIM => "领取线索",
        ACTION_RELEASE => "退回公海",
        ACTION_ASSIGN => "分配线索",
        ACTION_TRANSFER => "线索转移",
        ACTION_AUTO_RECYCLE => "超期自动回收",
        ACTION_RECYCLE => "管理员收回",
        ACTION_APPLY_PASS => "申领通过",
        _ => "入池",
    }
}

/// 统一入池入口：退回/管理员收回/自动回收共用
///
/// 单条线索一个事务：条件 UPDATE 抢占（写 pool_id/source_pool_id/status=8、清负责人与提醒标记、
/// 重置延期顺延）→ 关闭原负责人在负归属记录（冷却期锚点=end_time）→ 无在负记录时补插关闭流水。
/// 事务提交后向原负责人发送通知（§5.8 #5/#6）。
///
/// * `expected_owner` - Some=要求当前负责人匹配（退回/收回/自动回收）；None=要求当前无负责人
pub async fn enter_pool(
    db: &DbConn,
    lead_ids: &[i64],
    pool_id: i64,
    action_type: i16,
    operator_id: i64,
    expected_owner: Option<i64>,
    reason_type: Option<i16>,
    reason: &Option<String>,
) -> Result<Vec<PoolOpResultVO>> {
    let pool = pool_service::find_by_id(db, pool_id).await?
        .ok_or_else(|| Error::from("目标公海池不存在".to_string()))?;
    let pool_name = pool.name.clone().unwrap_or_else(|| format!("公海池#{}", pool_id));

    let mut results: Vec<PoolOpResultVO> = Vec::with_capacity(lead_ids.len());
    let mut notified: Vec<(i64, String)> = Vec::new();

    for lead_id in lead_ids {
        match enter_one(db, *lead_id, pool_id, action_type, operator_id, expected_owner, reason_type, reason).await {
            Ok(Some((old_owner, title))) => {
                results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: true, message: Some("操作成功".to_string()) });
                notified.push((old_owner, title));
            },
            Ok(None) => {
                results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: true, message: Some("操作成功".to_string()) });
            },
            Err(e) => {
                results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some(e.to_string()) });
            },
        }
    }

    for (owner, title) in &notified {
        let (nt_title, content) = match action_type {
            ACTION_RELEASE => {
                let reason_text = reason.clone().unwrap_or_default();
                let rt = match reason_type {
                    Some(1) => "无意向",
                    Some(2) => "无法联系",
                    Some(3) => "重复线索",
                    Some(4) => "区域不符",
                    _ => "其他",
                };
                (
                    "线索退回公海通知".to_string(),
                    format!(
                        "<p>您的线索<strong>【{}】</strong>已退回公海<strong>【{}】</strong>。</p><p>退回原因：{}{}</p><p>冷却期内您将无法再次领取该线索，请知悉。</p>",
                        title,
                        pool_name,
                        rt,
                        if reason_text.is_empty() { String::new() } else { format!("（{}）", reason_text) }
                    ),
                )
            },
            ACTION_AUTO_RECYCLE => (
                "线索自动回收通知".to_string(),
                format!("<p>您的线索<strong>【{}】</strong>因超期未跟进，已被自动回收至公海<strong>【{}】</strong>。</p><p>如需继续跟进，请前往公海重新领取或联系管理员分配。</p>", title, pool_name),
            ),
            _ => (
                "线索收回通知".to_string(),
                format!("<p>管理员已将您的线索<strong>【{}】</strong>收回至公海<strong>【{}】</strong>。</p><p>如对收回有异议，请联系管理员。</p>", title, pool_name),
            ),
        };
        let _ = send_pool_notice(db, *owner, &nt_title, &content, operator_id).await;
    }

    // §5.5：所有入池动作统一触发自动分配判断（池开启自动分配时，提交后立即分配）
    let entered_ids: Vec<i64> = results.iter()
        .filter(|r| r.success)
        .filter_map(|r| r.lead_id)
        .collect();
    if !entered_ids.is_empty() {
        if let Err(e) = crate::modules::crm::service::lead_auto_assign_service::trigger_on_enter(db, pool_id, &entered_ids).await {
            log::warn!("入池自动分配触发失败: {}", e);
        }
    }

    Ok(results)
}

/// 单条入池（事务内执行），成功返回 Some((原负责人, 线索标题))；无原负责人返回 None
async fn enter_one(
    db: &DbConn,
    lead_id: i64,
    pool_id: i64,
    action_type: i16,
    operator_id: i64,
    expected_owner: Option<i64>,
    reason_type: Option<i16>,
    reason: &Option<String>,
) -> Result<Option<(i64, String)>> {
    let txn: DatabaseTransaction = db.begin().await.map_err(map_db_err)?;

    let lead_model = lead::Entity::find_by_id(lead_id)
        .filter(lead::Column::Deleted.eq(0))
        .one(&txn)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("线索不存在".to_string()))?;

    if lead_model.converted_to_customer_id.is_some() {
        txn.rollback().await.ok();
        return err("该线索已转客户，不能入池");
    }

    let old_owner = lead_model.assigned_to;
    if old_owner.is_some() && expected_owner.is_none() {
        txn.rollback().await.ok();
        return err("线索仍归属他人，需先确认负责人");
    }

    let ts = now();
    let write_reason = action_type == ACTION_RELEASE;
    let payload = lead::ActiveModel {
        pool_id: Set(Some(pool_id)),
        source_pool_id: Set(Some(pool_id)),
        status: Set(Some(8)),
        assigned_to: Set(None),
        assigned_at: Set(None),
        reminded_at: Set(None),
        recycle_extend_days: Set(Some(0)),
        release_reason_type: Set(if write_reason { reason_type } else { None }),
        release_reason: Set(if write_reason { reason.clone() } else { None }),
        updated_by: Set(Some(operator_id)),
        update_time: Set(Some(ts.clone())),
        ..Default::default()
    };

    let mut query = lead::Entity::update_many()
        .set(payload)
        .filter(lead::Column::Id.eq(lead_id))
        .filter(lead::Column::Deleted.eq(0));
    query = match expected_owner {
        Some(owner) => query.filter(lead::Column::AssignedTo.eq(owner)),
        None => query.filter(lead::Column::AssignedTo.is_null()),
    };
    let res = query.exec(&txn).await.map_err(map_db_err)?;
    if res.rows_affected == 0 {
        txn.rollback().await.ok();
        return err("线索状态已变化，请刷新后重试");
    }

    match old_owner {
        Some(owner) => {
            close_open_history(&txn, lead_id, owner, action_type, reason_type, reason, operator_id).await?;
        },
        None => {
            insert_history(
                &txn,
                lead_id,
                operator_id,
                action_type,
                Some(ts.clone()),
                Some(ts.clone()),
                Some(pool_id),
                action_label(action_type).to_string(),
                reason_type,
                reason.clone(),
                operator_id,
            ).await?;
        },
    }

    txn.commit().await.map_err(map_db_err)?;

    Ok(old_owner.map(|owner| (owner, lead_title(&lead_model))))
}

/// 关闭指定负责人在负归属记录（冷却期以 end_time 为锚点）
/// 供同模块转移等操作复用（lead_transfer_service）。
#[allow(clippy::too_many_arguments)]
pub(crate) async fn close_open_history(
    txn: &DatabaseTransaction,
    lead_id: i64,
    owner_id: i64,
    action_type: i16,
    reason_type: Option<i16>,
    reason: &Option<String>,
    operator_id: i64,
) -> Result<()> {
    let current = lead_assign_history::Entity::find()
        .filter(lead_assign_history::Column::LeadId.eq(lead_id))
        .filter(lead_assign_history::Column::AdminId.eq(owner_id))
        .filter(lead_assign_history::Column::EndTime.is_null())
        .one(txn)
        .await
        .map_err(map_db_err)?;

    let ts = now();
    let reason_text = reason.clone().unwrap_or_default();
    match current {
        Some(record) => {
            let mut active: lead_assign_history::ActiveModel = record.into();
            active.end_time = Set(Some(ts));
            active.reason_type = Set(reason_type);
            active.reason = Set(reason.clone());
            active.remark = Set(Some(action_label(action_type).to_string()));
            active.operated_by = Set(Some(operator_id));
            active.update(txn).await.map_err(map_db_err)?;
        },
        None => {
            insert_history(
                txn,
                lead_id,
                owner_id,
                action_type,
                Some(ts.clone()),
                Some(ts),
                None,
                action_label(action_type).to_string(),
                reason_type,
                Some(reason_text),
                operator_id,
            ).await?;
        },
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn insert_history(
    txn: &impl ConnectionTrait,
    lead_id: i64,
    admin_id: i64,
    action_type: i16,
    start_time: Option<chrono::NaiveDateTime>,
    end_time: Option<chrono::NaiveDateTime>,
    pool_id: Option<i64>,
    remark: String,
    reason_type: Option<i16>,
    reason: Option<String>,
    operated_by: i64,
) -> Result<()> {
    let ts = now();
    let payload = lead_assign_history::ActiveModel {
        lead_id: Set(Some(lead_id)),
        admin_id: Set(Some(admin_id)),
        action_type: Set(Some(action_type)),
        start_time: Set(start_time.or(Some(ts.clone()))),
        end_time: Set(end_time),
        remark: Set(Some(remark)),
        reason_type: Set(reason_type),
        reason: Set(reason),
        pool_id: Set(pool_id),
        operated_by: Set(Some(operated_by)),
        create_time: Set(Some(ts)),
        ..Default::default()
    };
    lead_assign_history::Entity::insert(payload)
        .exec(txn)
        .await
        .map_err(map_db_err)?;
    Ok(())
}

/// 批量领取（语义=成为负责人，不再建客户）
///
/// 校验链（§5.4 领取）：池启用 → 我是池成员 → claim_mode=1 → 日限额 → 保有量 → 冷却期 → 条件 UPDATE 抢占。
/// 限额/保有量为事务外预检（§5.10 允许轻微超卖），抢占与归属历史在单条事务内保证一致；
/// 批量部分成功部分失败，逐条返回明细。
pub async fn claim(db: &DbConn, operator_id: i64, lead_ids: &[i64]) -> Result<Vec<PoolOpResultVO>> {
    let mut results: Vec<PoolOpResultVO> = Vec::with_capacity(lead_ids.len());
    let mut claimed_titles: Vec<String> = Vec::new();

    for lead_id in lead_ids {
        match claim_one(db, *lead_id, operator_id, false, ACTION_CLAIM).await {
            Ok(title) => {
                results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: true, message: Some("领取成功".to_string()) });
                claimed_titles.push(title);
            },
            Err(e) => {
                results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some(e.to_string()) });
            },
        }
    }

    if !claimed_titles.is_empty() {
        let preview: Vec<String> = claimed_titles.iter().take(5).cloned().collect();
        let mut content = format!(
            "<p>您已成功领取 <strong>{}</strong> 条公海线索，负责人责任已生效：</p><ul>{}</ul><p>请及时跟进，避免超期被自动回收。</p>",
            claimed_titles.len(),
            preview.iter().map(|t| format!("<li>{}</li>", t)).collect::<String>()
        );
        if claimed_titles.len() > 5 {
            content.push_str(&format!("<p>…等共 {} 条线索</p>", claimed_titles.len()));
        }
        let _ = send_pool_notice(db, operator_id, "线索领取成功通知", &content, operator_id).await;
    }

    Ok(results)
}

/// 单条领取：校验链 + 事务内抢占 + 归属历史
///
/// `bypass_claim_mode=true` 时跳过领取模式拦截（claim_mode=2/3），仅供申领审批通过后执行领取复用。
/// `history_action`：归属历史动作类型（1=领取 7=申领通过）。
async fn claim_one(db: &DbConn, lead_id: i64, user_id: i64, bypass_claim_mode: bool, history_action: i16) -> Result<String> {
    let lead_model = lead::Entity::find_by_id(lead_id)
        .filter(lead::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("线索不存在".to_string()))?;

    if lead_model.converted_to_customer_id.is_some() {
        return err("该线索已转客户，无法领取");
    }
    if lead_model.assigned_to.is_some() {
        return err("该线索已被他人领取");
    }

    let pool_id = lead_model.pool_id.unwrap_or(1);
    let pool = pool_service::find_by_id(db, pool_id).await?
        .ok_or_else(|| Error::from("公海池不存在".to_string()))?;
    if pool.status != Some(1) {
        return err("该线索所在公海池已停用，无法领取");
    }

    if !pool_member_service::is_member(db, pool_id, user_id).await? {
        return err("您不是该公海池的成员，无法领取");
    }

    let config = pool_config_service::get_raw(db, pool_id).await?
        .ok_or_else(|| Error::from("公海池配置缺失，请联系管理员".to_string()))?;
    match config.claim_mode.unwrap_or(1) {
        2 => return err("该池为申领模式，请提交申领申请"),
        3 => return err("该池已停用自主领取，请联系管理员分配"),
        _ => {},
    }

    let daily_limit = config.claim_daily_limit.unwrap_or(0);
    if daily_limit > 0 {
        let today_start = chrono::Local::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default();
        let claimed_today = lead_assign_history::Entity::find()
            .filter(lead_assign_history::Column::AdminId.eq(user_id))
            .filter(lead_assign_history::Column::ActionType.eq(ACTION_CLAIM))
            .filter(lead_assign_history::Column::PoolId.eq(pool_id))
            .filter(lead_assign_history::Column::StartTime.gte(today_start))
            .count(db)
            .await
            .map_err(map_db_err)?;
        if claimed_today >= daily_limit as u64 {
            return err(format!("已达今日领取上限（{} 条），请明日再试", daily_limit));
        }
    }

    let hold_limit = config.hold_limit.unwrap_or(0);
    if hold_limit > 0 {
        let holding = lead::Entity::find()
            .filter(lead::Column::AssignedTo.eq(user_id))
            .filter(lead::Column::Deleted.eq(0))
            .filter(lead::Column::ConvertedToCustomerId.is_null())
            .filter(lead::Column::Status.ne(4i32))
            .filter(lead::Column::Status.ne(5i32))
            .count(db)
            .await
            .map_err(map_db_err)?;
        if holding >= hold_limit as u64 {
            return err(format!("已达私海线索保有量上限（{} 条），请先跟进或退回部分线索", hold_limit));
        }
    }

    let cool_down_days = config.cool_down_days.unwrap_or(0);
    if cool_down_days > 0 {
        if let Some(last) = lead_assign_history::Entity::find()
            .filter(lead_assign_history::Column::LeadId.eq(lead_id))
            .filter(lead_assign_history::Column::AdminId.eq(user_id))
            .filter(lead_assign_history::Column::EndTime.is_not_null())
            .order_by_desc(lead_assign_history::Column::EndTime)
            .one(db)
            .await
            .map_err(map_db_err)?
        {
            if let Some(end_time) = last.end_time {
                let deadline = end_time + chrono::Duration::days(cool_down_days as i64);
                let ts = now();
                if deadline > ts {
                    let remain = (deadline - ts).num_days() + 1;
                    return err(format!("该线索处于冷却期，约 {} 天后可再次领取", remain));
                }
            }
        }
    }

    let title = lead_title(&lead_model);
    let ts = now();

    let txn: DatabaseTransaction = db.begin().await.map_err(map_db_err)?;
    let fresh = lead::Entity::find_by_id(lead_id)
        .filter(lead::Column::Deleted.eq(0))
        .one(&txn)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("线索不存在".to_string()))?;
    if fresh.assigned_to.is_some() {
        txn.rollback().await.ok();
        return err("该线索已被他人领取");
    }

    let payload = lead::ActiveModel {
        assigned_to: Set(Some(user_id)),
        assigned_at: Set(Some(ts.clone())),
        status: Set(Some(1)),
        // §5.4 提醒重置：领取=新的跟进周期，清空到期提醒幂等标记（与 assign_one 对齐）
        reminded_at: Set(None),
        updated_by: Set(Some(user_id)),
        update_time: Set(Some(ts.clone())),
        ..Default::default()
    };
    let res = lead::Entity::update_many()
        .set(payload)
        .filter(lead::Column::Id.eq(lead_id))
        .filter(lead::Column::Deleted.eq(0))
        .filter(lead::Column::AssignedTo.is_null())
        .exec(&txn)
        .await
        .map_err(map_db_err)?;
    if res.rows_affected == 0 {
        txn.rollback().await.ok();
        return err("该线索已被他人领取");
    }

    insert_history(
        &txn,
        lead_id,
        user_id,
        history_action,
        Some(ts.clone()),
        None,
        Some(pool_id),
        if history_action == ACTION_APPLY_PASS { "申领通过".to_string() } else { "领取线索".to_string() },
        None,
        None,
        user_id,
    ).await?;

    txn.commit().await.map_err(map_db_err)?;
    Ok(title)
}

/// 申领审批通过后执行领取（复用领取校验链，跳过 claim_mode 拦截，历史记为"申领通过"）
pub async fn claim_for_apply_audit(db: &DbConn, lead_id: i64, user_id: i64) -> Result<String> {
    claim_one(db, lead_id, user_id, true, ACTION_APPLY_PASS).await
}

/// 发送公海站内信（复用线索转移通知模式：创建+发布）
pub async fn send_pool_notice(db: &DbConn, to_user_id: i64, title: &str, content: &str, operator_id: i64) -> Result<()> {
    use crate::modules::system::model::notice::{NoticeModel, NoticeSaveDTO};

    let ts = chrono::Local::now().naive_local();
    let save_dto = NoticeSaveDTO {
        id: None,
        title: Some(title.to_string()),
        content: Some(content.to_string()),
        r#type: Some(4),
        level: Some("high".to_string()),
        target_type: Some(2),
        target_user_ids: Some(to_user_id.to_string()),
        publisher_id: Some(operator_id),
        publish_status: Some(0),
        publish_time: Some(ts.clone()),
        revoke_time: None,
        create_by: Some(operator_id),
        create_time: Some(ts.clone()),
        update_by: Some(operator_id),
        update_time: Some(ts),
    };

    let notice_id = NoticeModel::insert(db, &save_dto).await.map_err(map_db_err)?;
    let _ = crate::modules::system::service::notice_service::update_by_id_publish(
        db,
        &Some(notice_id),
        &Some(operator_id),
    )
    .await;
    Ok(())
}

/// 池配置快捷读取（供其他服务复用）
pub async fn load_pool_config(db: &DbConn, pool_id: i64) -> Result<lead_pool_config::Model> {
    pool_config_service::get_raw(db, pool_id).await?
        .ok_or_else(|| Error::from("公海池配置缺失，请联系管理员".to_string()))
}

/// 批量分配（管理员分配，§5.4 分配校验链）
///
/// 操作人是池管理员/超管 → 目标是池成员（超管放宽）→ 目标保有量校验（0=跳过）
/// → 条件 UPDATE 抢占（WHERE assigned_to IS NULL）→ 历史(action=3) → 通知新负责人（§5.8 #4）。
/// 单人多条=逐条；多人=按顺序均摊（第 i 条给 to_user_ids[i % len]）。
pub async fn assign(
    db: &DbConn,
    operator_id: i64,
    operator_name: &str,
    lead_ids: &[i64],
    to_user_ids: &[i64],
) -> Result<Vec<PoolOpResultVO>> {
    if to_user_ids.is_empty() {
        return err("请选择分配目标");
    }

    let mut results: Vec<PoolOpResultVO> = Vec::with_capacity(lead_ids.len());
    let mut assigned: Vec<(i64, Vec<String>)> = Vec::new();

    for (idx, lead_id) in lead_ids.iter().enumerate() {
        let to_user = to_user_ids[idx % to_user_ids.len()];
        match assign_one(db, *lead_id, to_user, operator_id, false).await {
            Ok(title) => {
                results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: true, message: Some("分配成功".to_string()) });
                match assigned.iter_mut().find(|(uid, _)| *uid == to_user) {
                    Some((_, titles)) => titles.push(title),
                    None => assigned.push((to_user, vec![title])),
                }
            },
            Err(e) => {
                results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some(e.to_string()) });
            },
        }
    }

    for (to_user, titles) in &assigned {
        let preview: Vec<String> = titles.iter().take(5).cloned().collect();
        let mut content = format!(
            "<p>主管<strong>{}</strong>分配了 <strong>{}</strong> 条线索给您：</p><ul>{}</ul><p>请及时跟进，避免超期被自动回收。</p>",
            operator_name,
            titles.len(),
            preview.iter().map(|t| format!("<li>{}</li>", t)).collect::<String>()
        );
        if titles.len() > 5 {
            content.push_str(&format!("<p>…等共 {} 条线索</p>", titles.len()));
        }
        let _ = send_pool_notice(db, *to_user, "线索分配通知", &content, operator_id).await;
    }

    Ok(results)
}

/// 单条分配：校验链 + 事务内抢占 + 历史留痕，成功返回线索标题
///
/// * `auto` - true=系统自动分配（§5.5）：跳过操作人权限校验，历史留痕标注"系统自动分配"
async fn assign_one(db: &DbConn, lead_id: i64, to_user_id: i64, operator_id: i64, auto: bool) -> Result<String> {
    let lead_model = lead::Entity::find_by_id(lead_id)
        .filter(lead::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("线索不存在".to_string()))?;

    if lead_model.converted_to_customer_id.is_some() {
        return err("该线索已转客户，无法分配");
    }
    if lead_model.assigned_to.is_some() {
        return err("该线索已有负责人，无法分配");
    }

    let pool_id = lead_model.pool_id.unwrap_or(1);
    if !auto {
        let is_super = delete_guard_service::is_super_admin(db, operator_id).await?;
        if !is_super && !pool_service::is_pool_manager(db, pool_id, operator_id).await? {
            return err("仅池管理员可分配线索");
        }

        if !is_super && !pool_member_service::is_member(db, pool_id, to_user_id).await? {
            return err("分配目标不是该公海池的成员");
        }
    }

    let config = load_pool_config(db, pool_id).await?;
    let hold_limit = config.hold_limit.unwrap_or(0);
    if hold_limit > 0 {
        let holding = lead::Entity::find()
            .filter(lead::Column::AssignedTo.eq(to_user_id))
            .filter(lead::Column::Deleted.eq(0))
            .filter(lead::Column::ConvertedToCustomerId.is_null())
            .filter(lead::Column::Status.ne(4i32))
            .filter(lead::Column::Status.ne(5i32))
            .count(db)
            .await
            .map_err(map_db_err)?;
        if holding >= hold_limit as u64 {
            return err(format!("目标成员已达私海线索保有量上限（{} 条）", hold_limit));
        }
    }

    let title = lead_title(&lead_model);
    let ts = now();

    let txn: DatabaseTransaction = db.begin().await.map_err(map_db_err)?;
    let payload = lead::ActiveModel {
        assigned_to: Set(Some(to_user_id)),
        assigned_at: Set(Some(ts.clone())),
        status: Set(Some(1)),
        reminded_at: Set(None),
        updated_by: Set(Some(operator_id)),
        update_time: Set(Some(ts.clone())),
        ..Default::default()
    };
    let res = lead::Entity::update_many()
        .set(payload)
        .filter(lead::Column::Id.eq(lead_id))
        .filter(lead::Column::Deleted.eq(0))
        .filter(lead::Column::AssignedTo.is_null())
        .exec(&txn)
        .await
        .map_err(map_db_err)?;
    if res.rows_affected == 0 {
        txn.rollback().await.ok();
        return err("线索状态已变化，请刷新后重试");
    }

    insert_history(
        &txn,
        lead_id,
        to_user_id,
        ACTION_ASSIGN,
        Some(ts.clone()),
        None,
        Some(pool_id),
        if auto { "系统自动分配".to_string() } else { "管理员分配".to_string() },
        None,
        None,
        operator_id,
    ).await?;

    txn.commit().await.map_err(map_db_err)?;
    Ok(title)
}

/// 单条自动分配（§5.5，系统触发）：跳过操作人权限校验，历史留痕标注"系统自动分配"
pub async fn assign_one_auto(db: &DbConn, lead_id: i64, to_user_id: i64) -> Result<String> {
    assign_one(db, lead_id, to_user_id, 0, true).await
}

/// 批量退回（销售本人退回或管理员代退，§5.4 退回校验链）
///
/// 是当前负责人（或池管理员/超管）→ 两级原因校验 → 目标池（target_pool_id 优先，否则 source_pool_id，否则当前池）
/// → enter_pool(action=2, expected_owner=当前负责人) 完成条件抢占/历史/通知。
pub async fn release(
    db: &DbConn,
    operator_id: i64,
    lead_ids: &[i64],
    reason_type: Option<i16>,
    reason: &Option<String>,
    target_pool_id: Option<i64>,
) -> Result<Vec<PoolOpResultVO>> {
    delete_guard_service::validate_release_reason(reason_type, reason)?;

    // 按目标池分组，组内批量走 enter_pool
    // 按（目标池, 当前负责人）分组：enter_pool 需要 expected_owner 做条件抢占
    let mut grouped: Vec<(i64, i64, Vec<i64>)> = Vec::new();
    let mut pre_errors: Vec<PoolOpResultVO> = Vec::with_capacity(lead_ids.len());

    for lead_id in lead_ids {
        let lead_model = match lead::Entity::find_by_id(*lead_id)
            .filter(lead::Column::Deleted.eq(0))
            .one(db)
            .await
            .map_err(map_db_err)?
        {
            Some(m) => m,
            None => {
                pre_errors.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some("线索不存在".to_string()) });
                continue;
            },
        };
        if lead_model.converted_to_customer_id.is_some() {
            pre_errors.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some("该线索已转客户，无法退回".to_string()) });
            continue;
        }
        if lead_model.assigned_to.is_none() {
            pre_errors.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some("该线索已在公海中".to_string()) });
            continue;
        }

        let owner = lead_model.assigned_to.unwrap_or(operator_id);
        let is_owner = lead_model.assigned_to == Some(operator_id);
        let pool_id = lead_model.pool_id.unwrap_or(1);
        let is_super = delete_guard_service::is_super_admin(db, operator_id).await?;
        if !is_owner && !is_super && !pool_service::is_pool_manager(db, pool_id, operator_id).await? {
            pre_errors.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some("仅当前负责人或池管理员可退回该线索".to_string()) });
            continue;
        }

        let target = target_pool_id
            .or(lead_model.source_pool_id)
            .unwrap_or(pool_id);
        match grouped.iter_mut().find(|(pid, oid, _)| *pid == target && *oid == owner) {
            Some((_, _, ids)) => ids.push(*lead_id),
            None => {
                grouped.push((target, owner, vec![*lead_id]));
            },
        }
    }

    let mut results = pre_errors;
    for (target, owner, ids) in &grouped {
        match enter_pool(db, ids, *target, ACTION_RELEASE, operator_id, Some(*owner), reason_type, reason).await {
            Ok(mut list) => results.append(&mut list),
            Err(e) => {
                for id in ids {
                    results.push(PoolOpResultVO { lead_id: Some(*id), success: false, message: Some(e.to_string()) });
                }
            },
        }
    }

    // 补齐未参与分组的线索（预检失败项已含），保证结果与入参同序同长
    for lead_id in lead_ids {
        if !results.iter().any(|r| r.lead_id == Some(*lead_id)) {
            results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some("未处理".to_string()) });
        }
    }
    Ok(results)
}

/// 管理员收回（私海→本池，§5.4 收回校验链）
///
/// 池管理员/超管 → 线索属私海（有负责人）→ enter_pool(action=6, expected_owner=原负责人)
/// 完成条件抢占/历史/通知（§5.8 #5 收回文案）。
pub async fn recycle(db: &DbConn, operator_id: i64, lead_ids: &[i64], remark: &Option<String>) -> Result<Vec<PoolOpResultVO>> {
    let mut grouped: Vec<(i64, i64, Vec<i64>)> = Vec::new();
    let mut results: Vec<PoolOpResultVO> = Vec::with_capacity(lead_ids.len());

    let is_super = delete_guard_service::is_super_admin(db, operator_id).await?;

    for lead_id in lead_ids {
        let lead_model = match lead::Entity::find_by_id(*lead_id)
            .filter(lead::Column::Deleted.eq(0))
            .one(db)
            .await
            .map_err(map_db_err)?
        {
            Some(m) => m,
            None => {
                results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some("线索不存在".to_string()) });
                continue;
            },
        };
        if lead_model.converted_to_customer_id.is_some() {
            results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some("该线索已转客户，无法收回".to_string()) });
            continue;
        }
        let old_owner = match lead_model.assigned_to {
            Some(o) => o,
            None => {
                results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some("该线索无负责人，无需收回".to_string()) });
                continue;
            },
        };

        let pool_id = lead_model.pool_id.unwrap_or(1);
        if !is_super && !pool_service::is_pool_manager(db, pool_id, operator_id).await? {
            results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some("仅池管理员可收回线索".to_string()) });
            continue;
        }

        let target = lead_model.source_pool_id.unwrap_or(pool_id);
        match grouped.iter_mut().find(|(pid, oid, _)| *pid == target && *oid == old_owner) {
            Some((_, _, ids)) => ids.push(*lead_id),
            None => grouped.push((target, old_owner, vec![*lead_id])),
        }
    }

    for (target, owner, ids) in &grouped {
        match enter_pool(db, ids, *target, ACTION_RECYCLE, operator_id, Some(*owner), None, remark).await {
            Ok(mut list) => results.append(&mut list),
            Err(e) => {
                for id in ids {
                    results.push(PoolOpResultVO { lead_id: Some(*id), success: false, message: Some(e.to_string()) });
                }
            },
        }
    }

    // 补齐未参与分组的线索，保证结果与入参同序同长
    for lead_id in lead_ids {
        if !results.iter().any(|r| r.lead_id == Some(*lead_id)) {
            results.push(PoolOpResultVO { lead_id: Some(*lead_id), success: false, message: Some("未处理".to_string()) });
        }
    }

    Ok(results)
}

/// 线索归属历史时间轴（§5.3：?lead_id=，按开始时间倒序）
pub async fn history_by_lead(db: &DbConn, lead_id: i64) -> Result<Vec<LeadAssignHistoryVO>> {
    let records = lead_assign_history::Entity::find()
        .filter(lead_assign_history::Column::LeadId.eq(lead_id))
        .order_by_desc(lead_assign_history::Column::StartTime)
        .all(db)
        .await
        .map_err(map_db_err)?;

    if records.is_empty() {
        return Ok(Vec::new());
    }

    // 批量联查用户名（admin_id + operated_by 合并一次 IN 查询）
    let all_ids: Vec<i64> = records.iter()
        .flat_map(|r| [r.admin_id, r.operated_by])
        .flatten()
        .collect();
    let name_map = crate::modules::system::service::admin_service::build_admin_name_map(db, all_ids).await;

    // 批量联查池名
    let pool_ids: Vec<i64> = records.iter().filter_map(|r| r.pool_id).collect();
    let mut pool_names: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    if !pool_ids.is_empty() {
        let pools = lead_pool::Entity::find()
            .filter(lead_pool::Column::Id.is_in(pool_ids))
            .all(db)
            .await
            .map_err(map_db_err)?;
        for p in pools {
            pool_names.insert(p.id, p.name.unwrap_or_else(|| format!("公海池#{}", p.id)));
        }
    }

    let data: Vec<LeadAssignHistoryVO> = records.into_iter().map(|item| {
        let pool_id = item.pool_id;
        let action_type = item.action_type;
        let mut vo = LeadAssignHistoryVO {
            id: Option::from(item.id),
            lead_id: item.lead_id,
            admin_id: item.admin_id,
            admin_name: None,
            action_type,
            action_label: Some(history_action_label(action_type.unwrap_or(0)).to_string()),
            start_time: item.start_time,
            end_time: item.end_time,
            pool_id,
            pool_name: pool_id.and_then(|pid| pool_names.get(&pid).cloned()),
            remark: item.remark,
            reason_type: item.reason_type,
            reason: item.reason,
            operated_by: item.operated_by,
            operated_by_name: None,
            create_time: item.create_time,
        };
        if let Some(aid) = vo.admin_id {
            vo.admin_name = name_map.get(&aid).cloned();
        }
        if let Some(oid) = vo.operated_by {
            vo.operated_by_name = name_map.get(&oid).cloned();
        }
        vo
    }).collect();

    Ok(data)
}

/// 归属历史操作类型文案
fn history_action_label(action_type: i16) -> &'static str {
    match action_type {
        1 => "领取",
        2 => "退回",
        3 => "管理员分配",
        4 => "转移",
        5 => "自动回收",
        6 => "管理员收回",
        7 => "申领通过",
        _ => "其他",
    }
}
