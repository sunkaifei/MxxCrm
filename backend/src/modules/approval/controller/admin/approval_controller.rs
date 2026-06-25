use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::response::MetaResp;
use crate::modules::approval::model::approval::{
    ApprovalProcessRequest, ApprovalSubmitRequest, FlowListQuery, FlowSaveRequest,
};
use crate::modules::approval::service::approval_service::ApprovalService;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
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

#[post("/approval/flow/save")]
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

#[get("/approval/flow/detail/{id}")]
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

#[get("/approval/flow/list")]
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

#[post("/approval/flow/toggle/{id}")]
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

#[post("/approval/submit")]
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

#[post("/approval/process")]
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

#[get("/approval/detail/{id}")]
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

#[get("/approval/list")]
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
