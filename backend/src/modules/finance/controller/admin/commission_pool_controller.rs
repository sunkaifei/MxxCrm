//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 团建资金池控制器
//!

use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::finance::service::commission_pool_service::{
    self, CommissionPoolSaveDTO, PoolExpenseDTO, PoolQuery,
};

/// 列表查询参数（GET query）
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub pool_name: Option<String>,
    pub department_id: Option<i64>,
    pub status: Option<i16>,
}

/// 流水查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 资金池列表
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<ListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let page = q.page.unwrap_or(1);
    let query = PoolQuery {
        page: q.page,
        page_size: q.page_size,
        pool_name: q.pool_name,
        department_id: q.department_id,
        status: q.status,
    };

    match commission_pool_service::get_pool_list(db, query).await {
        Ok((list, total)) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success_with_page(
                list,
                "local",
                page as u32,
                total as u32,
            )),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 资金池详情
pub async fn detail(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> HttpResponse {
    let db = &state.db;
    let id = path.into_inner();

    match commission_pool_service::get_pool_detail(db, id).await {
        Ok(vo) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(vo, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 新建/编辑资金池
pub async fn save(
    state: web::Data<AppState>,
    form_data: web::Json<CommissionPoolSaveDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match commission_pool_service::save_pool(db, dto).await {
        Ok(id) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 支出登记
pub async fn expense(
    state: web::Data<AppState>,
    form_data: web::Json<PoolExpenseDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match commission_pool_service::expense(db, dto).await {
        Ok(id) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 流水明细
pub async fn log(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    query: web::Query<LogQuery>,
) -> HttpResponse {
    let db = &state.db;
    let pool_id = path.into_inner();
    let q = query.0;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).max(1);

    match commission_pool_service::get_pool_log(db, pool_id, page, page_size).await {
        Ok((list, total)) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success_with_page(list, "local", page as u32, total as u32)),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance/commission-pool")
            .route(
                "/list",
                web::get()
                    .to(list)
                    .wrap(require_permission("finance:commission-pool:list")),
            )
            .route(
                "/detail/{id}",
                web::get()
                    .to(detail)
                    .wrap(require_permission("finance:commission-pool:list")),
            )
            .route(
                "/save",
                web::post()
                    .to(save)
                    .wrap(require_permission("finance:commission-pool:manage")),
            )
            .route(
                "/expense",
                web::post()
                    .to(expense)
                    .wrap(require_permission("finance:commission-pool:manage")),
            )
            .route(
                "/log/{id}",
                web::get()
                    .to(log)
                    .wrap(require_permission("finance:commission-pool:list")),
            ),
    );
}
