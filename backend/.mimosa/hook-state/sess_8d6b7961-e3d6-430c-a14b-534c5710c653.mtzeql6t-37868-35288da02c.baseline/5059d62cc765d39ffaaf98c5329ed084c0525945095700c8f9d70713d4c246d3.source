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
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::service::tax_invoice_service::{self, TaxInvoiceListQuery};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TaxInvoiceCreateRequest {
    pub invoice_id: i64,
    pub platform: Option<i32>,
    pub category: Option<i32>,
}

#[derive(Deserialize)]
pub struct TaxInvoiceIssueRequest {
    pub id: i64,
}

#[derive(Deserialize)]
pub struct TaxInvoiceVoidRequest {
    pub id: i64,
    pub reason: String,
}

pub async fn create(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<TaxInvoiceCreateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let result = tax_invoice_service::create_tax_invoice(db, form_data.invoice_id, form_data.platform, form_data.category, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn issue(state: web::Data<AppState>, form_data: web::Json<TaxInvoiceIssueRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = tax_invoice_service::issue_tax_invoice(db, form_data.id).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn void(state: web::Data<AppState>, form_data: web::Json<TaxInvoiceVoidRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = tax_invoice_service::void_tax_invoice(db, form_data.id, form_data.reason).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn info(state: web::Data<AppState>, query: web::Query<TaxInvoiceIssueRequest>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    if query.id == 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local"));
    }
    match tax_invoice_service::get_info(db, query.id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn list(state: web::Data<AppState>, query: web::Query<TaxInvoiceListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    match tax_invoice_service::get_list(db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/tax-invoice")
            .route(
                "/create",
                web::post().to(create).wrap(require_permission("sale:invoice:save")),
            )
            .route(
                "/issue",
                web::post().to(issue).wrap(require_permission("sale:invoice:update")),
            )
            .route(
                "/void",
                web::post().to(void).wrap(require_permission("sale:invoice:update")),
            )
            .route(
                "/info",
                web::get().to(info).wrap(require_permission("sale:invoice:list")),
            )
            .route(
                "/list",
                web::get().to(list).wrap(require_permission("sale:invoice:list")),
            ),
    );
}
