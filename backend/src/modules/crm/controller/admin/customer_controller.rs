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
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::crm::model::customer::{CustomerDetailVO, CustomerListQuery, CustomerListVO, CustomerSaveRequest, CustomerUpdateRequest};
use crate::modules::crm::service::customer_service;

#[post("/customer/save")]
#[protect("crm:customer:save")]
pub async fn customer_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<CustomerSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.company_name.as_ref().map_or(true, |name| name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "鍏徃鍚嶇О涓嶈兘涓虹┖", "local")));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = customer_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/customer/update")]
#[protect("crm:customer:update")]
pub async fn customer_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<CustomerUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "瀹㈡埛ID涓嶈兘涓虹┖", "local")));
    }

    if form_data.company_name.as_ref().map_or(true, |name| name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "鍏徃鍚嶇О涓嶈兘涓虹┖", "local")));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = customer_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/customer/bath_delete")]
#[protect("crm:customer:delete")]
pub async fn bath_delete_customer(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "鏈幏鍙栧埌鍒犻櫎鐨勫鎴稩D", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = customer_service::batch_delete_by_ids(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

#[get("/customer/info")]
#[protect("crm:customer:info")]
pub async fn customer_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "瀹㈡埛ID涓嶈兘涓虹┖", "local"));
    }

    match customer_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[get("/customer/list")]
#[protect("crm:customer:list")]
pub async fn customer_list(state: web::Data<AppState>, query: web::Query<CustomerListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    match customer_service::list(&db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}