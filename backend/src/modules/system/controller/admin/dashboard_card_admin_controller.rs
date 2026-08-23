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

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::dashboard_card::{
    DashboardCardAssignRolesRequest, DashboardCardListQuery, DashboardCardSaveRequest,
};
use crate::modules::system::service::{admin_service, dashboard_card_service};

/// 卡片管理列表（含角色分配信息）
pub async fn card_list(state: web::Data<AppState>, query: web::Query<DashboardCardListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    dashboard_card_service::get_by_page(db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

/// 新增卡片
pub async fn card_save(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<DashboardCardSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let admin = admin_service::get_by_detail(db, &Some(get_current_user_id(&req))).await?;
    let result = dashboard_card_service::insert(db, &form_data, &admin.user_name).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 更新卡片
pub async fn card_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<DashboardCardSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let admin = admin_service::get_by_detail(db, &Some(get_current_user_id(&req))).await?;
    let result = dashboard_card_service::update_by_id(db, &form_data, &admin.user_name).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 删除卡片（软删除 + 清理角色关联）
pub async fn card_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let ids = item.0.parse_ids();
    if ids.is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到有效的卡片ID", "local"));
    }
    let result = dashboard_card_service::batch_delete_by_ids(db, &ids).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

/// 分配卡片可见角色
pub async fn assign_roles(state: web::Data<AppState>, form_data: web::Json<DashboardCardAssignRolesRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = dashboard_card_service::update_card_roles(db, &form_data.0).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 当前用户可见卡片（前端页面动态渲染统计/概览卡片用，仅需登录）
pub async fn visible(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    dashboard_card_service::get_visible_cards(db, user_id).await.map(|list| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(list, "local"))
    })
}

/// 注册工作台卡片管理路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dashboard/card")
            // GET /dashboard/card/list - 卡片管理列表
            .route(
                "/list",
                web::get()
                    .to(card_list)
                    .wrap(require_permission("system:dashboard:list")),
            )
            // POST /dashboard/card/save - 新增卡片
            .route(
                "/save",
                web::post()
                    .to(card_save)
                    .wrap(require_permission("system:dashboard:save")),
            )
            // PUT /dashboard/card/update - 更新卡片
            .route(
                "/update",
                web::put()
                    .to(card_update)
                    .wrap(require_permission("system:dashboard:update")),
            )
            // DELETE /dashboard/card/bath_delete - 删除卡片
            .route(
                "/bath_delete",
                web::delete()
                    .to(card_delete)
                    .wrap(require_permission("system:dashboard:delete")),
            )
            // PUT /dashboard/card/assign_roles - 分配卡片可见角色
            .route(
                "/assign_roles",
                web::put()
                    .to(assign_roles)
                    .wrap(require_permission("system:dashboard:update")),
            )
            // GET /dashboard/card/visible - 当前用户可见卡片（仅需登录）
            .route("/visible", web::get().to(visible)),
    );
}
