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
use crate::modules::inventory::service::inventory_report_service;
use actix_web::{web, HttpRequest, HttpResponse};

/// 从 query_string 中提取指定 key 的值
fn q<'a>(qs: &'a str, key: &str) -> Option<&'a str> {
    qs.split('&')
        .find(|s| s.starts_with(&format!("{}=", key)))
        .and_then(|s| s.split('=').nth(1))
}

/// 收发存报表
pub async fn receive_send_report(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let qs = req.query_string();
    let warehouse_id = q(qs, "warehouseId").and_then(|s| s.parse().ok());
    let start_date = q(qs, "startDate").map(|s| s.to_string());
    let end_date = q(qs, "endDate").map(|s| s.to_string());

    match inventory_report_service::receive_send_stock_report(
        db,
        warehouse_id,
        start_date,
        end_date,
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 库存周转率报表
pub async fn turnover_report(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let qs = req.query_string();
    let warehouse_id = q(qs, "warehouseId").and_then(|s| s.parse().ok());
    let start_date = q(qs, "startDate").map(|s| s.to_string());
    let end_date = q(qs, "endDate").map(|s| s.to_string());

    match inventory_report_service::turnover_report(
        db,
        warehouse_id,
        start_date,
        end_date,
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 呆滞库存清单
pub async fn obsolete_report(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let qs = req.query_string();
    let warehouse_id = q(qs, "warehouseId").and_then(|s| s.parse().ok());
    let days = q(qs, "days").and_then(|s| s.parse().ok()).unwrap_or(90);

    match inventory_report_service::obsolete_stock_report(db, warehouse_id, days).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 库存成本报表
pub async fn cost_report(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let qs = req.query_string();
    let warehouse_id = q(qs, "warehouseId").and_then(|s| s.parse().ok());

    match inventory_report_service::cost_report(db, warehouse_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 入库汇总报表
pub async fn inbound_summary_report(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let qs = req.query_string();
    let warehouse_id = q(qs, "warehouseId").and_then(|s| s.parse().ok());
    let start_date = q(qs, "startDate").map(|s| s.to_string());
    let end_date = q(qs, "endDate").map(|s| s.to_string());

    match inventory_report_service::inbound_summary_report(
        db,
        warehouse_id,
        start_date,
        end_date,
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 出库汇总报表
pub async fn outbound_summary_report(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let qs = req.query_string();
    let warehouse_id = q(qs, "warehouseId").and_then(|s| s.parse().ok());
    let start_date = q(qs, "startDate").map(|s| s.to_string());
    let end_date = q(qs, "endDate").map(|s| s.to_string());

    match inventory_report_service::outbound_summary_report(
        db,
        warehouse_id,
        start_date,
        end_date,
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    // Keep existing routes
    cfg.service(
        web::scope("/inventory/report")
            .route("/receive_send", web::get().to(receive_send_report).wrap(require_permission("product:inventory:list")))
            .route("/turnover", web::get().to(turnover_report).wrap(require_permission("product:inventory:list")))
            .route("/obsolete", web::get().to(obsolete_report).wrap(require_permission("product:inventory:list")))
            .route("/cost", web::get().to(cost_report).wrap(require_permission("product:inventory:list"))),
    );
    // Add frontend-expected routes
    cfg.service(
        web::scope("/report")
            .route("/stock", web::get().to(receive_send_report).wrap(require_permission("product:inventory:list")))
            .route("/turnover", web::get().to(turnover_report).wrap(require_permission("product:inventory:list")))
            .route("/inbound_summary", web::get().to(inbound_summary_report).wrap(require_permission("product:inventory:list")))
            .route("/outbound_summary", web::get().to(outbound_summary_report).wrap(require_permission("product:inventory:list"))),
    );
}
