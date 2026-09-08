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
use crate::core::web::permission_guard::require_permission;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::service::scheduler_service;

pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<scheduler_service::SchedulerJobQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let page = q.page.unwrap_or(1) as u32;
    match scheduler_service::get_job_list(db, q).await {
        Ok((list, total)) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success_with_page(list, "local", page, total as u32)),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn detail(
    state: web::Data<AppState>,
    query: web::Query<crate::core::web::entity::common::InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "任务ID不能为空", "local"));
    }
    match scheduler_service::get_job_detail(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn update(
    state: web::Data<AppState>,
    form_data: web::Json<scheduler_service::SchedulerJobUpdateDTO>,
) -> HttpResponse {
    let db = &state.db;
    match scheduler_service::update_job(db, form_data.0).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success("更新成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn toggle(
    state: web::Data<AppState>,
    form_data: web::Json<scheduler_service::SchedulerToggleDTO>,
) -> HttpResponse {
    let db = &state.db;
    match scheduler_service::toggle_job(db, form_data.0).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success("操作成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn trigger(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<scheduler_service::SchedulerTriggerDTO>,
) -> HttpResponse {
    let db = &state.db;
    let (operator_id, username) = get_current_user(&req);
    let operator_name: &str = if username.is_empty() { "管理员" } else { &username };

    match scheduler_service::trigger_job(db, form_data.0, operator_id, operator_name).await {
        Ok(msg) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(msg, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn log_list(
    state: web::Data<AppState>,
    query: web::Query<scheduler_service::SchedulerLogQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let page = q.page.unwrap_or(1) as u32;
    match scheduler_service::get_log_list(db, q).await {
        Ok((list, total)) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success_with_page(list, "local", page, total as u32)),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/scheduler")
            .route("/list", web::get().to(list).wrap(require_permission("system:scheduler:list")))
            .route("/detail", web::get().to(detail).wrap(require_permission("system:scheduler:list")))
            .route("/update", web::post().to(update).wrap(require_permission("system:scheduler:manage")))
            .route("/toggle", web::post().to(toggle).wrap(require_permission("system:scheduler:manage")))
            .route("/trigger", web::post().to(trigger).wrap(require_permission("system:scheduler:manage")))
            .route("/log/list", web::get().to(log_list).wrap(require_permission("system:scheduler:list"))),
    );
}
