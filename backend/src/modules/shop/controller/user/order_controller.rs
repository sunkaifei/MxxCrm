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
use crate::core::web::base_controller::get_user_client_id;
use crate::core::web::response::MetaResp;
use actix_web::{get, post, put, web, HttpRequest, HttpResponse};

/// Create order
#[post("/order/create")]
pub async fn create_order(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call order_service::create(db, user_id, body.into_inner()).await
    let result = serde_json::json!({ "order_id": 0 });
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Pay order
#[post("/order/pay")]
pub async fn pay_order(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call order_service::pay(db, user_id, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Get order list
#[get("/order/list")]
pub async fn get_order_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call order_service::list(db, user_id, query.into_inner()).await
    let result = serde_json::json!({
        "list": [],
        "total": 0,
        "currentPage": 1,
        "pageSize": 10,
        "totalPages": 0
    });
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Get order detail
#[get("/order/detail")]
pub async fn get_order_detail(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call order_service::detail(db, user_id, order_id).await
    let result = serde_json::json!({});
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Cancel order
#[put("/order/cancel")]
pub async fn cancel_order(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call order_service::cancel(db, user_id, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Confirm receipt
#[put("/order/confirm")]
pub async fn confirm_receive(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call order_service::confirm_receive(db, user_id, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}