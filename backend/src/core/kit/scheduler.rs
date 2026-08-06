//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 定时任务调度器（动态加载版）
//! 从数据库 mxx_system_scheduler_job 加载启用的任务
//!
//! V7-2: 引入 SCHEDULER_HANDLE 全局句柄，支持 update_job/toggle_job 后动态重载
//! V7-3: 引入 SCHEDULER_REGISTRY 处理器注册器，新增处理器无需改 match
//! V7-4: handler_params 透传到处理器

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Datelike, Utc};
use sea_orm::{prelude::Json, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tokio::sync::{Mutex as AsyncMutex, OnceCell};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

use crate::modules::system::entity::scheduler_job;
use crate::modules::system::service::scheduler_service;

/// V7-2: 调度器全局句柄（用于动态重载）
/// 注：tokio-cron-scheduler 暂无按 job_id remove 的 API，采用"整调度器重启"策略
/// 任务数少（通常 < 10），重启开销可接受
pub static SCHEDULER_HANDLE: OnceCell<Arc<AsyncMutex<Option<JobScheduler>>>> = OnceCell::const_new();

/// V7-3: 处理器注册器
/// 新增处理器只需调用 SCHEDULER_REGISTRY.register(...)，无需改 scheduler.rs 的 match
pub type HandlerFn = Arc<
    dyn Fn(DatabaseConnection, Option<Json>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>
        + Send
        + Sync,
>;

pub struct SchedulerRegistry {
    handlers: AsyncMutex<HashMap<String, HandlerFn>>,
}

impl SchedulerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: AsyncMutex::new(HashMap::new()),
        }
    }

    pub async fn register(&self, name: &str, handler: HandlerFn) {
        self.handlers.lock().await.insert(name.to_string(), handler);
    }

    pub async fn get(&self, name: &str) -> Option<HandlerFn> {
        self.handlers.lock().await.get(name).cloned()
    }
}

pub static SCHEDULER_REGISTRY: OnceCell<SchedulerRegistry> = OnceCell::const_new();

/// 初始化处理器注册器（注册内置处理器）
async fn init_registry(registry: &SchedulerRegistry) {
    // 注册工资核算处理器
    registry
        .register(
            "salary_calculate",
            Arc::new(|db: DatabaseConnection, params: Option<Json>| {
                Box::pin(async move {
                    let (year, month) = parse_year_month_from_params(&params);
                    let count = crate::modules::finance::service::salary_service::calculate(
                        &db, year, month, 1, 0, "系统定时任务",
                    )
                    .await?;
                    Ok(format!(
                        "核算完成：{}年{}月，生成 {} 条工资记录",
                        year, month, count
                    ))
                })
            }),
        )
        .await;

    // 注册文章定时发布处理器
    registry
        .register(
            "article_publish",
            Arc::new(|db: DatabaseConnection, _params: Option<Json>| {
                Box::pin(async move {
                    let count = crate::modules::articles::model::article::ArticleModel::publish_scheduled(&db)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("定时发布完成：本次发布 {} 篇文章", count))
                })
            }),
        )
        .await;

    // G-2.3: 注册静态化生成处理器
    // 将全站动态页面渲染为 HTML 文件，Nginx 可直接读文件返回
    registry
        .register(
            "static_generate",
            Arc::new(|db: DatabaseConnection, _params: Option<Json>| {
                Box::pin(async move {
                    let (cat, art) = crate::modules::website::service::static_generate_service::generate_all(&db)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("静态化完成：栏目 {} / 文章 {}", cat, art))
                })
            }),
        )
        .await;

    // G-2.7: 注册内容采集处理器
    // 根据 mxx_website_collect_rule 表中启用的规则，定时采集外部内容
    registry
        .register(
            "content_collect",
            Arc::new(|db: DatabaseConnection, _params: Option<Json>| {
                Box::pin(async move {
                    let count = crate::modules::website::service::content_collector_service::collect_all(&db)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("内容采集完成：本次采集 {} 篇文章", count))
                })
            }),
        )
        .await;

    // 注册库存快照生成处理器（每日凌晨生成前一天的库存快照）
    registry
        .register(
            "stock_snapshot_generate",
            Arc::new(|db: DatabaseConnection, _params: Option<Json>| {
                Box::pin(async move {
                    let count = crate::modules::inventory::service::stock_snapshot_service::generate_daily_snapshot(&db)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("库存快照生成完成：本次生成 {} 条快照记录", count))
                })
            }),
        )
        .await;

    // 注册低库存采购建议扫描处理器
    registry
        .register(
            "low_stock_suggestion",
            Arc::new(|db: DatabaseConnection, _params: Option<Json>| {
                Box::pin(async move {
                    let suggestions = crate::modules::inventory::service::inventory_suggestion_service::scan_low_stock(&db)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!("低库存扫描完成：本次生成 {} 条采购建议", suggestions.len()))
                })
            }),
        )
        .await;
}

/// V7-4: 从 handler_params 解析 year/month，缺失时回退为"上月"
fn parse_year_month_from_params(params: &Option<Json>) -> (i32, i32) {
    let now = Utc::now();
    let default_year = if now.month() == 1 { now.year() - 1 } else { now.year() };
    let default_month = if now.month() == 1 { 12 } else { now.month() - 1 } as i32;

    if let Some(Json::Object(obj)) = params {
        let year = obj
            .get("year")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .unwrap_or(default_year);
        let month = obj
            .get("month")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .unwrap_or(default_month);
        (year, month)
    } else {
        (default_year, default_month)
    }
}

/// 启动定时任务调度器
pub async fn start_scheduler(db: DatabaseConnection) -> Result<JobScheduler, JobSchedulerError> {
    // 初始化注册器
    let registry = SCHEDULER_REGISTRY.get_or_init(|| async { SchedulerRegistry::new() }).await;
    init_registry(registry).await;

    let sched = JobScheduler::new().await?;

    // 从数据库加载所有启用的任务
    let jobs = scheduler_job::Entity::find()
        .filter(scheduler_job::Column::Enabled.eq(1))
        .filter(scheduler_job::Column::Deleted.eq(0))
        .all(&db)
        .await
        .unwrap_or_default();

    let mut loaded = 0;
    for job in jobs {
        match add_job_to_scheduler(&sched, &db, &job).await {
            Ok(_) => {
                loaded += 1;
                log::info!("[调度器] 已加载任务: {} ({})", job.job_name, job.job_code);
            }
            Err(e) => {
                log::error!("[调度器] 加载任务 {} 失败: {}", job.job_code, e);
            }
        }
    }

    sched.start().await?;
    log::info!("[调度器] 调度器已启动，成功加载 {} 个任务", loaded);

    // V7-2: 保存到全局句柄
    let handle = SCHEDULER_HANDLE
        .get_or_init(|| async { Arc::new(AsyncMutex::new(None)) })
        .await;
    *handle.lock().await = Some(sched.clone());

    Ok(sched)
}

/// 将数据库任务添加到调度器
async fn add_job_to_scheduler(
    sched: &JobScheduler,
    db: &DatabaseConnection,
    job: &scheduler_job::Model,
) -> Result<(), JobSchedulerError> {
    let job_id = job.id;
    let job_code = job.job_code.clone();
    let cron = job.cron_expression.clone();
    let db_clone = db.clone();
    // P2-6: 预读取重试配置（避免每次触发都重新读取，配置变更通过 reload_scheduler 生效）
    let max_retries = job.max_retries.unwrap_or(3).max(0);
    let retry_base = job.retry_interval_base.unwrap_or(60).max(1);

    let job_instance = Job::new_async(cron.as_str(), move |_uuid, _l| {
        let db = db_clone.clone();
        let job_code = job_code.clone();
        let job_id = job_id;
        let max_retries = max_retries;
        let retry_base = retry_base;
        Box::pin(async move {
            log::info!("[定时任务] 开始执行: {}", job_code);
            let start = std::time::Instant::now();

            // 查询最新的任务配置
            let current_job = match scheduler_job::Entity::find_by_id(job_id).one(&db).await {
                Ok(Some(j)) => j,
                Ok(None) => {
                    log::error!("[定时任务] 任务不存在: id={}", job_id);
                    return;
                }
                Err(e) => {
                    log::error!("[定时任务] 查询任务失败: id={}, err={}", job_id, e);
                    return;
                }
            };

            // P2-6: 带指数退避重试的执行
            // 策略：首次失败后按 base * 2^attempt 秒间隔重试，最多 max_retries 次
            // 全部失败后记录最终错误并发送告警
            let mut attempt = 0i32;
            let mut last_error: Option<String> = None;
            let mut final_msg: Option<String> = None;
            let mut success = false;

            loop {
                let result = execute_handler_by_code(
                    &current_job.handler,
                    &current_job.handler_params,
                    &db,
                )
                .await;

                match result {
                    Ok(msg) => {
                        final_msg = Some(msg);
                        success = true;
                        break;
                    }
                    Err(e) => {
                        last_error = Some(e.clone());
                        if attempt < max_retries {
                            // 指数退避：base * 2^attempt 秒
                            let delay_secs = (retry_base as u64).saturating_mul(2u64.saturating_pow(attempt as u32));
                            log::warn!(
                                "[定时任务] {} 第 {} 次执行失败，{} 秒后重试：{}",
                                job_code,
                                attempt + 1,
                                delay_secs,
                                e
                            );
                            // P2-6: 中间失败仅记日志，不调用 record_scheduled_execution
                            // 避免覆盖 last_run_status，最终状态由循环外统一记录
                            tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                            attempt += 1;
                        } else {
                            break;
                        }
                    }
                }
            }

            let elapsed = start.elapsed().as_millis() as i64;
            let (status, result_msg, error_msg) = if success {
                (1i32, final_msg, None::<String>)
            } else {
                (0i32, None, last_error.clone())
            };

            // P2-6: 先更新 last_retry_count（在 record_scheduled_execution 之前，避免 update_time 覆盖）
            if let Err(e) = scheduler_service::update_job_retry_count(&db, job_id, attempt).await {
                log::warn!("[定时任务] {} 更新重试次数失败：{}", job_code, e);
            }

            // 记录最终执行日志
            if let Err(e) = scheduler_service::record_scheduled_execution(
                &db,
                job_id,
                &job_code,
                status,
                result_msg.as_deref(),
                error_msg.as_deref(),
                elapsed,
            )
            .await
            {
                log::error!("[定时任务] 记录日志失败: {}", e);
            }

            if status == 1 {
                log::info!("[定时任务] {} 执行成功（重试 {} 次）：{}", job_code, attempt, result_msg.as_deref().unwrap_or_default());
            } else {
                log::error!(
                    "[定时任务] {} 执行失败（已重试 {} 次）：{}",
                    job_code,
                    attempt,
                    error_msg.as_deref().unwrap_or_default()
                );
                // P2-6: 重试耗尽后发送告警通知（站内信）
                if max_retries > 0 && attempt >= max_retries {
                    let alert_msg = format!(
                        "定时任务 [{}] 在 {} 重试 {} 次后仍失败：{}",
                        job_code,
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                        attempt,
                        error_msg.as_deref().unwrap_or_default()
                    );
                    send_job_failure_alert(&db, &job_code, &alert_msg).await;
                }
            }
        })
    })?;
    sched.add(job_instance).await?;
    Ok(())
}

/// P2-6: 定时任务重试耗尽后的告警通知
/// 通过系统通知（notice）发布告警，目标为全体管理员
async fn send_job_failure_alert(db: &DatabaseConnection, job_code: &str, message: &str) {
    use crate::modules::system::model::notice::{NoticeSaveDTO, NoticeSaveRequest};
    use crate::modules::system::service::notice_service;

    let now = chrono::Utc::now().naive_utc();
    let req = NoticeSaveRequest {
        title: Some(format!("定时任务 [{}] 执行失败告警", job_code)),
        content: Some(message.to_string()),
        r#type: Some(2), // 2=系统通知（按现有 notice_type 字典约定）
        level: Some("high".to_string()),
        target_type: Some(1), // 1=全体
        target_user_ids: None,
        publisher_id: Some(0), // 0=系统
        publish_status: Some(1), // 1=已发布
        publish_time: Some(now),
        revoke_time: None,
        create_by: Some(0),
    };
    let dto: NoticeSaveDTO = req.into();

    // 失败不影响主流程，仅记录日志
    if let Err(e) = notice_service::insert(db, &dto).await {
        log::error!(
            "[定时任务] {} 告警通知发送失败：{}；原始消息：{}",
            job_code,
            e,
            message
        );
    }
}

/// 根据处理器代码执行
/// V7-3: 优先从 SCHEDULER_REGISTRY 查找；未注册时回退到内置 match（向后兼容）
/// V7-4: 透传 handler_params 到处理器，支持参数化触发
async fn execute_handler_by_code(
    handler: &str,
    params: &Option<Json>,
    db: &DatabaseConnection,
) -> Result<String, String> {
    // V7-3: 优先从注册器查找
    if let Some(registry) = SCHEDULER_REGISTRY.get() {
        if let Some(h) = registry.get(handler).await {
            return h(db.clone(), params.clone()).await;
        }
    }

    // 回退到内置处理器（向后兼容）
    match handler {
        "salary_calculate" => {
            let (year, month) = parse_year_month_from_params(params);
            let count = crate::modules::finance::service::salary_service::calculate(
                db, year, month, 1, 0, "系统定时任务",
            )
            .await?;
            Ok(format!("核算完成：{}年{}月，生成 {} 条工资记录", year, month, count))
        }
        _ => Err(format!("未知的处理器: {}", handler)),
    }
}

/// V7-2: 重载调度器（整调度器重启策略）
/// 在 update_job / toggle_job 完成后调用，使新的 cron 表达式立即生效
pub async fn reload_scheduler(db: DatabaseConnection) -> Result<(), String> {
    let handle = SCHEDULER_HANDLE
        .get_or_init(|| async { Arc::new(AsyncMutex::new(None)) })
        .await;

    let mut guard = handle.lock().await;

    // 停止旧调度器
    if let Some(mut old_sched) = guard.take() {
        if let Err(e) = old_sched.shutdown().await {
            log::warn!("[调度器] 停止旧调度器失败: {}", e);
        }
    }

    // 启动新调度器
    match start_scheduler_inner(db).await {
        Ok(new_sched) => {
            *guard = Some(new_sched);
            log::info!("[调度器] 调度器已重载");
            Ok(())
        }
        Err(e) => {
            log::error!("[调度器] 重载失败: {}", e);
            Err(format!("调度器重载失败: {}", e))
        }
    }
}

/// 内部启动函数（不更新全局句柄，避免递归）
async fn start_scheduler_inner(db: DatabaseConnection) -> Result<JobScheduler, JobSchedulerError> {
    let registry = SCHEDULER_REGISTRY
        .get_or_init(|| async { SchedulerRegistry::new() })
        .await;
    init_registry(registry).await;

    let sched = JobScheduler::new().await?;

    let jobs = scheduler_job::Entity::find()
        .filter(scheduler_job::Column::Enabled.eq(1))
        .filter(scheduler_job::Column::Deleted.eq(0))
        .all(&db)
        .await
        .unwrap_or_default();

    let mut loaded = 0;
    for job in jobs {
        match add_job_to_scheduler(&sched, &db, &job).await {
            Ok(_) => {
                loaded += 1;
                log::info!("[调度器] 已加载任务: {} ({})", job.job_name, job.job_code);
            }
            Err(e) => {
                log::error!("[调度器] 加载任务 {} 失败: {}", job.job_code, e);
            }
        }
    }

    sched.start().await?;
    log::info!("[调度器] 调度器已启动，成功加载 {} 个任务", loaded);

    Ok(sched)
}
