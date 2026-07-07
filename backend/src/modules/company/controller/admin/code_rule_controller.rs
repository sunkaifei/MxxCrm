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
use crate::core::web::base_controller::get_user;
use crate::core::web::response::MetaResp;
use crate::modules::company::model::code_rule::{
    BatchRegenerateReq, CodeRuleSaveReq, GenerateCodeReq, PreviewCodeReq,
};
use crate::modules::company::service::code_rule_service;
use actix_web::{delete, get, post, put, web, HttpResponse, HttpRequest};
use actix_web_grants::protect;
use chrono::NaiveDate;
use serde::Deserialize;

/// 分页查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeRuleListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub module_code: Option<String>,
    pub enabled: Option<i16>,
}

/// 列表查询（分页）
#[get("/code-rule/list")]
#[protect("company:code:list")]
pub async fn list(state: web::Data<AppState>, query: web::Query<CodeRuleListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20);
    match code_rule_service::list(
        db,
        page,
        page_size,
        q.module_code.as_deref(),
        q.enabled,
    )
    .await
    {
        Ok(page_data) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(page_data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 详情查询
#[get("/code-rule/info/{id}")]
#[protect("company:code:list")]
pub async fn info(state: web::Data<AppState>, id: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    match code_rule_service::find_by_id(db, id.into_inner()).await {
        Ok(vo) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(vo, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 新增规则
#[post("/code-rule")]
#[protect("company:code:add")]
pub async fn create(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CodeRuleSaveReq>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt.id.unwrap_or_default();
    let form = item.0;
    if form.module_code.is_empty() {
        return Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "模块编码不能为空", "local")));
    }
    if form.module_name.is_empty() {
        return Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "模块名称不能为空", "local")));
    }
    if form.segments.is_empty() {
        return Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "段位配置不能为空", "local")));
    }
    match code_rule_service::create(db, form, user_id).await {
        Ok(id) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 修改规则
#[put("/code-rule/{id}")]
#[protect("company:code:update")]
pub async fn update(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
    item: web::Json<CodeRuleSaveReq>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt.id.unwrap_or_default();
    let form = item.0;
    if form.segments.is_empty() {
        return Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "段位配置不能为空", "local")));
    }
    match code_rule_service::update(db, id.into_inner(), form, user_id).await {
        Ok(rows) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(rows, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 删除规则（软删除）
#[delete("/code-rule/{id}")]
#[protect("company:code:delete")]
pub async fn delete(state: web::Data<AppState>, id: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    match code_rule_service::delete(db, id.into_inner()).await {
        Ok(rows) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(rows, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 启用/停用
#[put("/code-rule/toggle/{id}/{enabled}")]
#[protect("company:code:update")]
pub async fn toggle_enabled(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(i64, i16)>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt: JWTToken = get_user(&req).unwrap_or_default();
    let (id, enabled) = path.into_inner();
    match code_rule_service::toggle_enabled(db, id, enabled, jwt.id.unwrap_or_default()).await {
        Ok(rows) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(rows, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 预览编号（不入库，不递增计数器）
#[post("/code-rule/preview")]
#[protect("company:code:list")]
pub async fn preview(state: web::Data<AppState>, item: web::Json<PreviewCodeReq>) -> Result<HttpResponse> {
    let db = &state.db;
    match code_rule_service::preview(db, item.0).await {
        Ok(no) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(no, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 生成编号（递增流水号，入库）
/// 登录用户即可调用，实际由各业务模块在 service 层直接调用 generate_code
/// 支持传入 previous_version 进行版本递增（如 "V1" → "V2"）
#[post("/code-rule/generate")]
#[protect("company:code:list")]
pub async fn generate(state: web::Data<AppState>, item: web::Json<GenerateCodeReq>) -> Result<HttpResponse> {
    let db = &state.db;
    let form = item.0;
    let business_date = form
        .business_date
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    match code_rule_service::generate_code(
        db,
        &form.module_code,
        form.dept_code.as_deref(),
        business_date,
        form.previous_version.as_deref(),
    )
    .await
    {
        Ok(no) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(no, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 一键更新已有编号（异步任务，立即返回任务ID）
#[post("/code-rule/batch-regenerate")]
#[protect("company:code:regenerate")]
pub async fn batch_regenerate(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<BatchRegenerateReq>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt: JWTToken = get_user(&req).unwrap_or_default();
    match code_rule_service::start_batch_regenerate(db, item.0, jwt.id.unwrap_or_default()).await {
        Ok(task_id) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(task_id, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 查询一键更新任务进度
#[get("/code-rule/batch-regenerate/progress")]
#[protect("company:code:regenerate")]
pub async fn batch_regenerate_progress() -> Result<HttpResponse> {
    match code_rule_service::get_batch_progress() {
        Ok(p) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(p, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}
