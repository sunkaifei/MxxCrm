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
//!
//! 数据初始化（清除业务数据）：system:backup:clean
//!   仅超级管理员（user_type=1）可执行，需登录密码 + 一次性确认码双验证，执行前强制自动备份

use actix_web::{web, HttpRequest, HttpResponse};
use bcrypt::verify;
use serde::Deserialize;

use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::admin::AdminModel;
use crate::modules::system::service::{audit_service, backup_service, data_clean_service};

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
    /// 当前超管登录密码（bcrypt 校验）
    pub password: String,
    /// 邮箱验证码（一次性，5 分钟有效）
    pub otp: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupDeleteDTO {
    pub id: i64,
    /// 当前超管登录密码（bcrypt 校验）
    pub password: String,
    /// 邮箱验证码（一次性，5 分钟有效）
    pub otp: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupDownloadDTO {
    pub id: i64,
    /// 当前超管登录密码（bcrypt 校验）
    pub password: String,
    /// 邮箱验证码（一次性，5 分钟有效）
    pub otp: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpSendDTO {
    /// 验证码用途：delete（删除备份）/ restore（数据还原）/ download（下载备份）
    pub action: String,
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

/// 发送邮箱验证码（仅超管，60 秒限流），返回脱敏邮箱
pub async fn otp_send(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<OtpSendDTO>,
) -> HttpResponse {
    let db = &state.db;
    let (operator_id, username) = get_current_user(&req);
    if operator_id <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "登录状态已失效，请重新登录", "local"));
    }
    let action = form_data.0.action.clone();
    match crate::modules::system::service::otp_service::send(db, operator_id, &action).await {
        Ok(masked_email) => {
            log::warn!("[安全验证码] 管理员(id={}, {}) 申请「{}」操作验证码，已发送至 {}", operator_id, username, action, masked_email);
            HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::success(masked_email, "local"))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 高危操作统一校验：仅超管 + 登录密码（bcrypt）+ 邮箱验证码（一次性）
async fn verify_danger_action(
    db: &sea_orm::DatabaseConnection,
    req: &HttpRequest,
    password: &str,
    action: &str,
    otp: &str,
) -> Result<i64, String> {
    let (operator_id, username) = get_current_user(req);
    if operator_id <= 0 {
        return Err("登录状态已失效，请重新登录".to_string());
    }
    if password.trim().is_empty() {
        return Err("请输入当前超管的登录密码".to_string());
    }
    if otp.trim().is_empty() {
        return Err("请输入邮箱验证码".to_string());
    }
    // 超管身份 + 登录密码
    match check_super_admin_password(db, operator_id, password).await {
        Ok(true) => {}
        Ok(false) => {
            log::error!("[高危操作] 管理员(id={}) 密码校验失败，已拒绝「{}」", operator_id, action);
            return Err("密码校验失败或非超级管理员，已拒绝执行".to_string());
        }
        Err(e) => return Err(e),
    }
    // 邮箱验证码（一次性消费）
    crate::modules::system::service::otp_service::verify(operator_id, action, otp)
        .map_err(|e| format!("验证码校验失败: {}", e))?;
    log::warn!("[高危操作] 管理员(id={}, {}) 通过全部验证，执行「{}」", operator_id, username, action);
    Ok(operator_id)
}

/// 删除备份（危险操作：仅超管 + 登录密码 + 邮箱验证码；禁删最后一个成功备份）
pub async fn delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<BackupDeleteDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;
    let operator_id = match verify_danger_action(db, &req, &dto.password, "delete", &dto.otp).await {
        Ok(id) => id,
        Err(e) => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    };
    match backup_service::delete_backup(db, dto.id).await {
        Ok(msg) => {
            audit_service::record(
                db, &req, "system", "backup_delete", "database", operator_id,
                format!("删除备份记录 id={}：{}", dto.id, msg),
                None, None,
            ).await;
            HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::success(msg, "local"))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 数据恢复（危险操作：仅超管 + 登录密码 + 邮箱验证码；还原前自动备份当前数据）
pub async fn restore(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<BackupRestoreDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;
    let operator_id = match verify_danger_action(db, &req, &dto.password, "restore", &dto.otp).await {
        Ok(id) => id,
        Err(e) => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    };
    log::warn!(
        "[数据库恢复] 管理员(id={}) 通过验证，目标备份记录 id={}",
        operator_id,
        dto.id
    );
    match backup_service::restore_backup(db, dto.id).await {
        Ok(msg) => {
            audit_service::record(
                db, &req, "system", "backup_restore", "database", operator_id,
                format!("数据还原：使用备份记录 id={}，{}", dto.id, msg),
                None, None,
            ).await;
            HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::success(msg, "local"))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 下载备份文件（危险操作：仅超管 + 登录密码 + 邮箱验证码，二进制流，非 msgpack）
pub async fn download(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<BackupDownloadDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;
    let operator_id = match verify_danger_action(db, &req, &dto.password, "download", &dto.otp).await {
        Ok(id) => id,
        Err(e) => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    };
    match backup_service::download_backup(db, dto.id).await {
        Ok((data, file_name, mime)) => {
            audit_service::record(
                db, &req, "system", "backup_download", "database", operator_id,
                format!("下载备份文件：{}（{} 字节）", file_name, data.len()),
                None, None,
            ).await;
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

/// 数据初始化执行请求体
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanExecuteDTO {
    /// 当前超管登录密码（bcrypt 校验）
    pub password: String,
    /// 一次性确认码（预览接口返回，5 分钟有效）
    pub confirm_code: String,
}

/// 查询超管并校验密码
async fn check_super_admin_password(
    db: &sea_orm::DatabaseConnection,
    admin_id: i64,
    password: &str,
) -> Result<bool, String> {
    let admin = AdminModel::find_by_id(db, &Some(admin_id))
        .await
        .map_err(|e| format!("查询用户信息失败: {}", e))?
        .ok_or_else(|| "当前登录用户不存在".to_string())?;
    // 仅超级管理员（user_type=1）可执行数据初始化
    if admin.user_type != Some(1) {
        return Ok(false);
    }
    // 登录密码校验
    let stored = admin.password.clone().unwrap_or_default();
    let valid = verify(password, &stored).unwrap_or(false);
    Ok(valid)
}

/// 数据初始化预览：获取待清业务表清单 + 行数 + 一次性确认码（危险操作第一步）
pub async fn clean_preview(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let db = &state.db;
    let (operator_id, username) = get_current_user(&req);

    // 仅超管可获取预览
    let admin = match AdminModel::find_by_id(db, &Some(operator_id)).await {
        Ok(Some(a)) => a,
        _ => {
            return HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "当前登录用户不存在", "local"));
        }
    };
    if admin.user_type != Some(1) {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "仅超级管理员可执行数据初始化", "local"));
    }
    log::warn!("[数据初始化] 管理员(id={}, {}) 请求清除预览", operator_id, username);

    match data_clean_service::preview(db, operator_id).await {
        Ok(vo) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(vo, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 数据初始化执行（危险操作：超管 + 登录密码 + 一次性确认码 三重验证，执行前强制自动备份）
pub async fn clean_execute(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<CleanExecuteDTO>,
) -> HttpResponse {
    let db = &state.db;
    let (operator_id, username) = get_current_user(&req);

    if operator_id <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "登录状态已失效，请重新登录", "local"));
    }
    if form_data.0.password.trim().is_empty() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "请输入当前超管的登录密码", "local"));
    }
    if form_data.0.confirm_code.trim().is_empty() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "请输入确认码", "local"));
    }

    // 三重验证：超管身份 + 登录密码 + 确认码（确认码在 service 内校验）
    match check_super_admin_password(db, operator_id, &form_data.0.password).await {
        Ok(true) => {}
        Ok(false) => {
            log::error!("[数据初始化] 管理员(id={}) 密码校验失败，已拒绝清除", operator_id);
            return HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "密码校验失败或非超级管理员，已拒绝执行", "local"));
        }
        Err(e) => {
            return HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &e, "local"));
        }
    }

    log::warn!(
        "[数据初始化] 管理员(id={}, {}) 通过密码验证，开始清除全部业务数据",
        operator_id,
        username
    );

    let confirm_code = form_data.0.confirm_code.clone();
    match data_clean_service::execute(db, operator_id, &confirm_code).await {
        Ok(vo) => {
            // 审计记录（append-only，成功后才写）
            audit_service::record(
                db,
                &req,
                "system",
                "data_clean",
                "database",
                operator_id,
                format!(
                    "数据初始化：清除 {} 张业务表，清空商品图 {} 个文件，前置备份已生成",
                    vo.cleared_tables, vo.removed_files
                ),
                None,
                None,
            )
            .await;
            HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::success(vo, "local"))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/backup")
            .route("/list", web::get().to(list).wrap(require_permission("system:backup:list")))
            .route("/config", web::get().to(config).wrap(require_permission("system:backup:list")))
            .route("/config/update", web::post().to(update_config).wrap(require_permission("system:backup:update")))
            .route("/trigger", web::post().to(trigger).wrap(require_permission("system:backup:save")))
            .route("/otp/send", web::post().to(otp_send).wrap(require_permission("system:backup:list")))
            .route("/delete", web::post().to(delete).wrap(require_permission("system:backup:delete")))
            .route("/restore", web::post().to(restore).wrap(require_permission("system:backup:restore")))
            .route("/download", web::post().to(download).wrap(require_permission("system:backup:export")))
            .route("/clean/preview", web::get().to(clean_preview).wrap(require_permission("system:backup:clean")))
            .route("/clean/execute", web::post().to(clean_execute).wrap(require_permission("system:backup:clean"))),
    );
}
