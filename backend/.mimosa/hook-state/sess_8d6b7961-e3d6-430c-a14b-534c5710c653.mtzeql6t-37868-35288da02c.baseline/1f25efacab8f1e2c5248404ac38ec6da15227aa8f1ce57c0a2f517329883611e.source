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
use crate::modules::crm::entity::{lead, lead_pool, lead_pool_config};
use crate::modules::crm::service::lead_pool_ops_service;
use sea_orm::{ColumnTrait, ConnectionTrait, DbConn, EntityTrait, QueryFilter, Set, Statement};

fn map_db_err(e: sea_orm::DbErr) -> Error {
    Error::from(e.to_string())
}

/// 回收到期时点（§5.4/§5.7）：基线取「最后跟进时间」与「当前分配时间」的较大者，
/// 保证每次获得新归属都有完整跟进窗口，避免旧跟进记录导致再分配后立即二次可回收。
const DEADLINE_EXPR: &str =
    "(GREATEST(COALESCE(l.last_follow_up_at, l.assigned_at), l.assigned_at) \
     + ($2 + COALESCE(l.recycle_extend_days, 0)) * INTERVAL '1 day')";

/// 私海在负线索通用过滤（§5.4）
const PRIVATE_LEAD_WHERE: &str =
    "l.deleted = 0 AND l.assigned_to IS NOT NULL AND l.converted_to_customer_id IS NULL AND l.status NOT IN (4, 5)";

/// 线索超期自动回收（每日 00:30，§5.4/§5.6）
///
/// 逐池按配置扫描超期线索：`deleted=0 AND assigned_to IS NOT NULL AND converted_to_customer_id IS NULL
/// AND status NOT IN(4,5) AND now > 回收到期时点`，逐条走 enter_pool(ACTION_AUTO_RECYCLE) 归还原池、
/// 记时间轴(action=5)、通知原负责人（§5.8 #6），并联动自动分配触发。
pub async fn run_recycle(db: &DbConn) -> Result<String> {
    let configs = lead_pool_config::Entity::find()
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
            Err(e) => log::warn!("公海池 {} 自动回收失败: {}", config.pool_id, e),
        }
    }

    if total > 0 {
        Ok(format!("线索自动回收完成，共回收 {} 条（{}）", total, details.join("；")))
    } else {
        Ok("线索自动回收完成，无超期线索".to_string())
    }
}

/// 单池自动回收：扫描该池全部超期在负线索，逐条回还原池
async fn recycle_pool(db: &DbConn, config: &lead_pool_config::Model) -> Result<usize> {
    let pool_id = config.pool_id;
    let recycle_days = config.recycle_days.unwrap_or(30);

    let sql = format!(
        "SELECT l.id, l.assigned_to FROM mxx_crm_lead l \
         WHERE l.pool_id = $1 AND {} AND {} < NOW()",
        PRIVATE_LEAD_WHERE, DEADLINE_EXPR
    );
    let stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        &sql,
        [pool_id.into(), (recycle_days as i32).into()],
    );
    let rows = db.query_all_raw(stmt).await.map_err(map_db_err)?;
    if rows.is_empty() {
        return Ok(0);
    }

    let mut count: usize = 0;
    for row in rows {
        let lead_id: i64 = row.try_get("", "id").unwrap_or(0);
        let owner: Option<i64> = row.try_get::<Option<i64>>("", "assigned_to").ok().flatten();
        let Some(owner) = owner else { continue };

        let lead_ids = [lead_id];
        match lead_pool_ops_service::enter_pool(
            db,
            &lead_ids,
            pool_id,
            lead_pool_ops_service::ACTION_AUTO_RECYCLE,
            0,
            Some(owner),
            None,
            &None,
        ).await {
            Ok(results) if results.first().map(|r| r.success).unwrap_or(false) => count += 1,
            Ok(results) => {
                let msg = results.first().and_then(|r| r.message.clone()).unwrap_or_default();
                log::warn!("线索 {} 自动回收未生效: {}", lead_id, msg);
            },
            Err(e) => log::warn!("线索 {} 自动回收失败: {}", lead_id, e),
        }
    }
    Ok(count)
}

/// 线索到期提醒（每日 09:00，§5.4/§5.6）
///
/// 逐池扫描 `reminder_days > 0` 的池，对「回收到期时点已进入提醒窗口且 reminded_at IS NULL」的
/// 在负线索发送站内信（§5.8 #7：剩余天数 + 延期申请入口），发送后写 `reminded_at = now` 幂等标记。
pub async fn run_reminder(db: &DbConn) -> Result<String> {
    let configs = lead_pool_config::Entity::find()
        .filter(lead_pool_config::Column::ReminderDays.gt(0))
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
            Err(e) => log::warn!("公海池 {} 到期提醒失败: {}", config.pool_id, e),
        }
    }

    if total > 0 {
        Ok(format!("线索到期提醒完成，共提醒 {} 条（{}）", total, details.join("；")))
    } else {
        Ok("线索到期提醒完成，无待提醒线索".to_string())
    }
}

/// 单池到期提醒：扫描提醒窗口内未提醒线索，逐条发站内信并落幂等标记
async fn remind_pool(db: &DbConn, config: &lead_pool_config::Model) -> Result<usize> {
    let pool_id = config.pool_id;
    let recycle_days = config.recycle_days.unwrap_or(30);
    let reminder_days = config.reminder_days.unwrap_or(0);
    if reminder_days <= 0 {
        return Ok(0);
    }

    let pool_name = lead_pool::Entity::find_by_id(pool_id)
        .one(db)
        .await
        .map_err(map_db_err)?
        .and_then(|p| p.name)
        .unwrap_or_else(|| format!("公海池#{}", pool_id));

    let sql = format!(
        "SELECT l.id, l.company_name, l.contact_name, {} AS deadline FROM mxx_crm_lead l \
         WHERE l.pool_id = $1 AND {} AND l.reminded_at IS NULL \
         AND {} < NOW() + $3 * INTERVAL '1 day'",
        DEADLINE_EXPR, PRIVATE_LEAD_WHERE, DEADLINE_EXPR
    );
    let stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        &sql,
        [pool_id.into(), (recycle_days as i32).into(), (reminder_days as i32).into()],
    );
    let rows = db.query_all_raw(stmt).await.map_err(map_db_err)?;
    if rows.is_empty() {
        return Ok(0);
    }

    let mut count: usize = 0;
    for row in rows {
        let lead_id: i64 = row.try_get("", "id").unwrap_or_default();
        let Some(owner) = row.try_get::<Option<i64>>("", "assigned_to").ok().flatten() else {
            log::warn!("线索缺少负责人，跳过到期提醒");
            continue;
        };
        let company: Option<String> = row.try_get("", "company_name").ok().flatten();
        let contact: Option<String> = row.try_get("", "contact_name").ok().flatten();
        let deadline: chrono::NaiveDateTime = row.try_get("", "deadline").unwrap_or_default();

        let title = company
            .filter(|s| !s.trim().is_empty())
            .or_else(|| contact.filter(|s| !s.trim().is_empty()))
            .unwrap_or_else(|| format!("线索#{}", lead_id));

        // 剩余天数向上取整，已过期为 0
        let now_ts = chrono::Local::now().naive_local();
        let remain = (((deadline - now_ts).num_seconds() + 86399) / 86400).max(0);

        let content = format!(
            "<p>您的线索<strong>【{}】</strong>已超期未跟进，将在 <strong>{} 天</strong>后被自动回收至公海<strong>【{}】</strong>。</p><p>如需继续跟进，请及时处理；也可在私海对该线索提交延期申请。</p>",
            title, remain, pool_name
        );
        if let Err(e) = lead_pool_ops_service::send_pool_notice(db, owner, "线索到期提醒", &content, 0).await {
            log::warn!("线索 {} 到期提醒发送失败: {}", lead_id, e);
            continue;
        }

        // 写幂等标记：仅当 reminded_at 仍为空时生效，防并发重复提醒
        let res = lead::Entity::update_many()
            .set(lead::ActiveModel {
                reminded_at: Set(Some(now_ts)),
                updated_by: Set(Some(0)),
                ..Default::default()
            })
            .filter(lead::Column::Id.eq(lead_id))
            .filter(lead::Column::RemindedAt.is_null())
            .exec(db)
            .await
            .map_err(map_db_err)?;
        if res.rows_affected > 0 {
            count += 1;
        }
    }
    Ok(count)
}
