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
use crate::modules::website::model::website_user::WebsiteUserUpdateRequest;
use crate::modules::website::service::website_user_service;
use actix_web::{get, post, put, web, HttpRequest, HttpResponse};
use serde::Deserialize;

/// GET /api/user/website/profile - 获取当前登录用户信息
#[get("/profile")]
pub async fn get_profile(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    match website_user_service::get_profile(db, user_id).await {
        Ok(vo) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(vo, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// PUT /api/user/website/profile - 更新当前登录用户资料
#[put("/profile")]
pub async fn update_profile(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<WebsiteUserUpdateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    let result = website_user_service::update_profile(db, user_id, body.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<i64>::handle_result(result)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// POST /api/user/website/change_password - 修改密码
#[post("/change_password")]
pub async fn change_password(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<ChangePasswordRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    let result = website_user_service::change_password(
        db,
        user_id,
        body.old_password.clone(),
        body.new_password.clone(),
    )
    .await;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<i64>::handle_result(result)))
}
