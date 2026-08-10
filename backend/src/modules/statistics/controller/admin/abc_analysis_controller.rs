//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::statistics::service::abc_analysis_service;
use actix_web::{web, HttpResponse};

pub async fn run(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match abc_analysis_service::run_abc_analysis(db).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn summary(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match abc_analysis_service::get_abc_summary(db).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/statistics/abc")
            .route(
                "/run",
                web::post().to(run).wrap(require_permission("statistics:product:view")),
            )
            .route(
                "/summary",
                web::get().to(summary).wrap(require_permission("statistics:product:view")),
            ),
    );
}
