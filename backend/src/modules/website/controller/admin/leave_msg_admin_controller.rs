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
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::website::model::leave_msg::{ConvertLeadRequest, LeaveMsgListQuery};
use crate::modules::website::service::leave_msg_service;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

/// GET /message/list - 留言列表（分页）
pub async fn get_by_page(
    state: web::Data<AppState>,
    _req: HttpRequest,
    query: web::Query<LeaveMsgListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = query.0;
    let page = form_data.page.unwrap_or(1);
    let page_size = form_data.page_size.unwrap_or(10);
    leave_msg_service::get_by_page(db, page, page_size, form_data.website_id, form_data.status)
        .await
        .map(|page_data| HttpResponse::Ok().json(page_data))
}

/// GET /message/detail/{id} - 留言详情
pub async fn get_by_detail(
    state: web::Data<AppState>,
    _req: HttpRequest,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = leave_msg_service::find_by_id(db, id.into_inner()).await?;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// POST /message/convert_lead/{id} - 手动转线索
pub async fn convert_lead(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
    body: web::Json<ConvertLeadRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();
    if user_id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(401, "未登录", "local")));
    }

    let leave_msg_id = id.into_inner();
    let assigned_to = body.assigned_to;
    let result = leave_msg_service::convert_to_lead(db, leave_msg_id, assigned_to, user_id).await;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<i64>::handle_result(result)))
}

/// PUT /message/status/{id} - 更新留言状态（标记为已处理/已忽略）
pub async fn update_status(
    state: web::Data<AppState>,
    _req: HttpRequest,
    id: web::Path<i64>,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let status = body
        .get("status")
        .and_then(|v| v.as_i64())
        .unwrap_or(2) as i32;
    let result = leave_msg_service::update_status(db, id.into_inner(), status).await;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<i64>::handle_result(result)))
}

/// DELETE /message/batch_delete - 批量删除留言
pub async fn batch_delete(
    state: web::Data<AppState>,
    item: web::Json<BathDeleteIdRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok()
                .content_type("application/msgpack")
                .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let result = leave_msg_service::batch_delete(db, ids).await;
        Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册留言管理模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/message")
            // GET /message/list - 留言列表
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("website:message:list")),
            )
            // GET /message/detail/{id} - 留言详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("website:message:view")),
            )
            // POST /message/convert_lead/{id} - 手动转线索
            .route(
                "/convert_lead/{id}",
                web::post()
                    .to(convert_lead)
                    .wrap(require_permission("website:message:convert")),
            )
            // PUT /message/status/{id} - 更新留言状态
            .route(
                "/status/{id}",
                web::put()
                    .to(update_status)
                    .wrap(require_permission("website:message:update")),
            )
            // DELETE /message/batch_delete - 批量删除
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("website:message:delete")),
            ),
    );
}
