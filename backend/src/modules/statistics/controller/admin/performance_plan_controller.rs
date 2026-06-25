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
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::response::MetaResp;
use crate::modules::statistics::model::performance_plan::{
    CreatePlanRequest, SubmitPlanRequest, ReviewPlanRequest, ModifyPlanRequest, PlanQuery,
};
use crate::modules::statistics::service::performance_plan_service;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

/// 从Admin JWT中获取当前用户信息
fn get_admin_info(req: &HttpRequest) -> (i64, String) {
    let jwt_secret = crate::config::section::<String>("server", "jwt_secret_admin", "".to_string());
    let token_str = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim_start_matches("Bearer ")
        .to_string();

    match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(data) => (
            data.id.unwrap_or(0),
            data.username.unwrap_or_default(),
        ),
        Err(_) => (0, String::new()),
    }
}

/// 创建草稿计划
#[post("/statistics/performance/plan/create")]
#[protect("statistics:performance-plan:manage")]
pub async fn create_plan(state: web::Data<AppState>, req: web::Json<CreatePlanRequest>, http_req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let req = req.into_inner();
    let (employee_id, _) = get_admin_info(&http_req);

    match performance_plan_service::create_plan(db, employee_id, &req).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 提交计划（草稿→待审批）
#[post("/statistics/performance/plan/submit")]
#[protect("statistics:performance-plan:manage")]
pub async fn submit_plan(state: web::Data<AppState>, req: web::Json<SubmitPlanRequest>, http_req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let req = req.into_inner();
    let (user_id, user_name) = get_admin_info(&http_req);

    match performance_plan_service::submit_plan(db, req.plan_id, user_id, &user_name).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 审批通过
#[post("/statistics/performance/plan/approve")]
#[protect("statistics:performance-plan:approve")]
pub async fn approve_plan(state: web::Data<AppState>, req: web::Json<ReviewPlanRequest>, http_req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let req = req.into_inner();
    let (user_id, user_name) = get_admin_info(&http_req);

    match performance_plan_service::approve_plan(db, &req, user_id, &user_name).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 驳回
#[post("/statistics/performance/plan/reject")]
#[protect("statistics:performance-plan:approve")]
pub async fn reject_plan(state: web::Data<AppState>, req: web::Json<ReviewPlanRequest>, http_req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let req = req.into_inner();
    let (user_id, user_name) = get_admin_info(&http_req);

    match performance_plan_service::reject_plan(db, &req, user_id, &user_name).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 申请修改
#[post("/statistics/performance/plan/modify")]
#[protect("statistics:performance-plan:manage")]
pub async fn modify_plan(state: web::Data<AppState>, req: web::Json<ModifyPlanRequest>, http_req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let req = req.into_inner();
    let (user_id, user_name) = get_admin_info(&http_req);

    match performance_plan_service::modify_plan(db, &req, user_id, &user_name).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 查询计划列表
#[get("/statistics/performance/plan/list")]
#[protect("statistics:performance-plan:view")]
pub async fn get_plan_list(state: web::Data<AppState>, query: web::Query<PlanQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();

    match performance_plan_service::get_plan_list(db, query.employee_id, query.year, query.status).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 查询计划详情（含月度目标和审批记录）
#[get("/statistics/performance/plan/detail")]
#[protect("statistics:performance-plan:view")]
pub async fn get_plan_detail(state: web::Data<AppState>, query: web::Query<SubmitPlanRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let plan_id = query.plan_id;

    match performance_plan_service::get_plan_detail(db, plan_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 获取计划修改详情（编辑回显）
#[get("/statistics/performance/plan/modify-detail")]
#[protect("statistics:performance-plan:view")]
pub async fn get_plan_modify_detail(state: web::Data<AppState>, query: web::Query<SubmitPlanRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let plan_id = query.plan_id;

    match performance_plan_service::get_plan_modify_detail(db, plan_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}