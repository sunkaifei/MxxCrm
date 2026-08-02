//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 考勤扣款控制器
//!

use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::finance::service::attendance_service;

/// 列表查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub employee_id: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 详情查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailQuery {
    pub employee_id: i64,
    pub year: i32,
    pub month: i32,
}

/// 删除参数
#[derive(Deserialize)]
pub struct DeleteQuery {
    pub id: i64,
}

/// 批量导入请求体
#[derive(Deserialize)]
pub struct BatchImportDTO {
    pub records: Vec<attendance_service::AttendanceImportItem>,
}

/// 计算扣款查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeductionQuery {
    pub employee_id: i64,
    pub year: i32,
    pub month: i32,
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

    match attendance_service::get_attendance_list(
        db, q.year, q.month, q.employee_id, page, page_size,
    )
    .await
    {
        Ok((list, total)) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success_with_page(list, "local", page as u32, total as u32)),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 详情
pub async fn detail(
    state: web::Data<AppState>,
    query: web::Query<DetailQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;

    match attendance_service::get_attendance_detail(db, q.employee_id, q.year, q.month).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 新增/更新
pub async fn upsert(
    state: web::Data<AppState>,
    form_data: web::Json<attendance_service::AttendanceUpsertDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match attendance_service::upsert_attendance(db, dto).await {
        Ok(id) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 删除
pub async fn delete(
    state: web::Data<AppState>,
    query: web::Query<DeleteQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;

    match attendance_service::delete_attendance(db, q.id).await {
        Ok(_) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success("删除成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 批量导入
pub async fn batch_import(
    state: web::Data<AppState>,
    form_data: web::Json<BatchImportDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match attendance_service::batch_import(db, dto.records).await {
        Ok(count) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(count, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 计算扣款
pub async fn calculate_deduction(
    state: web::Data<AppState>,
    query: web::Query<DeductionQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;

    match attendance_service::calculate_deduction(db, q.employee_id, q.year, q.month).await {
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
        web::scope("/finance/attendance")
            .route(
                "/list",
                web::get()
                    .to(list)
                    .wrap(require_permission("finance:attendance:list")),
            )
            .route(
                "/detail",
                web::get()
                    .to(detail)
                    .wrap(require_permission("finance:attendance:list")),
            )
            .route(
                "/upsert",
                web::post()
                    .to(upsert)
                    .wrap(require_permission("finance:attendance:manage")),
            )
            .route(
                "/delete",
                web::post()
                    .to(delete)
                    .wrap(require_permission("finance:attendance:manage")),
            )
            .route(
                "/batch-import",
                web::post()
                    .to(batch_import)
                    .wrap(require_permission("finance:attendance:manage")),
            )
            .route(
                "/calculate-deduction",
                web::get()
                    .to(calculate_deduction)
                    .wrap(require_permission("finance:attendance:list")),
            ),
    );
}
