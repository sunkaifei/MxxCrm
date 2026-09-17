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
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web::http::StatusCode;
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
use crate::routes::{admin_routes, open_routes, user_routes, install_routes};
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

/// 根据扩展名推断 Content-Type
///
/// 注意必须传入**不含 `.br` 后缀**的原始路径，否则会被识别成 octet-stream。
fn mime_for(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") | Some("mjs") => "application/javascript",
        Some("json") | Some("map") => "application/json",
        Some("ico") => "image/x-icon",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}

/// 是否为 Vite 产出的带内容哈希的静态资源（形如 `js/index-a1b2c3d4.js`）
///
/// 哈希与内容一一对应，内容永不变 → 可安全使用 immutable 长效缓存；
/// 而 `index.html` 每次构建都可能变，必须每次 revalidate，否则发版后用户拿到旧入口。
fn is_hashed_asset(path: &str) -> bool {
    let file = path.rsplit('/').next().unwrap_or(path);
    let stem = file.rsplit_once('.').map(|(s, _)| s).unwrap_or(file);
    match stem.rsplit_once('-') {
        Some((_, hash)) => hash.len() >= 6 && hash.chars().all(|c| c.is_ascii_alphanumeric()),
        None => false,
    }
}

/// 缓存策略：入口 HTML 每次校验，哈希资源一年强缓存，其余一小时
fn cache_control_for(path: &str) -> &'static str {
    if path == "index.html" || path.ends_with("/index.html") {
        "no-cache, must-revalidate"
    } else if is_hashed_asset(path) {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    }
}

/// 为响应体生成 ETag（进程内稳定即可，无需跨版本一致）
fn etag_of(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("\"{:x}-{:x}\"", hasher.finish(), data.len())
}

/// 构造静态资源响应：统一的 ETag / 304 / Cache-Control / Content-Encoding 处理
fn build_static_response(
    logical_path: &str,
    data: &[u8],
    is_br: bool,
    if_none_match: Option<&str>,
) -> HttpResponse {
    let etag = etag_of(data);
    let cache_control = cache_control_for(logical_path);

    // 条件请求命中 → 304，只回头部，不传正文
    if let Some(inm) = if_none_match {
        let inm = inm.trim().trim_start_matches("W/");
        if inm == etag {
            let mut resp = HttpResponse::build(StatusCode::NOT_MODIFIED);
            resp.content_type(mime_for(logical_path))
                .insert_header(("Cache-Control", cache_control))
                .insert_header(("ETag", etag.as_str()));
            if is_br {
                resp.insert_header(("Vary", "Accept-Encoding"));
            }
            return resp.body(actix_web::body::BoxBody::new(()));
        }
    }

    let mut resp = HttpResponse::Ok();
    resp.content_type(mime_for(logical_path))
        .insert_header(("Cache-Control", cache_control))
        .insert_header(("ETag", etag.as_str()));

    // brotli 命中时必须同时声明 Content-Encoding 与 Vary，
    // 否则中间代理会把 .br 内容误发给不支持的客户端。
    if is_br {
        resp.insert_header(("Content-Encoding", "br"))
            .insert_header(("Vary", "Accept-Encoding"));
    }

    resp.body(data.to_vec())
}

/// 按相对路径提供内嵌后台前端资源；找不到时回退 index.html（SPA 兜底）
///
/// `path` 为不含前缀、不含开头 `/` 的相对路径（如 "js/index-xxx.js"、"index.html"）。
/// 供默认转发 handler 与可配置前缀的 admin handler 共用。
///
/// 优化点：
/// - 客户端声明 `Accept-Encoding: br` 时，直接返回预生成的 `.br` 产物（体积约为原始的 1/5）
/// - 带哈希的资源返回 immutable 一年强缓存，`index.html` 走 no-cache
/// - 全量 ETag + `If-None-Match` → 304，二次访问仅传输头部
pub fn serve_frontend_asset(path: &str) -> HttpResponse {
    serve_frontend_asset_with_headers(path, None, None)
}

/// 同 [`serve_frontend_asset`]，额外传入请求头以支持 brotli 协商与条件请求
pub fn serve_frontend_asset_with_headers(
    path: &str,
    accept_encoding: Option<&str>,
    if_none_match: Option<&str>,
) -> HttpResponse {
    let want_br = accept_encoding
        .unwrap_or("")
        .to_ascii_lowercase()
        .contains("br");

    // 优先取 brotli 版本，缺失则回退原始文件
    let (actual_path, maybe_file, is_br) = if want_br {
        let br_path = format!("{}.br", path);
        match FrontendAssets::get(&br_path) {
            Some(file) => (br_path, Some(file), true),
            None => (path.to_string(), FrontendAssets::get(path), false),
        }
    } else {
        (path.to_string(), FrontendAssets::get(path), false)
    };

    if let Some(file) = maybe_file {
        return build_static_response(path, file.data.as_ref(), is_br, if_none_match);
    }

    // SPA 兜底：任意未知路径都回 index.html
    match FrontendAssets::get("index.html") {
        Some(index) => build_static_response(
            "index.html",
            index.data.as_ref(),
            false,
            if_none_match,
        ),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

async fn serve_frontend(req: HttpRequest) -> HttpResponse {
    // 对 /api 路径返回 JSON 404，避免返回 HTML 导致前端 "Unknown content type" 错误
    if req.path().starts_with("/api") {
        return HttpResponse::NotFound()
            .content_type("application/json")
            .body(r#"{"code":404,"msg":"接口不存在","data":null}"#);
    }

    let path = req.path().trim_start_matches('/');

    let accept_encoding = header_to_str(&req, "accept-encoding");
    let if_none_match = header_to_str(&req, "if-none-match");

    serve_frontend_asset_with_headers(
        path,
        accept_encoding.as_deref(),
        if_none_match.as_deref(),
    )
}

/// 读取请求头中为合法 ASCII 字符串的值（非 ASCII 的头一律忽略，避免 pan‑ic）
fn header_to_str(req: &HttpRequest, name: &str) -> Option<String> {
    req.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod static_asset_tests {
    use super::*;

    #[test]
    fn mime_resolved_by_extension() {
        assert_eq!(mime_for("index.html"), "text/html; charset=utf-8");
        assert_eq!(mime_for("js/index-a1b2c3.js"), "application/javascript");
        assert_eq!(mime_for("css/bootstrap-CuKxi1oo.css"), "text/css");
        assert_eq!(mime_for("favicon.ico"), "image/x-icon");
        // 必须传原始路径：带上 .br 会被识别成未知类型
        assert_eq!(mime_for("index.html.br"), "application/octet-stream");
    }

    #[test]
    fn hashed_asset_detection() {
        assert!(is_hashed_asset("js/index-a1b2c3d4.js"));
        assert!(is_hashed_asset("jse/index-index-COzsSR4Z.js"));
        assert!(is_hashed_asset("css/bootstrap-CuKxi1oo.css"));
        // 无哈希的资源不能被 immutable 缓存
        assert!(!is_hashed_asset("index.html"));
        assert!(!is_hashed_asset("logo.png"));
        assert!(!is_hashed_asset("_app.config.js"));
    }

    #[test]
    fn cache_control_policy() {
        assert!(cache_control_for("index.html").contains("no-cache"));
        assert!(cache_control_for("js/index-a1b2c3d4.js").contains("immutable"));
        assert!(cache_control_for("logo.png").contains("max-age=3600"));
    }

    #[test]
    fn etag_is_stable_and_length_aware() {
        let a = etag_of(b"hello");
        let b = etag_of(b"hello");
        let c = etag_of(b"world");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn brotli_negotiation_returns_not_panicking() {
        let resp = serve_frontend_asset_with_headers(
            "index.html",
            Some("gzip, deflate, br"),
            None,
        );
        let status = resp.status();
        assert!(
            status.is_success() || status.as_u16() == 404,
            "unexpected status: {status}"
        );
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

        // A-1.2: 弱默认 JWT 密钥启动告警（生产环境必须更换，泄露即可伪造任意用户登录态）
        warn_if_default_jwt_secrets();

        // 启动 HTTP 服务（仅安装路由）
        let server = HttpServer::new(|| {
            let cors = Cors::default()
                .allow_any_origin()
                .allowed_methods(vec!["GET", "POST"])
                .supports_credentials()
                .max_age(36000);

            App::new()
                .wrap(crate::core::middleware::error_trace::ErrorTrace)
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

    let conn = match connect().await {
        Ok(c) => c,
        Err(e) => {
            log::error!("数据库连接失败，程序退出: {}", e);
            eprintln!(
                "数据库连接失败: {}\n请检查 config/config.ini 的 [db] url（数据库地址、端口、库名、密码）以及 PostgreSQL 服务是否启动",
                e
            );
            std::process::exit(1);
        }
    };

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

    // 初始化 HR 离职交接相关表（交接单三表 + 交接项模板预置 + resign_approval 流程模板）
    match crate::modules::system::migration::init_hr_resign_tables(&conn).await {
        Ok(_) => {
            log::info!("[HR离职交接] 数据库表初始化完成");
        }
        Err(e) => {
            log::error!("[HR离职交接] 数据库表初始化失败: {:?}", e);
        }
    }

    // 初始化可视化 PDF 模板设计器表与菜单（v89：模板扩展列 + 素材/字段元数据/版本表 + 权限 seed）
    match crate::modules::system::migration::init_pdf_designer_tables(&conn).await {
        Ok(_) => {
            log::info!("[PDF设计器] 数据库表与菜单初始化完成");
        }
        Err(e) => {
            log::error!("[PDF设计器] 数据库表初始化失败: {:?}", e);
        }
    }

    // 修正 PDF 设计器/素材菜单挂载位置与可见性（v90：从 PDF模板页下移到系统管理目录，设计器页恢复可见）
    match crate::modules::system::migration::fix_pdf_menu_structure(&conn).await {
        Ok(_) => {
            log::info!("[PDF设计器] 菜单结构校正完成");
        }
        Err(e) => {
            log::error!("[PDF设计器] 菜单结构校正失败: {:?}", e);
        }
    }

    // 初始化网站展示产品 SKU 销售库存列（mxx_website_product.sku_quantities）
    match crate::modules::website::migration::init_website_product_sku_quantities(&conn).await {
        Ok(_) => {
            log::info!("[网站产品] 数据库结构迁移完成");
        }
        Err(e) => {
            log::error!("[网站产品] 数据库结构迁移失败: {:?}", e);
        }
    }

    // 初始化 DB session 表（mem 缓存模式重启后降级验证用，防止重启丢登录态）
    crate::modules::system::service::session_service::ensure_session_table(&conn).await;

    // 初始化调度告警日志表（幂等；调度告警不再写公告表 mxx_notice）
    crate::modules::system::service::scheduler_service::ensure_alert_table(&conn).await;

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

    // A-1.2: 弱默认 JWT 密钥启动告警（生产环境必须更换，泄露即可伪造任意用户登录态）
    warn_if_default_jwt_secrets();

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
            .wrap(crate::core::middleware::error_trace::ErrorTrace)
            .wrap(cors)
            .app_data(web::Data::new(state.clone()), )
            .app_data(json_cfg)
            .configure(open_routes::configure_routes)
            .configure(admin_routes::configure_routes)
            .configure(user_routes::configure_routes)
            .default_service(web::get().to(serve_frontend))
    })
        .keep_alive(actix_http::KeepAlive::Timeout(std::time::Duration::from_secs(30)))
        .bind(format!("{}:{}", 
            config::section::<String>("server", "server_host", "127.0.0.1".to_string()),
            config::section::<u16>("server", "server_port", 8088)))?
        .run()
        .await

}

/// A-1.2: 弱默认 JWT 密钥启动告警（泄露即可伪造任意用户登录态）
fn warn_if_default_jwt_secrets() {
    let admin_secret = config::section::<String>("server", "jwt_secret_admin", "".to_string());
    let user_secret = config::section::<String>("server", "jwt_secret_user", "".to_string());
    if admin_secret.is_empty() || admin_secret == "mxx_b2b_admin56789" {
        log::warn!("[安全告警] jwt_secret_admin 使用默认弱密钥，生产环境必须更换，否则登录态可被伪造！");
    }
    if user_secret.is_empty() || user_secret == "mxx_b2b_user1234567" {
        log::warn!("[安全告警] jwt_secret_user 使用默认弱密钥，生产环境必须更换！");
    }
}
