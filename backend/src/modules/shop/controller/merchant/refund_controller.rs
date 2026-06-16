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
use crate::core::web::response::MetaResp;
use actix_web::{get, put, web, HttpRequest, HttpResponse};

/// Get refund list
#[get("/merchant/refund/list")]
pub async fn get_refund_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: extract merchant_id, call refund_service::list_by_merchant(db, merchant_id, query.into_inner()).await
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

/// Get refund detail
#[get("/merchant/refund/detail")]
pub async fn get_refund_detail(
    state: web::Data<AppState>,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call refund_service::detail(db, refund_id).await
    let result = serde_json::json!({});
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Agree refund
#[put("/merchant/refund/agree")]
pub async fn agree_refund(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call refund_service::agree(db, merchant_id, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Refuse refund
#[put("/merchant/refund/refuse")]
pub async fn refuse_refund(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call refund_service::refuse(db, merchant_id, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}