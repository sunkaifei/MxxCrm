//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 数据库版本化迁移工具
//!
//! 通过 mxx_schema_migration 版本表记录已执行的迁移批次：
//! 1. 避免启动时重复执行 CREATE TABLE/INDEX IF NOT EXISTS 刷 NOTICE 日志；
//! 2. 为后续版本升级提供统一的"批次是否已应用"判断入口；
//! 3. 老库兼容：目标表已存在但批次未记录时，直接标记已迁移，跳过重复建表。

use sea_orm::{ConnectionTrait, DbBackend, DbConn, DbErr, Statement};

/// 迁移记录表名
pub const MIGRATION_TABLE: &str = "mxx_schema_migration";

/// 检查指定表是否已存在（查询 pg_catalog，避免 IF NOT EXISTS 触发 NOTICE 日志）
pub async fn table_exists(db: &DbConn, table: &str) -> Result<bool, DbErr> {
    let sql = format!(
        "SELECT EXISTS (SELECT 1 FROM pg_catalog.pg_tables \
         WHERE schemaname = 'public' AND tablename = '{}') AS ok",
        table
    );
    let row = db
        .query_one_raw(Statement::from_string(DbBackend::Postgres, sql))
        .await?;
    Ok(match row {
        Some(r) => r.try_get("", "ok").unwrap_or(false),
        None => false,
    })
}

/// 确保迁移记录表存在（先查后建，不产生 NOTICE 日志）
pub async fn ensure_migration_table(db: &DbConn) -> Result<(), DbErr> {
    if table_exists(db, MIGRATION_TABLE).await? {
        return Ok(());
    }
    let sql = format!(
        "CREATE TABLE {} (
            id BIGSERIAL PRIMARY KEY,
            migration_name VARCHAR(100) NOT NULL UNIQUE,
            applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        MIGRATION_TABLE
    );
    db.execute_unprepared(&sql).await?;
    Ok(())
}

/// 迁移批次是否已应用
pub async fn migration_applied(db: &DbConn, name: &str) -> Result<bool, DbErr> {
    ensure_migration_table(db).await?;
    let sql = format!(
        "SELECT EXISTS (SELECT 1 FROM {} WHERE migration_name = '{}') AS ok",
        MIGRATION_TABLE, name
    );
    let row = db
        .query_one_raw(Statement::from_string(DbBackend::Postgres, sql))
        .await?;
    Ok(match row {
        Some(r) => r.try_get("", "ok").unwrap_or(false),
        None => false,
    })
}

/// 标记迁移批次已应用
pub async fn mark_migration_applied(db: &DbConn, name: &str) -> Result<(), DbErr> {
    let sql = format!(
        "INSERT INTO {} (migration_name) VALUES ('{}')",
        MIGRATION_TABLE, name
    );
    db.execute_unprepared(&sql).await?;
    Ok(())
}
