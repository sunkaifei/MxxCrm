//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::Mutex;

use crate::core::errors::error::{Error, Result};
use crate::core::kit::install;
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};

/// 测试连接请求
#[derive(Debug, Deserialize)]
pub struct TestConnectionRequest {
    pub mode: String, // "local" | "remote"
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

/// 测试连接响应
#[derive(Debug, Serialize)]
pub struct TestConnectionResponse {
    pub mode: String,
    pub database_exists: bool,
    pub table_count: i64,
    pub has_core_tables: bool,
    pub pgcrypto_installed: bool,
    pub pg_restore_available: bool,
}

/// 创建数据库请求
#[derive(Debug, Deserialize)]
pub struct CreateDatabaseRequest {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

/// 导入数据库请求
#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub mode: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub dump_file: Option<String>,
}

/// 完成安装请求
#[derive(Debug, Deserialize)]
pub struct CompleteRequest {
    pub mode: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    /// 超级管理员登录账号（仅安装时设置，之后系统内锁定不可改）
    pub admin_user_name: Option<String>,
    /// 超级管理员姓名
    pub admin_nick_name: Option<String>,
    /// 超级管理员邮箱
    pub admin_email: Option<String>,
    /// 超级管理员手机号
    pub admin_mobile: Option<String>,
    /// 超级管理员初始密码
    pub admin_password: Option<String>,
}

/// 许可协议接受请求
#[derive(Debug, Deserialize)]
pub struct LicenseAcceptRequest {
    pub accepted: bool,
    pub license_version: String,
}

/// 导入任务状态
#[derive(Debug, Clone, Serialize)]
pub struct ImportTask {
    pub task_id: String,
    pub status: String, // "pending", "running", "success", "failed"
    pub progress: u32,
    pub log: String,
}

/// 导入任务注册表
static IMPORT_TASKS: LazyLock<Mutex<HashMap<String, ImportTask>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 构建数据库连接串
pub fn build_db_url(host: &str, port: u16, username: &str, password: &str, database: &str) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{}",
        username, password, host, port, database
    )
}

/// 测试数据库连接
pub async fn test_connection(req: &TestConnectionRequest) -> Result<TestConnectionResponse> {
    install::ensure_license_accepted()?;

    // 本机模式校验 host
    if req.mode == "local" && !install::is_local_host(&req.host) {
        return Err(Error::BadRequest("本机模式仅支持 127.0.0.1".to_string()));
    }

    let db_url = build_db_url(&req.host, req.port, &req.username, &req.password, &req.database);

    install::log_install(format!(
        "测试连接: {}@{}:{}/{}",
        req.username, req.host, req.port, req.database
    ));

    match sea_orm::Database::connect(&db_url).await {
        Ok(conn) => {
            // 查询表数量
            let stmt = sea_orm::Statement::from_string(
                    sea_orm::DbBackend::Postgres,
                    "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'".to_string(),
                );
            let table_count = match conn.execute_raw(stmt).await
            {
                Ok(result) => {
                    result.rows_affected() as i64
                }
                Err(_) => 0,
            };

            // 检查核心表
            let has_core_tables = install::check_core_tables_exist(&conn).await;

            // 检查 pgcrypto
            let pgcrypto_installed = install::check_pgcrypto(&conn).await;

            let _ = conn.close().await;

            install::log_install(format!(
                "连接成功: 表数量={}, 核心表={}, pgcrypto={}",
                table_count, has_core_tables, pgcrypto_installed
            ));

            Ok(TestConnectionResponse {
                mode: req.mode.clone(),
                database_exists: table_count > 0,
                table_count,
                has_core_tables,
                pgcrypto_installed,
                pg_restore_available: install::check_pg_restore(),
            })
        }
        Err(e) => {
            install::log_install(format!("连接失败: {}", e));
            Err(Error::E(format!("无法连接到数据库: {}", e)))
        }
    }
}

/// 创建数据库
pub async fn create_database(req: &CreateDatabaseRequest) -> Result<String> {
    install::ensure_license_accepted()?;

    // 连接到 postgres 默认库
    let admin_url = format!(
        "postgres://{}:{}@{}:{}/postgres",
        req.username, req.password, req.host, req.port
    );

    install::log_install(format!("创建数据库: {}", req.database));

    match sea_orm::Database::connect(&admin_url).await {
        Ok(conn) => {
            let sql = format!(
                "CREATE DATABASE \"{}\" WITH ENCODING 'UTF8'",
                req.database
            );
            let stmt = sea_orm::Statement::from_string(sea_orm::DbBackend::Postgres, sql);
            match conn.execute_raw(stmt).await {
                Ok(_) => {
                    install::log_install(format!("数据库创建成功: {}", req.database));
                    let _ = conn.close().await;
                    Ok(format!("数据库 '{}' 创建成功", req.database))
                }
                Err(e) => {
                    let _ = conn.close().await;
                    install::log_install(format!("数据库创建失败: {}", e));
                    Err(Error::E(format!("创建数据库失败: {}", e)))
                }
            }
        }
        Err(e) => Err(Error::E(format!("无法连接到数据库服务器: {}", e))),
    }
}

/// 导入数据库（异步任务）
pub async fn import_database(req: &ImportRequest) -> Result<String> {
    install::ensure_license_accepted()?;

    let task_id = uuid::Uuid::new_v4().to_string();

    // 创建任务记录
    {
        let mut tasks = IMPORT_TASKS.lock().await;
        tasks.insert(
            task_id.clone(),
            ImportTask {
                task_id: task_id.clone(),
                status: "pending".to_string(),
                progress: 0,
                log: "等待开始...".to_string(),
            },
        );
    }

    // 确定 dump 文件路径
    let dump_path = match &req.dump_file {
        Some(path) if !path.is_empty() => path.clone(),
        _ => {
            // 使用内置 dump，提取到临时目录
            let dump = crate::embed_frontend::SqlDump::get("mxxcrm_data_full.dump")
                .ok_or_else(|| Error::E("内置数据包不存在，请先编译前端并确保 sql/mxxcrm_data_full.dump 存在".to_string()))?;
            let temp_path = std::env::temp_dir().join("mxxcrm_data_full.dump");
            std::fs::write(&temp_path, &dump.data)
                .map_err(|e| Error::E(format!("提取内置 dump 失败: {}", e)))?;
            temp_path.to_string_lossy().to_string()
        }
    };

    let task_id_clone = task_id.clone();
    let req_clone = ImportRequest {
        mode: req.mode.clone(),
        host: req.host.clone(),
        port: req.port,
        username: req.username.clone(),
        password: req.password.clone(),
        database: req.database.clone(),
        dump_file: Some(dump_path),
    };

    // 异步执行导入
    tokio::spawn(async move {
        run_import_task(task_id_clone, req_clone).await;
    });

    Ok(task_id)
}

/// 执行导入任务（在后台线程中运行）
async fn run_import_task(task_id: String, req: ImportRequest) {
    // 更新状态为 running
    {
        let mut tasks = IMPORT_TASKS.lock().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.status = "running".to_string();
            task.progress = 5;
            task.log = "正在准备导入...".to_string();
        }
    }

    install::log_install(format!("开始导入: {}@{}:{}/{}", req.username, req.host, req.port, req.database));

    // 1. 先检测并安装 pgcrypto
    {
        let db_url = build_db_url(&req.host, req.port, &req.username, &req.password, &req.database);
        if let Ok(conn) = sea_orm::Database::connect(&db_url).await {
            let pgcrypto_ok = install::check_pgcrypto(&conn).await;
            if !pgcrypto_ok {
                install::log_install("pgcrypto 扩展未安装，尝试安装...");
                let _ = install::install_pgcrypto(&conn).await;
            }
            let _ = conn.close().await;
        }
    }

    // 更新进度
    {
        let mut tasks = IMPORT_TASKS.lock().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.progress = 15;
            task.log = "正在执行 pg_restore...".to_string();
        }
    }

    // 2. 执行 pg_restore
    let dump_path = req.dump_file.as_deref().unwrap_or("");
    let output = tokio::process::Command::new("pg_restore")
        .arg("-h")
        .arg(&req.host)
        .arg("-p")
        .arg(req.port.to_string())
        .arg("-U")
        .arg(&req.username)
        .arg("-d")
        .arg(&req.database)
        .arg("--clean")
        .arg("--if-exists")
        .arg("--no-owner")
        .arg("--no-privileges")
        .arg(dump_path)
        .env("PGPASSWORD", &req.password)
        .output()
        .await;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            if !out.status.success() {
                // 检查是否有致命错误
                if stderr.contains("FATAL") {
                    install::log_install(format!("导入失败: {}", stderr));
                    let mut tasks = IMPORT_TASKS.lock().await;
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.status = "failed".to_string();
                        task.log = format!("导入失败: {}", stderr);
                    }
                    return;
                }
            }

            install::log_install("pg_restore 执行完成，验证数据...");

            // 更新进度
            {
                let mut tasks = IMPORT_TASKS.lock().await;
                if let Some(task) = tasks.get_mut(&task_id) {
                    task.progress = 80;
                    task.log = "正在验证导入数据...".to_string();
                }
            }

            // 3. 验证核心表
            let db_url = build_db_url(&req.host, req.port, &req.username, &req.password, &req.database);
            let verified = match sea_orm::Database::connect(&db_url).await {
                Ok(conn) => {
                    let ok = install::check_core_tables_exist(&conn).await;
                    let _ = conn.close().await;
                    ok
                }
                Err(_) => false,
            };

            let mut tasks = IMPORT_TASKS.lock().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                if verified {
                    task.status = "success".to_string();
                    task.progress = 100;
                    task.log = "导入成功，核心表验证通过".to_string();
                    install::log_install("导入成功，核心表验证通过");
                } else {
                    task.status = "failed".to_string();
                    task.log = "导入完成但核心表验证失败，请检查数据库".to_string();
                    install::log_install("导入完成但核心表验证失败");
                }
            }
        }
        Err(e) => {
            install::log_install(format!("pg_restore 执行失败: {}", e));
            let mut tasks = IMPORT_TASKS.lock().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = "failed".to_string();
                task.log = format!("pg_restore 执行失败: {}", e);
            }
        }
    }
}

/// 获取导入进度
pub async fn get_import_progress(task_id: &str) -> Result<ImportTask> {
    let tasks = IMPORT_TASKS.lock().await;
    tasks
        .get(task_id)
        .cloned()
        .ok_or_else(|| Error::E("任务不存在".to_string()))
}

/// 完成安装
pub async fn complete_install(req: &CompleteRequest) -> Result<String> {
    install::ensure_license_accepted()?;

    // 本机模式校验 host
    if req.mode == "local" && !install::is_local_host(&req.host) {
        return Err(Error::BadRequest("本机模式仅支持 127.0.0.1".to_string()));
    }

    // 远程模式校验核心表
    if req.mode == "remote" {
        let db_url = build_db_url(&req.host, req.port, &req.username, &req.password, &req.database);
        match sea_orm::Database::connect(&db_url).await {
            Ok(conn) => {
                let has_tables = install::check_core_tables_exist(&conn).await;
                let _ = conn.close().await;
                if !has_tables {
                    return Err(Error::BadRequest(
                        "远程数据库未检测到核心表，请先导入数据后再完成安装".to_string(),
                    ));
                }
            }
            Err(e) => {
                return Err(Error::E(format!("无法连接到远程数据库: {}", e)));
            }
        }
    }

    // 确保配置文件存在
    install::ensure_config_exists()?;

    // 写入配置文件
    let config_path = "config/config.ini";
    let mut ini = ini::Ini::load_from_file(config_path)
        .map_err(|e| Error::E(format!("读取配置文件失败: {}", e)))?;

    let db_url = build_db_url(&req.host, req.port, &req.username, &req.password, &req.database);
    ini.set_to(Some("db"), "url".to_string(), db_url);

    ini.write_to_file(config_path)
        .map_err(|e| Error::E(format!("写入配置文件失败: {}", e)))?;

    install::log_install("配置文件已更新");

    // 初始超级管理员账号设置（可选）：UPDATE 现有超管记录，id 与关联关系不变
    setup_super_admin(req).await?;

    // 创建锁文件
    install::create_lock_file()
        .map_err(|e| Error::E(format!("创建锁文件失败: {}", e)))?;

    install::log_install("安装完成，即将自动重启...");

    Ok("安装完成，程序即将自动重启".to_string())
}

/// 更新超级管理员账号信息（安装完成时的初始设置）
///
/// 数据导入后 admin 表中已有默认超级管理员（user_type=1），此处仅 UPDATE
/// user_name/nick_name/email/mobile/password 字段，id 与角色/部门/关联关系全部保持不变。
/// 以 admin_user_name 是否为作为是否执行的开关：未填写则跳过，不阻塞安装。
async fn setup_super_admin(req: &CompleteRequest) -> Result<String> {
    let Some(admin_user_name) = req
        .admin_user_name
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    else {
        return Ok("未提供管理员账号信息，跳过".to_string());
    };

    let db_url = build_db_url(&req.host, req.port, &req.username, &req.password, &req.database);
    let conn = sea_orm::Database::connect(&db_url)
        .await
        .map_err(|e| Error::E(format!("无法连接数据库更新管理员: {}", e)))?;

    use sea_orm::{
        ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
    };
    use crate::modules::system::entity::admin;

    // 定位超管（user_type=1 且未删除）
    let super_admin = admin::Entity::find()
        .filter(admin::Column::UserType.eq(1))
        .filter(admin::Column::Deleted.eq(0))
        .one(&conn)
        .await
        .map_err(|e| Error::E(format!("查询超级管理员失败: {}", e)))?
        .ok_or_else(|| Error::E("数据库中未找到超级管理员账号，请先完成数据导入".to_string()))?;

    let mut am: admin::ActiveModel = super_admin.clone().into();

    // 登录账号：统一小写去空格 + 唯一性校验
    let normalized_name = admin_user_name.replace(' ', "").to_lowercase();
    if normalized_name.is_empty() {
        return Err(Error::E("管理员登录账号不能为空".to_string()));
    }
    if normalized_name.len() > 38 {
        return Err(Error::E("管理员登录账号过长（最多 38 个字符）".to_string()));
    }
    let name_dup = admin::Entity::find()
        .filter(admin::Column::UserName.eq(&normalized_name))
        .filter(admin::Column::Id.ne(super_admin.id))
        .count(&conn)
        .await
        .map_err(|e| Error::E(e.to_string()))?;
    if name_dup > 0 {
        return Err(Error::E(format!(
            "登录账号 {} 已被其他用户使用",
            normalized_name
        )));
    }
    am.user_name = Set(Some(normalized_name));

    if let Some(nick) = req.admin_nick_name.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        am.nick_name = Set(Some(nick.to_string()));
    }

    if let Some(email) = req.admin_email.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let email = email.to_lowercase();
        if !email.contains('@') || email.contains(char::is_whitespace) {
            return Err(Error::E("管理员邮箱格式不正确".to_string()));
        }
        let email_dup = admin::Entity::find()
            .filter(admin::Column::Email.eq(&email))
            .filter(admin::Column::Id.ne(super_admin.id))
            .count(&conn)
            .await
            .map_err(|e| Error::E(e.to_string()))?;
        if email_dup > 0 {
            return Err(Error::E(format!("邮箱 {} 已被其他账号使用", email)));
        }
        am.email = Set(Some(email));
    }

    if let Some(mobile) = req.admin_mobile.as_deref().filter(|s| !s.trim().is_empty()) {
        let mobile =
            crate::utils::string_utils::normalize_and_validate_cn_mobile(Some(&mobile.to_string()))
                .map_err(Error::E)?;
        let mobile_dup = admin::Entity::find()
            .filter(admin::Column::Mobile.eq(&mobile))
            .filter(admin::Column::Id.ne(super_admin.id))
            .count(&conn)
            .await
            .map_err(|e| Error::E(e.to_string()))?;
        if mobile_dup > 0 {
            return Err(Error::E(format!("手机号 {} 已被其他账号绑定", mobile)));
        }
        am.mobile = Set(Some(mobile));
    }

    if let Some(pwd) = req.admin_password.as_deref() {
        let pwd = pwd.trim();
        if pwd.is_empty() {
            return Err(Error::E("管理员密码不能为空".to_string()));
        }
        if pwd.len() < 6 {
            return Err(Error::E("管理员密码长度不能少于 6 位".to_string()));
        }
        let hashed = bcrypt::hash(pwd, bcrypt::DEFAULT_COST)
            .map_err(|e| Error::E(format!("密码加密失败: {}", e)))?;
        am.password = Set(Some(hashed));
    }

    am.update_time = Set(Some(chrono::Local::now().naive_local()));
    am.update(&conn)
        .await
        .map_err(|e| Error::E(format!("更新超级管理员失败: {}", e)))?;

    install::log_install("超级管理员账号信息已更新");
    Ok("超级管理员账号信息已更新".to_string())
}