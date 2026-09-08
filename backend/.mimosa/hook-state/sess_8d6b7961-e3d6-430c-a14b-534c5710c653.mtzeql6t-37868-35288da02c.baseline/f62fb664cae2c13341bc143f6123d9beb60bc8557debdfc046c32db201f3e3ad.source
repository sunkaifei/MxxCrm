//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use crate::core::errors::error::Result;
use crate::core::kit::install;
use crate::core::web::response::{ok_success, ok_fail};
use crate::modules::install::service;

/// 获取免责协议
pub async fn get_license() -> Result<HttpResponse> {
    // 从内嵌资源中读取 license.txt
    let license = crate::embed_frontend::InstallAssets::get("license.txt")
        .ok_or_else(|| crate::core::errors::error::Error::E("协议文件不存在".to_string()))?;

    let content = std::str::from_utf8(&license.data)
        .map_err(|_| crate::core::errors::error::Error::E("协议文件编码错误".to_string()))?;

    let data = serde_json::json!({
        "title": "Mxx-CRM 软件使用许可与免责协议",
        "content": content,
        "version": "1.0",
        "updated_at": "2026-08-04"
    });

    Ok(ok_success(data))
}

/// 接受免责协议
pub async fn license_accept(body: web::Json<service::LicenseAcceptRequest>) -> Result<HttpResponse> {
    if !body.accepted {
        return Ok(ok_fail("请勾选同意协议"));
    }
    install::LICENSE_ACCEPTED.store(true, std::sync::atomic::Ordering::Relaxed);
    install::log_install("用户已同意免责协议");
    Ok(ok_success(serde_json::json!({"accepted": true})))
}

/// 环境检测
pub async fn env_check() -> Result<HttpResponse> {
    let os = install::detect_os();
    let pg_restore_available = install::check_pg_restore();
    let pg_service = install::check_postgres_service("127.0.0.1", 5432).await;

    let data = serde_json::json!({
        "os": {
            "type": os.os_type,
            "arch": os.arch,
            "name": os.name,
            "download_url": os.download_url,
            "install_hint": os.install_hint
        },
        "postgresql": {
            "installed": pg_service || pg_restore_available,
            "service_running": pg_service,
            "pg_restore_available": pg_restore_available,
            "download_url": os.download_url
        },
        "supported": true
    });

    Ok(ok_success(data))
}

/// 安装状态检查
pub async fn status() -> Result<HttpResponse> {
    let installed = install::is_installed();
    let data = serde_json::json!({
        "installed": installed,
        "pg_restore_available": install::check_pg_restore(),
        "version": env!("CARGO_PKG_VERSION"),
        "local_only": true
    });
    Ok(ok_success(data))
}

/// 测试数据库连接
pub async fn test_connection(body: web::Json<service::TestConnectionRequest>) -> Result<HttpResponse> {
    match service::test_connection(&body).await {
        Ok(resp) => Ok(ok_success(resp)),
        Err(e) => Ok(ok_fail(&e.to_string())),
    }
}

/// 创建数据库
pub async fn create_database(body: web::Json<service::CreateDatabaseRequest>) -> Result<HttpResponse> {
    match service::create_database(&body).await {
        Ok(msg) => Ok(ok_success(serde_json::json!({"message": msg}))),
        Err(e) => Ok(ok_fail(&e.to_string())),
    }
}

/// 导入数据库
pub async fn import_database(body: web::Json<service::ImportRequest>) -> Result<HttpResponse> {
    match service::import_database(&body).await {
        Ok(task_id) => Ok(ok_success(serde_json::json!({
            "task_id": task_id,
            "status": "pending"
        }))),
        Err(e) => Ok(ok_fail(&e.to_string())),
    }
}

/// 查询导入进度
#[derive(Deserialize)]
pub struct ProgressQuery {
    pub task_id: String,
}

pub async fn import_progress(query: web::Query<ProgressQuery>) -> Result<HttpResponse> {
    match service::get_import_progress(&query.task_id).await {
        Ok(task) => Ok(ok_success(task)),
        Err(e) => Ok(ok_fail(&e.to_string())),
    }
}

/// 完成安装
pub async fn complete_install(body: web::Json<service::CompleteRequest>) -> Result<HttpResponse> {
    match service::complete_install(&body).await {
        Ok(msg) => {
            // 安排自动重启
            install::schedule_restart();
            Ok(ok_success(serde_json::json!({"message": msg})))
        }
        Err(e) => Ok(ok_fail(&e.to_string())),
    }
}