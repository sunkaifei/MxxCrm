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
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::inventory::model::outbound::*;
use crate::modules::inventory::service::outbound_service;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::permission_guard::require_permission;

pub async fn outbound_save(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let body = body.0;

    let form_data: OutboundSaveRequest = serde_json::from_value(body)?;

    let result = outbound_service::create(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn outbound_audit(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let id = body.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }
    let audit_by = jwt_token.id.unwrap_or_default();
    if audit_by <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "获取用户信息失败", "local")));
    }
    let result = outbound_service::audit(&db, id, audit_by).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn outbound_reject(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let id = body.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }
    let audit_by = jwt_token.id.unwrap_or_default();
    if audit_by <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "获取用户信息失败", "local")));
    }
    let result = outbound_service::reject(&db, id, audit_by).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn outbound_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match outbound_service::get_detail(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn outbound_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    let query = OutboundListQuery {
        page_num: query_str.split("&").find(|s| s.starts_with("page=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<u64>().ok())).unwrap_or(1),
        page_size: query_str.split("&").find(|s| s.starts_with("pageSize=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<u64>().ok())).unwrap_or(20),
        outbound_no: query_str.split("&").find(|s| s.starts_with("outboundNo=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
        warehouse_id: query_str.split("&").find(|s| s.starts_with("warehouseId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        status: query_str.split("&").find(|s| s.starts_with("status=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i32>().ok())),
        outbound_type: query_str.split("&").find(|s| s.starts_with("outboundType=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
    };

    match outbound_service::get_list(&db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn outbound_update(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let body = body.0;

    let form_data: OutboundSaveRequest = serde_json::from_value(body)?;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    let result = outbound_service::update(&db, id, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn outbound_batch_delete(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let ids_str = req.query_string().split("&").find(|s| s.starts_with("ids=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())).unwrap_or_default();
    let ids: Vec<i64> = ids_str.split(',').filter_map(|s| s.parse::<i64>().ok()).collect();
    if ids.is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "参数无效：ids 必填", "local")));
    }

    let result = outbound_service::batch_delete(&db, &ids).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn outbound_submit(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();
    let result = outbound_service::submit_audit(db, id).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn outbound_print(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match outbound_service::get_print_data(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn outbound_export(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    let query = OutboundListQuery {
        page_num: query_str.split("&").find(|s| s.starts_with("page=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<u64>().ok())).unwrap_or(1),
        page_size: query_str.split("&").find(|s| s.starts_with("pageSize=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<u64>().ok())).unwrap_or(10000),
        outbound_no: query_str.split("&").find(|s| s.starts_with("outboundNo=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
        warehouse_id: query_str.split("&").find(|s| s.starts_with("warehouseId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        status: query_str.split("&").find(|s| s.starts_with("status=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i32>().ok())),
        outbound_type: query_str.split("&").find(|s| s.starts_with("outboundType=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
    };

    match outbound_service::export_list(&db, &query).await {
        Ok(csv) => Ok(HttpResponse::Ok()
            .content_type("text/csv; charset=utf-8")
            .insert_header(("Content-Disposition", "attachment; filename=\"outbound_list.csv\""))
            .body(csv)),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn outbound_import(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let body = body.0;

    let items: Vec<OutboundSaveRequest> = serde_json::from_value(body)
        .map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;

    let result = outbound_service::import_list(&db, items, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/outbound")
            .route("/save", web::post().to(outbound_save).wrap(require_permission("product:outbound:create")))
            .route("/update", web::put().to(outbound_update).wrap(require_permission("product:outbound:update")))
            .route("/submit/{id}", web::put().to(outbound_submit).wrap(require_permission("product:outbound:audit")))
            .route("/audit", web::post().to(outbound_audit).wrap(require_permission("product:outbound:audit")))
            .route("/reject", web::post().to(outbound_reject).wrap(require_permission("product:outbound:audit")))
            .route("/batch_delete", web::delete().to(outbound_batch_delete).wrap(require_permission("product:outbound:delete")))
            .route("/info", web::get().to(outbound_info).wrap(require_permission("product:outbound:list")))
            .route("/list", web::get().to(outbound_list).wrap(require_permission("product:outbound:list")))
            .route("/print/{id}", web::get().to(outbound_print).wrap(require_permission("product:outbound:list")))
            .route("/export", web::get().to(outbound_export).wrap(require_permission("product:outbound:list")))
            .route("/import", web::post().to(outbound_import).wrap(require_permission("product:outbound:create"))),
    );
}