//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::core::errors::error::Result;

/// 锁文件路径
const LOCK_FILE: &str = "config/.install.lock";

/// 安装日志文件路径
const INSTALL_LOG_FILE: &str = "logs/install.log";

/// 协议是否已同意（进程内存中保存）
pub static LICENSE_ACCEPTED: AtomicBool = AtomicBool::new(false);

/// 安装日志写入器
static INSTALL_LOG: LazyLock<Mutex<fs::File>> = LazyLock::new(|| {
    fs::create_dir_all("logs").ok();
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(INSTALL_LOG_FILE)
        .expect("无法创建安装日志文件");
    Mutex::new(file)
});

/// 写入安装日志
pub fn log_install(msg: impl AsRef<str>) {
    let msg = msg.as_ref();
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    if let Ok(mut file) = INSTALL_LOG.lock() {
        let _ = writeln!(file, "[{}] {}", timestamp, msg);
    }
    log::info!("[INSTALL] {}", msg);
}

/// 判断是否已安装（锁文件存在）
pub fn is_installed() -> bool {
    Path::new(LOCK_FILE).exists()
}

/// 创建锁文件
pub fn create_lock_file() -> std::io::Result<()> {
    let content = serde_json::json!({
        "installed_at": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    });
    fs::write(LOCK_FILE, content.to_string())?;
    log_install(format!("锁文件已创建: {}", LOCK_FILE));
    Ok(())
}

/// 校验主机地址是否为本机
pub fn is_local_host(host: &str) -> bool {
    matches!(
        host.trim().to_lowercase().as_str(),
        "127.0.0.1" | "localhost" | "::1"
    )
}

/// 校验协议已同意（未同意返回 Err）
pub fn ensure_license_accepted() -> Result<()> {
    if LICENSE_ACCEPTED.load(Ordering::Relaxed) {
        Ok(())
    } else {
        Err(crate::core::errors::error::Error::BadRequest(
            "请先阅读并同意软件许可与免责协议".to_string(),
        ))
    }
}

/// 确保配置文件存在，不存在则创建默认模板
pub fn ensure_config_exists() -> std::io::Result<()> {
    let config_path = "config/config.ini";
    if !Path::new(config_path).exists() {
        fs::create_dir_all("config")?;
        let default_config = "\
[server]
server_host=127.0.0.1
server_port=8080

[db]
url=

[log]
level=info
";
        fs::write(config_path, default_config)?;
        log_install("配置文件已创建: config/config.ini");
    }
    Ok(())
}

/// 检测 pg_restore 工具是否可用
pub fn check_pg_restore() -> bool {
    std::process::Command::new("pg_restore")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 当前操作系统信息
#[derive(serde::Serialize)]
pub struct OsInfo {
    pub os_type: String,
    pub arch: String,
    pub name: String,
    pub download_url: String,
    pub install_hint: String,
}

/// 检测当前操作系统
pub fn detect_os() -> OsInfo {
    let os_type = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unknown"
    };

    let arch = std::env::consts::ARCH;

    let os_name = std::env::consts::OS.to_string();

    let (download_url, install_hint) = match os_type {
        "windows" => (
            "https://www.postgresql.org/download/windows/",
            "请下载并安装 Windows 版 PostgreSQL（EDB 安装包，包含 pg_restore 工具）",
        ),
        "linux" => (
            "https://www.postgresql.org/download/linux/",
            "请使用包管理器安装：Debian/Ubuntu: apt install postgresql postgresql-client；CentOS/RHEL: yum install postgresql-server postgresql",
        ),
        "macos" => (
            "https://www.postgresql.org/download/macosx/",
            "请下载 macOS 安装包，或使用 Homebrew: brew install postgresql",
        ),
        _ => (
            "https://www.postgresql.org/download/",
            "请安装 PostgreSQL",
        ),
    };

    OsInfo {
        os_type: os_type.to_string(),
        arch: arch.to_string(),
        name: os_name,
        download_url: download_url.to_string(),
        install_hint: install_hint.to_string(),
    }
}

/// 检测本机 PostgreSQL 服务是否可用（5432 端口连通性）
pub async fn check_postgres_service(host: &str, port: u16) -> bool {
    tokio::net::TcpStream::connect((host, port)).await.is_ok()
}

/// 检查目标数据库是否已安装 pgcrypto 扩展
pub async fn check_pgcrypto(db: &sea_orm::DatabaseConnection) -> bool {
    use sea_orm::*;
    let sql = "SELECT 1 FROM pg_extension WHERE extname = 'pgcrypto'";
    match db.execute_unprepared(sql).await {
        Ok(result) => result.rows_affected() > 0,
        Err(_) => false,
    }
}

/// 安装 pgcrypto 扩展（需 superuser 权限）
pub async fn install_pgcrypto(db: &sea_orm::DatabaseConnection) -> Result<()> {
    use sea_orm::*;
    db.execute_unprepared("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .await
        .map_err(|e| {
            crate::core::errors::error::Error::E(format!("安装 pgcrypto 扩展失败: {}", e))
        })?;
    log_install("pgcrypto 扩展已安装");
    Ok(())
}

/// 找到可用端口（从 preferred 开始，最多尝试 50 个）
pub async fn find_available_port(host: &str, preferred: u16) -> u16 {
    let mut port = preferred;
    for _ in 0..50 {
        if tokio::net::TcpListener::bind((host, port)).await.is_ok() {
            return port;
        }
        port += 1;
    }
    port
}

/// 自动打开浏览器
#[cfg(target_os = "windows")]
pub fn open_browser(url: &str) {
    std::process::Command::new("cmd")
        .args(["/c", "start", url])
        .spawn()
        .ok();
}

#[cfg(target_os = "linux")]
pub fn open_browser(url: &str) {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .ok();
}

#[cfg(target_os = "macos")]
pub fn open_browser(url: &str) {
    std::process::Command::new("open")
        .arg(url)
        .spawn()
        .ok();
}

/// 检查核心表是否存在
pub async fn check_core_tables_exist(db: &sea_orm::DatabaseConnection) -> bool {
    use sea_orm::*;
    let sql = "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'mxx_system_admin'";
    match db.execute_unprepared(sql).await {
        Ok(result) => result.rows_affected() > 0,
        Err(_) => false,
    }
}

/// 检查数据库连接是否正常
pub async fn check_database_ready() -> bool {
    let url = crate::core::kit::config::section::<String>("db", "url", String::new());
    if url.is_empty() {
        return false;
    }
    match sea_orm::Database::connect(&url).await {
        Ok(conn) => {
            let ready = check_core_tables_exist(&conn).await;
            let _ = conn.close().await;
            ready
        }
        Err(_) => false,
    }
}

/// 安排自动重启
pub fn schedule_restart() {
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(2));
        let exe = std::env::current_exe().expect("获取程序路径失败");
        let _ = std::process::Command::new(exe)
            .current_dir(std::env::current_dir().unwrap())
            .spawn();
        std::process::exit(0);
    });
}