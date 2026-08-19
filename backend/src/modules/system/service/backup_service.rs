//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 数据库备份服务（应用内定时任务处理器 db_backup）
//!
//! 流程：pg_dump -Fc 全量 → pg_restore --list 校验 → 记录 mxx_system_backup_log → 清理超期备份
//! 配置（config.ini [backup] 节）：
//!   output_dir      备份输出目录（默认 ./storage/backup/）
//!   keep_days       日备保留天数（默认 14）
//!   pg_dump_path    pg_dump 可执行文件路径（默认 pg_dump，Windows 需全路径）
//!   pg_restore_path pg_restore 路径（留空则自动取 pg_dump 同目录）
//!
//! 依据：docs/数据备份-开发手册.md

use std::time::Instant;

use sea_orm::{ConnectionTrait, DatabaseConnection};
use tokio::process::Command;

use crate::core::kit::config;

/// 备份记录实体
pub use crate::modules::system::entity::backup_log;

/// 确保备份记录表存在（幂等，部署零 SQL 依赖）
pub async fn ensure_backup_log_table(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    db.execute_unprepared(
        r#"CREATE TABLE IF NOT EXISTS mxx_system_backup_log (
            id BIGSERIAL PRIMARY KEY,
            file_name VARCHAR(255) DEFAULT NULL,
            file_path VARCHAR(512) DEFAULT NULL,
            file_size BIGINT DEFAULT 0,
            table_count INT DEFAULT 0,
            status INT DEFAULT 0,
            cost_ms BIGINT DEFAULT 0,
            error_message VARCHAR(1024) DEFAULT NULL,
            create_time TIMESTAMP DEFAULT NULL
        )"#,
    )
    .await?;
    // V2: 记录操作类型（0=备份 1=数据恢复），旧库幂等补列
    db.execute_unprepared(
        "ALTER TABLE mxx_system_backup_log ADD COLUMN IF NOT EXISTS operate_type INT DEFAULT 0",
    )
    .await?;
    Ok(())
}

/// 读取保留天数：mxx_system_config(backup.keep_days) 优先，缺省回落 ini [backup] keep_days，再缺省 14
pub async fn effective_keep_days() -> i64 {
    if let Some(v) = crate::modules::system::service::config_service::find_value_by_key_from_db("backup.keep_days").await {
        if let Ok(n) = v.trim().parse::<i64>() {
            return n.max(1);
        }
    }
    config::section::<i64>("backup", "keep_days", 14).max(1)
}

/// 保存保留天数到系统配置表（不存在则插入）
pub async fn save_keep_days(db: &DatabaseConnection, keep_days: i64) -> Result<(), String> {
    use sea_orm::*;
    use crate::modules::system::entity::config as config_entity;

    let existing = config_entity::Entity::find()
        .filter(config_entity::Column::ConfigKey.eq("backup.keep_days"))
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(row) = existing {
        let mut active: config_entity::ActiveModel = row.into();
        active.config_value = sea_orm::Set(Some(keep_days.to_string()));
        active.update_time = sea_orm::Set(Some(chrono::Utc::now().naive_utc()));
        active.update(db).await.map_err(|e| e.to_string())?;
    } else {
        let active = config_entity::ActiveModel {
            config_name: sea_orm::Set(Some("数据库备份保留天数".to_string())),
            config_key: sea_orm::Set(Some("backup.keep_days".to_string())),
            config_value: sea_orm::Set(Some(keep_days.to_string())),
            config_type: sea_orm::Set(Some("N".to_string())),
            remark: sea_orm::Set(Some("数据备份与恢复页面设置：日备保留天数，超期自动清理".to_string())),
            create_time: sea_orm::Set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        active.insert(db).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 备份设置 VO（页面"备份设置"卡片数据）
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupConfigVO {
    pub job_id: i64,
    pub cron_expression: String,
    pub enabled: i32,
    pub keep_days: i64,
    pub output_dir: String,
    pub pg_dump_path: String,
    pub last_backup_time: Option<String>,
    pub last_backup_status: Option<i32>,
}

/// 获取备份设置汇总（任务来自 mxx_system_scheduler_job，保留天数优先系统配置表）
pub async fn get_config(db: &DatabaseConnection) -> Result<BackupConfigVO, String> {
    use sea_orm::*;
    let job = crate::modules::system::entity::scheduler_job::Entity::find()
        .filter(crate::modules::system::entity::scheduler_job::Column::JobCode.eq("db_backup"))
        .filter(crate::modules::system::entity::scheduler_job::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("备份任务未注册，请先执行 backend/sql/backup_log.sql")?;

    let last = backup_log::Entity::find()
        .filter(backup_log::Column::OperateType.eq(0))
        .order_by_desc(backup_log::Column::Id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(BackupConfigVO {
        job_id: job.id,
        cron_expression: job.cron_expression.clone(),
        enabled: job.enabled.unwrap_or(1),
        keep_days: effective_keep_days().await,
        output_dir: config::section::<String>("backup", "output_dir", "./storage/backup/".to_string()),
        pg_dump_path: config::section::<String>("backup", "pg_dump_path", "pg_dump".to_string()),
        last_backup_time: last.as_ref().and_then(|r| r.create_time).map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        last_backup_status: last.as_ref().and_then(|r| r.status),
    })
}

/// 备份记录 VO（页面表格行，camelCase 序列化）
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupLogVO {
    pub id: i64,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub table_count: Option<i32>,
    /// 0=备份, 1=数据恢复
    pub operate_type: Option<i32>,
    pub status: Option<i32>,
    pub cost_ms: Option<i64>,
    pub error_message: Option<String>,
    pub create_time: Option<String>,
}

impl From<backup_log::Model> for BackupLogVO {
    fn from(m: backup_log::Model) -> Self {
        Self {
            id: m.id,
            file_name: m.file_name,
            file_size: m.file_size,
            table_count: m.table_count,
            operate_type: m.operate_type,
            status: m.status,
            cost_ms: m.cost_ms,
            error_message: m.error_message,
            create_time: m.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 备份记录分页查询
pub async fn get_list(
    db: &DatabaseConnection,
    page: i64,
    page_size: i64,
) -> Result<(Vec<BackupLogVO>, i64), String> {
    use sea_orm::*;
    let page = std::cmp::max(page, 1);
    let page_size = page_size.clamp(1, 100);
    let paginator = backup_log::Entity::find()
        .order_by_desc(backup_log::Column::Id)
        .paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items: Vec<BackupLogVO> = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(BackupLogVO::from)
        .collect();
    Ok((items, total))
}

/// 删除备份：物理删除文件 + 删除记录（恢复类记录仅删记录）
pub async fn delete_backup(db: &DatabaseConnection, id: i64) -> Result<String, String> {
    use sea_orm::*;
    let rec = backup_log::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("备份记录不存在")?;
    let removed_file = rec
        .file_path
        .as_deref()
        .filter(|p| !p.is_empty())
        .map(std::path::Path::new)
        .filter(|p| p.exists())
        .map(|p| std::fs::remove_file(p).is_ok())
        .unwrap_or(false);
    backup_log::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(if removed_file { "备份文件与记录已删除".to_string() } else { "记录已删除（备份文件不存在或已移除）".to_string() })
}

/// 下载备份文件：返回 (字节, 文件名, MIME)
pub async fn download_backup(db: &DatabaseConnection, id: i64) -> Result<(Vec<u8>, String, String), String> {
    use sea_orm::*;
    let rec = backup_log::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("备份记录不存在")?;
    if rec.operate_type.unwrap_or(0) != 0 || rec.status.unwrap_or(0) != 1 {
        return Err("仅成功的备份文件支持下载".to_string());
    }
    let path = rec.file_path.clone().ok_or("文件路径缺失")?;
    let data = std::fs::read(&path).map_err(|e| format!("备份文件读取失败: {}", e))?;
    let name = rec.file_name.unwrap_or_else(|| "backup.dump".to_string());
    Ok((data, name, "application/octet-stream".to_string()))
}

/// 数据恢复：将指定备份 pg_restore 回当前库（危险操作）
///
/// 安全约束：
/// 1. 确认码必须为 RESTORE（前端弹窗强制输入）
/// 2. 仅"成功备份"(operate_type=0, status=1) 可恢复，文件必须在备份目录内（防路径穿越）
/// 3. 恢复前检查 pg_stat_activity，存在其他活跃连接则拒绝（避免恢复期间业务写入造成数据错乱）
/// 4. 恢复使用 --clean --if-exists --no-owner --no-privileges（先 DROP 后重建，不依赖原角色）
/// 5. 恢复后必须重启后端服务（连接池缓存的 prepared statement 失效），结果消息中明确提示
pub async fn restore_backup(
    db: &DatabaseConnection,
    id: i64,
    confirm: &str,
) -> Result<String, String> {
    use sea_orm::*;
    let start = Instant::now();

    if confirm.trim() != "RESTORE" {
        return Err("确认码不正确，请输入 RESTORE 以确认执行恢复".to_string());
    }
    ensure_backup_log_table(db).await.map_err(|e| e.to_string())?;

    // 1) 记录与文件校验
    let rec = backup_log::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("备份记录不存在")?;
    if rec.operate_type.unwrap_or(0) != 0 || rec.status.unwrap_or(0) != 1 {
        return Err("仅成功的备份文件支持恢复".to_string());
    }
    let file_path = rec.file_path.clone().ok_or("文件路径缺失")?;
    let canon = std::path::Path::new(&file_path)
        .canonicalize()
        .map_err(|_| "备份文件不存在或已被移动".to_string())?;
    let output_dir = config::section::<String>("backup", "output_dir", "./storage/backup/".to_string());
    let base = std::path::Path::new(&output_dir)
        .canonicalize()
        .map_err(|e| format!("备份目录不可用: {}", e))?;
    if !canon.starts_with(&base) {
        return Err("备份文件不在备份目录内，拒绝恢复".to_string());
    }
    let file_path_str = canon.to_string_lossy().to_string();
    let file_name = rec.file_name.clone().unwrap_or_else(|| "backup.dump".to_string());

    // 2) 活跃连接检查：存在其他 active 后端则拒绝（idle 连接不影响，DROP TABLE 只需等待/获取锁）
    let active_stmt = sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT count(*)::bigint AS c FROM pg_stat_activity WHERE datname = current_database() AND state = 'active' AND pid <> pg_backend_pid()".to_string(),
    );
    let active_row = db
        .query_one_raw(active_stmt)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("活跃连接检查失败")?;
    let active: i64 = active_row.try_get_by_index(0).unwrap_or(-1);
    if active != 0 {
        return Err(format!(
            "检测到 {} 个其他会话正在执行数据库操作，为避免数据错乱已中止恢复。请在无人使用的维护窗口执行",
            active
        ));
    }

    // 3) 执行恢复
    let db_url = config::section::<String>("db", "url", String::new());
    let pg_restore_path = config::section::<String>("backup", "pg_restore_path", String::new());
    let restore_bin = if pg_restore_path.is_empty() {
        guess_restore_path(&config::section::<String>("backup", "pg_dump_path", "pg_dump".to_string()))
    } else {
        pg_restore_path
    };
    let (user, pass, host, port, dbname) = parse_db_url(&db_url)?;

    let restore_fut = Command::new(&restore_bin)
        .env("PGPASSWORD", &pass)
        .args([
            "-h", &host,
            "-p", &port,
            "-U", &user,
            "-d", &dbname,
            "--clean", "--if-exists", "--no-owner", "--no-privileges",
            "-v",
            &file_path_str,
        ])
        .output();

    let output = tokio::time::timeout(std::time::Duration::from_secs(1800), restore_fut)
        .await
        .map_err(|_| "恢复超时（30 分钟）已中止".to_string())?
        .map_err(|e| format!("pg_restore 执行失败: {}", e))?;

    let cost_ms = start.elapsed().as_millis() as i64;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let tail: String = stderr.lines().rev().take(5).collect::<Vec<_>>().join(" | ");
        insert_log(db, Some(&file_name), Some(&file_path_str), 0, 0, 0, 1, cost_ms, Some(&tail)).await;
        return Err(format!("恢复失败（详见备份列表记录）: {}", tail));
    }

    insert_log(db, Some(&file_name), Some(&file_path_str), 0, 1, 0, 1, cost_ms, None).await;
    log::warn!("[数据库恢复] 已从 {} 恢复数据库，耗时 {} ms", file_name, cost_ms);

    Ok(format!(
        "恢复完成：{}，耗时 {} ms。请立即重启后端服务以刷新数据库连接缓存！",
        file_name, cost_ms
    ))
}

/// 从 postgres://user:pass@host:port/dbname 解析 (user, pass, host, port, dbname)
fn parse_db_url(url: &str) -> Result<(String, String, String, String, String), String> {
    let rest = url
        .strip_prefix("postgres://")
        .or_else(|| url.strip_prefix("postgresql://"))
        .ok_or_else(|| format!("数据库连接串格式不支持: {}", url))?;

    let (auth, host_part) = rest
        .split_once('@')
        .ok_or_else(|| "连接串缺少 @ 分隔".to_string())?;
    let (user, pass) = auth
        .split_once(':')
        .ok_or_else(|| "连接串缺少用户名/密码".to_string())?;

    let (host_port, dbname) = host_part
        .split_once('/')
        .ok_or_else(|| "连接串缺少数据库名".to_string())?;
    let (host, port) = match host_port.split_once(':') {
        Some((h, p)) => (h.to_string(), p.to_string()),
        None => (host_port.to_string(), "5432".to_string()),
    };
    // 去掉 dbname 上的查询参数（如 ?sslmode=disable）
    let dbname = dbname.split('?').next().unwrap_or(dbname).to_string();

    Ok((user.to_string(), pass.to_string(), host, port, dbname))
}

/// 执行一次全量备份（定时任务处理器 db_backup 入口，手动触发同用此函数）
///
/// 返回结果消息（由调度器写入运行日志）；失败返回 Err（调度器按重试策略退避重试）
pub async fn run_backup(db: &DatabaseConnection) -> Result<String, String> {
    let start = Instant::now();

    // 0) 兜底建表（幂等）
    if let Err(e) = ensure_backup_log_table(db).await {
        return Err(format!("备份记录表初始化失败: {}", e));
    }

    // 1) 读配置
    let output_dir = config::section::<String>("backup", "output_dir", "./storage/backup/".to_string());
    let keep_days = config::section::<i64>("backup", "keep_days", 14).max(1);
    let db_url = config::section::<String>("db", "url", String::new());
    let pg_dump_path = config::section::<String>("backup", "pg_dump_path", "pg_dump".to_string());
    let pg_restore_path = config::section::<String>("backup", "pg_restore_path", String::new());
    let (user, pass, host, port, dbname) = parse_db_url(&db_url)?;

    // 2) 准备目录与文件名
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let file_name = format!("{}_{}.dump", dbname, ts);
    let file_path = std::path::Path::new(&output_dir).join(&file_name);
    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        return Err(format!("备份目录创建失败 {}: {}", output_dir, e));
    }
    let file_path_str = file_path.to_string_lossy().to_string();

    // 3) 执行 pg_dump -Fc（密码经 PGPASSWORD 环境变量传入，不落命令行）
    let dump_output = Command::new(&pg_dump_path)
        .env("PGPASSWORD", &pass)
        .args([
            "-h", &host,
            "-p", &port,
            "-U", &user,
            "-Fc",
            "-f", &file_path_str,
            &dbname,
        ])
        .output()
        .await
        .map_err(|e| format!("pg_dump 执行失败（检查 backup.pg_dump_path 配置）: {}", e))?;

    if !dump_output.status.success() {
        // 失败清理残留文件，杜绝半成品备份
        let _ = std::fs::remove_file(&file_path);
        let stderr = String::from_utf8_lossy(&dump_output.stderr).trim().to_string();
        let msg = format!("pg_dump 失败: {}", stderr);
        insert_log(db, None, None, 0, 0, 0, 0, start.elapsed().as_millis() as i64, Some(&msg)).await;
        return Err(msg);
    }

    // 4) 校验：pg_restore --list 必须能列出目录，并统计表数量
    let restore_bin = if pg_restore_path.is_empty() {
        guess_restore_path(&pg_dump_path)
    } else {
        pg_restore_path
    };
    let verify = Command::new(&restore_bin)
        .env("PGPASSWORD", &pass)
        .args(["--list", &file_path_str])
        .output()
        .await
        .map_err(|e| format!("pg_restore 校验执行失败: {}", e))?;

    if !verify.status.success() {
        let _ = std::fs::remove_file(&file_path);
        let stderr = String::from_utf8_lossy(&verify.stderr).trim().to_string();
        let msg = format!("备份文件校验失败（已删除坏文件）: {}", stderr);
        insert_log(db, Some(&file_name), Some(&file_path_str), 0, 0, 0, 0, start.elapsed().as_millis() as i64, Some(&msg)).await;
        return Err(msg);
    }
    let toc = String::from_utf8_lossy(&verify.stdout);
    // 仅统计真正的建表条目（描述符 1259 = relation 定义），
    // 排除 "TABLE DATA"（数据行）与 "COMMENT ... TABLE"（表注释）行，与库内表数一致
    let table_count = toc
        .lines()
        .filter(|l| l.contains(" TABLE ") && l.contains(" 1259 "))
        .count() as i32;

    // 5) 记录成功日志
    let file_size = std::fs::metadata(&file_path).map(|m| m.len() as i64).unwrap_or(0);
    insert_log(
        db,
        Some(&file_name),
        Some(&file_path_str),
        file_size,
        1,
        table_count,
        0,
        start.elapsed().as_millis() as i64,
        None,
    )
    .await;

    // 6) 保留清理：删除超期日备（mtime 早于 keep_days 天前）
    let removed = cleanup_expired(&output_dir, keep_days as u64);

    Ok(format!(
        "备份完成: {}（{:.2} MB，{} 张表），耗时 {} ms{}",
        file_name,
        file_size as f64 / 1024.0 / 1024.0,
        table_count,
        start.elapsed().as_millis(),
        if removed > 0 { format!("，清理超期备份 {} 个", removed) } else { String::new() }
    ))
}

/// 按文件修改时间清理超期备份（仅匹配自动命名的日备，不动 _weekly/_monthly 归档）
fn cleanup_expired(dir: &str, keep_days: u64) -> u32 {
    let mut removed = 0u32;
    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(keep_days * 86400);
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_daily = name.ends_with(".dump")
                && name.split('_').count() >= 3
                && !name.contains("_weekly")
                && !name.contains("_monthly");
            if !is_daily {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if let Ok(mtime) = meta.modified() {
                    if mtime < cutoff {
                        if std::fs::remove_file(entry.path()).is_ok() {
                            removed += 1;
                        }
                    }
                }
            }
        }
    }
    removed
}

/// 未配置 pg_restore_path 时，从 pg_dump 路径推导（同目录 pg_restore[.exe]）
fn guess_restore_path(dump_path: &str) -> String {
    let p = std::path::Path::new(dump_path);
    let bin = if cfg!(windows) { "pg_restore.exe" } else { "pg_restore" };
    match p.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join(bin).to_string_lossy().to_string(),
        _ => bin.to_string(),
    }
}

/// 写备份记录（失败不影响主流程）
#[allow(clippy::too_many_arguments)]
async fn insert_log(
    db: &DatabaseConnection,
    file_name: Option<&str>,
    file_path: Option<&str>,
    file_size: i64,
    status: i32,
    table_count: i32,
    operate_type: i32,
    cost_ms: i64,
    error_message: Option<&str>,
) {
    let sql = format!(
        "INSERT INTO mxx_system_backup_log (file_name, file_path, file_size, table_count, operate_type, status, cost_ms, error_message, create_time) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, NOW())",
        opt_str(file_name),
        opt_str(file_path),
        file_size,
        table_count,
        operate_type,
        status,
        cost_ms,
        opt_str(error_message),
    );
    if let Err(e) = db.execute_unprepared(&sql).await {
        log::warn!("[数据库备份] 写备份记录失败: {}", e);
    }
}

/// SQL 字面量转义（单引号包裹；内部单引号翻倍）
fn opt_str(v: Option<&str>) -> String {
    match v {
        Some(s) => format!("'{}'", s.replace('\'', "''")),
        None => "NULL".to_string(),
    }
}
