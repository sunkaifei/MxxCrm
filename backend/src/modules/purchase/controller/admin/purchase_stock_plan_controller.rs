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
use crate::core::web::base_controller::get_user;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::purchase::model::purchase_stock_plan::{StockPlanListQuery, StockPlanSaveRequest};
use crate::modules::purchase::service::purchase_stock_plan_service;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::entity::common::BathDeleteIdRequest;

pub async fn stock_plan_save(state: web::Data<AppState>, form_data: web::Json<StockPlanSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let result = purchase_stock_plan_service::insert(&db, &form_data).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn stock_plan_update(state: web::Data<AppState>, form_data: web::Json<StockPlanSaveRequest>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();
    let form_data = form_data.0;

    let result = purchase_stock_plan_service::update(&db, id, &form_data).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn stock_plan_batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let ids = item.0.parse_ids();

    let result = purchase_stock_plan_service::batch_delete(&db, &ids).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn stock_plan_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match purchase_stock_plan_service::get_info(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn stock_plan_list(state: web::Data<AppState>, query: web::Query<StockPlanListQuery>) -> Result<HttpResponse> {
    let db = &state.db;

    match purchase_stock_plan_service::get_list(&db, &query.into_inner()).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn stock_plan_generate_pr(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();

    match purchase_stock_plan_service::generate_pr(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn stock_plan_recalculate(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();

    match purchase_stock_plan_service::recalculate(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn stock_plan_warning_list(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;

    match purchase_stock_plan_service::get_warning_list(&db).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/purchase/stock_plan")
            .route("/save", web::post().to(stock_plan_save).wrap(require_permission("purchase:stock_plan:save")))
            .route("/update/{id}", web::put().to(stock_plan_update).wrap(require_permission("purchase:stock_plan:update")))
            .route("/bath_delete", web::delete().to(stock_plan_batch_delete).wrap(require_permission("purchase:stock_plan:delete")))
            .route("/info", web::get().to(stock_plan_info).wrap(require_permission("purchase:stock_plan:view")))
            .route("/list", web::get().to(stock_plan_list).wrap(require_permission("purchase:stock_plan:list")))
            .route("/generate_pr/{id}", web::put().to(stock_plan_generate_pr).wrap(require_permission("purchase:stock_plan:convert")))
            .route("/recalculate/{id}", web::put().to(stock_plan_recalculate).wrap(require_permission("purchase:stock_plan:update")))
            .route("/warning_list", web::get().to(stock_plan_warning_list).wrap(require_permission("purchase:stock_plan:list"))),
    );
}