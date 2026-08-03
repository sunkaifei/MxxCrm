//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::model::website_notification_config::{
    NotificationConfigListQuery, NotificationConfigSaveDTO,
};
use crate::modules::website::service::website_notification_config_service;
use crate::modules::website::service::website_service;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ToggleEnabledRequest {
    pub enabled: i32,
}

/// 批量保存当前站点通知配置的请求体
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkUpsertRequest {
    pub configs: Vec<NotificationConfigSaveDTO>,
}

/// GET /website_notification/current - 获取当前（默认）站点的全部通知配置
///
/// 单站模式下不需要传 website_id，后端自动定位默认站点。
/// 供前端"网站设置 → 通知配置"Tab 一次性加载。
pub async fn get_current(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    let default_site = website_service::find_default(&db).await?;
    let website_id = default_site
        .id
        .ok_or_else(|| crate::core::errors::error::Error::from("默认站点ID为空"))?;
    let list = website_notification_config_service::find_current_all(&db, website_id).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success(list, "local")))
}

/// PUT /website_notification/current - 批量保存当前站点通知配置
///
/// 单站模式下不需要传 website_id，后端自动定位默认站点。
/// 请求体 `{ configs: [...] }`，每条按 scene_code 走 upsert 语义。
pub async fn update_current(
    state: web::Data<AppState>,
    body: web::Json<BulkUpsertRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let default_site = website_service::find_default(&db).await?;
    let website_id = default_site
        .id
        .ok_or_else(|| crate::core::errors::error::Error::from("默认站点ID为空"))?;
    let affected =
        website_notification_config_service::bulk_upsert(&db, website_id, body.into_inner().configs)
            .await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success(affected, "local")))
}

/// GET /website_notification/list - 通知配置列表（分页）
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<NotificationConfigListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match website_notification_config_service::get_by_page(db, query.into_inner()).await {
        Ok(page) => Ok(HttpResponse::Ok().json(page)),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /website_notification/detail/{id} - 通知配置详情
pub async fn detail(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match website_notification_config_service::get_by_id(db, id.into_inner()).await {
        Ok(vo) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(vo, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// POST /website_notification/create - 新增通知配置
pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<NotificationConfigSaveDTO>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = website_notification_config_service::create(db, body.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// PUT /website_notification/update/{id} - 编辑通知配置
pub async fn update(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<NotificationConfigSaveDTO>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result =
        website_notification_config_service::update(db, id.into_inner(), body.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// PUT /website_notification/toggle/{id} - 启用/停用
pub async fn toggle(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<ToggleEnabledRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result =
        website_notification_config_service::toggle_enabled(db, id.into_inner(), body.enabled).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// DELETE /website_notification/batch_delete - 批量删除
pub async fn batch_delete(
    state: web::Data<AppState>,
    item: web::Json<BathDeleteIdRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let result = website_notification_config_service::batch_delete(db, ids).await;
        Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册网站通知配置管理模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/website_notification")
            // GET /current - 获取当前站点全部通知配置（单站模式）
            .route(
                "/current",
                web::get().to(get_current).wrap(require_permission("website:notification:view")),
            )
            // PUT /current - 批量保存当前站点通知配置（单站模式）
            .route(
                "/current",
                web::put().to(update_current).wrap(require_permission("website:notification:update")),
            )
            .route(
                "/list",
                web::get().to(list).wrap(require_permission("website:notification:list")),
            )
            .route(
                "/detail/{id}",
                web::get().to(detail).wrap(require_permission("website:notification:view")),
            )
            .route(
                "/create",
                web::post()
                    .to(create)
                    .wrap(require_permission("website:notification:update")),
            )
            .route(
                "/update/{id}",
                web::put()
                    .to(update)
                    .wrap(require_permission("website:notification:update")),
            )
            .route(
                "/toggle/{id}",
                web::put()
                    .to(toggle)
                    .wrap(require_permission("website:notification:toggle")),
            )
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("website:notification:update")),
            ),
    );
}
