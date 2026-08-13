//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::sync::LazyLock;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse};
use actix_web::error::InternalError;
use utils::snowflake::Snowflake;
use crate::core::web::response::{MetaResp, MPACK};

#[allow(unused_imports)]
#[macro_use]
extern crate rust_i18n;

use crate::core::kit::db::connect;
use crate::core::kit::global::AppState;
use crate::core::kit::config;
use crate::core::kit::install;
use crate::routes::{admin_routes, merchant_routes, open_routes, user_routes, install_routes};
use crate::embed_frontend::{FrontendAssets, InstallAssets};

pub mod core;
pub mod utils;
pub mod modules;
pub mod routes;
pub mod embed_frontend;

rust_i18n::i18n!("locales");

pub static SNOWFLAKE: LazyLock<Snowflake> = LazyLock::new(|| {
    Snowflake::new(1,1,1)
});

async fn serve_frontend(req: HttpRequest) -> HttpResponse {
    let path = req.path().trim_start_matches('/');

    // 对 /api 路径返回 JSON 404，避免返回 HTML 导致前端 "Unknown content type" 错误
    if req.path().starts_with("/api") {
        return HttpResponse::NotFound()
            .content_type("application/json")
            .body(r#"{"code":404,"msg":"接口不存在","data":null}"#);
    }

    if let Some(file) = FrontendAssets::get(path) {
        let content_type = match path.split('.').last() {
            Some("html") => "text/html; charset=utf-8",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("ico") => "image/x-icon",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("svg") => "image/svg+xml",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            Some("ttf") => "font/ttf",
            _ => "application/octet-stream",
        };
        
        HttpResponse::Ok()
            .content_type(content_type)
            .body(file.data)
    } else {
        match FrontendAssets::get("index.html") {
            Some(index) => HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(index.data),
            None => HttpResponse::NotFound().body("404 Not Found"),
        }
    }
}

/// 安装模式下的默认页面服务
async fn serve_install_page(req: HttpRequest) -> HttpResponse {
    let path = req.path().trim_start_matches('/');

    // 对 /api 路径返回 JSON 404
    if req.path().starts_with("/api") {
        return HttpResponse::NotFound()
            .content_type("application/json")
            .body(r#"{"code":404,"msg":"接口不存在","data":null}"#);
    }

    // 尝试返回安装页面的静态资源
    if let Some(file) = InstallAssets::get(path) {
        let content_type = match path.split('.').last() {
            Some("html") => "text/html; charset=utf-8",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("ico") => "image/x-icon",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("svg") => "image/svg+xml",
            _ => "application/octet-stream",
        };
        HttpResponse::Ok()
            .content_type(content_type)
            .body(file.data)
    } else {
        // 默认返回安装页面 HTML
        match InstallAssets::get("index.html") {
            Some(index) => HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(index.data),
            None => HttpResponse::NotFound().body("404 Not Found - 安装页面未找到，请确认 static/install/index.html 存在"),
        }
    }
}

fn init_storage_dirs() {
    use std::fs;
    let upload_dirs = [
        "storage/upload/product/",
        "storage/upload/avatar/",
        "storage/upload/contract/",
        "storage/upload/invoice/",
        "storage/upload/quotation/",
        "storage/upload/payment/",
        "storage/upload/common/",
    ];
    for dir in upload_dirs {
        if let Err(e) = fs::create_dir_all(dir) {
            log::warn!("Failed to create storage directory {}: {}", dir, e);
        } else {
            log::info!("Created storage directory: {}", dir);
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("./config/log4rs.yaml", Default::default()).unwrap_or_default();

    // ===== 安装模式判定 =====
    let install_mode = !install::is_installed() && !install::check_database_ready().await;

    if install_mode {
        log::info!("======== 进入安装模式 ========");
        install::log_install("启动安装模式");

        // 安全：安装模式强制绑定 127.0.0.1，避免安装向导暴露到公网
        let host = "127.0.0.1".to_string();
        let preferred_port = config::section::<u16>("server", "server_port", 8080);
        // 端口冲突时自动顺延
        let port = install::find_available_port(&host, preferred_port).await;
        let url = format!("http://{}:{}", host, port);

        // 启动 HTTP 服务（仅安装路由）
        let server = HttpServer::new(|| {
            let cors = Cors::default()
                .allow_any_origin()
                .allowed_methods(vec!["GET", "POST"])
                .supports_credentials()
                .max_age(36000);

            App::new()
                .wrap(cors)
                .configure(install_routes::configure)
                .default_service(web::get().to(serve_install_page))
        })
        .bind(format!("{}:{}", host, port))?;

        // 延迟 1 秒后自动打开浏览器
        let browser_url = url.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            install::open_browser(&browser_url);
        });

        log::info!("安装向导已启动，请访问: {}", url);
        install::log_install(format!("安装向导已启动: {}", url));
        return server.run().await;
    }

    // ===== 以下为现有正常启动逻辑 =====
    log::info!("starting HTTP server at {:}",&config::section::<String>("server", "server_url", "http://127.0.0.1".to_string()));

    init_storage_dirs();

    // 初始化 PDF 模块字体（启动时加载一次，后续复用）
    crate::modules::system::service::typst_world::init_fonts();

    let conn = connect().await.unwrap_or_default();

    // 注入全局 DB 连接（供 permission_cache_service 等无法通过请求上下文获取 db 的场景使用）
    crate::core::kit::CONTEXT.set_db(conn.clone());

    // 初始化消息系统表
    match crate::modules::message::migration::init_message_tables(&conn).await {
        Ok(_) => {
            log::info!("[消息系统] 数据库表初始化完成");
        }
        Err(e) => {
            log::error!("[消息系统] 数据库表初始化失败: {:?}", e);
        }
    }

    // 初始化 DB session 表（mem 缓存模式重启后降级验证用，防止重启丢登录态）
    crate::modules::system::service::session_service::ensure_session_table(&conn).await;

    // 一次性数据迁移：ai_config / mail_config → 统一配置表（幂等，已迁移则跳过）
    match crate::modules::system::service::integration_config_service::migrate_legacy_configs(
        &conn,
    )
    .await
    {
        Ok(_) => log::info!("[配置迁移] 旧配置迁移检查完成"),
        Err(e) => log::error!("[配置迁移] 旧配置迁移失败: {:?}", e),
    }

    let state = AppState {
        db: conn.clone(),
    };

    // 启动定时任务调度器（每月1号 02:00 自动核算上月工资）
    match crate::core::kit::scheduler::start_scheduler(conn.clone()).await {
        Ok(_) => log::info!("[定时任务] 调度器启动成功"),
        Err(e) => log::error!("[定时任务] 调度器启动失败: {:?}", e),
    }

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .supports_credentials()
            .max_age(36000);

        let json_cfg = web::JsonConfig::default()
            .limit(1024 * 1024 * 10)
            .error_handler(|err, _req| {
                let body = MetaResp::<()>::fail(400, &err.to_string(), "local");
                let response = HttpResponse::BadRequest()
                    .content_type(MPACK)
                    .body(body);
                InternalError::from_response(err, response).into()
            });

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(state.clone()), )
            .app_data(json_cfg)
            .configure(open_routes::configure_routes)
            .configure(admin_routes::configure_routes)
            .configure(merchant_routes::configure_routes)
            .configure(user_routes::configure_routes)
            .default_service(web::get().to(serve_frontend))
    })
        .bind(format!("{}:{}", 
            config::section::<String>("server", "server_host", "127.0.0.1".to_string()),
            config::section::<u16>("server", "server_port", 8088)))?
        .run()
        .await

}