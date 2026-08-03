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
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::model::website_user::{WebsiteUserLoginRequest, WebsiteUserRegisterRequest};
use crate::modules::website::service::website_user_service;
use actix_web::{post, web, HttpRequest, HttpResponse};

/// POST /api/open/website/user/register - 前台用户注册
#[post("/api/open/website/user/register")]
pub async fn register(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<WebsiteUserRegisterRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
    let result = website_user_service::register(db, body.into_inner(), ip_address).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// POST /api/open/website/user/login - 前台用户登录
#[post("/api/open/website/user/login")]
pub async fn login(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<WebsiteUserLoginRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
    match website_user_service::login(db, body.into_inner(), ip_address).await {
        Ok(vo) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(vo, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}
