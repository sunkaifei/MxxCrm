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

/// Apply for refund
#[post("/refund/apply")]
pub async fn apply_refund(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call refund_service::apply(db, user_id, body.into_inner()).await
    let result = serde_json::json!({ "refund_id": 0 });
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Get refund detail
#[get("/refund/detail")]
pub async fn get_refund_detail(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call refund_service::detail(db, user_id, refund_id).await
    let result = serde_json::json!({});
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Cancel refund
#[put("/refund/cancel")]
pub async fn cancel_refund(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call refund_service::cancel(db, user_id, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Get refund list
#[get("/refund/list")]
pub async fn get_refund_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call refund_service::list(db, user_id, query.into_inner()).await
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