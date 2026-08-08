//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 系统设置与会话管理控制器（v1.1）
//!
//! 提供：
//! - 系统设置读取/保存（多设备模式、会话超时、并发设备数、注册开关）
//! - 在线会话列表（扫描缓存中的登录会话）
//! - 按会话踢人（从用户 token 集合中移除单个 token）
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_user;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::core::kit::jwt_util::JWTToken;
use crate::modules::system::service::{admin_service, config_service, permission_cache_service};

/// 系统设置保存请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingConfigUpdate {
    /// 是否允许多设备同时登录
    pub multi_device: Option<bool>,
    /// 会话超时（小时），由前端传入
    pub session_timeout: Option<i64>,
    /// 多设备模式最大在线设备数，0=不限制
    pub max_devices: Option<i64>,
    /// 是否开放员工注册
    pub register_enabled: Option<bool>,
}

/// 系统设置返回结构
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingConfigVO {
    pub multi_device: bool,
    /// 会话超时（小时）
    pub session_timeout: i64,
    pub max_devices: i64,
    /// 是否开放员工注册
    pub register_enabled: bool,
}

/// 在线会话返回结构
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionVO {
    pub user_id: i64,
    pub user_name: String,
    pub token: String,
    /// token 到期时间（Unix 秒，从 JWT exp 解析；无法解析则为 0）
    pub token_expire: i64,
    /// 是否当前管理员自己的会话（前端据此禁用"下线"按钮）
    pub current: bool,
}

/// 读取系统设置
///
/// 权限：system:setting:list
pub async fn get_setting_config(state: web::Data<AppState>) -> Result<HttpResponse, actix_web::Error> {
    let multi_device = permission_cache_service::is_multi_device_mode().await;
    // 会话超时：优先缓存配置（秒），转小时返回
    let timeout_secs = permission_cache_service::get_session_timeout_secs().await;
    let max_devices = permission_cache_service::get_max_devices().await as i64;
    let register_enabled = permission_cache_service::is_register_enabled().await;

    let config = SettingConfigVO {
        multi_device,
        session_timeout: (timeout_secs as i64) / 3600,
        max_devices,
        register_enabled,
    };

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(config, "local")))
}

/// 保存系统设置
///
/// 权限：system:setting:update
/// 将配置写入数据库 + 缓存，登录与请求中间件从缓存读取实现运行时即时生效。
pub async fn update_setting_config(
    state: web::Data<AppState>,
    item: web::Json<SettingConfigUpdate>,
) -> Result<HttpResponse, actix_web::Error> {
    let db = &state.db;
    let item = item.into_inner();

    // 1. 多设备登录开关
    if let Some(multi) = item.multi_device {
        let value = if multi { "1" } else { "0" };
        if let Err(e) = config_service::update_value_by_key(db, "login_multi_device", value).await {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &format!("保存失败: {}", e), "local")));
        }
        let _ = crate::core::kit::CONTEXT.cache_service.set_string("config:login_multi_device", value).await;
    }

    // 2. 会话超时（小时 -> 秒）落库 + 缓存
    if let Some(hours) = item.session_timeout {
        if hours > 0 {
            let seconds = (hours * 3600).to_string();
            if let Err(e) = config_service::update_value_by_key(db, "session_timeout", &seconds).await {
                return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &format!("保存失败: {}", e), "local")));
            }
            let _ = crate::core::kit::CONTEXT.cache_service.set_string("config:session_timeout", &seconds).await;
        }
    }

    // 3. 最大并发设备数
    if let Some(max) = item.max_devices {
        if max >= 0 {
            let value = max.to_string();
            if let Err(e) = config_service::update_value_by_key(db, "login_max_devices", &value).await {
                return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &format!("保存失败: {}", e), "local")));
            }
            let _ = crate::core::kit::CONTEXT.cache_service.set_string("config:login_max_devices", &value).await;
        }
    }

    // 4. 员工注册开关
    if let Some(enabled) = item.register_enabled {
        let value = if enabled { "1" } else { "0" };
        if let Err(e) = config_service::update_value_by_key(db, "register_enabled", value).await {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &format!("保存失败: {}", e), "local")));
        }
        let _ = crate::core::kit::CONTEXT.cache_service.set_string("config:register_enabled", value).await;
    }

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("保存成功".to_string(), "local")))
}

/// 在线会话列表
///
/// 权限：system:setting:list
/// 扫描缓存中的登录会话（单设备 user_* + 多设备 user_tokens_*），联查用户名。
pub async fn session_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let db = &state.db;
    // 当前管理员 id（用于标记 current）
    let current_admin: JWTToken = get_user(&req).unwrap_or_default();
    let current_uid = current_admin.id.unwrap_or_default();

    let sessions = permission_cache_service::list_online_sessions().await;
    let admin_secret = crate::core::kit::config::section::<String>("server", "jwt_secret_admin", "".to_string());

    let mut list: Vec<SessionVO> = Vec::new();
    for (user_id, tokens) in sessions {
        // 联查用户名
        let user_name = match admin_service::get_by_detail(db, &Some(user_id)).await {
            Ok(a) => a.user_name.unwrap_or_default(),
            Err(_) => String::new(),
        };
        for token in tokens {
            // 从 token 解析到期时间（解析失败则置 0）
            let token_expire = JWTToken::verify(&admin_secret, &token)
                .ok()
                .map(|t| t.exp as i64)
                .unwrap_or_default();
            list.push(SessionVO {
                user_id,
                user_name: user_name.clone(),
                token,
                token_expire,
                current: user_id == current_uid,
            });
        }
    }

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(list, "local")))
}

/// 按会话踢人（移除单个 token）
///
/// 权限：system:setting:list（与查看同一入口）
/// 路径参数：{user_id}/{token}
pub async fn kick_session(
    state: web::Data<AppState>,
    path: web::Path<(i64, String)>,
) -> Result<HttpResponse, actix_web::Error> {
    let _db = &state.db;
    let (user_id, token) = path.into_inner();

    // 不可踢超级管理员
    if user_id == 1 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "不能下线超级管理员的会话", "local")));
    }

    let removed = permission_cache_service::invalidate_session_by_token(user_id, &token).await;
    if removed {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("已强制下线".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "会话不存在或已失效", "local")))
    }
}

/// 路由注册
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/setting")
            // GET /setting/config - 读取系统设置
            .route(
                "/config",
                web::get()
                    .to(get_setting_config)
                    .wrap(require_permission("system:setting:list")),
            )
            // PUT /setting/config - 保存系统设置
            .route(
                "/config",
                web::put()
                    .to(update_setting_config)
                    .wrap(require_permission("system:setting:update")),
            ),
    );

    // 在线会话管理（独立 scope，复用 system:setting:list 权限）
    cfg.service(
        web::scope("/session")
            // GET /session/list - 在线会话列表
            .route(
                "/list",
                web::get()
                    .to(session_list)
                    .wrap(require_permission("system:setting:list")),
            )
            // POST /session/kick/{user_id}/{token} - 按会话踢人
            .route(
                "/kick/{user_id}/{token}",
                web::post()
                    .to(kick_session)
                    .wrap(require_permission("system:setting:list")),
            ),
    );
}
