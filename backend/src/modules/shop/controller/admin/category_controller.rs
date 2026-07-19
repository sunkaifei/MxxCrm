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
use crate::core::web::entity::common::InfoId;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use actix_web::{web, HttpRequest, HttpResponse};

/// Save category
pub async fn save(
    state: web::Data<AppState>,
    _req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call category_service::save(db, body.into_inner()).await
    let result = serde_json::json!({ "id": 0 });
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// Update category
pub async fn update(
    state: web::Data<AppState>,
    _req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call category_service::update(db, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Delete category
pub async fn delete(
    state: web::Data<AppState>,
    _req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call category_service::delete(db, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Get category tree
pub async fn tree(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call category_service::get_tree(db).await
    let result = serde_json::json!([]);
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

// ==================== 路由注册（单点维护）====================

/// 注册店铺分类模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(category_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/category")
            // POST /category/save - 保存分类
            .route(
                "/save",
                web::post()
                    .to(save)
                    .wrap(require_permission("system:category:save")),
            )
            // PUT /category/update - 更新分类
            .route(
                "/update",
                web::put()
                    .to(update)
                    .wrap(require_permission("system:category:update")),
            )
            // DELETE /category/delete - 删除分类
            .route(
                "/delete",
                web::delete()
                    .to(delete)
                    .wrap(require_permission("system:category:delete")),
            )
            // GET /category/tree - 分类树
            .route(
                "/tree",
                web::get()
                    .to(tree)
                    .wrap(require_permission("system:category:list")),
            ),
    );
}
