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
use crate::modules::sale::model::order::{OrderDetailVO, OrderListQuery, OrderListVO, OrderSaveRequest, OrderUpdateRequest};
use crate::modules::sale::service::order_service;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

#[post("/sale/order/save")]
#[protect("sale:order:save")]
pub async fn order_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<OrderSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;
    
    let result = order_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/sale/order/update")]
#[protect("sale:order:update")]
pub async fn order_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<OrderUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;
    
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "璁㈠崟ID涓嶈兘涓虹┖", "local")));
    }
    
    let result = order_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/sale/order/batchDelete")]
#[protect("sale:order:delete")]
pub async fn batch_delete_order(state: web::Data<AppState>, ids: web::Json<Vec<i64>>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = order_service::batch_delete(&db, &ids.0).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[get("/sale/order/info")]
#[protect("sale:order:view")]
pub async fn order_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID鏃犳晥", "local")));
    }
    
    match order_service::get_detail(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/sale/order/list")]
#[protect("sale:order:view")]
pub async fn order_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();
    
    let query = OrderListQuery {
        page_num: query_str.split("&").find(|s| s.starts_with("page=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        page_size: query_str.split("&").find(|s| s.starts_with("pageSize=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        keywords: query_str.split("&").find(|s| s.starts_with("keywords=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
        status: query_str.split("&").find(|s| s.starts_with("status=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
        customer_id: query_str.split("&").find(|s| s.starts_with("customerId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        order_type: query_str.split("&").find(|s| s.starts_with("orderType=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
    };
    
    match order_service::get_list(&db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}