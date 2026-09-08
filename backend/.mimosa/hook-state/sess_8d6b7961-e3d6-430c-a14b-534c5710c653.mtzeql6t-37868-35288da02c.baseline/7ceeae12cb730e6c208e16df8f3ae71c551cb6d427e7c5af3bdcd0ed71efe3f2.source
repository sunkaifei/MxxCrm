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
use crate::modules::inventory::service::bin_location_service;
use actix_web::{web, HttpResponse};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BinCreateRequest {
    pub warehouse_id: i64,
    pub area_id: Option<i64>,
    pub bin_code: String,
    pub bin_name: String,
    pub bin_type: Option<i32>,
    pub row: Option<i32>,
    pub col: Option<i32>,
    pub layer: Option<i32>,
    pub capacity: Option<Decimal>,
}

#[derive(Deserialize)]
pub struct BinAssignRequest {
    pub stock_id: i64,
    pub bin_id: i64,
    pub quantity: Decimal,
}

#[derive(Deserialize)]
pub struct BinWarehouseQuery {
    pub warehouse_id: i64,
}

pub async fn create(state: web::Data<AppState>, form_data: web::Json<BinCreateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.bin_code.is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "库位编码不能为空", "local")));
    }
    match bin_location_service::create_bin(
        db,
        form_data.warehouse_id,
        form_data.area_id,
        form_data.bin_code,
        form_data.bin_name,
        form_data.bin_type,
        form_data.row,
        form_data.col,
        form_data.layer,
        form_data.capacity,
    ).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn list(state: web::Data<AppState>, query: web::Query<BinWarehouseQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    if query.warehouse_id == 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "仓库ID不能为空", "local"));
    }
    match bin_location_service::get_bins_by_warehouse(db, query.warehouse_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn assign(state: web::Data<AppState>, form_data: web::Json<BinAssignRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.stock_id == 0 || form_data.bin_id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "库存ID和库位ID不能为空", "local")));
    }
    match bin_location_service::assign_stock_to_bin(db, form_data.stock_id, form_data.bin_id, form_data.quantity).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn utilization(state: web::Data<AppState>, query: web::Query<BinWarehouseQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    if query.warehouse_id == 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "仓库ID不能为空", "local"));
    }
    match bin_location_service::get_bin_utilization(db, query.warehouse_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/inventory/bin-location")
            .route(
                "/create",
                web::post().to(create).wrap(require_permission("inventory:warehouse:save")),
            )
            .route(
                "/list",
                web::get().to(list).wrap(require_permission("inventory:warehouse:list")),
            )
            .route(
                "/assign",
                web::post().to(assign).wrap(require_permission("inventory:stock:save")),
            )
            .route(
                "/utilization",
                web::get().to(utilization).wrap(require_permission("inventory:warehouse:list")),
            ),
    );
}
