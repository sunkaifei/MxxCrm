//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::user_auth::get_user_id_from_request;
use crate::core::web::response::MetaResp;
use crate::modules::website::model::website_order::OrderCreateRequest;
use crate::modules::website::service::website_order_service;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;

/// POST /api/user/website/order/create - 创建订单
#[post("/order/create")]
pub async fn create(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<OrderCreateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    match website_order_service::create_order(db, user_id, body.into_inner()).await {
        Ok(id) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<i32>,
}

/// GET /api/user/website/order/list - 用户订单列表
#[get("/order/list")]
pub async fn list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<OrderListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    match website_order_service::user_order_list(
        db,
        user_id,
        query.page.unwrap_or(1),
        query.page_size.unwrap_or(10),
        query.status,
    )
    .await
    {
        Ok(page) => Ok(HttpResponse::Ok().json(page)),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /api/user/website/order/detail/{id} - 订单详情
#[get("/order/detail/{id}")]
pub async fn detail(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    match website_order_service::get_order_detail(db, user_id, id.into_inner()).await {
        Ok(vo) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(vo, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CancelRequest {
    pub reason: Option<String>,
}

/// POST /api/user/website/order/cancel/{id} - 取消订单
#[post("/order/cancel/{id}")]
pub async fn cancel(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
    body: web::Json<CancelRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    let reason = body.reason.clone().unwrap_or_else(|| "用户主动取消".to_string());
    let result = website_order_service::user_cancel_order(db, user_id, id.into_inner(), reason).await;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<i64>::handle_result(result)))
}

/// POST /api/user/website/order/confirm_receive/{id} - 确认收货
#[post("/order/confirm_receive/{id}")]
pub async fn confirm_receive(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    let result = website_order_service::user_confirm_receive(db, user_id, id.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<i64>::handle_result(result)))
}
