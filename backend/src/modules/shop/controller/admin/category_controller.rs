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
use crate::modules::articles::model::category::{CategoryModel, CategoryPageDTO, CategorySaveDTO};
use crate::modules::articles::service::category_service;
use actix_web::{web, HttpRequest, HttpResponse};

/// Save category
pub async fn save(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let body = body.into_inner();

    let website_id = req
        .headers()
        .get("website_id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    let dto = CategorySaveDTO {
        id: None,
        parent_id: body
            .get("parentId")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| body.get("parentId").and_then(|v| v.as_i64())),
        short_url: body.get("shortUrl").and_then(|v| v.as_str()).map(|s| s.to_string()),
        website_id,
        category_name: body
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                body.get("categoryName")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }),
        sort: body
            .get("sortOrder")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .or_else(|| body.get("sort").and_then(|v| v.as_i64()).map(|v| v as i32)),
        is_show: body.get("isShow").and_then(|v| v.as_i64()).map(|v| v as i32),
        status: body.get("status").and_then(|v| v.as_i64()).map(|v| v as i32),
        page_type: body.get("pageType").and_then(|v| v.as_i64()).map(|v| v as i32),
        page_template_data_id: body
            .get("pageTemplateDataId")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| body.get("pageTemplateDataId").and_then(|v| v.as_i64())),
        banner_image: body
            .get("bannerImage")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        description: body
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        content_type: body
            .get("contentType")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32),
        link_url: body
            .get("linkUrl")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    };

    match category_service::save_category(db, dto).await {
        Ok(id) => {
            let result = serde_json::json!({ "id": id });
            Ok(HttpResponse::Ok()
                .content_type("application/msgpack")
                .body(MetaResp::success(result, "local")))
        }
        Err(err) => {
            let err_msg = err.to_string();
            Ok(HttpResponse::Ok()
                .content_type("application/msgpack")
                .body(MetaResp::<String>::fail(400, &err_msg, "local")))
        }
    }
}

/// Update category
pub async fn update(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let body = body.into_inner();

    let website_id = req
        .headers()
        .get("website_id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    let dto = CategorySaveDTO {
        id: body
            .get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| body.get("id").and_then(|v| v.as_i64())),
        parent_id: body
            .get("parentId")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| body.get("parentId").and_then(|v| v.as_i64())),
        short_url: body.get("shortUrl").and_then(|v| v.as_str()).map(|s| s.to_string()),
        website_id,
        category_name: body
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                body.get("categoryName")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }),
        sort: body
            .get("sortOrder")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .or_else(|| body.get("sort").and_then(|v| v.as_i64()).map(|v| v as i32)),
        is_show: body.get("isShow").and_then(|v| v.as_i64()).map(|v| v as i32),
        status: body.get("status").and_then(|v| v.as_i64()).map(|v| v as i32),
        page_type: body.get("pageType").and_then(|v| v.as_i64()).map(|v| v as i32),
        page_template_data_id: body
            .get("pageTemplateDataId")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| body.get("pageTemplateDataId").and_then(|v| v.as_i64())),
        banner_image: body
            .get("bannerImage")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        description: body
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        content_type: body
            .get("contentType")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32),
        link_url: body
            .get("linkUrl")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    };

    match category_service::update_by_id(db, dto).await {
        Ok(affected) if affected > 0 => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::success("修改成功".to_string(), "local"))),
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "更新失败", "local"))),
        Err(err) => {
            let err_msg = err.to_string();
            Ok(HttpResponse::Ok()
                .content_type("application/msgpack")
                .body(MetaResp::<String>::fail(400, &err_msg, "local")))
        }
    }
}

/// Delete category
pub async fn delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let body = body.into_inner();

    let website_id = req
        .headers()
        .get("website_id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .or_else(|| body.get("id").and_then(|v| v.as_i64()));

    if let Some(id_val) = id {
        let result = CategoryModel::batch_delete_by_ids(db, &website_id, vec![id_val]).await?;
        Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// Get category tree
pub async fn tree(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;

    let website_id = req
        .headers()
        .get("website_id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    let dto = CategoryPageDTO {
        category_name: None,
        website_id,
        is_show: None,
        status: None,
    };

    match category_service::select_all_list(db, dto).await {
        Ok(list) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(list, "local"))),
        Err(err) => {
            let err_msg = err.to_string();
            Ok(HttpResponse::Ok()
                .content_type("application/msgpack")
                .body(MetaResp::<String>::fail(400, &err_msg, "local")))
        }
    }
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
