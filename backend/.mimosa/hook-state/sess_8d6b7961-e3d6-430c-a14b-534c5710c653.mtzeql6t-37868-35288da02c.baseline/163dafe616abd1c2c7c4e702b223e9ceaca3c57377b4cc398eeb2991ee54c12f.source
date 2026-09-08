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
use chrono::NaiveDate;

use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::service::{permission_cache_service, resign_service};

/// 当前用户是否拥有指定权限码（从权限缓存读取，实时生效）
async fn has_permission(db: &sea_orm::DbConn, user_id: i64, code: &str) -> bool {
    permission_cache_service::get_or_load_permissions(db, user_id)
        .await
        .iter()
        .any(|p| p == code)
}

/// 是否可中止离职：HR（system:resign:confirm）或 发起人本人
async fn can_abort(db: &sea_orm::DbConn, record_id: i64, operator_id: i64) -> bool {
    if has_permission(db, operator_id, "system:resign:confirm").await {
        return true;
    }
    match resign_service::get_detail(db, record_id).await {
        Ok(detail) => detail.get("adminId").and_then(|v| v.as_i64()) == Some(operator_id),
        Err(_) => false,
    }
}

/// GET /resign/list - 交接单列表（system:resign:view）
pub async fn resign_list(
    state: web::Data<AppState>,
    query: web::Query<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let keyword = query.get("keyword").and_then(|v| v.as_str()).map(str::to_string);
    let status = query.get("status").and_then(|v| v.as_i64()).map(|v| v as i32);
    let page = query.get("page").and_then(|v| v.as_u64()).unwrap_or(1);
    let page_size = query.get("pageSize").and_then(|v| v.as_u64()).unwrap_or(10);
    match resign_service::get_list(db, keyword, status, page, page_size).await {
        Ok(data) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /resign/{id} - 交接单详情
/// 访问控制：system:resign:view 权限，或发起人本人/交接项确认人（本人数据兜底，方案 3.6.1）；
/// 离职原因按访问者身份过滤（方案 3.6.5：财务/系统管理员/交接确认人不可见）
pub async fn resign_detail(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
) -> HttpResponse {
    let db = &state.db;
    let id = path.into_inner();
    let operator_id = get_current_user_id(&req);
    if operator_id == 0 {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }
    let has_view = has_permission(db, operator_id, "system:resign:view").await;
    if !has_view {
        let allowed = matches!(
            resign_service::can_view_record(db, id, operator_id).await,
            Ok(true)
        );
        if !allowed {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(403, "您没有查看该交接单的权限", "local"));
        }
    }
    match resign_service::get_detail_for_operator(db, id, operator_id).await {
        Ok(data) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /resign/{id}/confirm-item - 交接项确认（assignee 本人或 HR 代确认）
pub async fn resign_confirm_item(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
    item: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let record_id = path.into_inner();
    let operator_id = get_current_user_id(&req);
    if operator_id == 0 {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }
    let item_id = item.get("itemId").and_then(|v| v.as_i64()).unwrap_or(0);
    let is_na = item.get("isNa").and_then(|v| v.as_bool()).unwrap_or(false);
    let remark = item.get("remark").and_then(|v| v.as_str()).map(str::to_string);
    if item_id <= 0 {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "参数错误：itemId 不能为空", "local"));
    }
    // assignee 本人确认无需权限码；非本人操作需 system:resign:confirm（HR 代确认兜底）
    let is_hr_override = has_permission(db, operator_id, "system:resign:confirm").await;
    match resign_service::confirm_item(db, record_id, item_id, operator_id, is_na, remark, is_hr_override).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /resign/{id}/settle - 财务结算确认（system:resign:settle，结算即完全离职）
pub async fn resign_settle(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    item: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let record_id = path.into_inner();
    let leave_date = item
        .get("leaveDate")
        .and_then(|v| v.as_str())
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    match resign_service::settle(db, record_id, leave_date).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /resign/{id}/abort - 离职中止（发起人本人或 HR，理由必填）
pub async fn resign_abort(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
    item: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let record_id = path.into_inner();
    let operator_id = get_current_user_id(&req);
    if operator_id == 0 {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }
    let reason = item.get("reason").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if !can_abort(db, record_id, operator_id).await {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(403, "您没有中止该离职流程的权限", "local"));
    }
    match resign_service::abort(db, record_id, reason).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /resign/{id}/transfer-assignee - 交接确认人转派（system:resign:confirm）
pub async fn resign_transfer_assignee(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    item: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let record_id = path.into_inner();
    let item_id = item.get("itemId").and_then(|v| v.as_i64()).unwrap_or(0);
    let new_assignee_id = item.get("newAssigneeId").and_then(|v| v.as_i64()).unwrap_or(0);
    if item_id <= 0 || new_assignee_id <= 0 {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "参数错误", "local"));
    }
    match resign_service::transfer_assignee(db, record_id, item_id, new_assignee_id).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 路由注册（挂载到 /api/system scope）
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/resign")
            // GET /api/system/resign/list - 交接单列表
            .route(
                "/list",
                web::get()
                    .to(resign_list)
                    .wrap(require_permission("system:resign:view")),
            )
            // GET /api/system/resign/{id} - 交接单详情
            // 访问控制在 handler 内：view 权限 或 发起人本人/交接项确认人（本人数据兜底）
            .route("/{id}", web::get().to(resign_detail))
            // POST /api/system/resign/{id}/confirm-item - 交接项确认
            .route("/{id}/confirm-item", web::post().to(resign_confirm_item))
            // POST /api/system/resign/{id}/settle - 财务结算确认
            .route(
                "/{id}/settle",
                web::post()
                    .to(resign_settle)
                    .wrap(require_permission("system:resign:settle")),
            )
            // POST /api/system/resign/{id}/abort - 离职中止
            .route("/{id}/abort", web::post().to(resign_abort))
            // POST /api/system/resign/{id}/transfer-assignee - 确认人转派
            .route(
                "/{id}/transfer-assignee",
                web::post()
                    .to(resign_transfer_assignee)
                    .wrap(require_permission("system:resign:confirm")),
            ),
    );
}
