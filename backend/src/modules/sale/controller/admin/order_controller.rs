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
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::sale::model::order::{OrderListQuery, OrderSaveRequest, OrderStatusUpdateRequest, OrderUpdateRequest};
use crate::modules::sale::service::order_service;
use actix_web::{get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

#[post("/sale/order/save")]
#[protect("sale:order:save")]
pub async fn order_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<OrderSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let result = order_service::insert(db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/sale/order/update")]
#[protect("sale:order:update")]
pub async fn order_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<OrderUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "订单ID不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let result = order_service::update(db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/sale/order/updateStatus")]
#[protect("sale:order:update")]
pub async fn order_update_status(state: web::Data<AppState>, form_data: web::Json<OrderStatusUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "订单ID不能为空", "local")));
    }
    if form_data.order_status.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "订单状态不能为空", "local")));
    }
    let result = order_service::update_status(db, &form_data).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[post("/sale/order/batch-delete")]
#[protect("sale:order:delete")]
pub async fn batch_delete_order(state: web::Data<AppState>, form_data: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = form_data.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids: Vec<i64> = ids_vec.into_iter().filter_map(|id| id.and_then(|s| s.parse().ok())).collect();
        let result = order_service::batch_delete(db, &ids).await;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

#[get("/sale/order/info")]
#[protect("sale:order:list")]
pub async fn order_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "订单ID不能为空", "local"));
    }
    match order_service::get_detail(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[get("/sale/order/list")]
#[protect("sale:order:list")]
pub async fn order_list(state: web::Data<AppState>, query: web::Query<OrderListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    match order_service::get_list(db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}
