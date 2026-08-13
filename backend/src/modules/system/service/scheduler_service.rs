//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::modules::system::entity::{scheduler_job, scheduler_log};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerJobVO {
    pub id: i64,
    pub job_code: String,
    pub job_name: String,
    pub cron_expression: String,
    pub handler: String,
    pub handler_params: Option<serde_json::Value>,
    pub description: Option<String>,
    pub job_type: Option<i32>,
    pub enabled: Option<i32>,
    pub last_run_time: Option<String>,
    pub last_run_status: Option<i32>,
    pub last_run_result: Option<String>,
    pub next_run_time: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

impl From<scheduler_job::Model> for SchedulerJobVO {
    fn from(m: scheduler_job::Model) -> Self {
        Self {
            id: m.id,
            job_code: m.job_code,
            job_name: m.job_name,
            cron_expression: m.cron_expression,
            handler: m.handler,
            handler_params: m.handler_params,
            description: m.description,
            job_type: m.job_type,
            enabled: m.enabled,
            last_run_time: m.last_run_time.map(|t| t.to_string()),
            last_run_status: m.last_run_status,
            last_run_result: m.last_run_result,
            next_run_time: m.next_run_time.map(|t| t.to_string()),
            create_time: m.create_time.map(|t| t.to_string()),
            update_time: m.update_time.map(|t| t.to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerLogVO {
    pub id: i64,
    pub job_id: i64,
    pub job_code: Option<String>,
    pub trigger_type: Option<i32>,
    pub status: Option<i32>,
    pub result_message: Option<String>,
    pub error_message: Option<String>,
    pub elapsed_ms: Option<i64>,
    pub operator_id: Option<i64>,
    pub operator_name: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

impl From<scheduler_log::Model> for SchedulerLogVO {
    fn from(m: scheduler_log::Model) -> Self {
        Self {
            id: m.id,
            job_id: m.job_id,
            job_code: m.job_code,
            trigger_type: m.trigger_type,
            status: m.status,
            result_message: m.result_message,
            error_message: m.error_message,
            elapsed_ms: m.elapsed_ms,
            operator_id: m.operator_id,
            operator_name: m.operator_name,
            start_time: m.start_time.map(|t| t.to_string()),
            end_time: m.end_time.map(|t| t.to_string()),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerJobQuery {
    pub job_code: Option<String>,
    pub job_name: Option<String>,
    pub enabled: Option<i32>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerJobUpdateDTO {
    pub id: i64,
    pub cron_expression: Option<String>,
    pub job_name: Option<String>,
    pub description: Option<String>,
    pub handler_params: Option<serde_json::Value>,
    pub enabled: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerToggleDTO {
    pub id: i64,
    pub enabled: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerTriggerDTO {
    pub id: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerLogQuery {
    pub job_id: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 任务列表
pub async fn get_job_list(
    db: &DatabaseConnection,
    query: SchedulerJobQuery,
) -> Result<(Vec<SchedulerJobVO>, i64), String> {
    let mut stmt = scheduler_job::Entity::find()
        .filter(scheduler_job::Column::Deleted.eq(0));

    if let Some(code) = &query.job_code {
        if !code.is_empty() {
            stmt = stmt.filter(scheduler_job::Column::JobCode.contains(code));
        }
    }
    if let Some(name) = &query.job_name {
        if !name.is_empty() {
            stmt = stmt.filter(scheduler_job::Column::JobName.contains(name));
        }
    }
    if let Some(enabled) = query.enabled {
        stmt = stmt.filter(scheduler_job::Column::Enabled.eq(enabled));
    }

    stmt = stmt.order_by_desc(scheduler_job::Column::CreateTime);

    let page = std::cmp::max(query.page.unwrap_or(1), 1);
    let page_size = std::cmp::max(query.page_size.unwrap_or(20), 1);
    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator.fetch_page((page - 1) as u64).await.map_err(|e| e.to_string())?;
    let vo_list: Vec<SchedulerJobVO> = items.into_iter().map(SchedulerJobVO::from).collect();
    Ok((vo_list, total))
}

/// 任务详情
pub async fn get_job_detail(db: &DatabaseConnection, id: i64) -> Result<SchedulerJobVO, String> {
    let job = scheduler_job::Entity::find_by_id(id)
        .one(db).await.map_err(|e| e.to_string())?
        .ok_or("任务不存在".to_string())?;
    Ok(SchedulerJobVO::from(job))
}

/// 更新任务
/// 注意：cron 表达式或 handler 修改后，内存中的调度器不会自动重载（tokio-cron-scheduler 限制）
/// 需要重启后端服务才能让新的 cron 生效。此处只更新数据库，并在响应中提示用户。
/// D-4: cron 修改后计算并回写 next_run_time
pub async fn update_job(
    db: &DatabaseConnection,
    dto: SchedulerJobUpdateDTO,
) -> Result<String, String> {
    let job = scheduler_job::Entity::find_by_id(dto.id)
        .one(db).await.map_err(|e| e.to_string())?
        .ok_or("任务不存在".to_string())?;

    let mut active: scheduler_job::ActiveModel = job.into();
    let mut cron_changed = false;
    if let Some(cron) = dto.cron_expression {
        // 简单校验 cron 格式（至少5个空格分隔的字段）
        let parts: Vec<&str> = cron.split_whitespace().collect();
        if parts.len() < 5 || parts.len() > 7 {
            return Err("cron 表达式格式错误，应为 5-7 个字段".to_string());
        }
        active.cron_expression = Set(cron.clone());
        cron_changed = true;

        // D-4: 计算并回写 next_run_time
        if let Some(next) = compute_next_run_time(&cron) {
            active.next_run_time = Set(Some(next.naive_utc()));
        }
    }
    if let Some(name) = dto.job_name { active.job_name = Set(name); }
    if let Some(desc) = dto.description { active.description = Set(Some(desc)); }
    if let Some(params) = dto.handler_params {
        active.handler_params = Set(Some(params));
    }
    if let Some(enabled) = dto.enabled { active.enabled = Set(Some(enabled)); }
    active.update_time = Set(Some(Utc::now().naive_utc()));

    active.update(db).await.map_err(|e| e.to_string())?;

    // V7-2: cron 变更或启用状态变更后，动态重载调度器（无需重启后端）
    if cron_changed || dto.enabled.is_some() {
        if let Err(e) = crate::core::kit::scheduler::reload_scheduler(db.clone()).await {
            log::warn!("[scheduler] 动态重载失败（不影响 DB 更新）: {}", e);
        }
    }

    if cron_changed {
        Ok("任务已更新，调度器已动态重载，新 cron 表达式立即生效".to_string())
    } else {
        Ok("任务已更新".to_string())
    }
}

/// D-4: 根据 cron 表达式计算下次执行时间（简化实现，支持 5/6/7 字段标准 cron，首字段为秒时自动跳过）
/// 从 from 时刻开始逐分钟向后扫描，最多 400 天，找到第一个匹配 cron 的时刻
pub fn compute_next_run_time_from(
    cron_expr: &str,
    from: chrono::DateTime<chrono::Utc>,
) -> Option<chrono::DateTime<chrono::Utc>> {
    let parts_all: Vec<&str> = cron_expr.split_whitespace().collect();
    // 支持 5 字段(分时日月周)；6 字段(秒分时日月周)；7 字段(秒分时日月周年)
    let parts: Vec<&str> = match parts_all.len() {
        5 => parts_all,
        6 => parts_all[1..].to_vec(),
        7 => parts_all[2..].to_vec(),
        _ => return None,
    };
    if parts.len() != 5 {
        return None;
    }

    use chrono::Timelike;
    let mut candidate = from
        .with_second(0)
        .unwrap_or(from)
        .with_nanosecond(0)
        .unwrap_or(from)
        + chrono::Duration::minutes(1);

    let max_minutes: i64 = 400 * 24 * 60;
    for _ in 0..max_minutes {
        if cron_matches(&parts, &candidate) {
            return Some(candidate);
        }
        candidate = candidate + chrono::Duration::minutes(1);
    }
    None
}

/// D-4: 从当前时间计算下次执行时间（用于展示）
pub fn compute_next_run_time(cron_expr: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    compute_next_run_time_from(cron_expr, chrono::Utc::now())
}

/// 简化版 cron 匹配：支持 * / 数字 / 逗号分隔 / 横线范围
fn cron_matches(parts: &[&str], dt: &chrono::DateTime<chrono::Utc>) -> bool {
    use chrono::Datelike;
    use chrono::Timelike;

    if parts.len() < 5 {
        return false;
    }

    let minute = dt.minute() as i32;
    let hour = dt.hour() as i32;
    let day = dt.day() as i32;
    let month = dt.month() as i32;
    let weekday = dt.weekday().num_days_from_sunday() as i32; // 0=Sunday

    if !cron_field_matches(parts[0], minute) { return false; }
    if !cron_field_matches(parts[1], hour) { return false; }
    if !cron_field_matches(parts[2], day) { return false; }
    if !cron_field_matches(parts[3], month) { return false; }
    if !cron_field_matches(parts[4], weekday) { return false; }
    true
}

fn cron_field_matches(field: &str, value: i32) -> bool {
    if field == "*" {
        return true;
    }
    // 处理逗号分隔
    for part in field.split(',') {
        let part = part.trim();
        // 处理 */N 步长
        if let Some(step_str) = part.strip_prefix("*/") {
            if let Ok(step) = step_str.parse::<i32>() {
                if step > 0 && value % step == 0 {
                    return true;
                }
            }
            continue;
        }
        // 处理范围 a-b
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                if let (Ok(lo), Ok(hi)) = (range[0].parse::<i32>(), range[1].parse::<i32>()) {
                    if value >= lo && value <= hi {
                        return true;
                    }
                }
            }
            continue;
        }
        // 纯数字
        if let Ok(v) = part.parse::<i32>() {
            if v == value {
                return true;
            }
        }
    }
    false
}

/// 启用/禁用
/// V7-2: 启用状态变更后动态重载调度器
pub async fn toggle_job(
    db: &DatabaseConnection,
    dto: SchedulerToggleDTO,
) -> Result<(), String> {
    let job = scheduler_job::Entity::find_by_id(dto.id)
        .one(db).await.map_err(|e| e.to_string())?
        .ok_or("任务不存在".to_string())?;

    let mut active: scheduler_job::ActiveModel = job.into();
    active.enabled = Set(Some(dto.enabled));
    active.update_time = Set(Some(Utc::now().naive_utc()));
    active.update(db).await.map_err(|e| e.to_string())?;

    // V7-2: 启用/禁用状态变更后动态重载调度器
    if let Err(e) = crate::core::kit::scheduler::reload_scheduler(db.clone()).await {
        log::warn!("[scheduler] 动态重载失败（不影响 DB 更新）: {}", e);
    }
    Ok(())
}

/// 手动触发执行
/// V7-5: 移除 enabled==0 校验，允许禁用状态下手动触发（用于调试场景）
pub async fn trigger_job(
    db: &DatabaseConnection,
    dto: SchedulerTriggerDTO,
    operator_id: i64,
    operator_name: &str,
) -> Result<String, String> {
    let job = scheduler_job::Entity::find_by_id(dto.id)
        .one(db).await.map_err(|e| e.to_string())?
        .ok_or("任务不存在".to_string())?;

    // V7-5: 移除 enabled 校验，允许禁用状态下手动触发
    // if job.enabled.unwrap_or(1) == 0 {
    //     return Err("任务已禁用，请先启用".to_string());
    // }

    // D-4: 两阶段日志——先记录"运行中"(status=2)，执行中进程退出时由下次启动标记为中断
    let log_id = start_run_log(db, job.id, &job.job_code, 1, operator_id, operator_name).await?;

    let start = std::time::Instant::now();
    let result = execute_handler(&job.handler, &job.handler_params, db).await;
    let elapsed = start.elapsed().as_millis() as i64;

    let (status, result_msg, error_msg) = match result {
        Ok(msg) => (1i32, Some(msg), None),
        Err(e) => (0, None, Some(e)),
    };

    // D-4: 回填运行日志并刷新任务 last_run_*/next_run_time
    finish_run_log(
        db,
        log_id,
        job.id,
        &job.job_code,
        status,
        result_msg.as_deref(),
        error_msg.as_deref(),
        elapsed,
    )
    .await?;

    if status == 1 {
        Ok(result_msg.unwrap_or_else(|| "执行成功".to_string()))
    } else {
        Err(error_msg.unwrap_or_else(|| "执行失败".to_string()))
    }
}

/// 执行日志列表
pub async fn get_log_list(
    db: &DatabaseConnection,
    query: SchedulerLogQuery,
) -> Result<(Vec<SchedulerLogVO>, i64), String> {
    let mut stmt = scheduler_log::Entity::find()
        .order_by_desc(scheduler_log::Column::StartTime);

    if let Some(job_id) = query.job_id {
        stmt = stmt.filter(scheduler_log::Column::JobId.eq(job_id));
    }

    let page = std::cmp::max(query.page.unwrap_or(1), 1);
    let page_size = std::cmp::max(query.page_size.unwrap_or(20), 1);
    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator.fetch_page((page - 1) as u64).await.map_err(|e| e.to_string())?;
    let vo_list: Vec<SchedulerLogVO> = items.into_iter().map(SchedulerLogVO::from).collect();
    Ok((vo_list, total))
}

/// 执行处理器（根据 handler 标识分发）
/// V7-4: 透传 handler_params 到处理器，支持参数化触发
async fn execute_handler(
    handler: &str,
    params: &Option<sea_orm::prelude::Json>,
    db: &DatabaseConnection,
) -> Result<String, String> {
    match handler {
        "salary_calculate" => {
            // V7-4: 解析 params 中的 year/month/department_id 可选字段
            let (year, month) = parse_year_month_from_params(params);
            let count = crate::modules::finance::service::salary_service::calculate(
                db, year, month, 1, 0, "系统定时任务"
            ).await?;
            Ok(format!("核算完成：{}年{}月，生成 {} 条工资记录", year, month, count))
        }
        _ => Err(format!("未知的处理器: {}", handler)),
    }
}

/// V7-4: 从 handler_params 解析 year/month，缺失时回退为"上月"
fn parse_year_month_from_params(params: &Option<sea_orm::prelude::Json>) -> (i32, i32) {
    use chrono::{Datelike, Utc};
    let now = Utc::now();
    let default_year = if now.month() == 1 { now.year() - 1 } else { now.year() };
    let default_month = if now.month() == 1 { 12 } else { now.month() - 1 } as i32;

    if let Some(sea_orm::prelude::Json::Object(obj)) = params {
        let year = obj.get("year")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .unwrap_or(default_year);
        let month = obj.get("month")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .unwrap_or(default_month);
        (year, month)
    } else {
        (default_year, default_month)
    }
}

/// 记录任务运行开始（status=2 运行中）
/// 任务执行过程中进程退出/重载时，该记录会保持"运行中"，由 mark_interrupted_runs 在下次启动时标记为中断
pub async fn start_run_log(
    db: &DatabaseConnection,
    job_id: i64,
    job_code: &str,
    trigger_type: i32,
    operator_id: i64,
    operator_name: &str,
) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let log = scheduler_log::ActiveModel {
        job_id: Set(job_id),
        job_code: Set(Some(job_code.to_string())),
        trigger_type: Set(Some(trigger_type)), // 0=定时 1=手动 2=补跑
        status: Set(Some(2)),                   // 运行中
        result_message: Set(None),
        error_message: Set(None),
        elapsed_ms: Set(None),
        operator_id: Set(Some(operator_id)),
        operator_name: Set(Some(operator_name.to_string())),
        start_time: Set(Some(now)),
        end_time: Set(None),
        ..Default::default()
    };
    let inserted = log.insert(db).await.map_err(|e| e.to_string())?;
    Ok(inserted.id)
}

/// 记录任务运行结束（status: 0=失败 1=成功），并刷新任务 last_run_* 与 next_run_time
pub async fn finish_run_log(
    db: &DatabaseConnection,
    log_id: i64,
    job_id: i64,
    job_code: &str,
    status: i32,
    result_msg: Option<&str>,
    error_msg: Option<&str>,
    elapsed_ms: i64,
) -> Result<(), String> {
    let now = Utc::now().naive_utc();

    if let Some(log) = scheduler_log::Entity::find_by_id(log_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
    {
        let mut active: scheduler_log::ActiveModel = log.into();
        active.status = Set(Some(status));
        active.result_message = Set(result_msg.map(|s| s.to_string()));
        active.error_message = Set(error_msg.map(|s| s.to_string()));
        active.elapsed_ms = Set(Some(elapsed_ms));
        active.end_time = Set(Some(now));
        active.update(db).await.map_err(|e| e.to_string())?;
    } else {
        log::warn!("[scheduler] finish_run_log: 日志不存在 id={}", log_id);
    }

    // 刷新任务最后执行信息 + 计算下次执行时间（修复 next_run_time 过期/缺失问题）
    if let Some(job) = scheduler_job::Entity::find_by_id(job_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
    {
        // 先取出 cron 表达式副本（job 后续会被 .into() 移动）
        let cron_expr = job.cron_expression.clone();
        let mut active: scheduler_job::ActiveModel = job.into();
        active.last_run_time = Set(Some(now));
        active.last_run_status = Set(Some(status));
        active.last_run_result = Set(
            if status == 1 {
                result_msg.map(|s| s.to_string())
            } else {
                error_msg.map(|s| s.to_string())
            },
        );
        active.next_run_time = Set(
            compute_next_run_time_from(&cron_expr, Utc::now()).map(|t| t.naive_utc()),
        );
        active.update_time = Set(Some(now));
        active.update(db).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// P2-6: 更新任务的 last_retry_count 字段
/// 在 finish_run_log 调用前单独调用，避免 update_time 相互覆盖
pub async fn update_job_retry_count(
    db: &DatabaseConnection,
    job_id: i64,
    retry_count: i32,
) -> Result<(), String> {
    let now = Utc::now().naive_utc();
    if let Some(job) = scheduler_job::Entity::find_by_id(job_id).one(db).await.map_err(|e| e.to_string())? {
        let mut active: scheduler_job::ActiveModel = job.into();
        active.last_retry_count = Set(Some(retry_count));
        active.update_time = Set(Some(now));
        active.update(db).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 启动/重载时扫描：将早于 cutoff 的"运行中"日志标记为"中断"(status=3)
/// 返回被标记的 (job_code, 条数) 列表，供上层记录日志与告警
pub async fn mark_interrupted_runs(
    db: &DatabaseConnection,
    cutoff: chrono::NaiveDateTime,
) -> Result<Vec<(String, i64)>, String> {
    let logs = scheduler_log::Entity::find()
        .filter(scheduler_log::Column::Status.eq(2))
        .filter(scheduler_log::Column::StartTime.lt(cutoff))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut map: HashMap<String, i64> = HashMap::new();
    let mut interrupted_job_ids: Vec<i64> = Vec::new();
    for log in logs {
        let mut active: scheduler_log::ActiveModel = log.clone().into();
        active.status = Set(Some(3)); // 中断
        active.error_message = Set(Some(
            "任务执行过程中进程退出/调度器重载，已被系统标记为中断".to_string(),
        ));
        active.end_time = Set(Some(cutoff));
        if let Err(e) = active.update(db).await {
            log::warn!("[scheduler] 标记中断日志失败: id={}, err={}", log.id, e);
        }
        interrupted_job_ids.push(log.job_id);
        let key = log.job_code.unwrap_or_default();
        *map.entry(key).or_insert(0) += 1;
    }

    // 同步更新 job 表 last_run_status=3（中断），让任务列表能看到中断状态
    for job_id in &interrupted_job_ids {
        if let Ok(Some(job)) = scheduler_job::Entity::find_by_id(*job_id).one(db).await {
            let mut active: scheduler_job::ActiveModel = job.into();
            active.last_run_status = Set(Some(3));
            active.last_run_result = Set(Some("任务执行过程中进程退出/调度器重载，已被系统标记为中断".to_string()));
            active.update_time = Set(Some(cutoff));
            let _ = active.update(db).await;
        }
    }

    Ok(map.into_iter().collect())
}

/// 漏跑检测：返回存在"错过执行"的启用任务
/// 判定规则：按 cron 从上次执行时间（或创建时间）计算的最近一次应执行时刻已早于当前时间 → 漏跑
pub async fn detect_missed_runs(db: &DatabaseConnection) -> Result<Vec<scheduler_job::Model>, String> {
    let jobs = scheduler_job::Entity::find()
        .filter(scheduler_job::Column::Enabled.eq(1))
        .filter(scheduler_job::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let now = Utc::now();
    let mut missed = Vec::new();
    for job in jobs {
        // 计算"应执行但可能已错过"的基准时刻
        let from = job
            .last_run_time
            .map(|t| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(t, chrono::Utc))
            .unwrap_or_else(|| {
                // 从未执行：以创建时间为基准
                job.create_time
                    .map(|t| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(t, chrono::Utc))
                    .unwrap_or(now)
            });

        if from >= now {
            continue;
        }

        if let Some(next) = compute_next_run_time_from(&job.cron_expression, from) {
            // 最近一次应执行时刻已到且未执行 → 漏跑
            if next <= now {
                missed.push(job);
            }
        }
    }
    Ok(missed)
}
