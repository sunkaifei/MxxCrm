//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::response::{MetaResp, MPACK};

use crate::modules::crm::model::work_log::WorkLogCreateDTO;
use crate::modules::crm::service::work_log_service;

/// GET /work-log/today - 今日工作日志（按 create_time 降序）
pub async fn work_log_today(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    if user_id <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }
    match work_log_service::find_today_list(db, user_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

/// GET /work-log/week-workload - 本周工作负载（按 work_date 升序）
pub async fn work_log_week_workload(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    if user_id <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }
    match work_log_service::find_week_workload(db, user_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

/// GET /work-log/today-summary - 今日待办汇总（已处理数 + 剩余数 + 完成率）
pub async fn work_log_today_summary(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    if user_id <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }
    match work_log_service::find_today_summary(db, user_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

/// POST /work-log/create - 内部接口，写入工作日志（需要登录）
pub async fn work_log_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<WorkLogCreateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    if user_id <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }
    let mut dto = payload.0;
    // 强制以登录用户为准，防止越权写入他人日志
    dto.user_id = user_id;
    match work_log_service::insert(db, &dto).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/work-log")
            .route("/today", web::get().to(work_log_today))
            .route("/week-workload", web::get().to(work_log_week_workload))
            .route("/today-summary", web::get().to(work_log_today_summary))
            .route("/create", web::post().to(work_log_create)),
    );
}
