//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{get, post, web, HttpResponse, Result};
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
use crate::modules::finance::model::finance_statistics::FinanceStatisticsQuery;
use crate::modules::finance::service::finance_statistics_service;

#[get("/statistics/summary")]
pub async fn summary(
    state: web::Data<AppState>
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = finance_statistics_service::get_summary(db).await;
    
    match result {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/list")]
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<FinanceStatisticsQuery>
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = finance_statistics_service::get_list(db, query.into_inner()).await;
    
    match result {
        Ok(list) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(list, "local"))),
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[post("/statistics/generate-daily")]
pub async fn generate_daily(
    state: web::Data<AppState>
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = finance_statistics_service::generate_daily_statistics(db).await;
    
    match result {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(summary);
    cfg.service(list);
    cfg.service(generate_daily);
}
