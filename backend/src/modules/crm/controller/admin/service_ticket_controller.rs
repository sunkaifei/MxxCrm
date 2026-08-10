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
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::crm::service::service_ticket_service::{self, TicketListQuery};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TicketCreateRequest {
    pub customer_id: i64,
    pub title: String,
    pub desc: Option<String>,
    pub priority: Option<i32>,
    pub ticket_type: Option<i32>,
    pub channel: Option<i32>,
    pub entitlement_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct TicketAssignRequest {
    pub id: i64,
    pub assigned_to: i64,
    pub dept_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct TicketRespondRequest {
    pub id: i64,
    pub content: String,
}

#[derive(Deserialize)]
pub struct TicketResolveRequest {
    pub id: i64,
    pub resolution: String,
}

#[derive(Deserialize)]
pub struct TicketCloseRequest {
    pub id: i64,
    pub satisfaction: Option<i32>,
    pub remark: Option<String>,
}

#[derive(Deserialize)]
pub struct TicketIdQuery {
    pub id: i64,
}

#[derive(Deserialize)]
pub struct TicketCustomerQuery {
    pub customer_id: i64,
}

pub async fn create(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<TicketCreateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match service_ticket_service::create_ticket(
        db,
        form_data.customer_id,
        form_data.title,
        form_data.desc,
        form_data.priority,
        form_data.ticket_type,
        form_data.channel,
        form_data.entitlement_id,
        jwt_token.id.unwrap_or_default(),
    ).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn assign(state: web::Data<AppState>, form_data: web::Json<TicketAssignRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "工单ID不能为空", "local")));
    }
    match service_ticket_service::assign_ticket(db, form_data.id, form_data.assigned_to, form_data.dept_id).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn respond(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<TicketRespondRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "工单ID不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match service_ticket_service::respond_ticket(db, form_data.id, form_data.content, jwt_token.id.unwrap_or_default()).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn resolve(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<TicketResolveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "工单ID不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match service_ticket_service::resolve_ticket(db, form_data.id, form_data.resolution, jwt_token.id.unwrap_or_default()).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn close(state: web::Data<AppState>, form_data: web::Json<TicketCloseRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "工单ID不能为空", "local")));
    }
    match service_ticket_service::close_ticket(db, form_data.id, form_data.satisfaction, form_data.remark).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn info(state: web::Data<AppState>, query: web::Query<TicketIdQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    if query.id == 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "工单ID不能为空", "local"));
    }
    match service_ticket_service::get_info(db, query.id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn list(state: web::Data<AppState>, query: web::Query<TicketListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    match service_ticket_service::get_list(db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn by_customer(state: web::Data<AppState>, query: web::Query<TicketCustomerQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    if query.customer_id == 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "客户ID不能为空", "local"));
    }
    match service_ticket_service::get_tickets_by_customer(db, query.customer_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/crm/service-ticket")
            .route(
                "/create",
                web::post().to(create).wrap(require_permission("crm:customer:save")),
            )
            .route(
                "/assign",
                web::post().to(assign).wrap(require_permission("crm:customer:save")),
            )
            .route(
                "/respond",
                web::post().to(respond).wrap(require_permission("crm:customer:save")),
            )
            .route(
                "/resolve",
                web::post().to(resolve).wrap(require_permission("crm:customer:save")),
            )
            .route(
                "/close",
                web::post().to(close).wrap(require_permission("crm:customer:save")),
            )
            .route(
                "/info",
                web::get().to(info).wrap(require_permission("crm:customer:list")),
            )
            .route(
                "/list",
                web::get().to(list).wrap(require_permission("crm:customer:list")),
            )
            .route(
                "/by-customer",
                web::get().to(by_customer).wrap(require_permission("crm:customer:list")),
            ),
    );
}
