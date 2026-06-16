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
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};

/// Add to cart
#[post("/cart/add")]
pub async fn add_cart(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call cart_service::add(db, user_id, body.into_inner()).await
    let result = serde_json::json!({ "id": 0 });
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Get cart list (grouped by shop)
#[get("/cart/list")]
pub async fn get_cart_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call cart_service::list(db, user_id).await
    let result = serde_json::json!([]);
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Update cart item (quantity/selected)
#[put("/cart/update")]
pub async fn update_cart(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call cart_service::update(db, user_id, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Batch delete cart items
#[delete("/cart/delete")]
pub async fn delete_cart(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let _user_id = get_user_client_id(&req)?;
    // TODO: call cart_service::batch_delete(db, user_id, ids).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}