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
use crate::modules::approval::model::approval::{
    ApprovalProcessRequest, ApprovalSubmitRequest, FlowListQuery, FlowSaveRequest,
};
use crate::modules::approval::service::approval_service::ApprovalService;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PageQuery {
    #[serde(rename = "page")]
    pub page_num: u64,
    #[serde(rename = "pageSize")]
    pub page_size: u64,
    #[serde(rename = "flowName")]
    pub flow_name: Option<String>,
    #[serde(rename = "businessType")]
    pub business_type: Option<String>,
}

pub async fn save_flow(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<FlowSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let operator = jwt_token.username.unwrap_or_default();
    match ApprovalService::save_flow(db, &payload.0, &operator).await {
        Ok(id) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn flow_detail(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match ApprovalService::find_flow_by_id(db, id.into_inner()).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn flow_list(
    state: web::Data<AppState>,
    query: web::Query<PageQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let q = FlowListQuery {
        page_num: query.page_num,
        page_size: query.page_size,
        flow_name: query.flow_name.clone(),
        business_type: query.business_type.clone(),
    };
    match ApprovalService::find_flow_list(db, &q).await {
        Ok(data) => {
            let page = data.current_page as u32;
            let total = data.total as u32;
            Ok(HttpResponse::Ok()
                .content_type("application/msgpack")
                .body(MetaResp::success_with_page(data, "local", page, total)))
        }
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn toggle_flow(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match ApprovalService::toggle_flow(db, id.into_inner()).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn delete_flow(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match ApprovalService::delete_flow(db, id.into_inner()).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn submit_approval(
    state: web::Data<AppState>,
    payload: web::Json<ApprovalSubmitRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match ApprovalService::submit(db, &payload.0).await {
        Ok(id) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn process_approval(
    state: web::Data<AppState>,
    payload: web::Json<ApprovalProcessRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match ApprovalService::process(db, &payload.0).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn approval_detail(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match ApprovalService::find_instance_by_id(db, id.into_inner()).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn approval_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<PageQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let approver_id = jwt_token.id.unwrap_or_default();
    match ApprovalService::find_instance_list(db, approver_id, query.page_num, query.page_size).await {
        Ok(data) => {
            let page = data.current_page as u32;
            let total = data.total as u32;
            Ok(HttpResponse::Ok()
                .content_type("application/msgpack")
                .body(MetaResp::success_with_page(data, "local", page, total)))
        }
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册审批模块所有路由
///
/// 修改路径、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(approval_controller::register)` 注册。
/// 注意：本 controller 原 handler 没有 `#[protect]` 宏，因此路由不挂权限中间件，
/// 仅依赖外层 `GrantsMiddleware::with_extractor(extract)` 做登录鉴权。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/approval")
            // POST /approval/flow/save - 保存审批流
            .route(
                "/flow/save",
                web::post().to(save_flow),
            )
            // GET /approval/flow/detail/{id} - 审批流详情
            .route(
                "/flow/detail/{id}",
                web::get().to(flow_detail),
            )
            // GET /approval/flow/list - 审批流列表
            .route(
                "/flow/list",
                web::get().to(flow_list),
            )
            // POST /approval/flow/toggle/{id} - 启用/停用审批流
            .route(
                "/flow/toggle/{id}",
                web::post().to(toggle_flow),
            )
            // POST /approval/flow/delete/{id} - 删除审批流
            .route(
                "/flow/delete/{id}",
                web::post().to(delete_flow),
            )
            // POST /approval/submit - 提交审批
            .route(
                "/submit",
                web::post().to(submit_approval),
            )
            // POST /approval/process - 处理审批
            .route(
                "/process",
                web::post().to(process_approval),
            )
            // GET /approval/detail/{id} - 审批实例详情
            .route(
                "/detail/{id}",
                web::get().to(approval_detail),
            )
            // GET /approval/list - 审批实例列表
            .route(
                "/list",
                web::get().to(approval_list),
            ),
    );
}
