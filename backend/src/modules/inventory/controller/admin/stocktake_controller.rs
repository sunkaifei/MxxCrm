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
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::inventory::model::stocktake::{StocktakeInputRequest, StocktakeListQuery, StocktakeSaveRequest};
use crate::modules::inventory::service::stocktake_service;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::permission_guard::require_permission;

pub async fn stocktake_save(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let body = body.0;

    let form_data: StocktakeSaveRequest = serde_json::from_value(body)?;

    let result = stocktake_service::create(db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn stocktake_update(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let id = path.into_inner();
    let body = body.0;

    let form_data: StocktakeSaveRequest = serde_json::from_value(body)?;

    let result = stocktake_service::update(db, id, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn stocktake_submit(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let id = path.into_inner();
    let result = stocktake_service::submit(db, id, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn stocktake_input(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>, body: web::Json<StocktakeInputRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let id = path.into_inner();
    let form_data = body.0;

    let result = stocktake_service::input(db, id, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn stocktake_complete(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let id = path.into_inner();
    let result = stocktake_service::complete(db, id, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn stocktake_cancel(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let id = path.into_inner();
    let result = stocktake_service::cancel(db, id, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn batch_delete_stocktake(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let ids = item.0.parse_ids();
    if ids.is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "请选择要删除的记录", "local")));
    }
    let result = stocktake_service::batch_delete(db, &ids).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn stocktake_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match stocktake_service::get_detail(db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn stocktake_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    fn q<'a>(qs: &'a str, key: &str) -> Option<&'a str> {
        qs.split('&').find(|s| s.starts_with(&format!("{}=", key)))
            .and_then(|s| s.split('=').nth(1))
    }

    let query = StocktakeListQuery {
        page_num: q(query_str, "page").and_then(|s| s.parse().ok()),
        page_size: q(query_str, "pageSize").and_then(|s| s.parse().ok()),
        stocktake_no: q(query_str, "checkNo").or_else(|| q(query_str, "stocktakeNo")).map(|s| s.to_string()),
        warehouse_id: q(query_str, "warehouseId").and_then(|s| s.parse().ok()),
        status: q(query_str, "status").and_then(|s| s.parse().ok()),
        stocktake_type: q(query_str, "checkType").or_else(|| q(query_str, "stocktakeType")).map(|s| s.to_string()),
    };

    match stocktake_service::get_list(db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 盘点审核（通过 POST JSON body 中的 id 完成审核）
pub async fn stocktake_audit(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let id = body.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }
    let result = stocktake_service::complete(db, id, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn stocktake_items(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let stocktake_id = req.query_string().split("&").find(|s| s.starts_with("stocktakeId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if stocktake_id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "stocktakeId无效", "local")));
    }

    match stocktake_service::get_items(db, stocktake_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/inventory/check")
            .route("/save", web::post().to(stocktake_save).wrap(require_permission("product:check:create")))
            .route("/update/{id}", web::put().to(stocktake_update).wrap(require_permission("product:check:update")))
            .route("/submit/{id}", web::put().to(stocktake_submit).wrap(require_permission("product:check:audit")))
            .route("/input/{id}", web::put().to(stocktake_input).wrap(require_permission("product:check:update")))
            .route("/complete/{id}", web::put().to(stocktake_complete).wrap(require_permission("product:check:audit")))
            .route("/cancel/{id}", web::put().to(stocktake_cancel).wrap(require_permission("product:check:update")))
            .route("/audit", web::post().to(stocktake_audit).wrap(require_permission("product:check:audit")))
            .route("/batch_delete", web::delete().to(batch_delete_stocktake).wrap(require_permission("product:check:delete")))
            .route("/info", web::get().to(stocktake_info).wrap(require_permission("product:check:list")))
            .route("/list", web::get().to(stocktake_list).wrap(require_permission("product:check:list")))
            .route("/items", web::get().to(stocktake_items).wrap(require_permission("product:check:list"))),
    );
}
