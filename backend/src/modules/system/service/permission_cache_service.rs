//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 权限缓存服务
//!
//! 提供「用户权限码集合」的缓存读写能力，支持 mem/redis 双模式。
//! 缓存键格式：`perm:{user_id}`，值为 JSON `["perm1","perm2",...]`。
//!
//! ## 核心函数
//!
//! - [`get_or_load_permissions`]：优先从缓存读取权限码，缓存miss时回查DB并回填
//! - [`set_permissions`]：将权限码写入缓存（登录时调用）
//! - [`invalidate_by_user_id`]：清除单个用户的权限缓存
//! - [`invalidate_by_user_ids`]：批量清除用户权限缓存
//! - [`invalidate_by_role_id`]：清除角色关联的所有用户权限缓存
//!

use std::sync::OnceLock;
use std::time::Duration;

use sea_orm::DbConn;

use crate::core::errors::error::Result;
use crate::core::kit::config;
use crate::core::kit::CONTEXT;
use crate::modules::system::model::admin_role_merge::AdminRoleMergeModel;
use crate::modules::system::service::config_service;
use crate::modules::system::service::menu_service;

/// 权限缓存键前缀
const PERM_KEY_PREFIX: &str = "perm:";

/// 用户Token缓存键前缀（单设备模式）
const USER_TOKEN_KEY_PREFIX: &str = "user_";

/// 多设备模式用户 Token 集合缓存键前缀
const USER_TOKENS_KEY_PREFIX: &str = "user_tokens_";

/// 默认缓存TTL（秒）
const DEFAULT_PERM_CACHE_TTL: u64 = 300;

/// 缓存TTL（启动后只读一次配置，避免每次写入都触发配置未找到的warn日志）
static PERM_CACHE_TTL: OnceLock<u64> = OnceLock::new();

/// 构建权限缓存的键
fn perm_key(user_id: i64) -> String {
    format!("{}{}", PERM_KEY_PREFIX, user_id)
}

/// 构建用户Token缓存的键（单设备模式）
fn user_token_key(user_id: i64) -> String {
    format!("{}{}", USER_TOKEN_KEY_PREFIX, user_id)
}

/// 构建多设备模式用户 Token 集合缓存的键
fn user_tokens_key(user_id: i64) -> String {
    format!("{}{}", USER_TOKENS_KEY_PREFIX, user_id)
}

/// 读取登录模式配置：false=单设备（默认），true=多设备
///
/// 优先从缓存读取，缓存未命中时回查数据库 mxx_system_config。
/// 全部由数据库控制，在线设置实时生效。
pub async fn is_multi_device_mode() -> bool {
    match CONTEXT.cache_service.get_string("config:login_multi_device").await {
        Ok(v) if !v.is_empty() => v == "1",
        _ => {
            let db_val = config_service::find_value_by_key_from_db("login_multi_device").await.unwrap_or_else(|| "0".to_string());
            let _ = CONTEXT.cache_service.set_string("config:login_multi_device", &db_val).await;
            db_val == "1"
        }
    }
}

/// 读取会话超时配置（秒），优先缓存，miss 时回查数据库
pub async fn get_session_timeout_secs() -> u64 {
    match CONTEXT.cache_service.get_string("config:session_timeout").await {
        Ok(v) if !v.is_empty() => v.parse::<u64>().unwrap_or(28800),
        _ => {
            let db_val = config_service::find_value_by_key_from_db("session_timeout").await.unwrap_or_else(|| "28800".to_string());
            let _ = CONTEXT.cache_service.set_string("config:session_timeout", &db_val).await;
            db_val.parse::<u64>().unwrap_or(28800)
        }
    }
}

/// 读取 accessToken 有效期配置（秒），优先缓存，miss 时回查数据库
///
/// 整改 v1.0 新增：JWT（accessToken）过期时间与 refreshToken（会话）过期时间分离，
/// accessToken 短期（默认 7200 秒），refreshToken 走 session_timeout 滑动续期。
pub async fn get_access_token_expire_secs() -> u64 {
    match CONTEXT.cache_service.get_string("config:access_token_expire").await {
        Ok(v) if !v.is_empty() => v.parse::<u64>().unwrap_or(7200),
        _ => {
            let db_val = config_service::find_value_by_key_from_db("access_token_expire").await.unwrap_or_else(|| "7200".to_string());
            let _ = CONTEXT.cache_service.set_string("config:access_token_expire", &db_val).await;
            db_val.parse::<u64>().unwrap_or(7200)
        }
    }
}

/// 读取多设备模式最大在线设备数，0=不限制
pub async fn get_max_devices() -> usize {
    match CONTEXT.cache_service.get_string("config:login_max_devices").await {
        Ok(v) if !v.is_empty() => v.parse::<usize>().unwrap_or(5),
        _ => {
            let db_val = config_service::find_value_by_key_from_db("login_max_devices").await.unwrap_or_else(|| "5".to_string());
            let _ = CONTEXT.cache_service.set_string("config:login_max_devices", &db_val).await;
            db_val.parse::<usize>().unwrap_or(5)
        }
    }
}

/// 读取员工注册开关：true=开放注册，false=关闭（默认）
pub async fn is_register_enabled() -> bool {
    match CONTEXT.cache_service.get_string("config:register_enabled").await {
        Ok(v) if !v.is_empty() => v == "1",
        _ => {
            let db_val = config_service::find_value_by_key_from_db("register_enabled").await.unwrap_or_else(|| "0".to_string());
            let _ = CONTEXT.cache_service.set_string("config:register_enabled", &db_val).await;
            db_val == "1"
        }
    }
}

/// 获取缓存TTL（秒），首次调用读配置，后续直接返回缓存值
fn cache_ttl() -> Duration {
    let ttl = *PERM_CACHE_TTL.get_or_init(|| {
        config::section::<u64>("server", "permission_cache_ttl", DEFAULT_PERM_CACHE_TTL)
    });
    Duration::from_secs(ttl)
}

/// 读取用户权限码集合：优先从缓存读取，缓存miss时回查DB并回填
///
/// 调用场景：`extract` 中间件每次请求调用
///
/// 性能：
/// - 缓存命中：mem ~0.001ms / redis ~0.3ms
/// - 缓存miss：1次DB查询（查询后自动回填缓存）
pub async fn get_or_load_permissions(db: &DbConn, user_id: i64) -> Vec<String> {
    if user_id <= 0 {
        return vec![];
    }

    let key = perm_key(user_id);

    // 尝试从缓存读取
    match CONTEXT.cache_service.get_json::<Vec<String>>(&key).await {
        Ok(perms) if !perms.is_empty() => {
            log::debug!("[权限缓存] 命中 user_id={}, perms={}", user_id, perms.len());
            return perms;
        }
        Ok(_) => {
            // 空数组或null，继续走DB查询
        }
        Err(e) => {
            log::warn!("[权限缓存] 读取失败 user_id={}, err={}", user_id, e);
        }
    }

    // 缓存miss，回查DB
    log::debug!("[权限缓存] 未命中 user_id={}, 回查DB", user_id);

    // 查询用户信息，判断是否超级管理员 + 检查禁用状态
    let admin = match crate::modules::system::service::admin_service::get_by_detail(db, &Some(user_id)).await {
        Ok(admin) => admin,
        Err(e) => {
            log::error!("[权限缓存] 查询用户失败 user_id={}, err={}", user_id, e);
            return vec![];
        }
    };

    // v2.0: 用户被禁用（status != 1）时返回空权限，不缓存，下次请求仍走DB查询
    if admin.status != Some(1) {
        log::warn!("[权限缓存] 用户已禁用 user_id={}, status={:?}", user_id, admin.status);
        return vec![];
    }

    let is_admin = admin.user_type == Some(1);

    let permissions = match menu_service::find_user_role_keys(db, &is_admin, &Some(user_id)).await {
        Ok(perms) => perms,
        Err(e) => {
            log::error!("[权限缓存] DB查询失败 user_id={}, err={}", user_id, e);
            return vec![];
        }
    };

    // 回填缓存
    if let Err(e) = set_permissions(user_id, &permissions).await {
        log::warn!("[权限缓存] 回填失败 user_id={}, err={}", user_id, e);
    }

    permissions
}

/// 将权限码写入缓存（登录时调用）
pub async fn set_permissions(user_id: i64, permissions: &[String]) -> Result<()> {
    let key = perm_key(user_id);
    CONTEXT.cache_service.set_string_ex(&key, &serde_json::to_string(permissions).unwrap_or_default(), Some(cache_ttl())).await?;
    Ok(())
}

/// 清除单个用户的权限缓存
pub async fn invalidate_by_user_id(user_id: i64) {
    let key = perm_key(user_id);
    if let Err(e) = CONTEXT.cache_service.del(&key).await {
        log::warn!("[权限缓存] 清除失败 user_id={}, err={}", user_id, e);
    }
}

/// 批量清除用户权限缓存
pub async fn invalidate_by_user_ids(user_ids: &[i64]) {
    for uid in user_ids {
        invalidate_by_user_id(*uid).await;
    }
    log::debug!("[权限缓存] 批量清除完成, count={}", user_ids.len());
}

/// 清除角色关联的所有用户权限缓存
///
/// 调用场景：角色菜单权限变更、角色删除
pub async fn invalidate_by_role_id(db: &DbConn, role_id: i64) {
    let admin_ids = AdminRoleMergeModel::find_admin_ids_by_role_id(db, &Some(role_id))
        .await
        .unwrap_or_default();

    log::debug!("[权限缓存] 角色ID={} 关联用户数={}", role_id, admin_ids.len());
    invalidate_by_user_ids(&admin_ids).await;
}

/// 清除用户的登录会话（Token + 权限缓存 + WebSocket 连接）
///
/// 调用场景：用户被禁用、删除、密码重置、踢下线、改密强制下线
/// 效果：用户下次请求时 token 验证失败 → 401 → 前端跳转登录页；
///       已建立的 WebSocket 连接通过 registry.kick 主动断开
pub async fn invalidate_user_session(user_id: i64) {
    // 清除权限缓存
    invalidate_by_user_id(user_id).await;
    // 清除Token缓存（使旧token失效，单设备模式）
    let token_key = user_token_key(user_id);
    if let Err(e) = CONTEXT.cache_service.del(&token_key).await {
        log::warn!("[权限缓存] 清除Token失败 user_id={}, err={}", user_id, e);
    }
    // 清除多设备模式 Token 集合
    let tokens_key = user_tokens_key(user_id);
    if let Err(e) = CONTEXT.cache_service.del(&tokens_key).await {
        log::warn!("[权限缓存] 清除多设备Token集合失败 user_id={}, err={}", user_id, e);
    }
    // 断开该用户所有 WebSocket 连接（消息模块）
    crate::modules::message::websocket::registry::ConnectionRegistry::global().kick(user_id);
    log::info!("[权限缓存] 用户会话已清除 user_id={}", user_id);
}

/// 按单个 token 踢出会话（多设备模式下从用户 Token 集合中移除指定 token）
///
/// 调用场景：在线会话列表的"按会话下线"操作
/// 返回：true=成功移除，false=token 不在集合中
pub async fn invalidate_session_by_token(user_id: i64, token: &str) -> bool {
    let tokens_key = user_tokens_key(user_id);
    let mut tokens: Vec<String> = match CONTEXT.cache_service.get_json(&tokens_key).await {
        Ok(t) => t,
        Err(_) => return false,
    };
    let before = tokens.len();
    tokens.retain(|t| t != token);
    if tokens.len() == before {
        return false;
    }
    if tokens.is_empty() {
        let _ = CONTEXT.cache_service.del(&tokens_key).await;
    } else {
        let _ = CONTEXT.cache_service.set_json(&tokens_key, &tokens).await;
    }
    // 单设备模式兼容：若存在 user_{id} 且匹配则一并清除
    if let Ok(cached) = CONTEXT.cache_service.get_string(&user_token_key(user_id)).await {
        if cached == token {
            let _ = CONTEXT.cache_service.del(&user_token_key(user_id)).await;
        }
    }
    log::info!("[权限缓存] 按会话踢出 user_id={}, 剩余会话数={}", user_id, tokens.len());
    true
}

/// 扫描在线用户会话，返回 (user_id, token列表) 的集合
///
/// 同时扫描单设备键 `user_*` 与多设备键 `user_tokens_*`。
/// 供安全设置页"在线用户列表"接口使用。
pub async fn list_online_sessions() -> Vec<(i64, Vec<String>)> {
    let mut result: std::collections::HashMap<i64, Vec<String>> = std::collections::HashMap::new();

    // 单设备模式：user_{id}
    if let Ok(keys) = CONTEXT.cache_service.keys(&format!("{}*", USER_TOKEN_KEY_PREFIX)).await {
        for k in keys {
            // 跳过 user_tokens_ 前缀的键（多设备模式）
            if k.starts_with(USER_TOKENS_KEY_PREFIX) {
                continue;
            }
            if let Some(id_str) = k.strip_prefix(USER_TOKEN_KEY_PREFIX) {
                if let Ok(uid) = id_str.parse::<i64>() {
                    if let Ok(token) = CONTEXT.cache_service.get_string(&k).await {
                        if !token.is_empty() {
                            result.entry(uid).or_default().push(token);
                        }
                    }
                }
            }
        }
    }

    // 多设备模式：user_tokens_{id}
    if let Ok(keys) = CONTEXT.cache_service.keys(&format!("{}*", USER_TOKENS_KEY_PREFIX)).await {
        for k in keys {
            if let Some(id_str) = k.strip_prefix(USER_TOKENS_KEY_PREFIX) {
                if let Ok(uid) = id_str.parse::<i64>() {
                    if let Ok(tokens) = CONTEXT.cache_service.get_json::<Vec<String>>(&k).await {
                        if !tokens.is_empty() {
                            result.entry(uid).or_default().extend(tokens);
                        }
                    }
                }
            }
        }
    }

    result.into_iter().collect()
}

/// 验证用户Token是否仍然有效
///
/// 调用场景：`extract` 中间件每次请求调用
/// 返回：true=token有效，false=token已被清除（用户被禁用/删除/退出登录）
pub async fn validate_user_token(user_id: i64, token: &str) -> bool {
    let key = user_token_key(user_id);
    match CONTEXT.cache_service.get_string(&key).await {
        Ok(cached_token) => {
            if cached_token.is_empty() {
                log::debug!("[权限缓存] Token缓存为空 user_id={}", user_id);
                return false;
            }
            // token匹配校验（支持多设备登录时，最新token覆盖旧token）
            if cached_token != token {
                log::debug!("[权限缓存] Token不匹配 user_id={}", user_id);
                return false;
            }
            true
        }
        Err(_) => {
            // 缓存中不存在（mem模式可能重启后丢失，或用户被禁用后清除）
            log::debug!("[权限缓存] Token缓存未找到 user_id={}", user_id);
            false
        }
    }
}

/// 统一会话校验（自动适配单/多设备模式）
///
/// - 单设备模式：校验 `user_{id}` 是否等于 token
/// - 多设备模式：校验 token 是否在 `user_tokens_{id}` 集合中
///
/// 调用场景：extract 中间件、WS 心跳
/// 返回：false 时请求方应返回 401 / 断开连接
///
/// 降级策略：mem 缓存模式重启后缓存丢失，缓存未命中时回查数据库 session 表，
/// DB 验证通过则回填缓存，确保重启不丢登录态。
pub async fn validate_session(user_id: i64, token: &str) -> bool {
    validate_session_with_db(user_id, token, None).await
}

/// 带 DB 降级的会话校验（extract 中间件传入 db 连接）
pub async fn validate_session_with_db(user_id: i64, token: &str, db: Option<&DbConn>) -> bool {
    let multi = is_multi_device_mode().await;

    let cached_ok = if multi {
        // 多设备模式：检查 token 是否在集合中
        let key = user_tokens_key(user_id);
        match CONTEXT.cache_service.get_json::<Vec<String>>(&key).await {
            Ok(tokens) => {
                if tokens.is_empty() {
                    false
                } else {
                    tokens.iter().any(|t| t == token)
                }
            }
            Err(_) => false,
        }
    } else {
        // 单设备模式：精确匹配
        validate_user_token(user_id, token).await
    };

    if cached_ok {
        return true;
    }

    // 缓存未命中（mem 模式重启后丢失），降级查数据库 session 表
    if let Some(db) = db {
        log::debug!("[权限缓存] 缓存未命中 user_id={}, 降级查 DB session", user_id);
        match crate::modules::system::service::session_service::get_session_store()
            .validate_session(db, user_id, token)
            .await
        {
            Ok(true) => {
                // DB 验证通过，回填缓存避免下次仍走 DB
                if multi {
                    let key = user_tokens_key(user_id);
                    let mut tokens: Vec<String> = CONTEXT.cache_service.get_json(&key).await.unwrap_or_default();
                    if !tokens.iter().any(|t| t == token) {
                        tokens.push(token.to_string());
                    }
                    let _ = CONTEXT.cache_service.set_json(&key, &tokens).await;
                } else {
                    let _ = CONTEXT.cache_service.set_string(&user_token_key(user_id), token).await;
                }
                true
            }
            _ => false,
        }
    } else {
        false
    }
}
