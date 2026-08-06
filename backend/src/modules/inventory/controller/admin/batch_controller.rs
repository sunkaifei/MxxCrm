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
use crate::core::web::response::{MetaResp, MPACK};
use crate::core::web::permission_guard::require_permission;
use crate::modules::inventory::model::batch::BatchListQuery;
use crate::modules::inventory::service::batch_service;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn batch_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    let query = BatchListQuery {
        page_num: query_str
            .split('&')
            .find(|s| s.starts_with("page="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1),
        page_size: query_str
            .split('&')
            .find(|s| s.starts_with("pageSize="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(20),
        batch_no: query_str
            .split('&')
            .find(|s| s.starts_with("batchNo="))
            .and_then(|s| s.split('=').nth(1))
            .map(|s| s.to_string()),
        product_id: query_str
            .split('&')
            .find(|s| s.starts_with("productId="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.parse::<i64>().ok()),
        warehouse_id: query_str
            .split('&')
            .find(|s| s.starts_with("warehouseId="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.parse::<i64>().ok()),
        status: query_str
            .split('&')
            .find(|s| s.starts_with("status="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.parse::<i32>().ok()),
    };

    match batch_service::get_list(&db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn batch_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req
        .query_string()
        .split('&')
        .find(|s| s.starts_with("id="))
        .and_then(|s| s.split('=').nth(1))
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match batch_service::get_detail(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn batch_trace(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req
        .query_string()
        .split('&')
        .find(|s| s.starts_with("id="))
        .and_then(|s| s.split('=').nth(1))
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match batch_service::trace(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn batch_list_by_product(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let product_id = req
        .query_string()
        .split('&')
        .find(|s| s.starts_with("productId="))
        .and_then(|s| s.split('=').nth(1))
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    if product_id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "产品ID无效", "local")));
    }

    match batch_service::list_by_product(&db, product_id).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/batch")
            .route(
                "/list",
                web::get()
                    .to(batch_list)
                    .wrap(require_permission("product:batch:list")),
            )
            .route(
                "/info",
                web::get()
                    .to(batch_info)
                    .wrap(require_permission("product:batch:list")),
            )
            .route(
                "/trace",
                web::get()
                    .to(batch_trace)
                    .wrap(require_permission("product:batch:list")),
            )
            .route(
                "/list_by_product",
                web::get()
                    .to(batch_list_by_product)
                    .wrap(require_permission("product:batch:list")),
            ),
    );
}
