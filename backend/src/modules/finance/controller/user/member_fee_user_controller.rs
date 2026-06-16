//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{get, post, web, HttpResponse, HttpRequest, Result, http::StatusCode};
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
use crate::modules::finance::model::member_fee::MemberFeeSaveRequest;
use crate::modules::finance::service::member_fee_service;
use crate::core::kit::jwt_util::JWTToken;
use crate::config;

#[get("/member-fee/my-info")]
pub async fn get_my_member_info(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let db = &state.db;

    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());
    let token_str = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(s) => s.trim_start_matches("Bearer ").to_string(),
            Err(e) => {
                log::error!("[获取会员信息] Authorization header解析失败: {:?}", e);
                return Ok(HttpResponse::Ok()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
            }
        },
        None => {
            log::error!("[获取会员信息] Authorization header不存在");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token不存在，请重新登录", "local")));
        }
    };

    let decoded_token = match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(t) => t,
        Err(e) => {
            log::error!("[获取会员信息] token验证失败: {:?}", e);
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效或已过期，请重新登录", "local")));
        }
    };

    if decoded_token.iss != "mxx_B2B_user" {
        log::error!("[获取会员信息] token签发者不匹配");
        return Ok(HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token类型错误，请重新登录", "local")));
    }

    let user_id = match decoded_token.id {
        Some(id) => id,
        None => {
            log::error!("[获取会员信息] token中未包含用户ID");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
        }
    };

    let result = member_fee_service::get_by_user_id(db, user_id).await;

    match result {
        Ok(Some(data)) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Ok(None) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(200, "暂无会员信息", "local"))),
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[post("/member-fee/create")]
pub async fn create_member_fee(
    req: HttpRequest,
    state: web::Data<AppState>,
    item: web::Json<MemberFeeSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;

    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());
    let token_str = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(s) => s.trim_start_matches("Bearer ").to_string(),
            Err(e) => {
                log::error!("[创建会员费用] Authorization header解析失败: {:?}", e);
                return Ok(HttpResponse::Ok()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
            }
        },
        None => {
            log::error!("[创建会员费用] Authorization header不存在");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token不存在，请重新登录", "local")));
        }
    };

    let decoded_token = match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(t) => t,
        Err(e) => {
            log::error!("[创建会员费用] token验证失败: {:?}", e);
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效或已过期，请重新登录", "local")));
        }
    };

    if decoded_token.iss != "mxx_B2B_user" {
        log::error!("[创建会员费用] token签发者不匹配");
        return Ok(HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token类型错误，请重新登录", "local")));
    }

    let user_id = match decoded_token.id {
        Some(id) => id,
        None => {
            log::error!("[创建会员费用] token中未包含用户ID");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
        }
    };

    let mut req = item.into_inner();
    req.user_id = user_id;

    let result = member_fee_service::insert(db, req).await;

    match result {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[post("/member-fee/purchase")]
pub async fn purchase_member(
    req: HttpRequest,
    state: web::Data<AppState>,
    item: web::Json<MemberFeeSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;

    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());
    let token_str = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(s) => s.trim_start_matches("Bearer ").to_string(),
            Err(e) => {
                log::error!("[购买会员] Authorization header解析失败: {:?}", e);
                return Ok(HttpResponse::Ok()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
            }
        },
        None => {
            log::error!("[购买会员] Authorization header不存在");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token不存在，请重新登录", "local")));
        }
    };

    let decoded_token = match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(t) => t,
        Err(e) => {
            log::error!("[购买会员] token验证失败: {:?}", e);
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效或已过期，请重新登录", "local")));
        }
    };

    if decoded_token.iss != "mxx_B2B_user" {
        log::error!("[购买会员] token签发者不匹配");
        return Ok(HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token类型错误，请重新登录", "local")));
    }

    let user_id = match decoded_token.id {
        Some(id) => id,
        None => {
            log::error!("[购买会员] token中未包含用户ID");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
        }
    };

    let mut req = item.into_inner();
    req.user_id = user_id;

    let result = member_fee_service::insert(db, req).await;

    match result {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}
