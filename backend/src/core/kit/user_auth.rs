//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.!
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{HttpRequest, HttpResponse, http::StatusCode};
use crate::core::web::response::{MetaResp, MPACK};
use crate::core::kit::jwt_util::JWTToken;
use crate::core::kit::{config, CONTEXT};

const USER_TOKEN_ISSUER: &str = "mxx_B2B_user";
const USER_TOKEN_ERROR_MSG_TOKEN_INVALID: &str = "token无效，请重新登录";
const USER_TOKEN_ERROR_MSG_TOKEN_NOT_EXIST: &str = "token不存在，请重新登录";
const USER_TOKEN_ERROR_MSG_TOKEN_EXPIRED: &str = "token无效或已过期，请重新登录";
const USER_TOKEN_ERROR_MSG_TOKEN_TYPE: &str = "token类型错误，请重新登录";
const USER_TOKEN_ERROR_MSG_USER_ID_INVALID: &str = "用户ID无效，请重新登录";
const USER_TOKEN_ERROR_MSG_USER_DISABLED: &str = "账号已被禁用，请重新登录";

pub async fn get_user_id_from_request(req: &HttpRequest) -> Result<i64, HttpResponse> {
    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());
    let token_str = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(s) => s.trim_start_matches("Bearer ").to_string(),
            Err(e) => {
                log::error!("[获取用户ID] Authorization header解析失败: {:?}", e);
                return Err(HttpResponse::Ok()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type(MPACK).body(MetaResp::<String>::fail(400, USER_TOKEN_ERROR_MSG_TOKEN_INVALID, "local")));
            }
        },
        None => {
            log::error!("[获取用户ID] Authorization header不存在");
            return Err(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type(MPACK).body(MetaResp::<String>::fail(400, USER_TOKEN_ERROR_MSG_TOKEN_NOT_EXIST, "local")));
        }
    };

    let decoded_token = match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(t) => t,
        Err(e) => {
            log::error!("[获取用户ID] token校验失败: {:?}", e);
            return Err(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type(MPACK).body(MetaResp::<String>::fail(400, USER_TOKEN_ERROR_MSG_TOKEN_EXPIRED, "local")));
        }
    };

    if decoded_token.iss != USER_TOKEN_ISSUER {
        log::error!("[获取用户ID] token签发者不匹配");
        return Err(HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .content_type(MPACK).body(MetaResp::<String>::fail(400, USER_TOKEN_ERROR_MSG_TOKEN_TYPE, "local")));
    }

    let user_id = match decoded_token.id {
        Some(id) => id,
        None => {
            log::error!("[获取用户ID] 用户ID无效");
            return Err(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type(MPACK).body(MetaResp::<String>::fail(400, USER_TOKEN_ERROR_MSG_USER_ID_INVALID, "local")));
        }
    };

    // v1.1: 校验用户是否被禁用（后台禁用时写入 user_disabled:{id} 标记，即时生效）
    if let Ok(flag) = CONTEXT.cache_service.get_string(&format!("user_disabled:{}", user_id)).await {
        if !flag.is_empty() {
            log::warn!("[获取用户ID] 用户已被禁用 user_id={}", user_id);
            return Err(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type(MPACK).body(MetaResp::<String>::fail(400, USER_TOKEN_ERROR_MSG_USER_DISABLED, "local")));
        }
    }

    Ok(user_id)
}
