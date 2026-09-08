//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::model::website_user::{WebsiteUserListQuery, WebsiteUserSaveDTO};
use crate::modules::website::service::website_user_service;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use crate::core::web::entity::common::BathDeleteIdRequest;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

/// GET /website_user/list - 前台用户列表（分页）
pub async fn get_by_page(
    state: web::Data<AppState>,
    query: web::Query<WebsiteUserListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match website_user_service::admin_get_by_page(db, query.into_inner()).await {
        Ok(page) => {
            let current_page = page.current_page as u32;
            let total = page.total as u32;
            Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::success_with_page(page, "local", current_page, total)))
        }
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /website_user/detail/{id} - 前台用户详情
pub async fn get_by_detail(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match website_user_service::admin_get_by_id(db, id.into_inner()).await {
        Ok(vo) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(vo, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// POST /website_user/create - 新增前台用户
pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<WebsiteUserSaveDTO>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = website_user_service::admin_create(db, body.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// PUT /website_user/update/{id} - 编辑前台用户
pub async fn update(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<WebsiteUserSaveDTO>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = website_user_service::admin_update(db, id.into_inner(), body.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ResetPasswordRequest {
    pub new_password: String,
}

/// PUT /website_user/reset_password/{id} - 重置密码
pub async fn reset_password(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<ResetPasswordRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = website_user_service::admin_reset_password(db, id.into_inner(), body.new_password.clone()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct UpdateStatusRequest {
    pub status: i32,
}

/// PUT /website_user/status/{id} - 启用/停用
pub async fn update_status(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<UpdateStatusRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = id.into_inner();
    let result = website_user_service::admin_update_status(db, user_id, body.status).await;
    // v1.1: 禁用（status=0）即时生效——写禁用标记，前台认证拦截；启用则清除标记
    if result.is_ok() {
        let key = format!("user_disabled:{}", user_id);
        if body.status == 0 {
            let _ = crate::core::kit::CONTEXT.cache_service.set_string(&key, "1").await;
        } else {
            let _ = crate::core::kit::CONTEXT.cache_service.del(&key).await;
        }
    }
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// DELETE /website_user/batch_delete - 批量删除
pub async fn batch_delete(
    state: web::Data<AppState>,
    item: web::Json<BathDeleteIdRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let result = website_user_service::admin_batch_delete(db, ids).await;
        Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册前台用户管理模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/website_user")
            .route("/list", web::get().to(get_by_page).wrap(require_permission("website:user:list")))
            .route("/detail/{id}", web::get().to(get_by_detail).wrap(require_permission("website:user:view")))
            .route("/create", web::post().to(create).wrap(require_permission("website:user:create")))
            .route("/update/{id}", web::put().to(update).wrap(require_permission("website:user:update")))
            .route("/reset_password/{id}", web::put().to(reset_password).wrap(require_permission("website:user:reset")))
            .route("/status/{id}", web::put().to(update_status).wrap(require_permission("website:user:status")))
            .route("/batch_delete", web::delete().to(batch_delete).wrap(require_permission("website:user:delete"))),
    );
}
