//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 团队提成控制器
//!

use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::finance::service::{team_commission_service, commission_allocation_service};
use crate::modules::finance::service::commission_allocation_service::{AllocateDTO, PendingQuery};

/// 列表查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub year: i32,
    pub month: i32,
    pub manager_id: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 汇总查询参数
#[derive(Deserialize)]
pub struct SummaryQuery {
    pub year: i32,
    pub month: i32,
}

/// 触发结算请求体
#[derive(Deserialize)]
pub struct CalculateDTO {
    pub year: i32,
    pub month: i32,
}

/// 待分配列表查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub allocator_id: Option<i64>,
}

/// 分配记录查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationLogQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub allocator_id: Option<i64>,
}

/// 查询团队提成列表
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<ListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).max(1);

    match team_commission_service::get_team_commission_list(
        db, q.year, q.month, q.manager_id, page, page_size,
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

/// 触发月度团队提成结算
pub async fn calculate(
    state: web::Data<AppState>,
    form_data: web::Json<CalculateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match team_commission_service::calc_monthly_settlement(db, dto.year, dto.month).await {
        Ok(count) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(count, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 团队提成汇总
pub async fn summary(
    state: web::Data<AppState>,
    query: web::Query<SummaryQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;

    match team_commission_service::get_team_summary(db, q.year, q.month).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 待分配列表（category=5 归集的待分配提成）
pub async fn pending_list(
    state: web::Data<AppState>,
    query: web::Query<PendingListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let page = q.page.unwrap_or(1);
    let query = PendingQuery {
        page: Some(page),
        page_size: q.page_size,
        year: q.year,
        month: q.month,
        allocator_id: q.allocator_id,
    };

    match commission_allocation_service::get_pending_list(db, query).await {
        Ok((list, total)) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success_with_page(
                list,
                "local",
                page as u32,
                total as u32,
            )),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 提交分配
pub async fn allocate(
    state: web::Data<AppState>,
    form_data: web::Json<AllocateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match commission_allocation_service::allocate(db, dto).await {
        Ok(count) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(count, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 分配记录
pub async fn allocation_log(
    state: web::Data<AppState>,
    query: web::Query<AllocationLogQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).max(1);

    match commission_allocation_service::get_allocation_log(
        db, q.year, q.month, q.allocator_id, page, page_size,
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

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance/team-commission")
            .route(
                "/list",
                web::get()
                    .to(list)
                    .wrap(require_permission("finance:team-commission:list")),
            )
            .route(
                "/calculate",
                web::post()
                    .to(calculate)
                    .wrap(require_permission("finance:team-commission:manage")),
            )
            .route(
                "/summary",
                web::get()
                    .to(summary)
                    .wrap(require_permission("finance:team-commission:list")),
            )
            // v2 新增：待分配 + 分配 + 分配记录
            .route(
                "/pending",
                web::get()
                    .to(pending_list)
                    .wrap(require_permission("finance:team-commission:list")),
            )
            .route(
                "/allocate",
                web::post()
                    .to(allocate)
                    .wrap(require_permission("finance:team-commission:manage")),
            )
            .route(
                "/allocation-log",
                web::get()
                    .to(allocation_log)
                    .wrap(require_permission("finance:team-commission:list")),
            ),
    );
}
