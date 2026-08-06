use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::purchase::service::purchase_report_service;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

pub async fn report_summary(state: web::Data<AppState>, query: web::Query<ReportQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    
    match purchase_report_service::summary(db, q.start_date, q.end_date).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn report_by_supplier(state: web::Data<AppState>, query: web::Query<ReportQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    
    match purchase_report_service::by_supplier(db, q.start_date, q.end_date).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn report_by_product(state: web::Data<AppState>, query: web::Query<ReportQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    
    match purchase_report_service::by_product(db, q.start_date, q.end_date).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn report_by_department(state: web::Data<AppState>, query: web::Query<ReportQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    
    match purchase_report_service::by_department(db, q.start_date, q.end_date).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn report_by_brand(state: web::Data<AppState>, query: web::Query<ReportQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    
    match purchase_report_service::by_brand(db, q.start_date, q.end_date).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/purchase/report")
            .route("/summary", web::get().to(report_summary).wrap(require_permission("purchase:report:list")))
            .route("/by_supplier", web::get().to(report_by_supplier).wrap(require_permission("purchase:report:list")))
            .route("/by_product", web::get().to(report_by_product).wrap(require_permission("purchase:report:list")))
            .route("/by_department", web::get().to(report_by_department).wrap(require_permission("purchase:report:list")))
            .route("/by_brand", web::get().to(report_by_brand).wrap(require_permission("purchase:report:list"))),
    );
}