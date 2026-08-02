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
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::website::model::website::{SiteSaveDTO, SiteUpdateRequest};
use crate::modules::website::service::website_service;
use actix_web::{web, HttpResponse};

/// 获取当前（默认）站点配置
///
/// 单站模式下不需要传 ID，直接返回 `is_default=1` 的站点。
/// 用于前端"网站设置"整页表单的初始化加载。
pub async fn get_current(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = website_service::find_default(&db).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

/// 更新当前（默认）站点配置
///
/// 单站模式下不需要传 ID，后端自动定位默认站点。
/// 接收完整 `SiteUpdateRequest` 表单，不需要路径参数 `id`，对前端整页设置表单更友好。
pub async fn update_current(state: web::Data<AppState>, item: web::Json<SiteUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    // 定位默认站点 ID
    let default_site = website_service::find_default(&db).await?;
    let site_id = default_site.id.ok_or_else(|| crate::core::errors::error::Error::from("默认站点ID为空"))?;

    // 复用既有更新逻辑：把请求转 DTO 并强制注入默认站点 ID
    let mut dto: SiteSaveDTO = item.into_inner().into();
    dto.id = Some(site_id);
    // 单站模式下保留默认标记，避免被误清空
    if dto.is_default.unwrap_or_default() == 0 {
        dto.is_default = Some(1);
    }
    // 保留原 user_id（避免前端没传时被置空）
    if dto.user_id.is_none() {
        dto.user_id = default_site.user_id;
    }

    website_service::update_by_id(&db, &dto).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success("修改成功", "local")))
}

// ==================== 路由注册（单点维护）====================

/// 注册网站模块所有路由（单站模式）
///
/// 单站模式下仅保留 `/site/current` 的 GET/PUT 两个接口：
/// - GET  /site/current  获取当前站点配置
/// - PUT  /site/current  更新当前站点配置
///
/// 多站遗留接口（add/batch_delete/update/{id}/update_status/update_default/detail/{id}/list）
/// 已全部移除，路由不再注册。如需保留兼容性，可参考 git 历史。
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(website_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/site")
            // GET /site/current - 获取当前（默认）站点配置
            .route(
                "/current",
                web::get()
                    .to(get_current)
                    .wrap(require_permission("system:site:view")),
            )
            // PUT /site/current - 更新当前（默认）站点配置
            .route(
                "/current",
                web::put()
                    .to(update_current)
                    .wrap(require_permission("system:site:update")),
            ),
    );
}