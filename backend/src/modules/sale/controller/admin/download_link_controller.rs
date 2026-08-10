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
use crate::modules::sale::service::download_link_service;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GenerateRequest {
    pub delivery_id: i64,
    pub expire_hours: Option<i64>,
}

#[derive(Deserialize)]
pub struct RevokeRequest {
    pub delivery_id: i64,
}

#[derive(Deserialize)]
pub struct AccessQuery {
    pub token: String,
    pub id: i64,
}

pub async fn generate(state: web::Data<AppState>, form_data: web::Json<GenerateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.delivery_id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "交付ID不能为空", "local")));
    }
    let expire_hours = form_data.expire_hours.unwrap_or(24);
    match download_link_service::generate_signed_url(db, form_data.delivery_id, expire_hours).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /sale/download/access - 公开接口，用 token 验证（无权限）
pub async fn access(state: web::Data<AppState>, query: web::Query<AccessQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    if query.token.is_empty() || query.id == 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "token和交付ID不能为空", "local"));
    }
    match download_link_service::verify_and_serve(db, query.token, query.id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn revoke(state: web::Data<AppState>, form_data: web::Json<RevokeRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.delivery_id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "交付ID不能为空", "local")));
    }
    match download_link_service::revoke_access(db, form_data.delivery_id).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/download")
            .route(
                "/generate",
                web::post().to(generate).wrap(require_permission("sale:delivery:save")),
            )
            // GET /sale/download/access - 公开接口，无权限（用 token 验证）
            .route(
                "/access",
                web::get().to(access),
            )
            .route(
                "/revoke",
                web::post().to(revoke).wrap(require_permission("sale:delivery:save")),
            ),
    );
}
