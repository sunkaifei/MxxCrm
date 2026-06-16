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
use crate::core::web::entity::common::InfoId;
use crate::core::web::response::MetaResp;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

/// Save category
#[post("/category/save")]
#[protect("system:category:save")]
pub async fn save(
    state: web::Data<AppState>,
    _req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call category_service::save(db, body.into_inner()).await
    let result = serde_json::json!({ "id": 0 });
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Update category
#[put("/category/update")]
#[protect("system:category:update")]
pub async fn update(
    state: web::Data<AppState>,
    _req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call category_service::update(db, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Delete category
#[delete("/category/delete")]
#[protect("system:category:delete")]
pub async fn delete(
    state: web::Data<AppState>,
    _req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call category_service::delete(db, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Get category tree
#[get("/category/tree")]
#[protect("system:category:list")]
pub async fn tree(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call category_service::get_tree(db).await
    let result = serde_json::json!([]);
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}