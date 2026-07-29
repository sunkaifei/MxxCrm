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
use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::sale::model::invoice::{InvoiceApprovalReq, InvoiceListQuery, InvoiceSaveRequest, InvoiceUpdateRequest};
use crate::modules::sale::service::invoice_service;

pub async fn invoice_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<InvoiceSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let result = invoice_service::insert(db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

pub async fn invoice_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<InvoiceUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "发票ID不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let result = invoice_service::update(db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

pub async fn bath_delete_invoice(state: web::Data<AppState>, form_data: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = form_data.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids: Vec<i64> = ids_vec.into_iter().filter_map(|id| id.and_then(|s| s.parse().ok())).collect();
        let result = invoice_service::batch_delete(db, &ids).await;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn invoice_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "发票ID不能为空", "local"));
    }
    match invoice_service::get_detail(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn invoice_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<InvoiceListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    match invoice_service::get_list(db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 发票审批 ===================

/// 提交审批
pub async fn invoice_submit(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let invoice_id = path.into_inner();
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match invoice_service::submit_invoice(db, invoice_id, jwt_token.id.unwrap_or_default(), &jwt_token.username.unwrap_or_default()).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 审批通过
pub async fn invoice_approve(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>, form_data: web::Json<InvoiceApprovalReq>) -> Result<HttpResponse> {
    let db = &state.db;
    let invoice_id = path.into_inner();
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match invoice_service::approve_invoice(db, invoice_id, jwt_token.id.unwrap_or_default(), &jwt_token.username.unwrap_or_default(), form_data.reason).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 驳回
pub async fn invoice_reject(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>, form_data: web::Json<InvoiceApprovalReq>) -> Result<HttpResponse> {
    let db = &state.db;
    let invoice_id = path.into_inner();
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match invoice_service::reject_invoice(db, invoice_id, jwt_token.id.unwrap_or_default(), &jwt_token.username.unwrap_or_default(), form_data.reason).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 审批详情
pub async fn invoice_approval_detail(state: web::Data<AppState>, path: web::Path<i64>) -> HttpResponse {
    let db = &state.db;
    let invoice_id = path.into_inner();
    match invoice_service::get_invoice_approval_detail(db, invoice_id).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册发票模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(invoice_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/invoice")
            // POST /sale/invoice/save - 新建发票
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/save",
                web::post()
                    .to(invoice_insert)
                    .wrap(require_permission("sale:invoice:save")),
            )
            // PUT /sale/invoice/update - 修改发票
            .route(
                "/update",
                web::put()
                    .to(invoice_update)
                    .wrap(require_permission("sale:invoice:update")),
            )
            // POST /sale/invoice/batch-delete - 批量删除发票
            .route(
                "/batch-delete",
                web::post()
                    .to(bath_delete_invoice)
                    .wrap(require_permission("sale:invoice:delete")),
            )
            // GET /sale/invoice/info - 发票详情
            .route(
                "/info",
                web::get()
                    .to(invoice_info)
                    .wrap(require_permission("sale:invoice:list")),
            )
            // GET /sale/invoice/list - 发票列表
            .route(
                "/list",
                web::get()
                    .to(invoice_list)
                    .wrap(require_permission("sale:invoice:list")),
            )
            // POST /sale/invoice/{id}/submit - 提交审批
            .route(
                "/{id}/submit",
                web::post()
                    .to(invoice_submit)
                    .wrap(require_permission("sale:invoice:update")),
            )
            // POST /sale/invoice/{id}/approve - 审批通过
            .route(
                "/{id}/approve",
                web::post()
                    .to(invoice_approve)
                    .wrap(require_permission("sale:invoice:update")),
            )
            // POST /sale/invoice/{id}/reject - 驳回
            .route(
                "/{id}/reject",
                web::post()
                    .to(invoice_reject)
                    .wrap(require_permission("sale:invoice:update")),
            )
            // GET /sale/invoice/{id}/approval-detail - 审批详情
            .route(
                "/{id}/approval-detail",
                web::get()
                    .to(invoice_approval_detail)
                    .wrap(require_permission("sale:invoice:list")),
            ),
    );
}
