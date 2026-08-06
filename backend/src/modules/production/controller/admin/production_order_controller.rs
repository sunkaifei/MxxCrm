//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::production::model::production_order::{ProductionOrderListQuery, ProductionOrderSaveRequest};
use crate::modules::production::service::production_order_service;
use crate::modules::production::service::production_service;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::entity::common::BathDeleteIdRequest;

pub async fn production_order_save(state: web::Data<AppState>, form_data: web::Json<ProductionOrderSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(resp) = production_service::check_production_access_response(&db).await {
        return Ok(resp);
    }
    let form_data = form_data.0;

    let result = production_order_service::insert(&db, &form_data).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn production_order_update(state: web::Data<AppState>, form_data: web::Json<ProductionOrderSaveRequest>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(resp) = production_service::check_production_access_response(&db).await {
        return Ok(resp);
    }
    let id = path.into_inner();
    let form_data = form_data.0;

    let result = production_order_service::update(&db, id, &form_data).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn production_order_batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(resp) = production_service::check_production_access_response(&db).await {
        return Ok(resp);
    }
    let ids = item.0.parse_ids();

    let result = production_order_service::batch_delete(&db, &ids).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn production_order_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(resp) = production_service::check_production_access_response(&db).await {
        return Ok(resp);
    }
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match production_order_service::get_info(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn production_order_list(state: web::Data<AppState>, query: web::Query<ProductionOrderListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(resp) = production_service::check_production_access_response(&db).await {
        return Ok(resp);
    }

    match production_order_service::get_list(&db, &query.into_inner()).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn production_order_release(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(resp) = production_service::check_production_access_response(&db).await {
        return Ok(resp);
    }
    let id = path.into_inner();

    match production_order_service::release(&db, id).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("操作成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn production_order_start(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(resp) = production_service::check_production_access_response(&db).await {
        return Ok(resp);
    }
    let id = path.into_inner();

    match production_order_service::start(&db, id).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("操作成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn production_order_complete(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(resp) = production_service::check_production_access_response(&db).await {
        return Ok(resp);
    }
    let id = path.into_inner();

    match production_order_service::complete(&db, id).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("操作成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn production_order_inbound(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(resp) = production_service::check_production_access_response(&db).await {
        return Ok(resp);
    }
    let id = path.into_inner();

    match production_order_service::inbound(&db, id).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("操作成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn production_order_close(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(resp) = production_service::check_production_access_response(&db).await {
        return Ok(resp);
    }
    let id = path.into_inner();

    match production_order_service::close(&db, id).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("操作成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/production/order")
            .route("/save", web::post().to(production_order_save).wrap(require_permission("production:order:save")))
            .route("/update/{id}", web::put().to(production_order_update).wrap(require_permission("production:order:update")))
            .route("/bath_delete", web::delete().to(production_order_batch_delete).wrap(require_permission("production:order:delete")))
            .route("/info", web::get().to(production_order_info).wrap(require_permission("production:order:view")))
            .route("/list", web::get().to(production_order_list).wrap(require_permission("production:order:list")))
            .route("/release/{id}", web::put().to(production_order_release).wrap(require_permission("production:order:release")))
            .route("/start/{id}", web::put().to(production_order_start).wrap(require_permission("production:order:start")))
            .route("/complete/{id}", web::put().to(production_order_complete).wrap(require_permission("production:order:complete")))
            .route("/inbound/{id}", web::put().to(production_order_inbound).wrap(require_permission("production:order:inbound")))
            .route("/close/{id}", web::put().to(production_order_close).wrap(require_permission("production:order:close"))),
    );
}