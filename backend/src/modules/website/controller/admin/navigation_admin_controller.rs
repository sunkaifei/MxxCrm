//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use crate::core::errors::error::Result;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::model::navigation::{NavigationSaveDTO, NavigationListQuery, NavigationDetailVO};
use crate::modules::website::service::{navigation_service, website_service};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

/// 获取默认站点ID
async fn get_default_site_id(db: &sea_orm::DbConn) -> Result<i64> {
    let site = website_service::find_default(db).await?;
    Ok(site.id.unwrap_or_default())
}

/// GET /navigation/list - 导航列表
pub async fn get_by_page(
    state: web::Data<AppState>,
    _req: HttpRequest,
    query: web::Query<NavigationListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let site_id = get_default_site_id(db).await?;
    let list = navigation_service::find_all(db, site_id).await?;

    // 按导航类型过滤（如只看 header / footer）
    let nav_type = query.nav_type.clone();
    let filtered: Vec<_> = if let Some(nt) = nav_type {
        list.into_iter().filter(|n| n.nav_type.as_deref() == Some(nt.as_str())).collect()
    } else {
        list
    };

    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success(filtered, "local")))
}

/// GET /navigation/detail/{id} - 导航详情
pub async fn get_by_detail(
    state: web::Data<AppState>,
    _req: HttpRequest,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let site_id = get_default_site_id(db).await?;
    let result: NavigationDetailVO = navigation_service::find_by_id(db, site_id, id.into_inner()).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success(result, "local")))
}

/// POST /navigation/add - 新增导航
pub async fn add(
    state: web::Data<AppState>,
    _req: HttpRequest,
    item: web::Json<NavigationSaveDTO>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let site_id = get_default_site_id(db).await?;
    let payload = item.0;

    if payload.name.is_none() {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "导航名称不能为空", "local")));
    }

    let result = navigation_service::insert(db, site_id, payload).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// PUT /navigation/update/{id} - 更新导航
pub async fn update_by_id(
    state: web::Data<AppState>,
    _req: HttpRequest,
    id: web::Path<i64>,
    item: web::Json<NavigationSaveDTO>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let site_id = get_default_site_id(db).await?;
    let mut payload = item.0;
    payload.id = id.into_inner();

    if payload.name.is_none() {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "导航名称不能为空", "local")));
    }

    let result = navigation_service::update(db, site_id, payload).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// DELETE /navigation/batch_delete - 批量删除
pub async fn batch_delete(
    state: web::Data<AppState>,
    item: web::Json<BathDeleteIdRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let site_id = get_default_site_id(db).await?;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let result = navigation_service::batch_delete(db, site_id, ids).await;
        Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

// ==================== 路由注册 ====================

/// 注册导航管理模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/navigation")
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("website:navigation:list")),
            )
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("website:navigation:view")),
            )
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("website:navigation:add")),
            )
            .route(
                "/update/{id}",
                web::put()
                    .to(update_by_id)
                    .wrap(require_permission("website:navigation:update")),
            )
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("website:navigation:delete")),
            ),
    );
}
