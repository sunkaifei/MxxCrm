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
use actix_web::{get, post, web, HttpRequest, HttpResponse};

/// Get order list
#[get("/merchant/order/list")]
pub async fn get_order_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: extract merchant_id, call order_service::list_by_merchant(db, merchant_id, query.into_inner()).await
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
#[get("/merchant/order/detail")]
pub async fn get_order_detail(
    state: web::Data<AppState>,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call order_service::detail(db, order_id).await
    let result = serde_json::json!({});
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Deliver order
#[post("/merchant/order/deliver")]
pub async fn deliver_order(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: extract merchant_id, call order_service::deliver(db, merchant_id, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}