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
use crate::modules::crm::model::contact::{ContactListQuery, ContactSaveRequest, ContactUpdateRequest, ContactBindRequest, ContactUnbindRequest, ContactSetRoleRequest};
use crate::modules::crm::service::contact_service;

#[post("/contact/save")]
#[protect("crm:contact:save")]
pub async fn contact_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ContactSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = contact_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/contact/update")]
#[protect("crm:contact:update")]
pub async fn contact_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ContactUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "联系人ID不能为空", "local")));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = contact_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/contact/bath_delete")]
#[protect("crm:contact:delete")]
pub async fn bath_delete_contact(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到删除的联系人ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = contact_service::batch_delete_by_ids(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

#[get("/contact/info")]
#[protect("crm:contact:info")]
pub async fn contact_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "联系人ID不能为空", "local"));
    }

    match contact_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[get("/contact/list")]
#[protect("crm:contact:list")]
pub async fn contact_list(state: web::Data<AppState>, query: web::Query<ContactListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    match contact_service::list(&db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 关联操作接口 ====================

/// 绑定联系人到客户（入职）
#[post("/contact/bind")]
#[protect("crm:contact:bind")]
pub async fn contact_bind(state: web::Data<AppState>, form_data: web::Json<ContactBindRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = contact_service::bind_contact(&db, &form_data.0).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

/// 解绑联系人（离职）
#[post("/contact/unbind")]
#[protect("crm:contact:unbind")]
pub async fn contact_unbind(state: web::Data<AppState>, form_data: web::Json<ContactUnbindRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = contact_service::unbind_contact(&db, &form_data.0).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

/// 设置联系人角色/标记
#[put("/contact/set_role")]
#[protect("crm:contact:set_role")]
pub async fn contact_set_role(state: web::Data<AppState>, form_data: web::Json<ContactSetRoleRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = contact_service::set_role(&db, &form_data.0).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}