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
use crate::modules::system::service::menu_service;

/// 权限缓存键前缀
const PERM_KEY_PREFIX: &str = "perm:";

/// 用户Token缓存键前缀
const USER_TOKEN_KEY_PREFIX: &str = "user_";

/// 默认缓存TTL（秒）
const DEFAULT_PERM_CACHE_TTL: u64 = 300;

/// 缓存TTL（启动后只读一次配置，避免每次写入都触发配置未找到的warn日志）
static PERM_CACHE_TTL: OnceLock<u64> = OnceLock::new();

/// 构建权限缓存的键
fn perm_key(user_id: i64) -> String {
    format!("{}{}", PERM_KEY_PREFIX, user_id)
}

/// 构建用户Token缓存的键
fn user_token_key(user_id: i64) -> String {
    format!("{}{}", USER_TOKEN_KEY_PREFIX, user_id)
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

/// 清除用户的登录会话（Token + 权限缓存）
///
/// 调用场景：用户被禁用、删除、密码重置
/// 效果：用户下次请求时 token 验证失败 → 401 → 前端跳转登录页
pub async fn invalidate_user_session(user_id: i64) {
    // 清除权限缓存
    invalidate_by_user_id(user_id).await;
    // 清除Token缓存（使旧token失效）
    let token_key = user_token_key(user_id);
    if let Err(e) = CONTEXT.cache_service.del(&token_key).await {
        log::warn!("[权限缓存] 清除Token失败 user_id={}, err={}", user_id, e);
    }
    log::info!("[权限缓存] 用户会话已清除 user_id={}", user_id);
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
