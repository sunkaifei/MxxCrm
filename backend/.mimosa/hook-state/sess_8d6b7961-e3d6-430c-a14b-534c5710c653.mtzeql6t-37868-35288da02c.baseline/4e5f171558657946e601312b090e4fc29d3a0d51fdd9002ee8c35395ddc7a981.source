//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::kit::global::AppState;
use crate::core::web::base_controller::{get_current_user, get_current_user_id};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::profile::{
    HrArchiveUpdateRequest, ProfileLogQuery, UnlockRequest,
};
use crate::modules::system::service::hr_archive_service;

#[derive(Debug, serde::Deserialize)]
pub struct ArchiveListQuery {
    pub keyword: Option<String>,
    /// true=仅看资料完善 / false=仅看有缺项 / None=全部
    pub filled: Option<bool>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// GET /hr-archive/list - 全员档案分页
pub async fn archive_list(
    state: web::Data<AppState>,
    query: web::Query<ArchiveListQuery>,
) -> HttpResponse {
    let q = query.0;
    match hr_archive_service::get_archive_page(
        &state.db,
        q.keyword,
        q.filled,
        q.page.unwrap_or(1).max(1),
        q.page_size.unwrap_or(20).clamp(1, 100),
    )
    .await
    {
        Ok(page) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(
            page.items, "local", page.current_page as u32, page.total as u32,
        )),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

/// GET /hr-archive/{adminId} - 完整档案详情
pub async fn archive_detail(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> HttpResponse {
    match hr_archive_service::get_archive_detail(&state.db, path.into_inner()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// PUT /hr-archive/{adminId} - HR 代改（写审计日志）
pub async fn archive_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
    payload: web::Json<HrArchiveUpdateRequest>,
) -> HttpResponse {
    let operator_id = get_current_user_id(&req);
    let (_, operator_name) = get_current_user(&req);
    match hr_archive_service::hr_update(
        &state.db,
        path.into_inner(),
        payload.0,
        operator_id,
        &operator_name,
    )
    .await
    {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i32>::success(1, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /hr-archive/{adminId}/unlock - 解锁身份证/银行卡
pub async fn archive_unlock(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
    payload: web::Json<UnlockRequest>,
) -> HttpResponse {
    let operator_id = get_current_user_id(&req);
    let (_, operator_name) = get_current_user(&req);
    match hr_archive_service::hr_unlock(
        &state.db,
        path.into_inner(),
        &payload.0.field,
        operator_id,
        &operator_name,
    )
    .await
    {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i32>::success(1, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /hr-archive/logs - 变更日志查询
pub async fn archive_logs(
    state: web::Data<AppState>,
    query: web::Query<ProfileLogQuery>,
) -> HttpResponse {
    match hr_archive_service::get_log_page(&state.db, query.0).await {
        Ok(page) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(
            page.items, "local", page.current_page as u32, page.total as u32,
        )),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/hr-archive")
            .route(
                "/list",
                web::get()
                    .to(archive_list)
                    .wrap(require_permission("system:hr-archive:list")),
            )
            .route(
                "/logs",
                web::get()
                    .to(archive_logs)
                    .wrap(require_permission("system:hr-archive:list")),
            )
            .route(
                "/{adminId}",
                web::get()
                    .to(archive_detail)
                    .wrap(require_permission("system:hr-archive:view")),
            )
            .route(
                "/{adminId}",
                web::put()
                    .to(archive_update)
                    .wrap(require_permission("system:hr-archive:update")),
            )
            .route(
                "/{adminId}/unlock",
                web::post()
                    .to(archive_unlock)
                    .wrap(require_permission("system:hr-archive:update")),
            ),
    );
}
