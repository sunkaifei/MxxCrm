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
use actix_web::{get, post, web, HttpRequest, HttpResponse};

/// Add review
#[post("/review/add")]
pub async fn add_review(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call review_service::add(db, user_id, body.into_inner()).await
    let result = serde_json::json!({ "id": 0 });
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Get review list
#[get("/review/list")]
pub async fn get_review_list(
    state: web::Data<AppState>,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call review_service::list(db, query.into_inner()).await
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

/// Get review stats
#[get("/review/stats")]
pub async fn get_review_stats(
    state: web::Data<AppState>,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call review_service::stats(db, spu_id).await
    let result = serde_json::json!({
        "total": 0,
        "good": 0,
        "medium": 0,
        "bad": 0
    });
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}