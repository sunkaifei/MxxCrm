//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 数据备份与恢复管理接口
//!
//! 页面：系统管理 → 数据备份与恢复（system/backup/index.vue）
//! 权限：system:backup:list / save(立即备份) / update(设置) / delete(删除) / restore(恢复) / export(下载)
//! 注：restore 为业务扩展动词（数据恢复，破坏性操作，标准动词无法表达）

use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::service::backup_service;

#[derive(Deserialize)]
pub struct BackupListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupConfigUpdateDTO {
    pub keep_days: Option<i64>,
    pub cron_expression: Option<String>,
    pub enabled: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRestoreDTO {
    pub id: i64,
    /// 确认码，必须为 RESTORE
    pub confirm: String,
}

/// 备份记录列表（含恢复记录）
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<BackupListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let page = q.page.unwrap_or(1) as u32;
    match backup_service::get_list(db, q.page.unwrap_or(1), q.page_size.unwrap_or(20)).await {
        Ok((items, total)) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success_with_page(items, "local", page, total as u32)),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 备份设置（任务状态/保留天数/目录）
pub async fn config(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match backup_service::get_config(db).await {
        Ok(vo) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(vo, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 保存备份设置：保留天数（系统配置表）+ cron/启用（调度任务，保存后动态重载）
pub async fn update_config(
    state: web::Data<AppState>,
    form_data: web::Json<BackupConfigUpdateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    if let Some(keep_days) = dto.keep_days {
        if !(1..=365).contains(&keep_days) {
            return HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "保留天数须在 1-365 之间", "local"));
        }
        if let Err(e) = backup_service::save_keep_days(db, keep_days).await {
            return HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &format!("保存保留天数失败: {}", e), "local"));
        }
    }

    if dto.cron_expression.is_some() || dto.enabled.is_some() {
        let cfg = match backup_service::get_config(db).await {
            Ok(c) => c,
            Err(e) => return HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &e, "local")),
        };
        let upd = crate::modules::system::service::scheduler_service::SchedulerJobUpdateDTO {
            id: cfg.job_id,
            cron_expression: dto.cron_expression,
            job_name: None,
            description: None,
            handler_params: None,
            enabled: dto.enabled,
        };
        if let Err(e) = crate::modules::system::service::scheduler_service::update_job(db, upd).await {
            return HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &format!("保存任务设置失败: {}", e), "local"));
        }
    }

    HttpResponse::Ok().content_type(MPACK)
        .body(MetaResp::success("保存成功".to_string(), "local"))
}

/// 立即备份（手动触发 db_backup 任务，复用调度器两阶段日志/操作人记录）
pub async fn trigger(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let db = &state.db;
    let (operator_id, username) = get_current_user(&req);
    let operator_name: &str = if username.is_empty() { "管理员" } else { &username };

    let cfg = match backup_service::get_config(db).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    };
    let dto = crate::modules::system::service::scheduler_service::SchedulerTriggerDTO { id: cfg.job_id };
    match crate::modules::system::service::scheduler_service::trigger_job(db, dto, operator_id, operator_name).await {
        Ok(msg) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(msg, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 删除备份（文件+记录）
pub async fn delete(
    state: web::Data<AppState>,
    query: web::Query<crate::core::web::entity::common::InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "记录ID不能为空", "local"));
    }
    match backup_service::delete_backup(db, item.id.unwrap()).await {
        Ok(msg) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(msg, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 数据恢复（危险操作：确认码 RESTORE + 活跃连接检查，详见 service 注释）
pub async fn restore(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<BackupRestoreDTO>,
) -> HttpResponse {
    let db = &state.db;
    let (operator_id, _) = get_current_user(&req);
    log::warn!(
        "[数据库恢复] 管理员(id={}) 发起恢复操作，目标备份记录 id={}",
        operator_id,
        form_data.0.id
    );
    match backup_service::restore_backup(db, form_data.0.id, &form_data.0.confirm).await {
        Ok(msg) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(msg, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 下载备份文件（二进制流，非 msgpack）
pub async fn download(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> HttpResponse {
    let db = &state.db;
    match backup_service::download_backup(db, path.into_inner()).await {
        Ok((data, file_name, mime)) => {
            let safe_name = file_name.replace('"', "_");
            HttpResponse::Ok()
                .content_type(mime.as_str())
                .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", safe_name)))
                .append_header(("Content-Length", data.len()))
                .body(data)
        }
        Err(e) => HttpResponse::NotFound()
            .content_type("application/json")
            .body(format!(r#"{{"code":404,"msg":"{}"}}"#, e.replace('"', "'"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/backup")
            .route("/list", web::get().to(list).wrap(require_permission("system:backup:list")))
            .route("/config", web::get().to(config).wrap(require_permission("system:backup:list")))
            .route("/config/update", web::post().to(update_config).wrap(require_permission("system:backup:update")))
            .route("/trigger", web::post().to(trigger).wrap(require_permission("system:backup:save")))
            .route("/delete", web::delete().to(delete).wrap(require_permission("system:backup:delete")))
            .route("/restore", web::post().to(restore).wrap(require_permission("system:backup:restore")))
            .route("/download/{id}", web::get().to(download).wrap(require_permission("system:backup:export"))),
    );
}
