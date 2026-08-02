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

/// D-4: 根据 cron 表达式计算下次执行时间（简化实现，支持 5 字段标准 cron）
/// 仅计算最近一次的下次执行时间，用于展示
fn compute_next_run_time(cron_expr: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let parts: Vec<&str> = cron_expr.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }

    let now = chrono::Utc::now();
    // 简化策略：从当前时间开始，逐分钟向后扫描，最多扫描 7*24*60=10080 分钟（7天）
    // 找到第一个匹配 cron 的时刻
    use chrono::Timelike;
    let mut candidate = now.with_second(0).unwrap_or(now).with_nanosecond(0).unwrap_or(now) + chrono::Duration::minutes(1);

    for _ in 0..10080 {
        if cron_matches(&parts, &candidate) {
            return Some(candidate);
        }
        candidate = candidate + chrono::Duration::minutes(1);
    }
    None
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

    let start = std::time::Instant::now();
    let result = execute_handler(&job.handler, &job.handler_params, db).await;
    let elapsed = start.elapsed().as_millis() as i64;

    let (status, result_msg, error_msg) = match result {
        Ok(msg) => (1i32, Some(msg), None),
        Err(e) => (0, None, Some(e)),
    };

    // 写日志
    let now = Utc::now().naive_utc();
    let log = scheduler_log::ActiveModel {
        job_id: Set(job.id),
        job_code: Set(Some(job.job_code.clone())),
        trigger_type: Set(Some(1)), // 手动
        status: Set(Some(status)),
        result_message: Set(result_msg.clone()),
        error_message: Set(error_msg.clone()),
        elapsed_ms: Set(Some(elapsed)),
        operator_id: Set(Some(operator_id)),
        operator_name: Set(Some(operator_name.to_string())),
        start_time: Set(Some(now)),
        end_time: Set(Some(now)),
        ..Default::default()
    };
    log.insert(db).await.map_err(|e| e.to_string())?;

    // 更新任务最后执行信息
    let mut active: scheduler_job::ActiveModel = job.into();
    active.last_run_time = Set(Some(now));
    active.last_run_status = Set(Some(status));
    active.last_run_result = Set(if status == 1 { result_msg.clone() } else { error_msg.clone() });
    active.update_time = Set(Some(now));
    active.update(db).await.map_err(|e| e.to_string())?;

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

/// 记录定时执行结果（供 scheduler.rs 调用）
/// P2-6: 新增 retry_count 参数，用于更新 last_retry_count 字段
pub async fn record_scheduled_execution(
    db: &DatabaseConnection,
    job_id: i64,
    job_code: &str,
    status: i32,
    result_msg: Option<&str>,
    error_msg: Option<&str>,
    elapsed_ms: i64,
) -> Result<(), String> {
    let now = Utc::now().naive_utc();
    let log = scheduler_log::ActiveModel {
        job_id: Set(job_id),
        job_code: Set(Some(job_code.to_string())),
        trigger_type: Set(Some(0)), // 定时
        status: Set(Some(status)),
        result_message: Set(result_msg.map(|s| s.to_string())),
        error_message: Set(error_msg.map(|s| s.to_string())),
        elapsed_ms: Set(Some(elapsed_ms)),
        operator_id: Set(Some(0)),
        operator_name: Set(Some("系统".to_string())),
        start_time: Set(Some(now)),
        end_time: Set(Some(now)),
        ..Default::default()
    };
    log.insert(db).await.map_err(|e| e.to_string())?;

    // 更新任务最后执行信息
    if let Some(job) = scheduler_job::Entity::find_by_id(job_id).one(db).await.map_err(|e| e.to_string())? {
        let mut active: scheduler_job::ActiveModel = job.into();
        active.last_run_time = Set(Some(now));
        active.last_run_status = Set(Some(status));
        active.last_run_result = Set(if status == 1 { result_msg.map(|s| s.to_string()) } else { error_msg.map(|s| s.to_string()) });
        active.update_time = Set(Some(now));
        active.update(db).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// P2-6: 更新任务的 last_retry_count 字段
/// 在 record_scheduled_execution 调用前/后单独调用，避免重复读写 job 表
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
