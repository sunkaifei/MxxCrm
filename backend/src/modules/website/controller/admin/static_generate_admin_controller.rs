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
use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::website::service::static_generate_service;

/// POST /static_generate/all - 生成所有静态页面
pub async fn generate_all(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match static_generate_service::generate_all(db).await {
        Ok((cat_count, art_count)) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(
                serde_json::json!({
                    "categories": cat_count,
                    "articles": art_count,
                }),
                "local",
            ))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// POST /static_generate/index - 生成首页
pub async fn generate_index(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match static_generate_service::generate_index(db).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success("首页静态化完成", "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// POST /static_generate/categories - 生成所有栏目页
pub async fn generate_categories(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match static_generate_service::generate_categories(db).await {
        Ok(count) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(
                serde_json::json!({ "count": count }),
                "local",
            ))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// POST /static_generate/articles - 生成所有文章页
pub async fn generate_articles(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match static_generate_service::generate_articles(db).await {
        Ok(count) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(
                serde_json::json!({ "count": count }),
                "local",
            ))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// DELETE /static_generate/clear - 清空静态化输出目录
pub async fn clear_output() -> Result<HttpResponse> {
    match static_generate_service::clear_output() {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success("静态化目录已清空", "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 注册静态化管理路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/static_generate")
            .route("/all", web::post().to(generate_all).wrap(require_permission("website:static:generate")))
            .route("/index", web::post().to(generate_index).wrap(require_permission("website:static:generate")))
            .route("/categories", web::post().to(generate_categories).wrap(require_permission("website:static:generate")))
            .route("/articles", web::post().to(generate_articles).wrap(require_permission("website:static:generate")))
            .route("/clear", web::delete().to(clear_output).wrap(require_permission("website:static:clear"))),
    );
}