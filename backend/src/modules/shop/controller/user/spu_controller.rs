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
use actix_web::{get, web, HttpRequest, HttpResponse};

/// Get SPU list (paginated)
#[get("/list")]
pub async fn get_spu_list(
    state: web::Data<AppState>,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call spu_service::list(db, query.into_inner()).await
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

/// Get SPU detail (with SKU and review stats)
#[get("/detail")]
pub async fn get_spu_detail(
    state: web::Data<AppState>,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call spu_service::get_detail(db, spu_id).await
    let result = serde_json::json!({
        "spu": {},
        "sku_list": [],
        "review_stats": {}
    });
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}