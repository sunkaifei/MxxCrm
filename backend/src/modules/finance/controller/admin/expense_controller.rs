//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 费用申请控制器
//!

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::finance::model::expense::{
    ExpenseApprovalReq, ExpenseListQuery, ExpensePaymentReq, ExpenseSaveRequest,
    ExpenseTypeSaveRequest,
};
use crate::modules::finance::service::expense_service;
use actix_web::{web, HttpRequest, HttpResponse};

/// 新建/编辑费用申请
pub async fn expense_save(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<ExpenseSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();

    let result = if form_data.id.unwrap_or_default() > 0 {
        expense_service::update(db, &form_data, user_id).await
    } else {
        expense_service::insert(db, &form_data, user_id).await
    };

    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<i64>::handle_result(result)))
}

/// 费用申请列表
pub async fn expense_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ExpenseListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    match expense_service::get_list(db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok()
                .content_type("application/msgpack")
                .body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 费用申请详情
pub async fn expense_info(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> HttpResponse {
    let db = &state.db;
    let id = path.into_inner();
    if id <= 0 {
        return HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "费用申请ID不能为空", "local"));
    }
    match expense_service::get_detail(db, id).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 提交审批
pub async fn expense_submit(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<ExpenseApprovalReq>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match expense_service::submit_expense(
        db,
        form_data.expense_id,
        jwt_token.id.unwrap_or_default(),
        &jwt_token.username.unwrap_or_default(),
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 审批通过
pub async fn expense_approve(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<ExpenseApprovalReq>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match expense_service::approve_expense(
        db,
        form_data.expense_id,
        jwt_token.id.unwrap_or_default(),
        form_data.reason,
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 驳回
pub async fn expense_reject(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<ExpenseApprovalReq>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match expense_service::reject_expense(
        db,
        form_data.expense_id,
        jwt_token.id.unwrap_or_default(),
        form_data.reason,
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 财务打款
pub async fn expense_payment(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<ExpensePaymentReq>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match expense_service::make_payment(db, &form_data, jwt_token.id.unwrap_or_default()).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 批量删除
pub async fn expense_batch_delete(
    state: web::Data<AppState>,
    form_data: web::Json<BathDeleteIdRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = form_data.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok()
                .content_type("application/msgpack")
                .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids: Vec<i64> = ids_vec
            .into_iter()
            .filter_map(|id| id.and_then(|s| s.parse().ok()))
            .collect();
        let result = expense_service::batch_delete(db, &ids).await;
        Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// 费用类型列表
pub async fn expense_type_list(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match expense_service::get_type_list(db).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 费用类型新建/编辑
pub async fn expense_type_save(
    state: web::Data<AppState>,
    form_data: web::Json<ExpenseTypeSaveRequest>,
) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;
    match expense_service::save_type(db, &form_data).await {
        Ok(id) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 费用类型批量删除
pub async fn expense_type_batch_delete(
    state: web::Data<AppState>,
    form_data: web::Json<BathDeleteIdRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = form_data.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok()
                .content_type("application/msgpack")
                .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids: Vec<i64> = ids_vec
            .into_iter()
            .filter_map(|id| id.and_then(|s| s.parse().ok()))
            .collect();
        let result = expense_service::batch_delete_type(db, &ids).await;
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

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance/expense")
            .route(
                "/save",
                web::post()
                    .to(expense_save)
                    .wrap(require_permission("finance:expense:add")),
            )
            .route(
                "/list",
                web::get()
                    .to(expense_list)
                    .wrap(require_permission("finance:expense:list")),
            )
            .route(
                "/info/{id}",
                web::get()
                    .to(expense_info)
                    .wrap(require_permission("finance:expense:list")),
            )
            .route(
                "/submit",
                web::post()
                    .to(expense_submit)
                    .wrap(require_permission("finance:expense:update")),
            )
            .route(
                "/approve",
                web::post()
                    .to(expense_approve)
                    .wrap(require_permission("finance:expense:approve")),
            )
            .route(
                "/reject",
                web::post()
                    .to(expense_reject)
                    .wrap(require_permission("finance:expense:approve")),
            )
            .route(
                "/payment",
                web::post()
                    .to(expense_payment)
                    .wrap(require_permission("finance:expense:payment")),
            )
            .route(
                "/batch-delete",
                web::post()
                    .to(expense_batch_delete)
                    .wrap(require_permission("finance:expense:delete")),
            )
            .route(
                "/type/list",
                web::get()
                    .to(expense_type_list)
                    .wrap(require_permission("finance:expense:type:list")),
            )
            .route(
                "/type/save",
                web::post()
                    .to(expense_type_save)
                    .wrap(require_permission("finance:expense:type:list")),
            )
            .route(
                "/type/batch-delete",
                web::post()
                    .to(expense_type_batch_delete)
                    .wrap(require_permission("finance:expense:type:list")),
            ),
    );
}
