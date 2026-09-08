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
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::inventory::model::stock_snapshot::StockSnapshotListQuery;
use crate::modules::inventory::service::stock_snapshot_service;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn snapshot_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    let query = StockSnapshotListQuery {
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
        snapshot_date: query_str
            .split('&')
            .find(|s| s.starts_with("snapshotDate="))
            .and_then(|s| s.split('=').nth(1))
            .map(|s| s.to_string()),
        start_date: query_str
            .split('&')
            .find(|s| s.starts_with("startDate="))
            .and_then(|s| s.split('=').nth(1))
            .map(|s| s.to_string()),
        end_date: query_str
            .split('&')
            .find(|s| s.starts_with("endDate="))
            .and_then(|s| s.split('=').nth(1))
            .map(|s| s.to_string()),
        warehouse_id: query_str
            .split('&')
            .find(|s| s.starts_with("warehouseId="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.parse::<i64>().ok()),
        product_id: query_str
            .split('&')
            .find(|s| s.starts_with("productId="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.parse::<i64>().ok()),
    };

    match stock_snapshot_service::get_list(&db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn snapshot_generate(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match stock_snapshot_service::generate(&db).await {
        Ok(count) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(count, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/snapshot")
            .route(
                "/list",
                web::get()
                    .to(snapshot_list)
                    .wrap(require_permission("product:snapshot:list")),
            )
            .route(
                "/generate",
                web::post()
                    .to(snapshot_generate)
                    .wrap(require_permission("product:snapshot:list")),
            ),
    );
}
