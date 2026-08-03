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
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::model::website_refund::{RefundHandleRequest, RefundListQuery};
use crate::modules::website::service::website_refund_service;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MarkRefundedRequest {
    pub transaction_id: Option<String>,
}

/// GET /website_refund/list - 退款单列表
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<RefundListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match website_refund_service::admin_list(db, query.into_inner()).await {
        Ok(page) => Ok(HttpResponse::Ok().json(page)),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /website_refund/detail/{id} - 退款详情
pub async fn detail(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match website_refund_service::admin_detail(db, id.into_inner()).await {
        Ok(vo) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(vo, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// POST /website_refund/handle/{id} - 审核退款（通过/拒绝）
pub async fn handle(
    state: web::Data<AppState>,
    req: actix_web::HttpRequest,
    id: web::Path<i64>,
    body: web::Json<RefundHandleRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let handle_by = jwt_token.id.unwrap_or_default();
    let result = website_refund_service::admin_handle(db, id.into_inner(), body.into_inner(), handle_by).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// POST /website_refund/mark_refunded/{id} - 标记退款已完成（实际打款后调用）
pub async fn mark_refunded(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<MarkRefundedRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = website_refund_service::admin_mark_refunded(db, id.into_inner(), body.transaction_id.clone()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// DELETE /website_refund/batch_delete - 批量删除退款单
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
        let result = website_refund_service::admin_batch_delete(db, ids).await;
        Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

// ==================== 路由注册 ====================

/// 注册网站退款管理模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/website_refund")
            .route("/list", web::get().to(list).wrap(require_permission("website:refund:list")))
            .route("/detail/{id}", web::get().to(detail).wrap(require_permission("website:refund:view")))
            .route("/handle/{id}", web::post().to(handle).wrap(require_permission("website:refund:handle")))
            .route("/mark_refunded/{id}", web::post().to(mark_refunded).wrap(require_permission("website:refund:refund")))
            .route("/batch_delete", web::delete().to(batch_delete).wrap(require_permission("website:refund:delete"))),
    );
}
