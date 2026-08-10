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
use crate::modules::statistics::service::customer_ltv_service;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LtvQuery {
    pub customer_id: i64,
}

#[derive(Deserialize)]
pub struct RepurchaseRateQuery {
    pub year: i32,
    pub month: i32,
}

#[derive(Deserialize)]
pub struct RepurchaseTrendQuery {
    pub months: Option<i32>,
}

#[derive(Deserialize)]
pub struct TopLtvQuery {
    pub limit: Option<i32>,
}

pub async fn ltv(state: web::Data<AppState>, query: web::Query<LtvQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.0;
    if query.customer_id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "客户ID不能为空", "local")));
    }
    match customer_ltv_service::get_customer_ltv(db, query.customer_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn repurchase_rate(state: web::Data<AppState>, query: web::Query<RepurchaseRateQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.0;
    match customer_ltv_service::get_repurchase_rate(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn repurchase_trend(state: web::Data<AppState>, query: web::Query<RepurchaseTrendQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.0;
    let months = query.months.unwrap_or(6);
    match customer_ltv_service::get_repurchase_analysis(db, months).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn top(state: web::Data<AppState>, query: web::Query<TopLtvQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.0;
    let limit = query.limit.unwrap_or(10);
    match customer_ltv_service::get_top_ltv_customers(db, limit).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/statistics/customer-ltv")
            .route(
                "/ltv",
                web::get().to(ltv).wrap(require_permission("statistics:customer:view")),
            )
            .route(
                "/repurchase-rate",
                web::get().to(repurchase_rate).wrap(require_permission("statistics:customer:view")),
            )
            .route(
                "/repurchase-trend",
                web::get().to(repurchase_trend).wrap(require_permission("statistics:customer:view")),
            )
            .route(
                "/top",
                web::get().to(top).wrap(require_permission("statistics:customer:view")),
            ),
    );
}
