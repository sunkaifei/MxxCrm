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
use actix_web::{put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

/// Audit supplier application
#[put("/supplier/audit")]
#[protect("system:supplier:audit")]
pub async fn audit_apply(
    state: web::Data<AppState>,
    _req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call audit_service::audit_supplier(db, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Audit SPU
#[put("/spu/audit")]
#[protect("system:spu:audit")]
pub async fn audit_spu(
    state: web::Data<AppState>,
    _req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call audit_service::audit_spu(db, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}