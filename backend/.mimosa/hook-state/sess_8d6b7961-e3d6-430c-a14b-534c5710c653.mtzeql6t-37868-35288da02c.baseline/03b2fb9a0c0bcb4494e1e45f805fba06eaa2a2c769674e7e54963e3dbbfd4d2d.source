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
use crate::modules::crm::entity::{contract, customer, customer_pool, customer_pool_config, opportunity};
use crate::modules::crm::service::customer_pool_service;
use crate::modules::crm::service::customer_pool_workbench_service::{
    close_open_history, customer_display_name, insert_history, notify_pool_admins, ACTION_AUTO_RECYCLE, ACTION_FROZEN,
};
use crate::modules::crm::service::lead_pool_ops_service::send_pool_notice;
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, DbConn, DbErr, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use std::collections::{HashMap, HashSet};

fn map_db_err(e: DbErr) -> Error {
    Error::from(e.to_string())
}

fn now() -> chrono::NaiveDateTime {
    chrono::Local::now().naive_local()
}

/// 有效合同状态集合默认值（草拟 1 / 履行中 2 / 已完成 3 / 已收款 4 / 终止 5 的对齐见 config 注释；
/// 默认取 [2,3,4]，即草拟与终止不算有效合同）
const DEFAULT_DEAL_STATUS: [i32; 3] = [2, 3, 4];

fn parse_deal_status(value: &Option<serde_json::Value>) -> Vec<i32> {
    match value {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|x| x.as_i64().map(|n| n as i32))
            .collect(),
        _ => DEFAULT_DEAL_STATUS.to_vec(),
    }
}

fn contract_status_num(s: &crate::core::r#enum::contract_status_enum::ContractStatus) -> i32 {
    match s {
        crate::core::r#enum::contract_status_enum::ContractStatus::Draft => 1,
        crate::core::r#enum::contract_status_enum::ContractStatus::Signed => 2,
        crate::core::r#enum::contract_status_enum::ContractStatus::Executing => 3,
        crate::core::r#enum::contract_status_enum::ContractStatus::Completed => 4,
        crate::core::r#enum::contract_status_enum::ContractStatus::Terminated => 5,
    }
}

/// 池名（缺失回退 公海池#id）
async fn pool_name_of(db: &DbConn, pool_id: i64) -> String {
    customer_pool::Entity::find_by_id(pool_id)
        .one(db)
        .await
        .ok()
        .flatten()
        .and_then(|p| p.name)
        .unwrap_or_else(|| format!("公海池#{}", pool_id))
}

/// 三条独立回收规则的到期线（§5.7 多规则）：「未跟进 recycle_days」「未新增商机 no_opportunity_days」
/// 「未签合同 no_contract_days」+ 绝对到期硬上限 max_recycle_days，任一到期即回收；
/// 返回 (到期线, 触发原因文案)。无规则或 assigned_at 缺失返回空向量。
fn rule_deadlines(
    cfg: &customer_pool_config::Model,
    assigned_at: Option<chrono::NaiveDateTime>,
    last_follow_up_at: Option<chrono::NaiveDateTime>,
    opp_last_at: Option<chrono::NaiveDateTime>,
    contract_last_at: Option<chrono::NaiveDateTime>,
    extend_days: i64,
) -> Vec<(chrono::NaiveDateTime, &'static str)> {
    let Some(assigned_at) = assigned_at else {
        return Vec::new();
    };

    let mut deadlines: Vec<(chrono::NaiveDateTime, &'static str)> = Vec::new();

    // 规则1：未跟进。基线=最后跟进与分配时间的较大者，保证新归属有完整跟进窗口
    if let Some(days) = cfg.recycle_days.filter(|d| *d > 0) {
        let anchor = match last_follow_up_at {
            Some(l) => l.max(assigned_at),
            None => assigned_at,
        };
        deadlines.push((
            anchor + chrono::Duration::days(days as i64 + extend_days),
            "超期未跟进",
        ));
    }
    // 规则2：未新增商机。锚点=最近一次非删除商机的创建时间，从未有商机则从分配日起算
    if let Some(days) = cfg.no_opportunity_days.filter(|d| *d > 0) {
        let anchor = opp_last_at.unwrap_or(assigned_at);
        deadlines.push((
            anchor + chrono::Duration::days(days as i64 + extend_days),
            "长期未新增商机",
        ));
    }
    // 规则3：未签合同。锚点=最近一次非删除合同的创建时间，从未有合同则从分配日起算
    if let Some(days) = cfg.no_contract_days.filter(|d| *d > 0) {
        let anchor = contract_last_at.unwrap_or(assigned_at);
        deadlines.push((
            anchor + chrono::Duration::days(days as i64 + extend_days),
            "长期未签约",
        ));
    }
    // 绝对到期硬上限：无论是否有跟进/商机/合同，在持不得超过该期限（取较早者）
    if let Some(cap) = cfg.max_recycle_days.filter(|d| *d > 0) {
        deadlines.push((
            assigned_at + chrono::Duration::days(cap as i64 + extend_days),
            "超过最长保护期",
        ));
    }
    deadlines
}

/// 回收引擎（每日 00:30，§5.6/§5.7）
///
/// 遍历启用任一回收规则的池：按池扫在负客户（含存量归默认池），逐条计算多规则到期线，
/// 命中到期者先做豁免三源检查（成单保护/商机保护命中则跳过并通知一次），
/// 事务内条件 UPDATE（WHERE assigned_to=<快照负责人> AND update_time=<快照>）防与人工操作竞态，
/// 成功即回写池字段、被动 release_count+1 并做冻结判定（写 action=9）、关闭在负归属流水(6)、
/// 通知原负责人与池管理员。整任务幂等，重复执行不产生二次回收。
pub async fn run_recycle(db: &DbConn) -> Result<String> {
    let configs = customer_pool_config::Entity::find()
        .all(db)
        .await
        .map_err(map_db_err)?;

    let mut total: usize = 0;
    let mut details: Vec<String> = Vec::new();
    for config in &configs {
        match recycle_pool(db, config).await {
            Ok(n) if n > 0 => {
                total += n;
                details.push(format!("池#{} 回收 {} 条", config.pool_id, n));
            },
            Ok(_) => {},
            Err(e) => log::warn!("客户公海池 {} 自动回收失败: {}", config.pool_id, e),
        }
    }

    if total > 0 {
        Ok(format!("客户自动回收完成，共回收 {} 条（{}）", total, details.join("；")))
    } else {
        Ok("客户自动回收完成，无超期客户".to_string())
    }
}

/// 单池自动回收：扫描该池全部在负客户并逐条回收
async fn recycle_pool(db: &DbConn, cfg: &customer_pool_config::Model) -> Result<usize> {
    let deal_status = parse_deal_status(&cfg.deal_status);
    let deal_protect = cfg.deal_protect_enabled == Some(1);
    let opp_protect = cfg.opportunity_protect_enabled == Some(1);
    let freeze_threshold = cfg.freeze_release_count.unwrap_or(0);

    let candidates = pool_candidates(db, cfg).await?;
    if candidates.is_empty() {
        return Ok(0);
    }
    let ids: Vec<i64> = candidates.iter().map(|c| c.id).collect();

    // 一次批量取商机/合同信息，避免 N+1（按规则开关按需加载）
    let (opp_last_map, active_opp) = if cfg.no_opportunity_days.is_some() || opp_protect {
        load_opportunity_context(db, &ids, opp_protect).await?
    } else {
        (HashMap::new(), HashSet::new())
    };
    let (contract_last_map, valid_contract) = if cfg.no_contract_days.is_some() || deal_protect {
        load_contract_context(db, &ids, &deal_status, deal_protect).await?
    } else {
        (HashMap::new(), HashSet::new())
    };

    let ts = now();
    let mut count: usize = 0;
    for row in &candidates {
        let Some(owner) = row.assigned_to else { continue };
        let opp_last = opp_last_map.get(&row.id).copied();
        let contract_last = contract_last_map.get(&row.id).copied();
        let deadlines = rule_deadlines(
            cfg,
            row.assigned_at,
            row.last_follow_up_at,
            opp_last,
            contract_last,
            row.recycle_extend_days.unwrap_or(0) as i64,
        );
        if deadlines.is_empty() {
            continue;
        }
        let overdue: Vec<&str> = deadlines
            .iter()
            .filter(|(d, _)| *d <= ts)
            .map(|(_, label)| *label)
            .collect();
        if overdue.is_empty() {
            continue;
        }

        // 豁免：成单保护（有有效合同）/ 商机保护（有进行中商机）
        let protected = (deal_protect && valid_contract.contains(&row.id))
            || (opp_protect && active_opp.contains(&row.id));
        if protected {
            let reason = if deal_protect && valid_contract.contains(&row.id) {
                "存在有效合同，受成单保护"
            } else {
                "存在进行中商机，受商机保护"
            };
            // 仅首次到期时通知（reminded_at 幂等），避免每日重复打扰
            if mark_reminded_once(db, row.id, owner).await? {
                let content = format!(
                    "<p>您的客户<strong>【{}】</strong>已到回收期，但因{}，本次不回收。</p><p>请持续跟进，保护条件消失后仍将按池规则执行回收。</p>",
                    customer_display_name(row),
                    reason
                );
                let _ = send_pool_notice(db, owner, "公海客户回收保护通知", &content, 0).await;
            }
            continue;
        }

        // 冻结判定：被动回收计数+1，达到阈值即冻结（决策点 #14：负责人主动退回不计）
        let old_count = row.release_count.unwrap_or(0);
        let new_count = old_count + 1;
        let frozen = freeze_threshold > 0 && new_count >= freeze_threshold;

        let eff_pool = cfg.pool_id;
        let txn: DatabaseTransaction = db.begin().await.map_err(map_db_err)?;
        let snapshot = row.update_time;
        let mut cond = Condition::all()
            .add(customer::Column::Id.eq(row.id))
            .add(customer::Column::Deleted.eq(0))
            .add(customer::Column::AssignedTo.eq(owner));
        cond = match snapshot {
            Some(s) => cond.add(customer::Column::UpdateTime.eq(s)),
            None => cond.add(customer::Column::UpdateTime.is_null()),
        };
        let payload = customer::ActiveModel {
            pool_id: Set(Some(eff_pool)),
            source_pool_id: Set(Some(eff_pool)),
            assigned_to: Set(None),
            assigned_at: Set(None),
            entered_pool_at: Set(Some(ts)),
            reminded_at: Set(None),
            recycle_extend_days: Set(Some(0)),
            from_pool: Set(Some(1)),
            release_count: Set(Some(new_count)),
            release_frozen: Set(if frozen { Some(1) } else { None }),
            updated_by: Set(Some(0)),
            update_time: Set(Some(ts)),
            ..Default::default()
        };
        let res = customer::Entity::update_many()
            .set(payload)
            .filter(cond)
            .exec(&txn)
            .await
            .map_err(map_db_err)?;
        if res.rows_affected == 0 {
            // 归属已变化（被领取/转移），本次不回收，交由下轮扫描
            txn.rollback().await.ok();
            continue;
        }

        close_open_history(&txn, row.id, owner, ACTION_AUTO_RECYCLE, None, &None, 0, eff_pool).await?;
        if frozen {
            insert_history(
                &txn,
                row.id,
                owner,
                ACTION_FROZEN,
                Some(ts),
                Some(ts),
                Some(eff_pool),
                "触发冻结".to_string(),
                None,
                Some(format!("连续 {} 次被动回收，已冻结", new_count)),
                0,
            )
            .await?;
        }
        txn.commit().await.map_err(map_db_err)?;

        count += 1;
        let pool_name = pool_name_of(db, eff_pool).await;
        let reason_text = overdue.join("、");
        let mut content = format!(
            "<p>您的客户<strong>【{}】</strong>因{}，已于今日自动回收至公海<strong>【{}】</strong>。</p>",
            customer_display_name(row),
            reason_text,
            pool_name
        );
        if frozen {
            content.push_str(&format!(
                "<p>因连续 {} 次被动回收，该客户已<strong>冻结</strong>，需联系管理员解冻后方可再次领取。</p>",
                new_count
            ));
        } else {
            content.push_str("<p>如仍需跟进，请在冷却期满后重新领取，并注意按期跟进避免再次回收。</p>");
        }
        let _ = send_pool_notice(db, owner, "公海客户回收通知", &content, 0).await;
        let _ = notify_pool_admins(
            db,
            eff_pool,
            "公海客户自动回收",
            &format!(
                "<p>客户<strong>【{}】</strong>因{}已被系统自动回收至本池（{}）。</p>",
                customer_display_name(row),
                reason_text,
                if frozen { format!("已冻结，回收次数 {}", new_count) } else { "未达冻结阈值".to_string() }
            ),
            0,
        )
        .await;
    }
    Ok(count)
}

/// 客户到期提醒（每日 09:00，§5.6/§5.7）
///
/// 扫描 `reminder_days > 0` 且启用任一回收规则的池，对「到期线已进入提醒窗口且 reminded_at IS NULL」
/// 的在负客户发送站内信（剩余天数 + 延期申请入口），发送成功后写 `reminded_at=now` 幂等标记。
pub async fn run_reminder(db: &DbConn) -> Result<String> {
    let configs = customer_pool_config::Entity::find()
        .all(db)
        .await
        .map_err(map_db_err)?;

    let mut total: usize = 0;
    let mut details: Vec<String> = Vec::new();
    for config in &configs {
        match remind_pool(db, config).await {
            Ok(n) if n > 0 => {
                total += n;
                details.push(format!("池#{} 提醒 {} 条", config.pool_id, n));
            },
            Ok(_) => {},
            Err(e) => log::warn!("客户公海池 {} 到期提醒失败: {}", config.pool_id, e),
        }
    }

    if total > 0 {
        Ok(format!("客户到期提醒完成，共提醒 {} 条（{}）", total, details.join("；")))
    } else {
        Ok("客户到期提醒完成，无待提醒客户".to_string())
    }
}

/// 单池到期提醒：扫描提醒窗口内未提醒客户，逐条发站内信并落幂等标记
async fn remind_pool(db: &DbConn, cfg: &customer_pool_config::Model) -> Result<usize> {
    let pool_id = cfg.pool_id;
    let reminder_days = cfg.reminder_days.unwrap_or(0);
    if reminder_days <= 0 {
        return Ok(0);
    }
    let deal_status = parse_deal_status(&cfg.deal_status);
    let deal_protect = cfg.deal_protect_enabled == Some(1);
    let opp_protect = cfg.opportunity_protect_enabled == Some(1);

    let candidates = pool_candidates(db, cfg).await?;
    if candidates.is_empty() {
        return Ok(0);
    }
    let ids: Vec<i64> = candidates.iter().map(|c| c.id).collect();

    // 按规则开关按需加载商机/合同上下文
    let (opp_last_map, active_opp) = if cfg.no_opportunity_days.is_some() || opp_protect {
        load_opportunity_context(db, &ids, opp_protect).await?
    } else {
        (HashMap::new(), HashSet::new())
    };
    let (contract_last_map, valid_contract) = if cfg.no_contract_days.is_some() || deal_protect {
        load_contract_context(db, &ids, &deal_status, deal_protect).await?
    } else {
        (HashMap::new(), HashSet::new())
    };

    let pool_name = pool_name_of(db, pool_id).await;
    let ts = now();
    let window_end = ts + chrono::Duration::days(reminder_days as i64);
    let mut count: usize = 0;

    for row in &candidates {
        let Some(owner) = row.assigned_to else { continue };
        if row.reminded_at.is_some() {
            continue;
        }
        let opp_last = opp_last_map.get(&row.id).copied();
        let contract_last = contract_last_map.get(&row.id).copied();
        let deadlines = rule_deadlines(
            cfg,
            row.assigned_at,
            row.last_follow_up_at,
            opp_last,
            contract_last,
            row.recycle_extend_days.unwrap_or(0) as i64,
        );
        let Some((deadline, _)) = deadlines.iter().min_by_key(|(d, _)| *d) else {
            continue;
        };
        if *deadline <= ts || *deadline > window_end {
            continue;
        }
        // 当前受保护客户不会在到期时被回收，跳过提醒避免误导
        if (deal_protect && valid_contract.contains(&row.id)) || (opp_protect && active_opp.contains(&row.id)) {
            continue;
        }
        if !mark_reminded_once(db, row.id, owner).await? {
            continue;
        }
        let remain = (((*deadline - ts).num_seconds()) + 86399) / 86400;
        let content = format!(
            "<p>您的客户<strong>【{}】</strong>将于 <strong>{}</strong>（剩 {} 天）到期，届时将自动回收至公海<strong>【{}】</strong>。</p><p>请及时跟进、推进商机或合同，如需继续保留可在该客户详情中提交延期申请。</p>",
            customer_display_name(row),
            deadline.format("%Y-%m-%d"),
            remain.max(0),
            pool_name
        );
        if let Err(e) = send_pool_notice(db, owner, "公海客户到期提醒", &content, 0).await {
            log::warn!("客户 {} 到期提醒发送失败: {}", row.id, e);
            continue;
        }
        count += 1;
    }
    Ok(count)
}

/// 池内候选客户（在负 + 未冻结 + 归属该池），供回收与提醒共用
async fn pool_candidates(db: &DbConn, cfg: &customer_pool_config::Model) -> Result<Vec<customer::Model>> {
    let pool_id = cfg.pool_id;
    let mut scope = Condition::any()
        .add(customer::Column::PoolId.eq(pool_id))
        .add(customer::Column::SourcePoolId.eq(pool_id));
    if pool_id == customer_pool_service::DEFAULT_CUSTOMER_POOL_ID {
        scope = scope.add(
            Condition::all()
                .add(customer::Column::PoolId.is_null())
                .add(customer::Column::SourcePoolId.is_null()),
        );
    }
    let mut cond = Condition::all()
        .add(customer::Column::Deleted.eq(0))
        .add(customer::Column::AssignedTo.is_not_null())
        .add(scope)
        .add(
            Condition::any()
                .add(customer::Column::ReleaseFrozen.eq(0))
                .add(customer::Column::ReleaseFrozen.is_null()),
        );
    // 自建客户按池规则回收开关（0=仅统计不强制；默认保护自建客户）
    if cfg.self_built_recycle_enabled != Some(1) {
        cond = cond.add(
            Condition::any()
                .add(customer::Column::FromPool.eq(1))
                .add(customer::Column::FromPool.is_null()),
        );
    }
    customer::Entity::find()
        .filter(cond)
        .all(db)
        .await
        .map_err(map_db_err)
}

/// 批量取商机上下文：最近创建时间（全部非删除商机，供「未新增商机」规则）；
/// 进行中商机集合（stage 非 5/6 终态，供商机保护豁免）
async fn load_opportunity_context(
    db: &DbConn,
    ids: &[i64],
    need_active: bool,
) -> Result<(HashMap<i64, chrono::NaiveDateTime>, HashSet<i64>)> {
    let mut last_map: HashMap<i64, chrono::NaiveDateTime> = HashMap::new();
    let mut active: HashSet<i64> = HashSet::new();
    if ids.is_empty() {
        return Ok((last_map, active));
    }
    let rows = opportunity::Entity::find()
        .filter(opportunity::Column::CustomerId.is_in(ids.to_vec()))
        .filter(opportunity::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(map_db_err)?;
    for row in rows {
        if let Some(ct) = row.create_time {
            let entry = last_map.entry(row.customer_id.unwrap_or(0)).or_insert(ct);
            if ct > *entry {
                *entry = ct;
            }
        }
        if need_active {
            let is_active = match row.stage {
                Some(s) => s != 5 && s != 6,
                None => true,
            };
            if is_active {
                if let Some(cid) = row.customer_id {
                    active.insert(cid);
                }
            }
        }
    }
    Ok((last_map, active))
}

/// 批量取合同上下文：最近创建时间（全部非删除合同，供「未签合同」规则）；
/// 有效合同集合（status 命中配置集合，供成单保护豁免）
async fn load_contract_context(
    db: &DbConn,
    ids: &[i64],
    deal_status: &[i32],
    need_valid: bool,
) -> Result<(HashMap<i64, chrono::NaiveDateTime>, HashSet<i64>)> {
    let mut last_map: HashMap<i64, chrono::NaiveDateTime> = HashMap::new();
    let mut valid: HashSet<i64> = HashSet::new();
    if ids.is_empty() {
        return Ok((last_map, valid));
    }
    let rows = contract::Entity::find()
        .filter(contract::Column::CustomerId.is_in(ids.to_vec()))
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(map_db_err)?;
    for row in rows {
        if let Some(ct) = row.create_time {
            let entry = last_map.entry(row.customer_id.unwrap_or(0)).or_insert(ct);
            if ct > *entry {
                *entry = ct;
            }
        }
        if need_valid {
            if let (Some(cid), Some(st)) = (row.customer_id, row.status) {
                if deal_status.contains(&contract_status_num(&st)) {
                    valid.insert(cid);
                }
            }
        }
    }
    Ok((last_map, valid))
}

/// 写到期提醒/保护通知幂等标记：仅当 reminded_at 仍为空时置 now，返回是否生效
async fn mark_reminded_once(db: &DbConn, customer_id: i64, owner: i64) -> Result<bool> {
    let ts = now();
    let res = customer::Entity::update_many()
        .set(customer::ActiveModel {
            reminded_at: Set(Some(ts)),
            ..Default::default()
        })
        .filter(customer::Column::Id.eq(customer_id))
        .filter(customer::Column::Deleted.eq(0))
        .filter(customer::Column::AssignedTo.eq(owner))
        .filter(customer::Column::RemindedAt.is_null())
        .exec(db)
        .await
        .map_err(map_db_err)?;
    Ok(res.rows_affected > 0)
}
