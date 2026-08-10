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
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::inventory::service::serial_number_service::{self, SerialByProductQuery};
use actix_web::{web, HttpResponse};
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SerialImportRequest {
    pub product_id: i64,
    pub warehouse_id: i64,
    pub serials: Vec<SerialImportItem>,
}

#[derive(Deserialize)]
pub struct SerialImportItem {
    pub serial_no: String,
    pub production_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
}

#[derive(Deserialize)]
pub struct SerialBindRequest {
    pub serial_id: i64,
    pub order_item_id: i64,
}

#[derive(Deserialize)]
pub struct SerialUnbindRequest {
    pub serial_id: i64,
}

#[derive(Deserialize)]
pub struct SerialNoQuery {
    pub serial_no: String,
}

#[derive(Deserialize)]
pub struct ExpiryAlertQuery {
    pub days: Option<i32>,
}

pub async fn import_serials(state: web::Data<AppState>, form_data: web::Json<SerialImportRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let serials: Vec<(String, Option<NaiveDate>, Option<NaiveDate>)> = form_data.serials
        .into_iter()
        .map(|item| (item.serial_no, item.production_date, item.expiry_date))
        .collect();
    match serial_number_service::import_serials(db, form_data.product_id, form_data.warehouse_id, serials).await {
        Ok(count) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(count, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn bind(state: web::Data<AppState>, form_data: web::Json<SerialBindRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.serial_id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "序列号ID不能为空", "local")));
    }
    match serial_number_service::bind_serial_to_order(db, form_data.serial_id, form_data.order_item_id).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn unbind(state: web::Data<AppState>, form_data: web::Json<SerialUnbindRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.serial_id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "序列号ID不能为空", "local")));
    }
    match serial_number_service::unbind_serial(db, form_data.serial_id).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn by_product(state: web::Data<AppState>, query: web::Query<SerialByProductQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    if query.product_id == 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "产品ID不能为空", "local"));
    }
    match serial_number_service::get_serials_by_product(db, query.product_id, query.status).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn expiry_alerts(state: web::Data<AppState>, query: web::Query<ExpiryAlertQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let days = query.days.unwrap_or(30);
    match serial_number_service::check_expiry_alerts(db, days).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn info(state: web::Data<AppState>, query: web::Query<SerialNoQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    if query.serial_no.is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "序列号不能为空", "local"));
    }
    match serial_number_service::get_serial_info(db, query.serial_no).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/inventory/serial-number")
            .route(
                "/import",
                web::post().to(import_serials).wrap(require_permission("inventory:stock:save")),
            )
            .route(
                "/bind",
                web::post().to(bind).wrap(require_permission("inventory:outbound:save")),
            )
            .route(
                "/unbind",
                web::post().to(unbind).wrap(require_permission("inventory:inbound:save")),
            )
            .route(
                "/by-product",
                web::get().to(by_product).wrap(require_permission("inventory:stock:list")),
            )
            .route(
                "/expiry-alerts",
                web::get().to(expiry_alerts).wrap(require_permission("inventory:stock:list")),
            )
            .route(
                "/info",
                web::get().to(info).wrap(require_permission("inventory:stock:list")),
            ),
    );
}
