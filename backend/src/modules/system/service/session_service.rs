//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! Session 管理器（登录认证整改 v1.0）
//!
//! 提供统一的会话存储接口，支持 DB 和 Redis 两种后端。
//! 配置项 `server.session_store` 控制使用哪种后端。
//!
//! ## 验证策略
//! 1. JWT 签名验证（在 extract 中间件中完成）
//! 2. SessionStoreType::validate_session() 验证用户状态
//! 3. 过期 session 在验证时自动清理
//! 4. 定时任务可以调用 clean_expired() 批量清理
//!
//! ## v1.0 整改变更
//! - 全量改用 SeaORM 参数化查询，消除 SQL 字符串拼接
//! - 新增 refreshToken 能力：落库仅存 SHA-256 哈希，支持旋转替换与精确登出
//! - 时间列命名规范化：created_at → create_time

use sea_orm::{sea_query::Expr, ActiveModelTrait, ColumnTrait, ConnectionTrait, DbConn, DeleteResult, EntityTrait, QueryFilter, Set, Statement};

use crate::core::errors::error::{Error, Result};
use crate::core::kit::config;
use crate::core::kit::CONTEXT;
use crate::modules::system::entity::system_session;
use crate::modules::system::entity::system_session::Entity as SessionEntity;

/// 生成 refreshToken 明文（64 字节加密随机数，hex 编码 128 字符）
pub fn generate_refresh_token() -> String {
    use rand::RngCore;
    let mut buf = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

/// 计算 SHA-256 hex（64 字符）。refreshToken 明文不落库，仅存哈希
pub fn sha256_hex(input: &str) -> String {
    use sha2::Digest;
    use sha2::Sha256;
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// R8：取数据库当前时间（显式转 timestamp 匹配 NaiveDateTime）
///
/// 会话窗口的写入列（expire_time）与判定时钟统一取自 DB，
/// 消除多实例部署 / NTP 漂移下"写入时刻"与"判定时刻"来源不一致的问题
async fn db_now(db: &DbConn) -> Result<chrono::NaiveDateTime> {
    let stmt = Statement::from_string(
        db.get_database_backend(),
        "SELECT CURRENT_TIMESTAMP::timestamp AS db_now",
    );
    let row = db
        .query_one_raw(stmt)
        .await?
        .ok_or_else(|| Error::from("获取数据库时间失败".to_string()))?;
    row.try_get("", "db_now")
        .map_err(|e| Error::from(format!("解析数据库时间失败: {}", e)))
}

/// 刷新会话定位信息（按 refreshToken 哈希查得）
#[derive(Debug, Clone)]
pub struct RefreshSessionInfo {
    pub session_id: i64,
    pub user_id: i64,
    /// 旧 accessToken（用于同步更新缓存 token 集合）
    pub old_token: String,
}

/// Session 存储类型枚举
pub enum SessionStoreType {
    /// 数据库存储
    Db(DbSessionStore),
    /// Redis 存储
    Redis(RedisSessionStore),
}

impl SessionStoreType {
    /// 创建会话（登录成功后调用）
    ///
    /// v2.0（R1 拆参）：access/refresh 生命周期分离写入——
    /// `access_expire_secs` 决定 expire_time（与 JWT exp 对齐），
    /// `refresh_expire_secs` 决定 refresh_expire_time（与配置 session_timeout 对齐）
    pub async fn create_session(
        &self,
        db: &DbConn,
        user_id: i64,
        token: &str,
        refresh_hash: &str,
        ip: &str,
        access_expire_secs: i64,
        refresh_expire_secs: i64,
    ) -> Result<()> {
        match self {
            SessionStoreType::Db(store) => {
                store
                    .create_session(db, user_id, token, refresh_hash, ip, access_expire_secs, refresh_expire_secs)
                    .await
            }
            SessionStoreType::Redis(store) => {
                store
                    .create_session(db, user_id, token, refresh_hash, ip, access_expire_secs, refresh_expire_secs)
                    .await
            }
        }
    }

    /// 验证会话是否有效（accessToken 精确匹配）
    pub async fn validate_session(&self, db: &DbConn, user_id: i64, token: &str) -> Result<bool> {
        match self {
            SessionStoreType::Db(store) => store.validate_session(db, user_id, token).await,
            SessionStoreType::Redis(store) => store.validate_session(db, user_id, token).await,
        }
    }

    /// 删除用户的所有会话（踢人下线 / 改密强制下线）
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

    /// 按 refreshToken 哈希查找有效会话（刷新接口用）
    ///
    /// 返回 None 的情况：哈希不存在、refreshToken 已过期（过期行直接删除）
    pub async fn find_valid_by_refresh(
        &self,
        db: &DbConn,
        refresh_hash: &str,
    ) -> Result<Option<RefreshSessionInfo>> {
        match self {
            SessionStoreType::Db(store) => store.find_valid_by_refresh(db, refresh_hash).await,
            SessionStoreType::Redis(store) => store.find_valid_by_refresh(db, refresh_hash).await,
        }
    }

    /// 旋转会话凭据（刷新成功后调用）
    ///
    /// 原子更新同一 session 行：新 accessToken + 新 refreshToken 哈希 + 重置过期时间。
    /// 旧 refreshToken 随之行内替换，立即作废（Redis 模式显式删除旧反查索引）。
    pub async fn rotate_session(
        &self,
        db: &DbConn,
        info: &RefreshSessionInfo,
        old_refresh_hash: &str,
        new_token: &str,
        new_refresh_hash: &str,
        access_expire_secs: i64,
        refresh_expire_secs: i64,
    ) -> Result<()> {
        match self {
            SessionStoreType::Db(store) => {
                store
                    .rotate_session(db, info, old_refresh_hash, new_token, new_refresh_hash, access_expire_secs, refresh_expire_secs)
                    .await
            }
            SessionStoreType::Redis(store) => {
                store
                    .rotate_session(db, info, old_refresh_hash, new_token, new_refresh_hash, access_expire_secs, refresh_expire_secs)
                    .await
            }
        }
    }

    /// 按 refreshToken 哈希删除会话（精确登出当前设备）
    ///
    /// 返回删除的行数（0 表示凭据无效，幂等）
    pub async fn remove_by_refresh(&self, db: &DbConn, refresh_hash: &str) -> Result<u64> {
        match self {
            SessionStoreType::Db(store) => store.remove_by_refresh(db, refresh_hash).await,
            SessionStoreType::Redis(store) => store.remove_by_refresh(db, refresh_hash).await,
        }
    }

    /// 按 accessToken 删除会话（登出时无 refreshToken 的兜底定位）
    ///
    /// 返回删除的行数（0 表示会话不存在，幂等）
    pub async fn remove_by_token(&self, db: &DbConn, user_id: i64, token: &str) -> Result<u64> {
        match self {
            SessionStoreType::Db(store) => store.remove_by_token(db, user_id, token).await,
            SessionStoreType::Redis(store) => store.remove_by_token(db, user_id, token).await,
        }
    }
}

/// 数据库 Session 存储实现（SeaORM 参数化查询）
pub struct DbSessionStore;

impl DbSessionStore {
    async fn create_session(
        &self,
        db: &DbConn,
        user_id: i64,
        token: &str,
        refresh_hash: &str,
        ip: &str,
        access_expire_secs: i64,
        refresh_expire_secs: i64,
    ) -> Result<()> {
        // R8：时间源取自数据库
        let now = db_now(db).await?;
        // R1 双时间拆参：access 窗口与 refresh 窗口分列
        let expire_time = now + chrono::Duration::seconds(access_expire_secs);
        let refresh_expire_time = now + chrono::Duration::seconds(refresh_expire_secs);

        // 先删除该用户 refresh 窗口已终结的旧 session，再插入新 session。
        // 判定统一以 refresh_expire_time 为准（同 clean_expired / find_valid_by_refresh）：
        // 仅 access 过期的行仍承载"待静默刷新"会话（他端设备），不可误删
        let _: DeleteResult = SessionEntity::delete_many()
            .filter(system_session::Column::UserId.eq(user_id))
            .filter(system_session::Column::RefreshExpireTime.lt(now))
            .exec(db)
            .await?;

        let session = system_session::ActiveModel {
            user_id: Set(user_id),
            token: Set(token.to_string()),
            refresh_token: Set(Some(refresh_hash.to_string())),
            login_ip: Set(if ip.is_empty() { None } else { Some(ip.to_string()) }),
            login_time: Set(Some(now)),
            expire_time: Set(Some(expire_time)),
            refresh_expire_time: Set(Some(refresh_expire_time)),
            status: Set(Some(1)),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        session.insert(db).await?;
        Ok(())
    }

    async fn validate_session(&self, db: &DbConn, user_id: i64, token: &str) -> Result<bool> {
        // R8：时间源取自数据库
        let now = db_now(db).await?;

        let session = SessionEntity::find()
            .filter(system_session::Column::UserId.eq(user_id))
            .filter(system_session::Column::Token.eq(token))
            .one(db)
            .await;

        match session {
            Ok(Some(row)) => {
                // 过期清理
                if row.expire_time.map(|t| t < now).unwrap_or(true) {
                    let _ = SessionEntity::delete_by_id(row.id).exec(db).await;
                    return Ok(false);
                }
                Ok(row.status.unwrap_or(0) == 1)
            }
            Ok(None) => Ok(false),
            Err(e) => {
                log::warn!("[Session] 查询失败 user_id={}, err={}", user_id, e);
                Ok(false)
            }
        }
    }

    async fn remove_session(&self, db: &DbConn, user_id: i64) -> Result<()> {
        SessionEntity::delete_many()
            .filter(system_session::Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(())
    }

    async fn clean_expired(&self, db: &DbConn) -> Result<u64> {
        // R8：时间源取自数据库
        let now = db_now(db).await?;
        // 清理语义：仅删 refresh 窗口彻底终结（refresh_expire_time 已过）的会话行。
        // 不能用 expire_time < now —— access 过期但 refresh 窗口仍有效的行是"待静默刷新"的会话
        // （用户 2h~8h 内回来应能 refresh 成功），被删将导致其被迫重新登录。
        let result = SessionEntity::delete_many()
            .filter(system_session::Column::RefreshExpireTime.lt(now))
            .exec(db)
            .await?;
        let count = result.rows_affected;
        if count > 0 {
            log::info!("[Session] 清理过期会话 {} 条", count);
        }
        Ok(count)
    }

    async fn find_valid_by_refresh(
        &self,
        db: &DbConn,
        refresh_hash: &str,
    ) -> Result<Option<RefreshSessionInfo>> {
        // R8：时间源取自数据库
        let now = db_now(db).await?;

        let row = SessionEntity::find()
            .filter(system_session::Column::RefreshToken.eq(refresh_hash))
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        match row {
            Some(row) => {
                // refreshToken 过期：删除该行并判定无效
                if row.refresh_expire_time.map(|t| t < now).unwrap_or(true) {
                    let _ = SessionEntity::delete_by_id(row.id).exec(db).await;
                    log::debug!("[Session] refreshToken 已过期 session_id={}", row.id);
                    return Ok(None);
                }
                Ok(Some(RefreshSessionInfo {
                    session_id: row.id,
                    user_id: row.user_id,
                    old_token: row.token,
                }))
            }
            None => Ok(None),
        }
    }

    async fn rotate_session(
        &self,
        db: &DbConn,
        info: &RefreshSessionInfo,
        old_refresh_hash: &str,
        new_token: &str,
        new_refresh_hash: &str,
        access_expire_secs: i64,
        refresh_expire_secs: i64,
    ) -> Result<()> {
        // R8：时间源取自数据库
        let now = db_now(db).await?;
        let expire_time = now + chrono::Duration::seconds(access_expire_secs);
        let refresh_expire_time = now + chrono::Duration::seconds(refresh_expire_secs);

        // R7：乐观锁条件更新——仅当行上 refresh_token 仍是本次请求持有的旧哈希时才旋转，
        // 防止同一旧 refreshToken 被并发重放消费两次（复用攻击窗口）。
        // rows_affected == 0 说明会话已被旋转/删除，上层应转 401 让客户端重新登录。
        let result = SessionEntity::update_many()
            .col_expr(system_session::Column::Token, Expr::value(new_token.to_string()))
            .col_expr(system_session::Column::RefreshToken, Expr::value(Some(new_refresh_hash.to_string())))
            .col_expr(system_session::Column::ExpireTime, Expr::value(Some(expire_time)))
            .col_expr(system_session::Column::RefreshExpireTime, Expr::value(Some(refresh_expire_time)))
            .filter(system_session::Column::Id.eq(info.session_id))
            .filter(system_session::Column::RefreshToken.eq(old_refresh_hash))
            .exec(db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::from("会话已失效或被并发消费".to_string()));
        }
        Ok(())
    }

    async fn remove_by_refresh(&self, db: &DbConn, refresh_hash: &str) -> Result<u64> {
        let result = SessionEntity::delete_many()
            .filter(system_session::Column::RefreshToken.eq(refresh_hash))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    async fn remove_by_token(&self, db: &DbConn, user_id: i64, token: &str) -> Result<u64> {
        let result = SessionEntity::delete_many()
            .filter(system_session::Column::UserId.eq(user_id))
            .filter(system_session::Column::Token.eq(token))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }
}

/// Redis Session 存储实现
///
/// 复用现有 CacheService，存储结构：
/// - Key: `session:{user_id}:{token}`，Value: json 会话信息，TTL 与 JWT expire 一致
/// - Key: `refresh_session:{refresh_hash}`，Value: json {user_id, token}，TTL 同上（刷新接口反查用）
pub struct RedisSessionStore;

impl RedisSessionStore {
    fn session_key(user_id: i64, token: &str) -> String {
        format!("session:{}:{}", user_id, token)
    }

    fn refresh_key(refresh_hash: &str) -> String {
        format!("refresh_session:{}", refresh_hash)
    }

    async fn create_session(
        &self,
        _db: &DbConn,
        user_id: i64,
        token: &str,
        refresh_hash: &str,
        ip: &str,
        access_expire_secs: i64,
        refresh_expire_secs: i64,
    ) -> Result<()> {
        let now = chrono::Local::now().naive_local();
        // R1 双时间拆参：会话键承载"会话行"语义，须存活至 refresh 窗口终结（与 Db 行一致，
        // D1：session_timeout 写两键 TTL），access 窗口过期由键值内嵌 expire_time + validate_session
        // 解析判定；refresh_session:{hash} 索引键 TTL 同为 refresh 窗口。若会话键仅存活 access 窗口，
        // 闲置 2h~8h 用户回访时 find_valid_by_refresh 的 liveness 校验将误杀待静默刷新的会话
        let refresh_ttl = std::time::Duration::from_secs(refresh_expire_secs as u64);

        let session_value = serde_json::json!({
            "login_ip": ip,
            "login_time": now.format("%Y-%m-%d %H:%M:%S").to_string(),
            "expire_time": (now + chrono::Duration::seconds(access_expire_secs)).format("%Y-%m-%d %H:%M:%S").to_string(),
        });
        CONTEXT
            .cache_service
            .set_string_ex(&Self::session_key(user_id, token), &session_value.to_string(), Some(refresh_ttl))
            .await?;

        // refreshToken 反查索引
        let refresh_index = serde_json::json!({ "user_id": user_id, "token": token });
        CONTEXT
            .cache_service
            .set_string_ex(&Self::refresh_key(refresh_hash), &refresh_index.to_string(), Some(refresh_ttl))
            .await?;
        Ok(())
    }

    async fn validate_session(&self, _db: &DbConn, user_id: i64, token: &str) -> Result<bool> {
        // 会话键 TTL = refresh 窗口（同 Db 行存活期），access 窗口的过期判定须解析键值内嵌
        // expire_time（与 Db 实现校验 expire_time 列语义对齐）；键不存活或内嵌时间已过 → 失效
        let key = Self::session_key(user_id, token);
        match CONTEXT.cache_service.get_string(&key).await {
            Ok(val) if !val.is_empty() => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&val) {
                    if let Some(exp) = v.get("expire_time").and_then(|e| e.as_str()) {
                        if let Ok(exp_ts) = chrono::NaiveDateTime::parse_from_str(exp, "%Y-%m-%d %H:%M:%S") {
                            if exp_ts < chrono::Local::now().naive_local() {
                                // access 窗口已过：删除会话键，会话进入"待静默刷新"状态，
                                // 与 Db 实现 validate_session 过期即删行行为一致
                                let _ = CONTEXT.cache_service.del(&key).await;
                                return Ok(false);
                            }
                        }
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    async fn remove_session(&self, _db: &DbConn, user_id: i64) -> Result<()> {
        // R4 修复：原实现删除的 `user_{id}` 键与会话实际存储键 `session:{user_id}:{token}` 不匹配，
        // 导致 Redis 模式下踢下线 / 改密撤销永远删不到会话；改为按前缀扫描逐键删除
        let pattern = format!("session:{}:*", user_id);
        let ks = CONTEXT.cache_service.keys(&pattern).await?;
        for k in ks {
            let _ = CONTEXT.cache_service.del(&k).await;
        }
        Ok(())
    }

    async fn clean_expired(&self, _db: &DbConn) -> Result<u64> {
        // Redis 自带 TTL 过期，无需手动清理
        Ok(0)
    }

    async fn find_valid_by_refresh(
        &self,
        _db: &DbConn,
        refresh_hash: &str,
    ) -> Result<Option<RefreshSessionInfo>> {
        let raw = CONTEXT
            .cache_service
            .get_string(&Self::refresh_key(refresh_hash))
            .await
            .unwrap_or_default();
        if raw.is_empty() {
            return Ok(None);
        }
        let idx: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };
        let user_id = idx.get("user_id").and_then(|v| v.as_i64()).unwrap_or_default();
        let token = idx.get("token").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        if user_id <= 0 || token.is_empty() {
            return Ok(None);
        }
        // 确认会话主键仍有效
        let alive = CONTEXT
            .cache_service
            .get_string(&Self::session_key(user_id, &token))
            .await
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        if !alive {
            return Ok(None);
        }
        Ok(Some(RefreshSessionInfo {
            // Redis 模式无自增主键，session_id 置 0（rotate 按索引键操作）
            session_id: 0,
            user_id,
            old_token: token,
        }))
    }

    async fn rotate_session(
        &self,
        _db: &DbConn,
        info: &RefreshSessionInfo,
        old_refresh_hash: &str,
        new_token: &str,
        new_refresh_hash: &str,
        access_expire_secs: i64,
        refresh_expire_secs: i64,
    ) -> Result<()> {
        // R1：新会话键与反查索引键的 TTL 均按 refresh 窗口（会话行存活至 refresh 终结，
        // 同 create_session；access 过期由键值内嵌 expire_time 在 validate_session 判定）
        let refresh_ttl = std::time::Duration::from_secs(refresh_expire_secs as u64);
        let now = chrono::Local::now().naive_local();

        // 写新会话与反查索引
        let session_value = serde_json::json!({
            "rotate_time": now.format("%Y-%m-%d %H:%M:%S").to_string(),
            "expire_time": (now + chrono::Duration::seconds(access_expire_secs)).format("%Y-%m-%d %H:%M:%S").to_string(),
        });
        CONTEXT
            .cache_service
            .set_string_ex(&Self::session_key(info.user_id, new_token), &session_value.to_string(), Some(refresh_ttl))
            .await?;
        let refresh_index = serde_json::json!({ "user_id": info.user_id, "token": new_token });
        CONTEXT
            .cache_service
            .set_string_ex(&Self::refresh_key(new_refresh_hash), &refresh_index.to_string(), Some(refresh_ttl))
            .await?;

        // 删除旧键（旧 refreshToken 反查索引与旧 accessToken 会话同时作废）
        let _ = CONTEXT.cache_service.del(&Self::refresh_key(old_refresh_hash)).await;
        let _ = CONTEXT
            .cache_service
            .del(&Self::session_key(info.user_id, &info.old_token))
            .await;
        Ok(())
    }

    async fn remove_by_refresh(&self, _db: &DbConn, refresh_hash: &str) -> Result<u64> {
        let raw = CONTEXT
            .cache_service
            .get_string(&Self::refresh_key(refresh_hash))
            .await
            .unwrap_or_default();
        if raw.is_empty() {
            return Ok(0);
        }
        let idx: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(_) => return Ok(0),
        };
        let user_id = idx.get("user_id").and_then(|v| v.as_i64()).unwrap_or_default();
        let token = idx.get("token").and_then(|v| v.as_str()).unwrap_or_default();
        if user_id > 0 && !token.is_empty() {
            let _ = CONTEXT
                .cache_service
                .del(&Self::session_key(user_id, token))
                .await;
        }
        let _ = CONTEXT.cache_service.del(&Self::refresh_key(refresh_hash)).await;
        Ok(1)
    }

    async fn remove_by_token(&self, _db: &DbConn, user_id: i64, token: &str) -> Result<u64> {
        // 会话键删除后 refreshToken 反查键即使残留也会因会话键不存在而判定无效（liveness 校验兜底）
        CONTEXT
            .cache_service
            .del(&Self::session_key(user_id, token))
            .await?;
        Ok(1)
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

/// 确保 mxx_system_session 表存在（启动时调用，版本化迁移，幂等）
pub async fn ensure_session_table(db: &DbConn) {
    const MIGRATION: &str = "session_table_v1";
    // 批次已应用则跳过
    if let Ok(true) = crate::core::db_migration::migration_applied(db, MIGRATION).await {
        return;
    }
    // 老库兼容：表已存在则直接标记已迁移
    if let Ok(true) = crate::core::db_migration::table_exists(db, "mxx_system_session").await {
        let _ = crate::core::db_migration::mark_migration_applied(db, MIGRATION).await;
        return;
    }

    let create_table_sql = "CREATE TABLE IF NOT EXISTS mxx_system_session (
        id BIGSERIAL PRIMARY KEY,
        user_id BIGINT NOT NULL,
        token VARCHAR(512) NOT NULL,
        refresh_token VARCHAR(128),
        login_ip VARCHAR(64),
        login_time TIMESTAMP NOT NULL DEFAULT NOW(),
        expire_time TIMESTAMP NOT NULL,
        refresh_expire_time TIMESTAMP,
        status SMALLINT NOT NULL DEFAULT 1,
        create_time TIMESTAMP NOT NULL DEFAULT NOW()
    )";

    let create_idx1 = "CREATE INDEX IF NOT EXISTS idx_session_user_id ON mxx_system_session(user_id)";
    let create_idx2 = "CREATE INDEX IF NOT EXISTS idx_session_token ON mxx_system_session(token)";
    let create_idx3 = "CREATE INDEX IF NOT EXISTS idx_session_refresh_token ON mxx_system_session(refresh_token)";

    match db.execute_unprepared(create_table_sql).await {
        Err(e) => {
            log::warn!("[Session] 创建表失败: {}（可能已存在，忽略）", e);
        }
        _ => {
            log::info!("[Session] 表 mxx_system_session 已就绪");
        }
    }

    // 索引创建失败不影响主流程，仅记录日志
    let _ = db.execute_unprepared(create_idx1).await;
    let _ = db.execute_unprepared(create_idx2).await;
    let _ = db.execute_unprepared(create_idx3).await;

    // 记录批次已应用
    let _ = crate::core::db_migration::mark_migration_applied(db, MIGRATION).await;
}
