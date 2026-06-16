//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{get, post, web, HttpResponse, http::StatusCode, HttpRequest, http::header::{ContentType, HeaderValue}};
use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use reqwest::Client;
use sea_orm::TransactionTrait;
use serde_json::{self, Value};
use chrono::Datelike;
use crate::core::errors::error::{Error, Result};
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller;
use crate::core::web::response::MetaResp;
use crate::modules::user::model::user_platform::{WechatAuthResponse, UserPlatformModel, UserPlatformSaveDTO, UserInfo, WechatLoginRequest, TokenRefreshRequest, TempTokenRequest, UserBasicInfoVO, WechatPhoneResponse, WechatPhoneLoginRequest, WechatCodeLoginRequest};
use crate::modules::user::model::user::{UserModel, UserSaveDTO, UpdateNicknameRequest, UpdateAvatarRequest};
use crate::modules::user::service::{user_platform_service, user_service};
use crate::modules::system::service::config_service;
use crate::modules::upload::model::attachment::{StorageType, UploadImageResult};
use crate::modules::upload::service::upload_service;
use crate::config;
use crate::utils::time_utils;
use std::fs;
use std::path::Path;
use std::ffi::OsStr;

/// 微信手机号授权登录/注册
/// 通过手机号授权获取用户手机号，用于登录或注册
/// 已注册用户：查询手机号获取登录信息，更新用户表手机号
/// 未注册用户：使用手机号创建新用户并完成注册
/// 支持同时传入login_code绑定微信平台信息
#[post("/wechat/phone-login")]
pub async fn wechat_phone_login(state: web::Data<AppState>, item: web::Json<WechatPhoneLoginRequest>) -> Result<HttpResponse> {
    let db = &state.db;

    let app_id = config::section::<String>("wechat", "app_id", "".to_string());
    let app_secret = config::section::<String>("wechat", "app_secret", "".to_string());

    if app_id.is_empty() || app_secret.is_empty() {
        log::error!("[微信手机号登录] 微信配置未设置");
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "微信配置未设置", "local")));
    }

    let client = Client::new();
    
    // 获取access_token（用于获取手机号）
    let access_token_url = format!(
        "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
        app_id, app_secret
    );
    
    let access_resp = client.get(&access_token_url).send().await.map_err(|e| {
        log::error!("[微信手机号登录] 获取access_token失败: {:?}", e);
        Error::from(format!("获取access_token失败: {:?}", e))
    })?;
    
    let access_result: serde_json::Value = access_resp.json().await.map_err(|e| {
        log::error!("[微信手机号登录] 解析access_token响应失败: {:?}", e);
        Error::from(format!("解析access_token响应失败: {:?}", e))
    })?;
    
    let _access_token = access_result.get("access_token").and_then(|v| v.as_str())
        .ok_or(Error::from("获取access_token失败"))?;

    // 获取手机号
    let phone_url = format!(
        "https://api.weixin.qq.com/wxa/business/getuserphonenumber?access_token={}",
        _access_token
    );
    let phone_body = serde_json::json!({ "code": item.phone_code });
    
    let phone_resp = client.post(&phone_url)
        .json(&phone_body)
        .send()
        .await
        .map_err(|e| {
            log::error!("[微信手机号登录] 获取手机号失败: {:?}", e);
            Error::from(format!("获取手机号失败: {:?}", e))
        })?;

    let phone_result: WechatPhoneResponse = phone_resp.json().await.map_err(|e| {
        log::error!("[微信手机号登录] 解析手机号响应失败: {:?}", e);
        Error::from(format!("解析手机号响应失败: {:?}", e))
    })?;

    if phone_result.errcode != Some(0) && phone_result.errcode != Some(200) {
        let errmsg = phone_result.errmsg.unwrap_or("获取手机号失败".to_string());
        log::error!("[微信手机号登录] 获取手机号失败: {}", errmsg);
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &format!("获取手机号失败: {}", errmsg), "local")));
    }

    let phone_number = phone_result.phone_info
        .and_then(|p| p.pure_phone_number.or(p.phone_number))
        .ok_or(Error::from("未能获取到手机号"))?;

    log::info!("[微信手机号登录] 获取手机号成功: {}", phone_number);

    // 如果有login_code，获取微信平台信息
    let wechat_response: Option<WechatAuthResponse> = if let Some(ref login_code) = item.login_code {
        if !login_code.is_empty() {
            let url = format!(
                "https://api.weixin.qq.com/sns/jscode2session?appid={}&secret={}&js_code={}&grant_type=authorization_code",
                app_id, app_secret, login_code
            );

            let response = client.get(&url).send().await.map_err(|e| {
                log::error!("[微信手机号登录] 获取微信平台信息失败: {:?}", e);
                Error::from(format!("获取微信平台信息失败: {:?}", e))
            })?;

            let resp: WechatAuthResponse = response.json().await.map_err(|e| {
                log::error!("[微信手机号登录] 解析微信平台信息失败: {:?}", e);
                Error::from(format!("解析微信平台信息失败: {:?}", e))
            })?;

            if let Some(errcode) = resp.errcode {
                if errcode != 0 {
                    let errmsg = resp.errmsg.unwrap_or("获取微信平台信息失败".to_string());
                    log::warn!("[微信手机号登录] 获取微信平台信息失败: {}", errmsg);
                    None
                } else {
                    Some(resp)
                }
            } else {
                Some(resp)
            }
        } else {
            None
        }
    } else {
        None
    };

    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());

    // 先通过手机号查询用户
    let mut user_id: Option<i64> = None;
    let mut existing_user: Option<crate::modules::user::entity::user::Model> = None;
    let mut mobile_conflict = false;
    
    if let Some(user_model) = UserModel::find_by_mobile(db, &phone_number).await? {
        // 检查手机号是否被其他人占用
        let phone_owned_by_other = if let Some(ref resp) = wechat_response {
            if let Some(ref openid) = resp.openid {
                if let Some(platform_model) = UserPlatformModel::find_by_weixin_openid(db, openid).await? {
                    // 如果有openid，检查openid对应的用户是否是同一个
                    platform_model.user_id != user_model.id
                } else {
                    // openid未注册，说明是用其他方式注册的用户
                    true
                }
            } else {
                // 没有openid，手机号已被他人使用
                true
            }
        } else {
            // 没有openid凭证，无法验证是否是本人使用，手机号已被他人使用
            true
        };
        
        if phone_owned_by_other {
            let existing_user_id = user_model.id;
            log::warn!("[微信手机号登录] 手机号已被其他用户占用，phone_number={}, existing_user_id={}", phone_number, existing_user_id);
            mobile_conflict = true;
        } else {
            user_id = Some(user_model.id);
            existing_user = Some(user_model);
            log::info!("[微信手机号登录] 用户已存在且手机号属于本人，user_id={}", user_id.unwrap());
        }
    }

    if user_id.is_none() && !mobile_conflict {
        // 用户不存在或手机号冲突，尝试通过微信openid查询
        if let Some(ref resp) = wechat_response {
            if let Some(ref openid) = resp.openid {
                if let Some(platform_model) = UserPlatformModel::find_by_weixin_openid(db, openid).await? {
                    let openid_user_id = platform_model.user_id;
                    
                    // 检查该用户的手机号是否被其他用户占用
                    if let Some(openid_user) = UserModel::find_by_id(db, &Some(openid_user_id)).await? {
                        if let Some(ref user_mobile) = openid_user.mobile {
                            if user_mobile != &phone_number {
                                // 用户已有手机号，但不是当前请求的手机号
                                // 检查这个手机号是否被其他用户占用
                                if let Some(mobile_owner) = UserModel::find_by_mobile(db, user_mobile).await? {
                                    if mobile_owner.id != openid_user_id {
                                        // 手机号被其他用户占用，应该报错
                                        log::warn!("[微信手机号登录] openid对应用户的手机号被其他用户占用，user_id={}, occupied_by_user_id={}", 
                                            openid_user_id, mobile_owner.id);
                                        mobile_conflict = true;
                                    }
                                }
                            }
                        }
                    }
                    
                    if !mobile_conflict {
                        user_id = Some(openid_user_id);
                        existing_user = UserModel::find_by_id(db, &Some(openid_user_id)).await?.or_else(|| {
                            log::warn!("[微信手机号登录] 平台记录存在但用户不存在，user_id={}", openid_user_id);
                            None
                        });
                        if let Some(uid) = user_id {
                            log::info!("[微信手机号登录] 通过openid找到用户，user_id={}", uid);
                        }
                    }
                }
            }
        }
    }
    
    // 如果手机号冲突，直接返回错误
    if mobile_conflict {
        log::error!("[微信手机号登录] 手机号已被其他用户占用，phone_number={}", phone_number);
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "该手机号已被其他用户使用，请使用其他手机号", "local")));
    }

    if user_id.is_none() {
        // 用户不存在，创建新用户（可能同时绑定微信）
        let new_user_id = (*db).transaction(|conn| {
            let user_save_dto = UserSaveDTO {
                id: None,
                username: None,
                nickname: None,
                avatar: None,
                email: None,
                mobile: Some(phone_number.clone()),
                loginfailure: Some(0),
                lastlogintime: None,
                lastloginip: None,
                password: None,
                salt: None,
                motto: None,
                status: Some("1".to_string()),
            };
            
            Box::pin(async move {
                let new_user_id = user_service::insert(conn, &user_save_dto).await?;

                // 如果有微信信息，绑定平台记录
                if let Some(ref resp) = wechat_response.clone() {
                    let mut save_dto: UserPlatformSaveDTO = resp.clone().into();
                    save_dto.user_id = Some(new_user_id);
                    user_platform_service::insert(conn, &save_dto).await?;
                    log::info!("[微信手机号登录] 创建用户并绑定微信成功，user_id={}", new_user_id);
                }

                Ok::<i64, Error>(new_user_id)
            })
        }).await.map_err(|e| {
            log::error!("[微信手机号登录] 创建用户失败: {:?}", e);
            Error::from(format!("创建用户失败: {:?}", e))
        })?;

        user_id = Some(new_user_id);
        log::info!("[微信手机号登录] 创建新用户成功，user_id={}", new_user_id);
    } else {
        // 用户已存在，更新手机号（如果之前没有）
        if let Some(ref mut user) = existing_user {
            if user.mobile.is_none() || user.mobile != Some(phone_number.clone()) {
                let save_dto = UserSaveDTO {
                    id: Some(user.id),
                    username: user.username.clone(),
                    nickname: user.nickname.clone(),
                    avatar: user.avatar.clone(),
                    email: user.email.clone(),
                    mobile: Some(phone_number.clone()),
                    loginfailure: user.loginfailure,
                    lastlogintime: None,
                    lastloginip: user.lastloginip.clone(),
                    password: user.password.clone(),
                    salt: user.salt.clone(),
                    motto: user.motto.clone(),
                    status: user.status.clone(),
                };
                user_service::update_by_id(db, &save_dto).await?;
                log::info!("[微信手机号登录] 更新用户手机号成功，user_id={}", user.id);
            }

            // 如果有微信信息且未绑定，绑定平台记录
            if let Some(ref resp) = wechat_response {
                if let Some(ref openid) = resp.openid {
                    if UserPlatformModel::find_by_weixin_openid(db, openid).await?.is_none() {
                        let mut save_dto: UserPlatformSaveDTO = resp.clone().into();
                        save_dto.user_id = Some(user.id);
                        user_platform_service::insert(db, &save_dto).await?;
                        log::info!("[微信手机号登录] 绑定微信平台信息成功，user_id={}", user.id);
                    }
                }
            }
        }
    }

    // 生成token
    if let Some(uid) = user_id {
        let jwt_token = JWTToken::new(
            Some(uid),
            None,
            vec![],
            Some("mxx_B2B_user"),
        );
        
        let token_str = jwt_token.create_token(&jwt_secret)?;
        let expire_time = jwt_token.exp;
        
        let user_model = UserModel::find_by_id(db, &Some(uid)).await?
            .ok_or(Error::from("用户不存在"))?;
        
        let user_info = UserInfo {
            id: uid,
            nick_name: user_model.nickname.unwrap_or_default(),
            avatar_url: user_model.avatar.unwrap_or_default(),
            gender: 0,
        };
        
        let mut result = serde_json::Map::new();
        result.insert("token".to_string(), serde_json::Value::String(token_str));
        result.insert("expireTime".to_string(), serde_json::Value::Number((expire_time * 1000).into()));
        result.insert("userInfo".to_string(), serde_json::to_value(user_info)?);
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")));
    }

    log::error!("[微信手机号登录] 登录失败");
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "登录失败", "local")))
}

/// 微信平台code登录/注册
/// 通过微信登录code获取用户openid，用于登录或注册平台信息
#[post("/wechat/code-login")]
pub async fn wechat_code_login(state: web::Data<AppState>, item: web::Json<WechatCodeLoginRequest>) -> Result<HttpResponse> {
    let db = &state.db;

    let app_id = config::section::<String>("wechat", "app_id", "".to_string());
    let app_secret = config::section::<String>("wechat", "app_secret", "".to_string());

    if app_id.is_empty() || app_secret.is_empty() {
        log::error!("[微信code登录] 微信配置未设置");
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "微信配置未设置", "local")));
    }

    let client = Client::new();
    let url = format!(
        "https://api.weixin.qq.com/sns/jscode2session?appid={}&secret={}&js_code={}&grant_type=authorization_code",
        app_id, app_secret, item.code
    );

    let response = client.get(&url).send().await.map_err(|e| {
        log::error!("[微信code登录] 微信API请求失败: {:?}", e);
        Error::from(format!("微信API请求失败: {:?}", e))
    })?;

    let wechat_response: WechatAuthResponse = response.json().await.map_err(|e| {
        log::error!("[微信code登录] 微信API响应解析失败: {:?}", e);
        Error::from(format!("微信API响应解析失败: {:?}", e))
    })?;

    if let Some(errcode) = wechat_response.errcode {
        if errcode != 0 {
            let errmsg = wechat_response.errmsg.unwrap_or("未知错误".to_string());
            log::error!("[微信code登录] 微信登录失败: errcode={}, errmsg={}", errcode, errmsg);
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &format!("微信登录失败: {}", errmsg), "local")));
        }
    }

    let openid = wechat_response.openid.clone().ok_or(Error::from("获取openid失败"))?;
    let unionid = wechat_response.unionid.clone();
    let login_type = item.r#type.clone();

    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());

    let mut user_id: Option<i64> = None;
    
    // 优先通过unionid查询
    if let Some(ref uid) = unionid {
        if let Some(platform_model) = UserPlatformModel::find_by_weixin_unionid(db, uid).await? {
            user_id = Some(platform_model.user_id);
        }
    }

    // 通过openid查询
    if user_id.is_none() {
        if let Some(platform_model) = UserPlatformModel::find_by_weixin_openid(db, &openid).await? {
            user_id = Some(platform_model.user_id);
        }
    }

    if user_id.is_none() {
        // 用户不存在，创建新用户、平台记录
        let new_user_id = (*db).transaction(|conn| {
            let user_save_dto = UserSaveDTO {
                id: None,
                username: None,
                nickname: None,
                avatar: None,
                email: None,
                mobile: None,
                loginfailure: Some(0),
                lastlogintime: None,
                lastloginip: None,
                password: None,
                salt: None,
                motto: None,
                status: Some("1".to_string()),
            };
            
            Box::pin(async move {
                // 1. 创建用户
                let new_user_id = user_service::insert(conn, &user_save_dto).await?;

                // 2. 创建微信平台记录
                let mut platform_save_dto: UserPlatformSaveDTO = wechat_response.clone().into();
                platform_save_dto.user_id = Some(new_user_id);
                user_platform_service::insert(conn, &platform_save_dto).await?;

                log::info!("[微信code登录] 创建用户及平台记录成功，user_id={}", new_user_id);

                Ok::<i64, Error>(new_user_id)
            })
        }).await.map_err(|e| {
            log::error!("[微信code登录] 事务执行失败: {:?}", e);
            Error::from(format!("事务执行失败: {:?}", e))
        })?;

        user_id = Some(new_user_id);
    } else {
        // 更新平台信息
        if let Some(ref uid) = unionid {
            if let Some(platform_model) = UserPlatformModel::find_by_weixin_unionid(db, uid).await? {
                let mut save_dto: UserPlatformSaveDTO = wechat_response.clone().into();
                save_dto.id = Some(platform_model.id);
                save_dto.user_id = Some(platform_model.user_id);
                user_platform_service::update_by_id(db, platform_model.id, &save_dto).await?;
            }
        } else if let Some(platform_model) = UserPlatformModel::find_by_weixin_openid(db, &openid).await? {
            let mut save_dto: UserPlatformSaveDTO = wechat_response.clone().into();
            save_dto.id = Some(platform_model.id);
            save_dto.user_id = Some(platform_model.user_id);
            user_platform_service::update_by_id(db, platform_model.id, &save_dto).await?;
        }
    }

    // 生成token
    if let Some(uid) = user_id {
        let jwt_token = JWTToken::new(
            Some(uid),
            None,
            vec![],
            Some("mxx_B2B_user"),
        );
        
        let token_str = jwt_token.create_token(&jwt_secret)?;
        let expire_time = jwt_token.exp;
        
        let user_model = UserModel::find_by_id(db, &Some(uid)).await?
            .ok_or(Error::from("用户不存在"))?;
        
        let user_info = UserInfo {
            id: uid,
            nick_name: user_model.nickname.unwrap_or_default(),
            avatar_url: user_model.avatar.unwrap_or_default(),
            gender: 0,
        };
        
        let mut result = serde_json::Map::new();
        result.insert("token".to_string(), serde_json::Value::String(token_str));
        result.insert("expireTime".to_string(), serde_json::Value::Number((expire_time * 1000).into()));
        result.insert("userInfo".to_string(), serde_json::to_value(user_info)?);
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")));
    }

    log::error!("[微信code登录] 登录失败: 未找到用户信息");
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "登录失败", "local")))
}


#[get("/wechat/user-info")]
pub async fn get_user_info(req: HttpRequest, state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());

    let token_str = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(s) => s.trim_start_matches("Bearer ").to_string(),
            Err(e) => {
                log::error!("[获取用户信息] Authorization header解析失败: {:?}", e);
                return Ok(HttpResponse::Ok()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
            }
        },
        None => {
            log::error!("[获取用户信息] Authorization header不存在");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token不存在，请重新登录", "local")));
        }
    };
    
    let decoded_token = match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(t) => t,
        Err(e) => {
            log::error!("[获取用户信息] token校验失败: {:?}", e);
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效或已过期，请重新登录", "local")));
        }
    };

    if decoded_token.iss != "mxx_B2B_user" {
        log::error!("[获取用户信息] token签发者不匹配");
        return Ok(HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token类型错误，请重新登录", "local")));
    }

    let user_id = match decoded_token.id {
        Some(id) => id,
        None => {
            log::error!("[获取用户信息] token中未包含用户ID");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
        }
    };

    let user_model = match UserModel::find_by_id(db, &Some(user_id)).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::error!("[获取用户信息] 用户不存在, user_id={}", user_id);
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户不存在", "local")));
        }
        Err(e) => {
            log::error!("[获取用户信息] 查询用户信息失败: {:?}", e);
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "查询用户信息失败", "local")));
        }
    };

    let user_info = UserBasicInfoVO {
        id: Some(user_model.id),
        nickname: user_model.nickname,
        avatar: user_model.avatar,
        like_count: Some(0),
        follower_count: Some(0),
        follow_count: Some(0),
        member_level: Some("普通会员".to_string()),
    };

    Ok(HttpResponse::Ok()
        .insert_header(("Content-Type", "application/json;charset=utf-8"))
        .content_type("application/msgpack").body(MetaResp::success(user_info, "local")))
}

#[post("/wechat/update-nickname")]
pub async fn update_nickname(req: HttpRequest, state: web::Data<AppState>, item: web::Json<UpdateNicknameRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());

    let token_str = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(s) => s.trim_start_matches("Bearer ").to_string(),
            Err(e) => {
                log::error!("[修改昵称] Authorization header解析失败: {:?}", e);
                return Ok(HttpResponse::Ok()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
            }
        },
        None => {
            log::error!("[修改昵称] Authorization header不存在");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token不存在，请重新登录", "local")));
        }
    };

    let decoded_token = match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(t) => t,
        Err(e) => {
            log::error!("[修改昵称] token校验失败: {:?}", e);
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效或已过期，请重新登录", "local")));
        }
    };

    if decoded_token.iss != "mxx_B2B_user" {
        log::error!("[修改昵称] token签发者不匹配");
        return Ok(HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token类型错误，请重新登录", "local")));
    }

    let user_id = match decoded_token.id {
        Some(id) => id,
        None => {
            log::error!("[修改昵称] token中未包含用户ID");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
        }
    };

    let user_model = match UserModel::find_by_id(db, &Some(user_id)).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::error!("[修改昵称] 用户不存在, user_id={}", user_id);
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户不存在", "local")));
        }
        Err(e) => {
            log::error!("[修改昵称] 查询用户信息失败: {:?}", e);
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "查询用户信息失败", "local")));
        }
    };

    let save_dto = UserSaveDTO {
        id: Some(user_model.id),
        username: user_model.username,
        nickname: Some(item.nickname.clone()),
        avatar: user_model.avatar,
        email: user_model.email,
        mobile: user_model.mobile,
        loginfailure: user_model.loginfailure,
        lastlogintime: None,
        lastloginip: user_model.lastloginip,
        password: user_model.password,
        salt: user_model.salt,
        motto: user_model.motto,
        status: user_model.status,
    };

    match user_service::update_by_id(db, &save_dto).await {
        Ok(_) => {
            log::info!("[修改昵称] 用户昵称修改成功, user_id={}, nickname={}", user_id, item.nickname);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("修改成功".to_string(), "local")))
        }
        Err(e) => {
            log::error!("[修改昵称] 用户昵称修改失败: {:?}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "修改失败", "local")))
        }
    }
}

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

fn b64_msgpack_response(bytes: Vec<u8>) -> HttpResponse {
    let b64 = STANDARD.encode(&bytes);
    HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(b64)
}

/// 文件上传请求结构体
#[derive(Debug, MultipartForm)]
pub struct AvatarUploadRequest {
    #[multipart(rename = "avatar")]
    pub avatar: TempFile,
}

#[post("/wechat/update-avatar")]
pub async fn update_avatar(req: HttpRequest, state: web::Data<AppState>, MultipartForm(form): MultipartForm<AvatarUploadRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    
    log::info!("[修改头像] 接收到头像上传请求");
    
    // 验证用户登录状态并获取用户ID
    let user_id = match base_controller::get_user_client_id(&req) {
        Ok(id) => {
            log::info!("[修改头像] 用户ID: {}", id);
            id
        }
        Err(e) => {
            log::error!("[修改头像] 获取用户ID失败: {}", e);
            return Ok(b64_msgpack_response(MetaResp::<String>::fail(400, "用户未登录", "local")));
        }
    };
    
    // 查询用户信息获取注册日期
    let user = match UserModel::find_by_id(db, &Some(user_id)).await
        .map_err(|e| { log::error!("[修改头像] 查询用户失败: {}", e); e })?
    {
        Some(u) => u,
        None => {
            log::error!("[修改头像] 用户不存在, user_id={}", user_id);
            return Ok(b64_msgpack_response(MetaResp::<String>::fail(400, "用户不存在", "local")));
        }
    };
    
    // 检查表单是否有文件
    if form.avatar.file_name.is_none() {
        log::error!("[修改头像] 表单中没有文件字段");
        return Ok(b64_msgpack_response(MetaResp::<String>::fail(400, "表单中没有文件字段", "local")));
    }
    
    // 获取文件名
    let file_name = &form.avatar.file_name.clone().unwrap_or_else(|| "".to_string());
    log::info!("[修改头像] 文件名: {}", file_name);
    
    if file_name.is_empty() {
        log::error!("[修改头像] 文件名为空");
        return Ok(b64_msgpack_response(MetaResp::<String>::fail(400, "没有获取到文件名，上传失败", "local")));
    }
    
    // 检查文件扩展名
    let ext = get_extension(file_name.as_str());
    log::info!("[修改头像] 文件扩展名: {}", ext);
    
    if !is_image(ext.clone()) {
        log::error!("[修改头像] 文件类型不正确，扩展名: {}", ext);
        return Ok(b64_msgpack_response(MetaResp::<String>::fail(400, "请上传正确的图片类型（png, jpg, jpeg, gif, bmp, svg）", "local")));
    }
    
    // 检查文件大小
    let file_size = fs::metadata(&form.avatar.file.path()).map(|m| m.len()).unwrap_or(0);
    log::info!("[修改头像] 文件大小: {} bytes", file_size);
    
    if file_size == 0 {
        log::error!("[修改头像] 文件为空，大小为0");
        return Ok(b64_msgpack_response(MetaResp::<String>::fail(400, "上传的文件为空", "local")));
    }
    
    // 从用户注册日期提取年月日
    let (year, month, day) = match user.create_time {
        Some(dt) => {
            (dt.year(), dt.month(), dt.day())
        }
        None => {
            // 如果没有注册日期，使用当前日期
            let now = time_utils::now();
            (now.year(), now.month(), now.day())
        }
    };
    
    // 构建目录结构：avatar/年/月/日/
    let directory = format!("avatar/{}/{:02}/{:02}", year, month, day);
    
    // 构建文件名：头像_用户id.后缀
    let name = format!("avatar_{}.{}", user_id, ext);
    
    // 构建完整路径和URL
    let base_path = config::section::<String>("attach", "upload_path", "./storage/upload/".to_string());
    let base_url = config::section::<String>("attach", "upload_url", "/upload/".to_string());
    let path = format!("{}{}/{}", base_path, directory, name);
    let url = format!("{}{}/{}", base_url, directory, name);
    
    // 获取存储类型配置（使用统一的上传服务，带默认回退）
    let storage_cfg = upload_service::get_storage_config(db).await
        .unwrap_or_else(|_| upload_service::StorageConfig {
            storage_type: StorageType::Local,
            storage_url: String::new(),
        });
    
    // 上传到本地存储
    match &storage_cfg.storage_type {
        StorageType::Local => {
            // 删除旧头像文件（如果存在）
            if Path::new(&path).exists() {
                if let Err(e) = fs::remove_file(&path) {
                    log::warn!("[修改头像] 删除旧头像失败: {}", e);
                }
            }
            if let Err(e) = upload_to_local_storage(&form.avatar, &Some(path.clone())).await {
                log::error!("[修改头像] 保存头像文件失败: path={}, err={}", path, e);
                return Ok(b64_msgpack_response(MetaResp::<String>::fail(400, "保存文件失败", "local")));
            }
        },
        StorageType::Qiniu => {
            return Ok(b64_msgpack_response(MetaResp::<String>::fail(400, "七牛云存储暂未实现", "local")));
        },
        StorageType::Aliyun => {
            return Ok(b64_msgpack_response(MetaResp::<String>::fail(400, "阿里云存储暂未实现", "local")));
        },
        StorageType::Tencent => {
            return Ok(b64_msgpack_response(MetaResp::<String>::fail(400, "腾讯云存储暂未实现", "local")));
        },
    }
    
    // 更新用户头像字段
    let static_url = config::section::<String>("attach", "static_url", "https://static.s88.cn/".to_string());
    let avatar_url = format!("{}{}", storage_cfg.storage_url, url);
    let display_url = if storage_cfg.storage_url.is_empty() {
        format!("{}{}", static_url, url)
    } else {
        avatar_url.clone()
    };
    
    match UserModel::update_avatar(db, user_id, Some(avatar_url)).await {
        Err(e) => {
            log::error!("[修改头像] 更新用户头像失败: {:?}", e);
            if fs::remove_file(&path).is_err() {
                log::warn!("[修改头像] 删除已上传文件失败");
            }
            return Ok(b64_msgpack_response(MetaResp::<String>::fail(400, "更新头像失败", "local")));
        }
        Ok(_) => {
            log::info!("[修改头像] 头像上传成功, user_id={}, url={}", user_id, display_url);
            
            let result = UploadImageResult {
                file_name: name,
                url: display_url,
                id: user_id.to_string(),
            };
            
            Ok(b64_msgpack_response(MetaResp::success(result, "local")))
        }
    }
}

/// # 上传到本地存储
async fn upload_to_local_storage(file: &TempFile, save_path: &Option<String>) -> Result<bool> {
    if let Some(path) = save_path {
        if path.trim().is_empty() {
            return Err(Error::from("保存路径不能为空"));
        }
        
        // 确保目录存在
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| Error::from(format!("创建目录失败: {}", e)))?;
            }
        }
        
        // 复制文件
        fs::copy(&file.file, path).map_err(|e| Error::from(format!("文件复制失败: {}", e)))?;
        
        // 验证文件是否存在
        if !Path::new(path).exists() {
            return Err(Error::from("上传失败"));
        }
        
        // Linux下设置权限
        if cfg!(target_os = "linux") {
            let target_path = &config::section::<String>("attach", "upload_path", "".to_string());
            if let Ok(metadata) = fs::metadata(target_path) {
                let permissions = metadata.permissions();
                let _ = fs::set_permissions(path, permissions);
            }
        }
        
        Ok(true)
    } else {
        Err(Error::from("保存路径不能为空"))
    }
}

/// # 获取文件扩展名
fn get_extension(filename: &str) -> String {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_string()
}

/// # 检查是否为图片类型
fn is_image(extension: String) -> bool {
    matches!(extension.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg")
}

/// # 获取上传存储URL
async fn upload_storage_url(db: &sea_orm::DatabaseConnection, storage_type: &Option<i32>) -> Result<String> {
    match storage_type {
        Some(1) => Ok(config::section::<String>("attach", "static_url", "https://static.s88.cn/".to_string())),
        Some(2) => {
            let config_detail = config_service::select_by_key(db, &"qiniuDomain".to_string()).await?;
            Ok(config_detail.config_value.unwrap_or_else(|| "".to_string()))
        },
        Some(3) => {
            let config_detail = config_service::select_by_key(db, &"aliyunDomain".to_string()).await?;
            Ok(config_detail.config_value.unwrap_or_else(|| "".to_string()))
        },
        Some(4) => {
            let config_detail = config_service::select_by_key(db, &"tencentDomain".to_string()).await?;
            Ok(config_detail.config_value.unwrap_or_else(|| "".to_string()))
        },
        _ => Ok(config::section::<String>("attach", "static_url", "https://static.s88.cn/".to_string())),
    }
}

#[post("/wechat/refresh-token")]
pub async fn refresh_token(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());

    let token_str = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(s) => s.trim_start_matches("Bearer ").to_string(),
            Err(e) => {
                log::error!("[Token刷新] Authorization header解析失败: {:?}", e);
                return Ok(HttpResponse::Ok()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效", "local")));
            }
        },
        None => {
            log::error!("[Token刷新] Authorization header不存在");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token不存在，请重新登录", "local")));
        }
    };

    let decoded_token = match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(t) => t,
        Err(e) => {
            log::error!("[Token刷新] token校验失败: {:?}", e);
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效或已过期，请重新登录", "local")));
        }
    };
    
    if decoded_token.iss != "mxx_B2B_user" {
        log::error!("[Token刷新] token签发者不匹配");
        return Ok(HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token类型错误，请重新登录", "local")));
    }
    
    let user_id = match decoded_token.id {
        Some(id) => id,
        None => {
            log::error!("[Token刷新] token中未包含用户ID");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
        }
    };
    
    let user_model: crate::modules::user::entity::user::Model = match UserModel::find_by_id(db, &Some(user_id)).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::error!("[Token刷新] 用户不存在, user_id={}", user_id);
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户不存在，请重新登录", "local")));
        }
        Err(e) => {
            log::error!("[Token刷新] 查询用户信息失败: {:?}", e);
            return Ok(HttpResponse::Ok()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "服务器内部错误", "local")));
        }
    };
    
    let jwt_token = JWTToken::new(
        Some(user_model.id),
        user_model.username.clone(),
        vec![],
        Some("mxx_B2B_user"),
    );
    
    let new_token = jwt_token.create_token(&jwt_secret)?;
    let expire_time = jwt_token.exp;
    
    let user_info = UserInfo {
        id: user_model.id,
        nick_name: user_model.nickname.unwrap_or_default(),
        avatar_url: user_model.avatar.unwrap_or_default(),
        gender: 0,
    };
    
    let mut result = serde_json::Map::new();
    result.insert("token".to_string(), serde_json::Value::String(new_token));
    result.insert("expireTime".to_string(), serde_json::Value::Number(expire_time.into()));
    result.insert("userInfo".to_string(), serde_json::to_value(user_info)?);
    
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}
