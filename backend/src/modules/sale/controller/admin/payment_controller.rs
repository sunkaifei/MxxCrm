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
use crate::core::web::base_controller::{get_current_user, get_current_user_id};
use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::model::payment::{
    PaymentApplyRequest, PaymentApprovalReq, PaymentListQuery, PaymentSaveRequest, PaymentUpdateRequest,
};
use crate::modules::sale::service::payment_service;

pub async fn payment_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PaymentSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let result = payment_service::insert(&db, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn payment_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PaymentUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "收款记录ID不能为空", "local")));
    }

    let result = payment_service::update(&db, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn bath_delete_payment(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到删除的收款记录ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = payment_service::batch_delete_by_ids(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

pub async fn payment_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "收款记录ID不能为空", "local"));
    }

    match payment_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn payment_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PaymentListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    let current_user_id = get_current_user_id(&req);

    match payment_service::list(&db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 确认回款：status→2，设 confirm_time/confirm_by，联动订单 paid_amount
pub async fn payment_confirm(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let payment_id = match form_data.0.id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "回款ID不能为空", "local")),
    };

    let user_id = get_current_user_id(&req);

    match payment_service::confirm(db, payment_id, user_id).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 驳回回款：status→3
pub async fn payment_reject(
    state: web::Data<AppState>,
    form_data: web::Json<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let payment_id = match form_data.0.id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "回款ID不能为空", "local")),
    };

    match payment_service::reject(db, payment_id).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 核销：将回款金额分配到一个或多个回款计划
pub async fn payment_apply(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<PaymentApplyRequest>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    let user_id = get_current_user_id(&req);

    match payment_service::apply(db, &dto, user_id).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 取消核销：回滚 payment 和 plan 金额，软删除核销记录
pub async fn payment_application_cancel(
    state: web::Data<AppState>,
    form_data: web::Json<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let application_id = match form_data.0.id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "核销记录ID不能为空", "local")),
    };

    match payment_service::cancel_apply(db, application_id).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 查询回款未核销金额及可核销计划列表
pub async fn payment_unapplied(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let payment_id = match query.0.id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "回款ID不能为空", "local")),
    };

    match payment_service::get_unapplied(db, payment_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 查询回款的核销明细列表
pub async fn payment_application_list(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let payment_id = match query.0.id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "回款ID不能为空", "local")),
    };

    match payment_service::get_applications(db, payment_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 回款审批 ====================

/// 提交审批
pub async fn payment_submit(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
) -> HttpResponse {
    let db = &state.db;
    let payment_id = path.into_inner();
    let (operator_id, operator_name) = get_current_user(&req);
    match payment_service::submit_payment(db, payment_id, operator_id, &operator_name).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 审批通过
pub async fn payment_approve(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
    form_data: web::Json<PaymentApprovalReq>,
) -> HttpResponse {
    let db = &state.db;
    let payment_id = path.into_inner();
    let (operator_id, operator_name) = get_current_user(&req);
    match payment_service::approve_payment(db, payment_id, operator_id, &operator_name, form_data.0.reason).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 驳回
pub async fn payment_approval_reject(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
    form_data: web::Json<PaymentApprovalReq>,
) -> HttpResponse {
    let db = &state.db;
    let payment_id = path.into_inner();
    let (operator_id, operator_name) = get_current_user(&req);
    match payment_service::reject_payment(db, payment_id, operator_id, &operator_name, form_data.0.reason).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 审批详情
pub async fn payment_approval_detail(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> HttpResponse {
    let db = &state.db;
    let payment_id = path.into_inner();
    match payment_service::get_payment_approval_detail(db, payment_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册回款模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(payment_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/payment")
            // POST /sale/payment/save - 新建回款
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/save",
                web::post()
                    .to(payment_insert)
                    .wrap(require_permission("sale:payment:save")),
            )
            // PUT /sale/payment/update - 修改回款
            .route(
                "/update",
                web::put()
                    .to(payment_update)
                    .wrap(require_permission("sale:payment:update")),
            )
            // DELETE /sale/payment/bath_delete - 批量删除回款
            .route(
                "/bath_delete",
                web::delete()
                    .to(bath_delete_payment)
                    .wrap(require_permission("sale:payment:delete")),
            )
            // GET /sale/payment/info - 回款详情
            .route(
                "/info",
                web::get()
                    .to(payment_info)
                    .wrap(require_permission("sale:payment:view")),
            )
            // GET /sale/payment/list - 回款列表
            .route(
                "/list",
                web::get()
                    .to(payment_list)
                    .wrap(require_permission("sale:payment:list")),
            )
            // POST /sale/payment/confirm - 确认回款
            .route(
                "/confirm",
                web::post()
                    .to(payment_confirm)
                    .wrap(require_permission("sale:payment:confirm")),
            )
            // POST /sale/payment/reject - 驳回回款
            .route(
                "/reject",
                web::post()
                    .to(payment_reject)
                    .wrap(require_permission("sale:payment:confirm")),
            )
            // POST /sale/payment/application/apply - 核销
            .route(
                "/application/apply",
                web::post()
                    .to(payment_apply)
                    .wrap(require_permission("sale:payment:confirm")),
            )
            // POST /sale/payment/application/cancel - 取消核销
            .route(
                "/application/cancel",
                web::post()
                    .to(payment_application_cancel)
                    .wrap(require_permission("sale:payment:confirm")),
            )
            // GET /sale/payment/unapplied - 查询未核销金额及可核销计划
            .route(
                "/unapplied",
                web::get()
                    .to(payment_unapplied)
                    .wrap(require_permission("sale:payment:list")),
            )
            // GET /sale/payment/application/list - 查询核销明细列表
            .route(
                "/application/list",
                web::get()
                    .to(payment_application_list)
                    .wrap(require_permission("sale:payment:list")),
            )
            // POST /sale/payment/{id}/submit - 提交审批
            .route(
                "/{id}/submit",
                web::post()
                    .to(payment_submit)
                    .wrap(require_permission("sale:payment:confirm")),
            )
            // POST /sale/payment/{id}/approve - 审批通过
            .route(
                "/{id}/approve",
                web::post()
                    .to(payment_approve)
                    .wrap(require_permission("sale:payment:confirm")),
            )
            // POST /sale/payment/{id}/reject - 驳回
            .route(
                "/{id}/reject",
                web::post()
                    .to(payment_approval_reject)
                    .wrap(require_permission("sale:payment:confirm")),
            )
            // GET /sale/payment/{id}/approval-detail - 审批详情
            .route(
                "/{id}/approval-detail",
                web::get()
                    .to(payment_approval_detail)
                    .wrap(require_permission("sale:payment:view")),
            ),
    );
}