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
use actix_web::{get, post, put, web, HttpRequest, HttpResponse};

/// Create SPU
#[post("/merchant/spu/create")]
pub async fn create_spu(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: extract merchant_id, call spu_service::create(db, merchant_id, body.into_inner()).await
    let result = serde_json::json!({ "id": 0 });
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Update SPU
#[put("/merchant/spu/update")]
pub async fn update_spu(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call spu_service::update(db, merchant_id, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Get SPU list
#[get("/merchant/spu/list")]
pub async fn get_spu_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call spu_service::list(db, merchant_id, query.into_inner()).await
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

/// Get SPU detail
#[get("/merchant/spu/detail")]
pub async fn get_spu_detail(
    state: web::Data<AppState>,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call spu_service::detail(db, spu_id).await
    let result = serde_json::json!({});
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Offline SPU
#[put("/merchant/spu/offline")]
pub async fn offline_spu(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call spu_service::offline(db, merchant_id, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Batch update SKU stock
#[put("/merchant/sku/stock")]
pub async fn batch_update_sku_stock(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call sku_service::batch_update_stock(db, merchant_id, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}