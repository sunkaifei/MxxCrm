//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 调薪记录控制器
//!

use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::finance::service::salary_adjustment_service;

/// 列表查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub employee_id: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 历史/对比查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeQuery {
    pub employee_id: i64,
}

/// 审批参数
#[derive(Deserialize)]
pub struct ApproveQuery {
    pub id: i64,
}

/// 列表
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<ListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).max(1);

    match salary_adjustment_service::get_adjustment_list(db, q.employee_id, page, page_size).await {
        Ok((list, total)) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success_with_page(list, "local", page as u32, total as u32)),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 调薪历史
pub async fn history(
    state: web::Data<AppState>,
    query: web::Query<EmployeeQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;

    match salary_adjustment_service::get_employee_history(db, q.employee_id).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 创建调薪
pub async fn create(
    state: web::Data<AppState>,
    form_data: web::Json<salary_adjustment_service::SalaryAdjustmentCreateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match salary_adjustment_service::create_adjustment(db, dto).await {
        Ok(id) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 审批通过
pub async fn approve(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ApproveQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let approver_id = jwt_token.id.unwrap_or(0);
    let approver_name = jwt_token.username.as_deref().unwrap_or("审批人");

    match salary_adjustment_service::approve_adjustment(db, q.id, approver_id, approver_name).await {
        Ok(_) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success("审批通过".to_string(), "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 审批驳回
pub async fn reject(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<salary_adjustment_service::RejectDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let approver_id = jwt_token.id.unwrap_or(0);
    let approver_name = jwt_token.username.as_deref().unwrap_or("审批人");

    match salary_adjustment_service::reject_adjustment(
        db,
        dto.id,
        approver_id,
        approver_name,
        &dto.reason,
    )
    .await
    {
        Ok(_) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success("已驳回".to_string(), "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 调薪前后对比
pub async fn comparison(
    state: web::Data<AppState>,
    query: web::Query<EmployeeQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;

    match salary_adjustment_service::get_comparison(db, q.employee_id).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance/salary-adjustment")
            .route(
                "/list",
                web::get()
                    .to(list)
                    .wrap(require_permission("finance:adjustment:list")),
            )
            .route(
                "/history",
                web::get()
                    .to(history)
                    .wrap(require_permission("finance:adjustment:list")),
            )
            .route(
                "/create",
                web::post()
                    .to(create)
                    .wrap(require_permission("finance:adjustment:manage")),
            )
            .route(
                "/approve",
                web::post()
                    .to(approve)
                    .wrap(require_permission("finance:adjustment:manage")),
            )
            .route(
                "/reject",
                web::post()
                    .to(reject)
                    .wrap(require_permission("finance:adjustment:manage")),
            )
            .route(
                "/comparison",
                web::get()
                    .to(comparison)
                    .wrap(require_permission("finance:adjustment:list")),
            ),
    );
}
