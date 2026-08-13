//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::sync::Mutex;
use std::time::{Duration, Instant};

use chrono::NaiveDate;
use sea_orm::{ConnectionTrait, DbBackend, DbConn, Statement, TransactionTrait};
use crate::core::errors::error::{Error, Result};

/// 汇总 topic 常量
pub const TOPIC_CONTRACT: &str = "contract";
pub const TOPIC_PAYMENT: &str = "payment";
pub const TOPIC_EMPLOYEE: &str = "employee";
pub const TOPIC_CUSTOMER: &str = "customer";
pub const ALL_TOPICS: [&str; 4] = [TOPIC_CONTRACT, TOPIC_PAYMENT, TOPIC_EMPLOYEE, TOPIC_CUSTOMER];

/// 汇总新鲜度静态缓存（60s），避免每次查询都扫批次表
static FRESH_CACHE: Mutex<Option<(Instant, bool, String)>> = Mutex::new(None);
const FRESH_CACHE_TTL: Duration = Duration::from_secs(60);

/// 双源查询参数：明细源起始日 / 汇总源结束日
///
/// 统一模式：明细表只取当日及以后（历史走汇总表），汇总表只取昨天及以前。
/// - use_agg=true：detail_start=今天，hist_end=昨天
/// - use_agg=false：detail_start=查询起点（明细覆盖全区间），hist_end=过去哨兵日（汇总源恒空）
pub fn dual_source_params(range: &crate::modules::statistics::service::stats_range::StatsRange, use_agg: bool) -> (Option<NaiveDate>, Option<NaiveDate>) {
    let today = chrono::Local::now().date_naive();
    let yesterday = today - chrono::Duration::days(1);
    if use_agg {
        let detail_start = range.start.map_or(Some(today), |s| Some(s.max(today)));
        (detail_start, Some(yesterday))
    } else {
        (range.start, NaiveDate::from_ymd_opt(1900, 1, 1))
    }
}

/// 汇总表是否新鲜：该 topic 最近成功批次的 end_date >= 昨天（即覆盖到昨天）
/// 结果缓存 60 秒
pub async fn agg_fresh(db: &DbConn, topic: &str) -> bool {
    if let Ok(mut guard) = FRESH_CACHE.lock() {
        if let Some((t, fresh, cached_topic)) = guard.as_ref() {
            if t.elapsed() < FRESH_CACHE_TTL && cached_topic == topic {
                return *fresh;
            }
        }
    }
    let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
    let fresh = check_agg_fresh(db, topic, yesterday).await.unwrap_or(false);
    if let Ok(mut guard) = FRESH_CACHE.lock() {
        *guard = Some((Instant::now(), fresh, topic.to_string()));
    }
    fresh
}

async fn check_agg_fresh(db: &DbConn, topic: &str, yesterday: NaiveDate) -> Result<bool> {
    let row = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT MAX(end_date) AS latest FROM mxx_statistics_agg_batch
               WHERE topic = $1 AND status = 1"#,
            [topic.into()],
        ))
        .await?;
    Ok(match row {
        Some(r) => {
            let latest: Option<NaiveDate> = r.try_get("", "latest").ok().flatten();
            latest.map_or(false, |d| d >= yesterday)
        }
        None => false,
    })
}

/// 手动/定时触发后清除新鲜度缓存
pub fn invalidate_fresh_cache() {
    if let Ok(mut guard) = FRESH_CACHE.lock() {
        *guard = None;
    }
}

/// 各 topic 的重算 INSERT...SELECT（在事务内执行，先 DELETE 后 INSERT）
async fn refresh_topic_in_txn<C: ConnectionTrait>(
    txn: &C,
    topic: &str,
    start: NaiveDate,
    end: NaiveDate,
    batch_id: i64,
) -> Result<u64> {
    let (table, insert_sql) = match topic {
        TOPIC_CONTRACT => (
            "mxx_statistics_daily_contract",
            r#"INSERT INTO mxx_statistics_daily_contract
               (stat_date, employee_id, dept_id, customer_id, contract_type, status, contract_count, contract_amount, batch_id)
               SELECT sign_date,
                      COALESCE(assigned_to, 0),
                      0,
                      COALESCE(customer_id, 0),
                      COALESCE(contract_type::int, 0)::smallint,
                      COALESCE(status::int, 0)::smallint,
                      COUNT(*)::int,
                      COALESCE(SUM(amount), 0),
                      $3::int8
               FROM mxx_crm_contract
               WHERE deleted = 0 AND sign_date IS NOT NULL AND sign_date BETWEEN $1::date AND $2::date
               GROUP BY 1, 2, 3, 4, 5, 6"#,
        ),
        TOPIC_PAYMENT => (
            "mxx_statistics_daily_payment",
            r#"INSERT INTO mxx_statistics_daily_payment
               (stat_date, employee_id, dept_id, customer_id, payment_count, payment_amount, contract_amount, batch_id)
               SELECT payment_date,
                      COALESCE(owner_user_id, 0),
                      0,
                      COALESCE(customer_id, 0),
                      COUNT(*)::int,
                      COALESCE(SUM(amount), 0),
                      0,
                      $3::int8
               FROM mxx_sale_payment
               WHERE deleted = 0 AND status = 2 AND payment_date IS NOT NULL AND payment_date BETWEEN $1::date AND $2::date
               GROUP BY 1, 2, 3, 4"#,
        ),
        TOPIC_CUSTOMER => (
            "mxx_statistics_daily_customer",
            r#"INSERT INTO mxx_statistics_daily_customer
               (stat_date, employee_id, customer_type, source, industry, new_count, contract_count, contract_amount, batch_id)
               SELECT create_time::date,
                      COALESCE(assigned_to, 0),
                      COALESCE(customer_type::int, 0)::smallint,
                      COALESCE(source::int, 0)::smallint,
                      COALESCE(industry::int, 0)::smallint,
                      COUNT(*)::int,
                      0,
                      0,
                      $3::int8
               FROM mxx_crm_customer
               WHERE deleted = 0 AND create_time IS NOT NULL AND create_time::date BETWEEN $1::date AND $2::date
               GROUP BY 1, 2, 3, 4, 5"#,
        ),
        TOPIC_EMPLOYEE => (
            "mxx_statistics_daily_employee",
            r#"INSERT INTO mxx_statistics_daily_employee
               (stat_date, employee_id, dept_id, new_customers, contract_customers, followup_total,
                followup_customer, followup_opportunity, new_leads, new_opportunities,
                won_opportunities, lost_opportunities, contract_count, contract_amount, batch_id)
               SELECT d, eid, 0,
                      SUM(nc)::int, SUM(cc)::int, SUM(ft)::int,
                      SUM(fc)::int, SUM(fo)::int, 0,
                      SUM(no_)::int, SUM(wo)::int, SUM(lo)::int,
                      SUM(cnt)::int, SUM(amt), $3::int8
               FROM (
                   SELECT create_time::date AS d, COALESCE(assigned_to, 0) AS eid,
                          1 AS nc, 0 AS cc, 0 AS ft, 0 AS fc, 0 AS fo, 0 AS no_, 0 AS wo, 0 AS lo, 0 AS cnt, 0 AS amt
                   FROM mxx_crm_customer
                   WHERE deleted = 0 AND create_time IS NOT NULL AND assigned_to IS NOT NULL
                     AND create_time::date BETWEEN $1::date AND $2::date
                   UNION ALL
                   SELECT sign_date, COALESCE(assigned_to, 0),
                          0, 0, 0, 0, 0, 0, 0, 0, 1, COALESCE(amount, 0)
                   FROM mxx_crm_contract
                   WHERE deleted = 0 AND sign_date IS NOT NULL AND assigned_to IS NOT NULL
                     AND sign_date BETWEEN $1::date AND $2::date
                   UNION ALL
                   SELECT sign_date, COALESCE(assigned_to, 0),
                          0, COUNT(DISTINCT customer_id)::int8, 0, 0, 0, 0, 0, 0, 0, 0
                   FROM mxx_crm_contract
                   WHERE deleted = 0 AND sign_date IS NOT NULL AND assigned_to IS NOT NULL AND customer_id IS NOT NULL
                     AND sign_date BETWEEN $1::date AND $2::date
                   GROUP BY 1, 2
                   UNION ALL
                   SELECT create_time::date, COALESCE(assigned_to, created_by, 0),
                          0, 0, 1, CASE WHEN source_type = 2 THEN 1 ELSE 0 END, CASE WHEN source_type = 3 THEN 1 ELSE 0 END,
                          0, 0, 0, 0, 0
                   FROM mxx_crm_followup
                   WHERE deleted = 0 AND create_time IS NOT NULL AND COALESCE(assigned_to, created_by, 0) > 0
                     AND create_time::date BETWEEN $1::date AND $2::date
                   UNION ALL
                   SELECT create_time::date, COALESCE(assigned_to, created_by, 0),
                          0, 0, 0, 0, 0, 1, CASE WHEN stage = 4 THEN 1 ELSE 0 END, CASE WHEN stage = 5 THEN 1 ELSE 0 END, 0, 0
                   FROM mxx_crm_opportunity
                   WHERE deleted = 0 AND create_time IS NOT NULL AND COALESCE(assigned_to, created_by, 0) > 0
                     AND create_time::date BETWEEN $1::date AND $2::date
               ) src
               GROUP BY 1, 2"#,
        ),
        _ => return Err(Error::from(format!("未知汇总 topic: {}", topic))),
    };

    // 幂等：先删后插（同一事务内原子生效）
    let del_stmt = Statement::from_sql_and_values(
        DbBackend::Postgres,
        format!("DELETE FROM {} WHERE stat_date BETWEEN $1::date AND $2::date", table),
        [crate::modules::statistics::service::stats_range::date_param(Some(start)), crate::modules::statistics::service::stats_range::date_param(Some(end))],
    );
    txn.execute_raw(del_stmt)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let ins_stmt = Statement::from_sql_and_values(
        DbBackend::Postgres,
        insert_sql,
        [crate::modules::statistics::service::stats_range::date_param(Some(start)), crate::modules::statistics::service::stats_range::date_param(Some(end)), batch_id.into()],
    );
    let ins = txn
        .execute_raw(ins_stmt)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // row_count 语义：写入行数（INSERT 影响行数），供批次表追溯
    Ok(ins.rows_affected())
}

/// 记录批次（成功/失败）
async fn insert_batch(
    db: &DbConn,
    topic: &str,
    start: NaiveDate,
    end: NaiveDate,
    row_count: i32,
    trigger_type: i16,
    trigger_by: i64,
    status: i16,
    message: Option<String>,
) -> i64 {
    let result = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO mxx_statistics_agg_batch
               (topic, start_date, end_date, row_count, trigger_type, trigger_by, status, message, create_time, update_time)
               VALUES ($1, $2::date, $3::date, $4, $5, $6, $7, $8, NOW(), NOW())
               RETURNING id"#,
            [
                topic.into(),
                crate::modules::statistics::service::stats_range::date_param(Some(start)),
                crate::modules::statistics::service::stats_range::date_param(Some(end)),
                row_count.into(),
                trigger_type.into(),
                trigger_by.into(),
                status.into(),
                message.into(),
            ],
        ))
        .await;
    match result {
        Ok(Some(r)) => r.try_get::<i64>("", "id").unwrap_or(0),
        _ => 0,
    }
}

/// 重算单个 topic（单 topic 一个事务：DELETE+INSERT+批次 原子完成）
/// 失败时记录失败批次（独立写入），不影响其他 topic
pub async fn refresh_topic(
    db: &DbConn,
    topic: &str,
    start: NaiveDate,
    end: NaiveDate,
    trigger_type: i16,
    trigger_by: i64,
) -> Result<(i64, u64)> {
    if end < start {
        return Err(Error::from("重算区间无效：end < start"));
    }
    // 预生成批次 id（成功批次在事务内不再单独插入，事务外插成功记录）
    // 事务：DELETE + INSERT 原子（项目惯例 begin/commit 模式）
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let tx_res = match refresh_topic_in_txn(&txn, topic, start, end, chrono::Utc::now().timestamp_millis()).await {
        Ok(n) => {
            txn.commit().await.map_err(|e| Error::from(e.to_string())).map(|_| n)
        }
        Err(e) => {
            let _ = txn.rollback().await;
            Err(e)
        }
    };

    match tx_res {
        Ok(affected) => {
            let batch_id = insert_batch(
                db, topic, start, end, affected as i32, trigger_type, trigger_by, 1, None,
            )
            .await;
            invalidate_fresh_cache();
            Ok((batch_id, affected))
        }
        Err(e) => {
            insert_batch(
                db,
                topic,
                start,
                end,
                0,
                trigger_type,
                trigger_by,
                2,
                Some(e.to_string().chars().take(480).collect()),
            )
            .await;
            Err(Error::from(format!("topic {} 重算失败: {}", topic, e)))
        }
    }
}

/// 重算全部 topic（定时任务 trigger_type=1 / 手动全量 trigger_type=2）
pub async fn refresh_all(db: &DbConn, start: NaiveDate, end: NaiveDate) -> Result<u64> {
    refresh_all_with(db, start, end, 1, 0).await
}

/// 重算全部 topic（带触发类型，供手动全量透传 trigger_type=2）
pub async fn refresh_all_with(db: &DbConn, start: NaiveDate, end: NaiveDate, trigger_type: i16, trigger_by: i64) -> Result<u64> {
    let mut total = 0u64;
    let mut errs = Vec::new();
    for topic in ALL_TOPICS {
        match refresh_topic(db, topic, start, end, trigger_type, trigger_by).await {
            Ok((_, n)) => total += n,
            Err(e) => errs.push(e.to_string()),
        }
    }
    if errs.is_empty() {
        Ok(total)
    } else {
        Err(Error::from(format!("部分 topic 失败: {}", errs.join("; "))))
    }
}

// ==================== 回填检测（B2.2）====================
// 日常任务只覆盖近 4 天；用户把签约/回款日期改到窗口外（或历史日期数据被确认/修改）时，
// 同步定向重算目标月（单月明细聚合秒级），关闭漂移缺口。月度全量重算作为最后兜底。

/// 重算某个日期所在月（1 号 ~ 月末）
async fn refresh_month(db: &DbConn, topic: &str, d: NaiveDate, trigger_by: i64) {
    let start = d.with_day(1).unwrap_or(d);
    let end = {
        let next_month = if d.month() == 12 {
            chrono::NaiveDate::from_ymd_opt(d.year() + 1, 1, 1)
        } else {
            chrono::NaiveDate::from_ymd_opt(d.year(), d.month() + 1, 1)
        };
        next_month.map_or(Some(d), |n| Some(n - chrono::Duration::days(1)))
            .unwrap_or(d)
    };
    if let Err(e) = refresh_topic(db, topic, start, end, 2, trigger_by).await {
        log::warn!("[stats][回填] {} 月度重算失败: {} ~ {} err={}", topic, start, end, e);
    }
}

use chrono::Datelike;

/// 合同签约日期变更回填检测（事务提交后调用，best-effort 不阻断业务）
pub async fn on_contract_date_changed(db: &DbConn, old_date: Option<NaiveDate>, new_date: Option<NaiveDate>, trigger_by: i64) {
    let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
    let mut months: Vec<NaiveDate> = Vec::new();
    for d in [old_date, new_date].into_iter().flatten() {
        // 只需重算已落汇总表的日期（< 今天）
        if d < chrono::Local::now().date_naive() && d <= yesterday {
            let m = d.with_day(1).unwrap_or(d);
            if !months.contains(&m) {
                months.push(m);
            }
        }
    }
    for m in months {
        refresh_month(db, TOPIC_CONTRACT, m, trigger_by).await;
        // 合同主题变化同步影响员工汇总（合同数/金额/成交客户）
        refresh_month(db, TOPIC_EMPLOYEE, m, trigger_by).await;
    }
}

/// 回款日期变更/确认回填检测（事务提交后调用，best-effort）
pub async fn on_payment_date_changed(db: &DbConn, old_date: Option<NaiveDate>, new_date: Option<NaiveDate>, trigger_by: i64) {
    let today = chrono::Local::now().date_naive();
    let mut months: Vec<NaiveDate> = Vec::new();
    for d in [old_date, new_date].into_iter().flatten() {
        if d < today {
            let m = d.with_day(1).unwrap_or(d);
            if !months.contains(&m) {
                months.push(m);
            }
        }
    }
    for m in months {
        refresh_month(db, TOPIC_PAYMENT, m, trigger_by).await;
    }
}
