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
use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, Result};
use actix_web::error::InternalError;
use utils::snowflake::Snowflake;
use crate::core::web::response::MetaResp;

#[allow(unused_imports)]
#[macro_use]
extern crate rust_i18n;

use crate::core::kit::db::connect;
use crate::core::kit::global::AppState;
use crate::core::kit::config;
use crate::routes::{admin_routes, merchant_routes, open_routes, user_routes};
use crate::embed_frontend::FrontendAssets;

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

    log::info!("starting HTTP server at {:}",&config::section::<String>("server", "server_url", "http://127.0.0.1".to_string()));

    init_storage_dirs();

    let conn = connect().await.unwrap_or_default();

    let state = AppState {
        db: conn,
    };

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
                    .content_type("application/msgpack")
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