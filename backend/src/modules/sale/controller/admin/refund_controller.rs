//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 销售退货单控制器
//!

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::{get_current_user, get_current_user_id};
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::model::refund::{
    RefundApprovalReq, RefundListQuery, RefundPaymentRequest, RefundQualityCheckReq,
    RefundReceiveReq, RefundSaveRequest, RefundUpdateRequest,
};
use crate::modules::sale::service::refund_service;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn refund_insert(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<RefundSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let user_id = get_current_user_id(&req);
    let result = refund_service::insert(db, &form_data, user_id).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

pub async fn refund_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<RefundUpdateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "退货单ID不能为空", "local")));
    }
    let user_id = get_current_user_id(&req);
    let result = refund_service::update(db, &form_data, user_id).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

pub async fn batch_delete_refund(
    state: web::Data<AppState>,
    form_data: web::Json<BathDeleteIdRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = form_data.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids: Vec<i64> = ids_vec
            .into_iter()
            .filter_map(|id| id.and_then(|s| s.parse().ok()))
            .collect();
        let result = refund_service::batch_delete(db, &ids).await;
        Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn refund_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "退货单ID不能为空", "local"));
    }
    match refund_service::get_detail(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn refund_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<RefundListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let current_user_id = get_current_user_id(&req);
    match refund_service::get_list(db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ========== 退货流程操作 ==========

/// 提交审批
pub async fn refund_submit(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<InfoId>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "退货单ID不能为空", "local")));
    }
    let (operator_id, operator_name) = get_current_user(&req);
    match refund_service::submit_refund(
        db,
        item.id.unwrap(),
        operator_id,
        &operator_name,
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 审批通过
pub async fn refund_approve(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<RefundApprovalReq>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    match refund_service::approve_refund(
        db,
        form_data.refund_id,
        get_current_user_id(&req),
        form_data.reason,
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 驳回
pub async fn refund_reject(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<RefundApprovalReq>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    match refund_service::reject_refund(
        db,
        form_data.refund_id,
        get_current_user_id(&req),
        form_data.reason,
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 仓库收货
pub async fn refund_receive(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<RefundReceiveReq>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    match refund_service::receive_refund(db, &form_data, get_current_user_id(&req)).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 质检完成
pub async fn refund_quality_check(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<RefundQualityCheckReq>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    match refund_service::quality_check(db, &form_data, get_current_user_id(&req)).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 取消退货单
pub async fn refund_cancel(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<InfoId>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "退货单ID不能为空", "local")));
    }
    match refund_service::cancel_refund(db, item.id.unwrap(), get_current_user_id(&req)).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 发起退款
pub async fn refund_payment(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<RefundPaymentRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    match refund_service::create_payment(db, &form_data, get_current_user_id(&req)).await {
        Ok(payment_id) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(payment_id, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/refund")
            .route(
                "/save",
                web::post()
                    .to(refund_insert)
                    .wrap(require_permission("sale:refund:save")),
            )
            .route(
                "/update",
                web::put()
                    .to(refund_update)
                    .wrap(require_permission("sale:refund:update")),
            )
            .route(
                "/batch-delete",
                web::post()
                    .to(batch_delete_refund)
                    .wrap(require_permission("sale:refund:delete")),
            )
            .route(
                "/info",
                web::get()
                    .to(refund_info)
                    .wrap(require_permission("sale:refund:list")),
            )
            .route(
                "/list",
                web::get()
                    .to(refund_list)
                    .wrap(require_permission("sale:refund:list")),
            )
            .route(
                "/submit",
                web::post()
                    .to(refund_submit)
                    .wrap(require_permission("sale:refund:update")),
            )
            .route(
                "/approve",
                web::post()
                    .to(refund_approve)
                    .wrap(require_permission("sale:refund:audit")),
            )
            .route(
                "/reject",
                web::post()
                    .to(refund_reject)
                    .wrap(require_permission("sale:refund:audit")),
            )
            .route(
                "/receive",
                web::post()
                    .to(refund_receive)
                    .wrap(require_permission("sale:refund:update")),
            )
            .route(
                "/quality-check",
                web::post()
                    .to(refund_quality_check)
                    .wrap(require_permission("sale:refund:update")),
            )
            .route(
                "/cancel",
                web::post()
                    .to(refund_cancel)
                    .wrap(require_permission("sale:refund:update")),
            )
            .route(
                "/payment",
                web::post()
                    .to(refund_payment)
                    .wrap(require_permission("sale:refund:payment")),
            ),
    );
}
