//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 外勤拜访专用控制器
//! - 签到/签退：基于 followup 表（activity_type=2）扩展实现
//! - 数据权限：列表/统计接口使用 data_scope_service 过滤

use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::crm::model::followup::{VisitCheckInRequest, VisitListQuery};
use crate::modules::crm::service::followup_service;

/// GET /visit/list - 外勤拜访列表（筛选 activity_type=2 的跟进记录）
pub async fn visit_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<VisitListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    let query = query.0;

    match followup_service::visit_list(db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /visit/info/{id} - 拜访详情
pub async fn visit_info(state: web::Data<AppState>, id: web::Path<i64>) -> HttpResponse {
    let db = &state.db;
    let id = id.into_inner();

    match followup_service::visit_info(db, id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /visit/check-in - 签到（创建一条 activity_type=2 的跟进记录）
pub async fn visit_check_in(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<VisitCheckInRequest>,
) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let created_by = jwt_token.id.unwrap_or_default();

    if created_by <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }

    match followup_service::visit_check_in(db, &form_data, created_by).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /visit/check-out/{id} - 签退（更新 check_out_time）
pub async fn visit_check_out(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
) -> HttpResponse {
    let db = &state.db;
    let id = id.into_inner();

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let updated_by = jwt_token.id.unwrap_or_default();

    if updated_by <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }

    match followup_service::visit_check_out(db, id, updated_by).await {
        Ok(rows) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(rows, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /visit/statistics - 拜访统计（按人/按日统计）
pub async fn visit_statistics(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();

    if current_user_id <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }

    match followup_service::visit_statistics(db, current_user_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 注册外勤拜访模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/visit")
            // GET /visit/list - 外勤拜访列表
            .route(
                "/list",
                web::get()
                    .to(visit_list)
                    .wrap(require_permission("crm:visit:list")),
            )
            // GET /visit/info/{id} - 拜访详情
            .route(
                "/info/{id}",
                web::get()
                    .to(visit_info)
                    .wrap(require_permission("crm:visit:list")),
            )
            // POST /visit/check-in - 签到
            .route(
                "/check-in",
                web::post()
                    .to(visit_check_in)
                    .wrap(require_permission("crm:visit:add")),
            )
            // POST /visit/check-out/{id} - 签退
            .route(
                "/check-out/{id}",
                web::post()
                    .to(visit_check_out)
                    .wrap(require_permission("crm:visit:add")),
            )
            // GET /visit/statistics - 拜访统计
            .route(
                "/statistics",
                web::get()
                    .to(visit_statistics)
                    .wrap(require_permission("crm:visit:list")),
            ),
    );
}
