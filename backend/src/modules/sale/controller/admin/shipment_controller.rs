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
use crate::core::web::entity::common::InfoId;
use crate::core::web::response::MetaResp;
use crate::modules::sale::model::shipment::{ShipmentListQuery, ShipmentSaveRequest, ShipmentUpdateRequest};
use crate::modules::sale::service::shipment_service;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

#[get("/sale/shipment/list")]
#[protect("sale:order:list")]
pub async fn shipment_list(state: web::Data<AppState>, query: web::Query<ShipmentListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    match shipment_service::get_list(db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[get("/sale/shipment/info")]
#[protect("sale:order:list")]
pub async fn shipment_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "发货单ID不能为空", "local"));
    }
    match shipment_service::get_detail(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[post("/sale/shipment/save")]
#[protect("sale:order:edit")]
pub async fn shipment_insert(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<ShipmentSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let result = shipment_service::create(db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/sale/shipment/update")]
#[protect("sale:order:edit")]
pub async fn shipment_update(
    state: web::Data<AppState>,
    form_data: web::Json<ShipmentUpdateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "发货单ID不能为空", "local")));
    }
    let result = shipment_service::update(db, &form_data).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/sale/shipment/delete")]
#[protect("sale:order:edit")]
pub async fn shipment_delete(state: web::Data<AppState>, item: web::Query<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "发货单ID不能为空", "local")));
    }
    let result = shipment_service::delete(db, item.id.unwrap()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[post("/sale/shipment/sign")]
#[protect("sale:order:edit")]
pub async fn shipment_sign(state: web::Data<AppState>, item: web::Query<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "发货单ID不能为空", "local")));
    }
    let result = shipment_service::sign(db, item.id.unwrap()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}
