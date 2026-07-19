use crate::core::errors::error::Result;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::permission_guard::require_permission;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::response::MetaResp;
use crate::modules::ai::service::background_check_service;
use crate::modules::system::service::admin_service;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundCheckRequest {
    pub company_name: String,
    pub lead_id: Option<i64>,
    pub company_id: Option<i64>,
}

pub async fn perform_background_check(state: web::Data<AppState>, req: HttpRequest, item: web::Json<BackgroundCheckRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;

    match background_check_service::perform_background_check(
        &db,
        &item.company_name,
        item.lead_id,
        item.company_id,
        jwt_token.id.unwrap_or_default(),
        &admin.user_name.clone().unwrap_or_default(),
    ).await {
        Ok(result) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_by_lead_id(state: web::Data<AppState>, lead_id: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    match background_check_service::get_by_lead_id(&db, lead_id.into_inner()).await {
        Ok(list) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(list, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_latest_by_lead_id(state: web::Data<AppState>, lead_id: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    match background_check_service::get_latest_by_lead_id(&db, lead_id.into_inner()).await {
        Ok(result) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_by_company_id(state: web::Data<AppState>, company_id: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    match background_check_service::get_by_company_id(&db, company_id.into_inner()).await {
        Ok(list) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(list, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_latest_by_company_id(state: web::Data<AppState>, company_id: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    match background_check_service::get_latest_by_company_id(&db, company_id.into_inner()).await {
        Ok(result) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_detail(state: web::Data<AppState>, id: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    match background_check_service::get_by_id(&db, id.into_inner()).await {
        Ok(Some(detail)) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(detail, "local"))),
        Ok(None) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "背调记录不存在", "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_timeline(state: web::Data<AppState>, query: web::Query<TimelineQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    match background_check_service::get_timeline_by_company_name(&db, &query.company_name).await {
        Ok(list) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(list, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineQuery {
    pub company_name: String,
}

pub async fn delete_by_id(state: web::Data<AppState>, id: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    match background_check_service::delete_by_id(&db, id.into_inner()).await {
        Ok(count) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(count, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/background-check")
            .route("/perform", web::post().to(perform_background_check).wrap(require_permission("crm:lead:list")))
            .route("/lead/{lead_id}", web::get().to(get_by_lead_id).wrap(require_permission("crm:lead:list")))
            .route("/latest/{lead_id}", web::get().to(get_latest_by_lead_id).wrap(require_permission("crm:lead:list")))
            .route("/company/{company_id}", web::get().to(get_by_company_id).wrap(require_permission("crm:lead:list")))
            .route("/company-latest/{company_id}", web::get().to(get_latest_by_company_id).wrap(require_permission("crm:lead:list")))
            .route("/detail/{id}", web::get().to(get_detail).wrap(require_permission("crm:lead:list")))
            .route("/timeline", web::get().to(get_timeline).wrap(require_permission("crm:lead:list")))
            .route("/delete/{id}", web::delete().to(delete_by_id).wrap(require_permission("crm:lead:edit"))),
    );
}
