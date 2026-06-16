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
use actix_web::{get, put, web, HttpRequest, HttpResponse};

/// Get shop info
#[get("/merchant/shop/info")]
pub async fn get_shop_info(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: extract merchant_id from token, call shop_service::get_merchant_shop(db, merchant_id).await
    let result = serde_json::json!({});
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Update shop
#[put("/merchant/shop/update")]
pub async fn update_shop(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: extract merchant_id from token, call shop_service::update_shop(db, merchant_id, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}