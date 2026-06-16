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
use actix_web::HttpRequest;
use crate::core::kit::config;
use crate::core::kit::jwt_util::JWTToken;

/// 获取管理员信息
pub fn get_user(req: &HttpRequest) -> Result<JWTToken> {
    let token = req
        .headers()
        .get("Authorization")
        .map(|v| v.to_str().unwrap_or_default().to_string())
        .unwrap_or_default()
        .split("Bearer ")
        .collect::<Vec<&str>>()
        .pop()
        .unwrap_or_default()
        .to_string();
    let jwt_token = JWTToken::verify(&config::section::<String>("server", "jwt_secret_admin", "".to_string()), &token)?;
    Ok(jwt_token)
}

/// 获取管理员ID
pub fn get_admin_id(req: &HttpRequest) -> Result<i64> {
    let jwt_token = get_user(req)?;
    Ok(jwt_token.id.unwrap_or_default())
}

/// 获取用户端用户信息
pub fn get_user_client(req: &HttpRequest) -> Result<JWTToken> {
    let token = req
        .headers()
        .get("Authorization")
        .map(|v| v.to_str().unwrap_or_default().to_string())
        .unwrap_or_default()
        .split("Bearer ")
        .collect::<Vec<&str>>()
        .pop()
        .unwrap_or_default()
        .to_string();
    let jwt_token = JWTToken::verify(&config::section::<String>("server", "jwt_secret_user", "".to_string()), &token)?;
    Ok(jwt_token)
}

/// 获取用户端用户ID
pub fn get_user_client_id(req: &HttpRequest) -> Result<i64> {
    let jwt_token = get_user_client(req)?;
    Ok(jwt_token.id.unwrap_or_default())
}


