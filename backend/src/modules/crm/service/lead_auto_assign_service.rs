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

use crate::core::errors::error::Result;
use crate::modules::crm::entity::{lead, lead_pool, lead_pool_config, lead_pool_member};
use crate::modules::system::entity::admin;
use crate::modules::crm::service::{lead_pool_ops_service, pool_config_service, pool_member_service};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, QueryOrder};

/// 入池触发自动分配（§5.5）
///
/// 所有入池动作统一收敛到 enter_pool，事务提交后调用本方法判断：
/// 池开启自动分配（auto_assign_enabled=1）时，逐条为本批线索选取目标成员并分配；
/// 未开启则不做任何处理（线索留在池中等待领取，开启后由兜底任务消化存量）。
/// 单条失败自动跳过（由兜底扫描重试），不阻断入池主流程。
pub async fn trigger_on_enter(db: &DbConn, pool_id: i64, lead_ids: &[i64]) -> Result<usize> {
    let config = match pool_config_service::get_raw(db, pool_id).await? {
        Some(c) if c.auto_assign_enabled.unwrap_or(0) == 1 => c,
        _ => return Ok(0),
    };
    run_for_pool(db, pool_id, &config, lead_ids).await
}

/// 兜底扫描（定时任务每 5 分钟，§5.5）
///
/// 扫描所有开启自动分配的池，将 `assigned_to IS NULL` 的存量未分配线索逐条自动分配。
pub async fn sweep_all(db: &DbConn) -> Result<String> {
    let configs = lead_pool_config::Entity::find()
        .filter(lead_pool_config::Column::AutoAssignEnabled.eq(1i16))
        .all(db)
        .await
        .map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;

    let mut total: usize = 0;
    let mut details: Vec<String> = Vec::new();
    for config in &configs {
        match sweep_pool(db, config.pool_id).await {
            Ok(n) if n > 0 => {
                total += n;
                details.push(format!("池#{} 分配 {} 条", config.pool_id, n));
            },
            Ok(_) => {},
            Err(e) => log::warn!("公海池 {} 兜底自动分配失败: {}", config.pool_id, e),
        }
    }

    if total > 0 {
        Ok(format!("自动分配兜底扫描完成，共分配 {} 条（{}）", total, details.join("；")))
    } else {
        Ok("自动分配兜底扫描完成，无待分配线索".to_string())
    }
}

/// 单池兜底扫描：查未分配线索并逐条分配
async fn sweep_pool(db: &DbConn, pool_id: i64) -> Result<usize> {
    let config = pool_config_service::get_raw(db, pool_id).await?
        .ok_or_else(|| crate::core::errors::error::Error::from(format!("公海池 {} 配置缺失", pool_id)))?;

    let rows = lead::Entity::find()
        .filter(lead::Column::PoolId.eq(pool_id))
        .filter(lead::Column::AssignedTo.is_null())
        .filter(lead::Column::Deleted.eq(0))
        .filter(lead::Column::ConvertedToCustomerId.is_null())
        .all(db)
        .await
        .map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;

    if rows.is_empty() {
        return Ok(0);
    }
    let lead_ids: Vec<i64> = rows.iter().map(|l| l.id).collect();
    run_for_pool(db, pool_id, &config, &lead_ids).await
}

/// 对一批线索执行自动分配（逐条选人 → 单条分配 → 按成员聚合发送 §5.8 #9 通知）
async fn run_for_pool(db: &DbConn, pool_id: i64, config: &lead_pool_config::Model, lead_ids: &[i64]) -> Result<usize> {
    if lead_ids.is_empty() {
        return Ok(0);
    }

    let pool_name = lead_pool::Entity::find_by_id(pool_id)
        .one(db)
        .await
        .map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?
        .and_then(|p| p.name)
        .unwrap_or_else(|| format!("公海池#{}", pool_id));

    let mut assigned: HashMap<i64, Vec<String>> = HashMap::new();
    let mut count: usize = 0;

    for lead_id in lead_ids {
        // 逐条选人：每条分配后私海数/游标均已变化，重查保证轮询与负载均衡均匀
        match pick_target(db, pool_id, config).await? {
            Some(to_user) => match lead_pool_ops_service::assign_one_auto(db, *lead_id, to_user).await {
                Ok(title) => {
                    assigned.entry(to_user).or_default().push(title);
                    count += 1;
                },
                Err(e) => log::warn!("线索 {} 自动分配失败: {}", lead_id, e),
            },
            // 无可分配成员（无普通成员/全员停用/全员达保有量），后续线索同样无法分配
            None => break,
        }
    }

    // §5.8 #9 通知：来源池 + 规则名
    for (to_user, titles) in &assigned {
        let preview: Vec<String> = titles.iter().take(5).cloned().collect();
        let mut content = format!(
            "<p>公海池<strong>【{}】</strong>的 <strong>{}</strong> 条线索已按「{}」规则自动分配给您：</p><ul>{}</ul><p>请及时跟进，避免超期被自动回收。</p>",
            pool_name,
            titles.len(),
            mode_label(config.auto_assign_mode),
            preview.iter().map(|t| format!("<li>{}</li>", t)).collect::<String>()
        );
        if titles.len() > 5 {
            content.push_str(&format!("<p>…等共 {} 条线索</p>", titles.len()));
        }
        if let Err(e) = lead_pool_ops_service::send_pool_notice(db, *to_user, "线索自动分配通知", &content, 0).await {
            log::warn!("发送自动分配通知失败: {}", e);
        }
    }

    Ok(count)
}

/// 选取自动分配目标成员（§5.5）
///
/// 候选队列 = 池内普通成员（member_type=2）按加入顺序升序，剔除停用/离职成员；
/// 保有量上限（hold_limit>0）生效时剔除已达上限成员；
/// 轮询（mode=1）：游标乐观锁抢占，失败重取，最多 3 次；
/// 负载均衡（mode=2）：当前私海线索数最小者，并列取加入顺序靠前者。
async fn pick_target(db: &DbConn, pool_id: i64, config: &lead_pool_config::Model) -> Result<Option<i64>> {
    // 候选队列：普通成员升序
    let members = lead_pool_member::Entity::find()
        .filter(lead_pool_member::Column::PoolId.eq(pool_id))
        .filter(lead_pool_member::Column::MemberType.eq(2i16))
        .order_by_asc(lead_pool_member::Column::Id)
        .all(db)
        .await
        .map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
    let mut queue: Vec<i64> = members.iter().filter_map(|m| m.user_id).collect();
    if queue.is_empty() {
        return Ok(None);
    }

    // 跳过停用/离职成员（仅在职且启用状态的成员参与分配）
    let actives = admin::Entity::find()
        .filter(admin::Column::Id.is_in(queue.clone()))
        .filter(admin::Column::Status.eq(1))
        .filter(admin::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
    let active_ids: std::collections::HashSet<i64> = actives.into_iter().map(|a| a.id).collect();
    queue.retain(|uid| active_ids.contains(uid));
    if queue.is_empty() {
        return Ok(None);
    }

    // 保有量过滤（0=不启用）
    let hold_limit = config.hold_limit.unwrap_or(0);
    if hold_limit > 0 {
        let counts = pool_member_service::private_lead_counts(db, &queue).await?;
        queue.retain(|uid| counts.get(uid).copied().unwrap_or(0) < hold_limit as i64);
        if queue.is_empty() {
            return Ok(None);
        }
    }

    if config.auto_assign_mode.unwrap_or(1) == 2 {
        // 负载均衡：私海线索数最小者，并列取队列序（加入顺序）靠前
        let counts = pool_member_service::private_lead_counts(db, &queue).await?;
        Ok(queue.into_iter().min_by_key(|uid| (counts.get(uid).copied().unwrap_or(0), *uid)))
    } else {
        // 轮询：游标乐观锁抢占（assign_cursor = assign_cursor + 1 WHERE assign_cursor = 旧值），失败重取，最多 3 次
        use sea_orm::sea_query::{Expr, ExprTrait};

        let mut cursor = std::cmp::max(config.assign_cursor.unwrap_or(0), 0);
        for _ in 0..3 {
            let candidate = queue[(cursor as usize) % queue.len()];
            let res = lead_pool_config::Entity::update_many()
                .col_expr(lead_pool_config::Column::AssignCursor, Expr::col(lead_pool_config::Column::AssignCursor).add(1))
                .filter(lead_pool_config::Column::PoolId.eq(pool_id))
                .filter(lead_pool_config::Column::AssignCursor.eq(cursor))
                .exec(db)
                .await
                .map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
            if res.rows_affected > 0 {
                return Ok(Some(candidate));
            }
            // 并发冲突：重取游标再试
            cursor = std::cmp::max(
                pool_config_service::get_raw(db, pool_id).await?.and_then(|c| c.assign_cursor).unwrap_or(0),
                0,
            );
        }
        // 连续冲突放弃本次，留给兜底扫描
        Ok(None)
    }
}

/// 自动分配模式标签（§5.8 #9 通知用）
fn mode_label(mode: Option<i16>) -> &'static str {
    match mode {
        Some(2) => "负载均衡",
        _ => "轮询",
    }
}
