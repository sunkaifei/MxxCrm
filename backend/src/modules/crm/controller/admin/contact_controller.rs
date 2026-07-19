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
use crate::core::web::permission_guard::require_permission;
use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::crm::model::contact::{ContactListQuery, ContactSaveRequest, ContactUpdateRequest, ContactBindRequest, ContactUnbindRequest, ContactSetRoleRequest, ContactCheckRequest};
use crate::modules::crm::service::contact_service;

pub async fn contact_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ContactSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = contact_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

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

pub async fn contact_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<ContactListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    match contact_service::list(&db, &query, jwt_token.id.unwrap_or_default()).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 联系人查重：检查手机、电话、微信、QQ、邮箱是否已存在
pub async fn contact_check(state: web::Data<AppState>, form_data: web::Json<ContactCheckRequest>) -> HttpResponse {
    let db = &state.db;
    match contact_service::check_duplicate(&db, &form_data.0).await {
        Ok(results) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(results, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 关联操作接口 ====================

/// 绑定联系人到客户（入职）
pub async fn contact_bind(state: web::Data<AppState>, form_data: web::Json<ContactBindRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = contact_service::bind_contact(&db, &form_data.0).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

/// 解绑联系人（离职）
pub async fn contact_unbind(state: web::Data<AppState>, form_data: web::Json<ContactUnbindRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = contact_service::unbind_contact(&db, &form_data.0).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

/// 设置联系人角色/标记
pub async fn contact_set_role(state: web::Data<AppState>, form_data: web::Json<ContactSetRoleRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = contact_service::set_role(&db, &form_data.0).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

// ==================== 路由注册（单点维护）====================

/// 注册联系人模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(contact_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/contact")
            // POST /contact/save - 新建联系人
            .route(
                "/save",
                web::post()
                    .to(contact_insert)
                    .wrap(require_permission("crm:contact:create")),
            )
            // PUT /contact/update - 修改联系人
            .route(
                "/update",
                web::put()
                    .to(contact_update)
                    .wrap(require_permission("crm:contact:update")),
            )
            // DELETE /contact/bath_delete - 批量删除联系人
            .route(
                "/bath_delete",
                web::delete()
                    .to(bath_delete_contact)
                    .wrap(require_permission("crm:contact:delete")),
            )
            // GET /contact/info - 联系人详情
            .route(
                "/info",
                web::get()
                    .to(contact_info)
                    .wrap(require_permission("crm:contact:info")),
            )
            // GET /contact/list - 联系人列表
            .route(
                "/list",
                web::get()
                    .to(contact_list)
                    .wrap(require_permission("crm:contact:list")),
            )
            // POST /contact/check - 联系人查重
            .route(
                "/check",
                web::post()
                    .to(contact_check)
                    .wrap(require_permission("crm:contact:list")),
            )
            // POST /contact/bind - 绑定联系人到客户
            .route(
                "/bind",
                web::post()
                    .to(contact_bind)
                    .wrap(require_permission("crm:contact:bind")),
            )
            // POST /contact/unbind - 解绑联系人
            .route(
                "/unbind",
                web::post()
                    .to(contact_unbind)
                    .wrap(require_permission("crm:contact:unbind")),
            )
            // PUT /contact/set_role - 设置联系人角色
            .route(
                "/set_role",
                web::put()
                    .to(contact_set_role)
                    .wrap(require_permission("crm:contact:set_role")),
            ),
    );
}