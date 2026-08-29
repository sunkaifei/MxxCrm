//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 自定义字段一键加速服务（混合方案 P1，设计文档 8.1/6.3）
//! - 对模块下 filterable=1 且启用中的字段建表达式索引 + 每模块一张 GIN jsonb_path_ops（P1-4/P1-5）
//! - CREATE INDEX CONCURRENTLY 禁止运行在事务块内：走独立连接、非事务执行，用完即关，不占用业务连接池
//! - 执行后查 pg_index.indisvalid：CONCURRENTLY 失败/中断遗留的 INVALID 索引自动 DROP 并重试（P1-14）
//! - 成功后回写元数据表 indexed=1，重建/迁移可追溯（6.3）
//! - 索引表达式与 build_filter_expr 严格同源（转型不一致会导致索引不命中，12 风险表）

use sea_orm::{ConnectionTrait, DbConn, Statement};

use crate::core::errors::error::{Error, Result};
use crate::core::kit::db;
use crate::modules::system::model::field_def::{FieldDefModel, FieldIndexFailVO, FieldIndexReportVO};
use crate::modules::system::service::field_def_service::{check_field_key_format, resolve_table};

/// 一键加速入口（P1-4）：对模块下全部 filterable=1 且启用中的字段建表达式索引 + 模块级 GIN
/// - 管理页「加速」按钮调用，低频管理操作，同步执行并返回逐项报告
/// - db 仅用于元数据读取与 indexed 标记回写；DDL 走函数内创建的专用连接
pub async fn accelerate(db: &DbConn, module: &str) -> Result<FieldIndexReportVO> {
    // 表名来自模块白名单常量（无注入面），module 同时进入索引命名，必须先过白名单
    let table = resolve_table(module)
        .ok_or_else(|| Error::from(format!("业务模块「{}」未接入自定义字段", module)))?;

    // 仅启用中字段参与加速（停用字段不出现在筛选，无需索引，7.4 规则 6）
    let defs = FieldDefModel::find_by_module(db, module, true)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let targets: Vec<_> = defs
        .into_iter()
        .filter(|d| d.filterable == Some(1))
        .collect();

    // CONCURRENTLY 专用连接：独立于业务连接池、非事务（6.3 硬约束），用完即关
    let ddl_conn = db::connect()
        .await
        .map_err(|e| Error::from(format!("索引加速专用数据库连接创建失败: {}", e)))?;

    let mut report = FieldIndexReportVO {
        module: Some(module.to_string()),
        ..Default::default()
    };
    let mut ok_ids: Vec<i64> = Vec::new();

    for def in &targets {
        let key = def.field_key.clone().unwrap_or_default();
        // 防御：索引名/表达式内联拼 SQL，key 必须过格式白名单（save 链已校验，此处兜底，P1-2 同源防注入）
        if check_field_key_format(&key).is_err() {
            report.failed.push(FieldIndexFailVO {
                field_key: Some(key),
                error: Some("字段键格式非法，无法生成索引".to_string()),
            });
            continue;
        }
        let index_name = format!("idx_{}_{}_cf", module, key);
        let expr = scalar_index_expr(def.field_type.unwrap_or(1), &key);
        let create_sql = format!(
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS {} ON {} {}",
            index_name, table, expr
        );
        match ensure_index(&ddl_conn, &index_name, &create_sql).await {
            Ok(true) => {
                log::info!("[field_index] 表达式索引创建成功: {}", index_name);
                report.created.push(key);
                ok_ids.push(def.id);
            }
            Ok(false) => {
                report.skipped.push(key);
                ok_ids.push(def.id);
            }
            Err(e) => {
                log::warn!("[field_index] 表达式索引创建失败: {} err={}", index_name, e);
                report.failed.push(FieldIndexFailVO {
                    field_key: Some(key),
                    error: Some(e.to_string()),
                });
            }
        }
    }

    // 模块级 GIN（jsonb_path_ops）：覆盖多选/成员/附件「包含」筛选（P1-5/P1-15），无 filterable 字段也建（覆盖未来新增）
    let gin_name = format!("idx_{}_custom_fields_gin", module);
    let gin_sql = format!(
        "CREATE INDEX CONCURRENTLY IF NOT EXISTS {} ON {} USING GIN (custom_fields jsonb_path_ops)",
        gin_name, table
    );
    report.gin_status = Some(match ensure_index(&ddl_conn, &gin_name, &gin_sql).await {
        Ok(true) => {
            log::info!("[field_index] GIN 索引创建成功: {}", gin_name);
            "created".to_string()
        }
        Ok(false) => "skipped".to_string(),
        Err(e) => {
            log::warn!("[field_index] GIN 索引创建失败: {} err={}", gin_name, e);
            format!("failed: {}", e)
        }
    });

    // 释放专用连接（close 失败不影响报告，连接随进程生命周期兜底回收）
    let _ = ddl_conn.close().await;

    // 成功/跳过的字段回写 indexed=1（6.3 元数据可追溯）；失败项保持原状，修复后可重跑
    if !ok_ids.is_empty() {
        FieldDefModel::set_indexed_flag(db, &ok_ids, 1)
            .await
            .map_err(|e| Error::from(format!("indexed 标记回写失败: {}", e)))?;
    }

    Ok(report)
}

/// 幂等确保索引存在且有效，返回 true=本次新建、false=已存在有效而跳过
/// - 预检 indisvalid：valid 跳过（重复点加速零开销）；INVALID 遗留（CONCURRENTLY 中断产物）自动 DROP（P1-14）
/// - 创建失败时同样清理遗留 INVALID 后重试一次，仍失败才报错（8.1：自动 DROP 并报错重试）
/// - 创建成功后复检 indisvalid，防「索引存在但不可用」虚标 indexed=1（12 风险表）
async fn ensure_index(conn: &DbConn, index_name: &str, create_sql: &str) -> Result<bool> {
    match index_valid(conn, index_name).await? {
        Some(true) => return Ok(false),
        Some(false) => {
            drop_index(conn, index_name).await?;
        }
        None => {}
    }

    let mut last_err = String::new();
    for _ in 0..2 {
        match conn.execute_unprepared(create_sql).await {
            Ok(_) => {
                if index_valid(conn, index_name).await? == Some(true) {
                    return Ok(true);
                }
                last_err = "索引创建后 indisvalid=false（已清理，可重试）".to_string();
            }
            Err(e) => last_err = e.to_string(),
        }
        // 失败或不可用：清掉遗留 INVALID 索引后重试一次
        drop_index(conn, index_name).await?;
    }
    Err(Error::from(format!(
        "索引 {} 创建失败（已自动清理遗留 INVALID 索引）: {}",
        index_name, last_err
    )))
}

/// 查询索引是否存在且有效：Some(true)=存在且 valid，Some(false)=存在但 INVALID，None=不存在
/// 索引名在 schema 内唯一，限定 current_schema() 防同名混淆
async fn index_valid(conn: &DbConn, index_name: &str) -> Result<Option<bool>> {
    let sql = r#"SELECT i.indisvalid
                 FROM pg_index i
                 JOIN pg_class c ON c.oid = i.indexrelid
                 JOIN pg_namespace n ON n.oid = c.relnamespace
                 WHERE n.nspname = current_schema() AND c.relname = $1"#;
    let rows = conn
        .query_all_raw(Statement::from_sql_and_values(
            conn.get_database_backend(),
            sql,
            [index_name.into()],
        ))
        .await
        .map_err(|e| Error::from(format!("查询索引状态 {} 失败: {}", index_name, e)))?;
    Ok(rows
        .first()
        .and_then(|row| row.try_get::<Option<bool>>("", "indisvalid").ok().flatten()))
}

/// DROP INDEX CONCURRENTLY（同样禁事务块，走专用连接；IF EXISTS 幂等）
async fn drop_index(conn: &DbConn, index_name: &str) -> Result<()> {
    let sql = format!("DROP INDEX CONCURRENTLY IF EXISTS {}", index_name);
    conn.execute_unprepared(&sql)
        .await
        .map_err(|e| Error::from(format!("清理遗留索引 {} 失败: {}", index_name, e)))?;
    Ok(())
}

/// 按字段类型生成与 6.3 模板 / build_filter_expr 同源的索引表达式
/// 文本/单选/日期裸 ->>（ISO 字典序=时间序）；数字/金额 ::numeric；布尔 ::boolean；
/// 数组型（7/9/10）不建表达式索引，走模块级 GIN（无序语义）
fn scalar_index_expr(field_type: i32, key: &str) -> String {
    match field_type {
        3 | 11 => format!("(((custom_fields->>'{}')::numeric))", key),
        8 => format!("(((custom_fields->>'{}')::boolean))", key),
        _ => format!("(((custom_fields->>'{}')))", key),
    }
}
