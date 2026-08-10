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
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::service::exchange_service::{self, ExchangeItemInput, ExchangeListQuery};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ExchangeCreateRequest {
    pub refund_id: Option<i64>,
    pub order_id: i64,
    pub items: Vec<ExchangeItemInput>,
}

#[derive(Deserialize)]
pub struct ExchangeIdRequest {
    pub id: i64,
}

pub async fn create(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ExchangeCreateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let result = exchange_service::create_exchange(db, form_data.refund_id, form_data.order_id, form_data.items, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn submit(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ExchangeIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match exchange_service::submit_exchange(db, form_data.id, jwt_token.id.unwrap_or_default(), &jwt_token.username.clone().unwrap_or_default()).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn approve(state: web::Data<AppState>, form_data: web::Json<ExchangeIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    match exchange_service::approve_exchange(db, form_data.id).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn info(state: web::Data<AppState>, query: web::Query<ExchangeIdRequest>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    if query.id == 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local"));
    }
    match exchange_service::get_info(db, query.id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn list(state: web::Data<AppState>, query: web::Query<ExchangeListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    match exchange_service::get_list(db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/exchange")
            .route(
                "/create",
                web::post().to(create).wrap(require_permission("sale:refund:save")),
            )
            .route(
                "/submit",
                web::post().to(submit).wrap(require_permission("sale:refund:save")),
            )
            .route(
                "/approve",
                web::post().to(approve).wrap(require_permission("sale:refund:save")),
            )
            .route(
                "/info",
                web::get().to(info).wrap(require_permission("sale:refund:list")),
            )
            .route(
                "/list",
                web::get().to(list).wrap(require_permission("sale:refund:list")),
            ),
    );
}
