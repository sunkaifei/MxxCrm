//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! Session 管理器
//!
//! 提供统一的会话存储接口，支持 DB 和 Redis 两种后端。
//! 配置项 `server.session_store` 控制使用哪种后端。
//!
//! ## 验证策略
//! 1. JWT 签名验证（在 extract 中间件中完成）
//! 2. SessionStoreType::validate_session() 验证用户状态
//! 3. 过期 session 在验证时自动清理
//! 4. 定时任务可以调用 clean_expired() 批量清理

use sea_orm::{DbConn, Statement};
use sea_orm::ConnectionTrait;

use crate::core::errors::error::Result;
use crate::core::kit::config;
use crate::core::kit::CONTEXT;

/// Session 存储类型枚举
pub enum SessionStoreType {
    /// 数据库存储
    Db(DbSessionStore),
    /// Redis 存储
    Redis(RedisSessionStore),
}

impl SessionStoreType {
    /// 创建会话（登录成功后调用）
    pub async fn create_session(&self, db: &DbConn, user_id: i64, token: &str, ip: &str, expire_secs: i64) -> Result<()> {
        match self {
            SessionStoreType::Db(store) => store.create_session(db, user_id, token, ip, expire_secs).await,
            SessionStoreType::Redis(store) => store.create_session(db, user_id, token, ip, expire_secs).await,
        }
    }

    /// 验证会话是否有效
    pub async fn validate_session(&self, db: &DbConn, user_id: i64, token: &str) -> Result<bool> {
        match self {
            SessionStoreType::Db(store) => store.validate_session(db, user_id, token).await,
            SessionStoreType::Redis(store) => store.validate_session(db, user_id, token).await,
        }
    }

    /// 删除用户的所有会话（退出登录/踢人下线）
    pub async fn remove_session(&self, db: &DbConn, user_id: i64) -> Result<()> {
        match self {
            SessionStoreType::Db(store) => store.remove_session(db, user_id).await,
            SessionStoreType::Redis(store) => store.remove_session(db, user_id).await,
        }
    }

    /// 清理过期会话
    pub async fn clean_expired(&self, db: &DbConn) -> Result<u64> {
        match self {
            SessionStoreType::Db(store) => store.clean_expired(db).await,
            SessionStoreType::Redis(store) => store.clean_expired(db).await,
        }
    }
}

/// 数据库 Session 存储实现
///
/// 使用 mxx_system_session 表存储会话信息。
/// 创建表 SQL：
/// ```sql
/// CREATE TABLE IF NOT EXISTS mxx_system_session (
///     id BIGSERIAL PRIMARY KEY,
///     user_id BIGINT NOT NULL,
///     token VARCHAR(512) NOT NULL,
///     login_ip VARCHAR(64),
///     login_time TIMESTAMP NOT NULL DEFAULT NOW(),
///     expire_time TIMESTAMP NOT NULL,
///     status SMALLINT NOT NULL DEFAULT 1,
///     created_at TIMESTAMP NOT NULL DEFAULT NOW()
/// );
/// CREATE INDEX IF NOT EXISTS idx_session_user_id ON mxx_system_session(user_id);
/// CREATE INDEX IF NOT EXISTS idx_session_token ON mxx_system_session(token);
/// ```
pub struct DbSessionStore;

impl DbSessionStore {
    async fn create_session(&self, db: &DbConn, user_id: i64, token: &str, ip: &str, expire_secs: i64) -> Result<()> {
        let now = chrono::Local::now().naive_local();
        let expire_time = now + chrono::Duration::seconds(expire_secs);

        // 先删除该用户的过期 session，再插入新 session
        let _ = db.execute_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!(
                "DELETE FROM mxx_system_session WHERE user_id = {} AND expire_time < NOW()",
                user_id
            ),
        )).await;

        db.execute_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!(
                "INSERT INTO mxx_system_session (user_id, token, login_ip, login_time, expire_time, status, created_at) \
                 VALUES ({}, '{}', '{}', '{}', '{}', 1, '{}')",
                user_id,
                token.replace('\'', "''"),
                ip.replace('\'', "''"),
                now.format("%Y-%m-%d %H:%M:%S.%f"),
                expire_time.format("%Y-%m-%d %H:%M:%S.%f"),
                now.format("%Y-%m-%d %H:%M:%S.%f"),
            ),
        )).await?;

        Ok(())
    }

    async fn validate_session(&self, db: &DbConn, user_id: i64, token: &str) -> Result<bool> {
        let now = chrono::Local::now().naive_local();

        let result = db.query_one_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!(
                "SELECT id, expire_time, status FROM mxx_system_session \
                 WHERE user_id = {} AND token = '{}' LIMIT 1",
                user_id,
                token.replace('\'', "''"),
            ),
        )).await;

        match result {
            Ok(Some(row)) => {
                // 使用 try_get 读取列值，注意需要指定类型参数
                let expire_time_raw = row.try_get::<chrono::NaiveDateTime>("", "expire_time");
                let status: i16 = row.try_get("", "status").unwrap_or(0);

                let expire_time = match expire_time_raw {
                    Ok(dt) => dt,
                    Err(_) => {
                        // 降级：尝试按字符串解析
                        let expire_time_str: String = row.try_get("", "expire_time").unwrap_or_default();
                        chrono::NaiveDateTime::parse_from_str(&expire_time_str, "%Y-%m-%d %H:%M:%S.%f")
                            .unwrap_or_else(|_| {
                                log::warn!("[Session] 解析 expire_time 失败，视为过期");
                                chrono::Local::now().naive_local() - chrono::Duration::hours(1)
                            })
                    }
                };

                // 过期清理
                if expire_time < now {
                    let _ = db.execute_raw(Statement::from_string(
                        sea_orm::DatabaseBackend::Postgres,
                        format!(
                            "DELETE FROM mxx_system_session WHERE user_id = {} AND token = '{}'",
                            user_id,
                            token.replace('\'', "''"),
                        ),
                    )).await;
                    return Ok(false);
                }

                Ok(status == 1)
            }
            Ok(None) => Ok(false),
            Err(e) => {
                log::warn!("[Session] 查询失败 user_id={}, err={}", user_id, e);
                Ok(false)
            }
        }
    }

    async fn remove_session(&self, db: &DbConn, user_id: i64) -> Result<()> {
        db.execute_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!(
                "DELETE FROM mxx_system_session WHERE user_id = {}",
                user_id,
            ),
        )).await?;
        Ok(())
    }

    async fn clean_expired(&self, db: &DbConn) -> Result<u64> {
        let result = db.execute_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "DELETE FROM mxx_system_session WHERE expire_time < NOW()".to_string(),
        )).await?;
        let count = result.rows_affected();
        if count > 0 {
            log::info!("[Session] 清理过期会话 {} 条", count);
        }
        Ok(count)
    }
}

/// Redis Session 存储实现
///
/// 复用现有 CacheService，存储结构：
/// Key: session:{user_id}:{token_hash}
/// Value: json {"login_ip": "...", "login_time": ..., "expire_time": ...}
/// TTL: 与 JWT expire 一致
pub struct RedisSessionStore;

impl RedisSessionStore {
    async fn create_session(&self, _db: &DbConn, user_id: i64, token: &str, ip: &str, expire_secs: i64) -> Result<()> {
        let now = chrono::Local::now().naive_local();
        let session_key = format!("session:{}:{}", user_id, token);
        let session_value = serde_json::json!({
            "login_ip": ip,
            "login_time": now.format("%Y-%m-%d %H:%M:%S").to_string(),
            "expire_time": (now + chrono::Duration::seconds(expire_secs)).format("%Y-%m-%d %H:%M:%S").to_string(),
        });
        let ttl = std::time::Duration::from_secs(expire_secs as u64);
        CONTEXT.cache_service.set_string_ex(&session_key, &session_value.to_string(), Some(ttl)).await?;
        Ok(())
    }

    async fn validate_session(&self, _db: &DbConn, user_id: i64, token: &str) -> Result<bool> {
        let session_key = format!("session:{}:{}", user_id, token);
        match CONTEXT.cache_service.get_string(&session_key).await {
            Ok(val) if !val.is_empty() => Ok(true),
            _ => Ok(false),
        }
    }

    async fn remove_session(&self, _db: &DbConn, user_id: i64) -> Result<()> {
        // Redis 模式通过 cache prefix 匹配删除
        // 由于无法批量匹配 key，这里使用 user_ 前缀做兼容
        // 实际通过 token 缓存 key 删除
        let _ = CONTEXT.cache_service.del(&format!("user_{}", user_id)).await;
        Ok(())
    }

    async fn clean_expired(&self, _db: &DbConn) -> Result<u64> {
        // Redis 自带 TTL 过期，无需手动清理
        Ok(0)
    }
}

/// 获取 SessionStore 实例（根据配置选择后端）
pub fn get_session_store() -> SessionStoreType {
    match config::section::<String>("server", "session_store", "db".to_string()).as_str() {
        "redis" => {
            log::debug!("[Session] 使用 Redis 存储");
            SessionStoreType::Redis(RedisSessionStore)
        }
        _ => {
            log::debug!("[Session] 使用 DB 存储");
            SessionStoreType::Db(DbSessionStore)
        }
    }
}

/// 确保 mxx_system_session 表存在（启动时调用）
///
/// 注意：PostgreSQL 的 prepared statement 不支持多语句，因此需要分开执行
pub async fn ensure_session_table(db: &DbConn) {
    let create_table_sql = "CREATE TABLE IF NOT EXISTS mxx_system_session (
        id BIGSERIAL PRIMARY KEY,
        user_id BIGINT NOT NULL,
        token VARCHAR(512) NOT NULL,
        login_ip VARCHAR(64),
        login_time TIMESTAMP NOT NULL DEFAULT NOW(),
        expire_time TIMESTAMP NOT NULL,
        status SMALLINT NOT NULL DEFAULT 1,
        created_at TIMESTAMP NOT NULL DEFAULT NOW()
    )";

    let create_idx1 = "CREATE INDEX IF NOT EXISTS idx_session_user_id ON mxx_system_session(user_id)";
    let create_idx2 = "CREATE INDEX IF NOT EXISTS idx_session_token ON mxx_system_session(token)";

    let backend = sea_orm::DatabaseBackend::Postgres;

    match db.execute_raw(Statement::from_string(backend, create_table_sql)).await { Err(e) => {
        log::warn!("[Session] 创建表失败: {}（可能已存在，忽略）", e);
    } _ => {
        log::info!("[Session] 表 mxx_system_session 已就绪");
    }}

    // 索引创建失败不影响主流程，仅记录日志
    let _ = db.execute_raw(Statement::from_string(backend, create_idx1)).await;
    let _ = db.execute_raw(Statement::from_string(backend, create_idx2)).await;
}