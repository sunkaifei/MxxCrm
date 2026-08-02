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
use crate::modules::website::model::website_cart::{CartAddRequest, CartBatchDeleteRequest, CartUpdateRequest};
use crate::modules::website::service::website_cart_service;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};

/// POST /api/user/website/cart/add - 添加购物车
#[post("/cart/add")]
pub async fn add(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CartAddRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    let result = website_cart_service::add(db, user_id, body.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<i64>::handle_result(result)))
}

/// GET /api/user/website/cart/list - 购物车列表
#[get("/cart/list")]
pub async fn list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    match website_cart_service::list(db, user_id).await {
        Ok(vo) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(vo, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// PUT /api/user/website/cart/update/{id} - 更新购物车项
#[put("/cart/update/{id}")]
pub async fn update(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
    body: web::Json<CartUpdateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    let result = website_cart_service::update(db, user_id, id.into_inner(), body.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<i64>::handle_result(result)))
}

/// DELETE /api/user/website/cart/delete/{id} - 删除购物车项
#[delete("/cart/delete/{id}")]
pub async fn delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    let result = website_cart_service::delete(db, user_id, id.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<i64>::handle_result(result)))
}

/// POST /api/user/website/cart/batch_delete - 批量删除购物车
#[post("/cart/batch_delete")]
pub async fn batch_delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CartBatchDeleteRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    let result = website_cart_service::batch_delete(db, user_id, body.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<i64>::handle_result(result)))
}
