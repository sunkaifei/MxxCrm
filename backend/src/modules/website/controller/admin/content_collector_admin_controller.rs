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
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::service::content_collector_service;

/// POST /content_collector/run - 执行所有启用的采集规则
pub async fn run_collect(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match content_collector_service::collect_all(db).await {
        Ok(count) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(
                serde_json::json!({ "collected": count }),
                "local",
            ))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 注册内容采集器管理路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/content_collector")
            .route("/run", web::post().to(run_collect).wrap(require_permission("website:collector:run"))),
    );
}