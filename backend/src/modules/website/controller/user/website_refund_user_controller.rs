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
use crate::modules::website::model::website_refund::RefundApplyRequest;
use crate::modules::website::service::website_refund_service;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;

/// POST /api/user/website/refund/apply - 申请退款
#[post("/refund/apply")]
pub async fn apply(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<RefundApplyRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    match website_refund_service::apply(db, user_id, body.into_inner()).await {
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
pub struct RefundListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<i32>,
}

/// GET /api/user/website/refund/list - 用户退款列表
#[get("/refund/list")]
pub async fn list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<RefundListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    match website_refund_service::user_list(
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

/// GET /api/user/website/refund/detail/{id} - 退款详情
#[get("/refund/detail/{id}")]
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
    match website_refund_service::get_detail(db, Some(user_id), id.into_inner()).await {
        Ok(vo) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(vo, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// POST /api/user/website/refund/cancel/{id} - 取消退款申请
#[post("/refund/cancel/{id}")]
pub async fn cancel(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    let result = website_refund_service::user_cancel(db, user_id, id.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<i64>::handle_result(result)))
}
