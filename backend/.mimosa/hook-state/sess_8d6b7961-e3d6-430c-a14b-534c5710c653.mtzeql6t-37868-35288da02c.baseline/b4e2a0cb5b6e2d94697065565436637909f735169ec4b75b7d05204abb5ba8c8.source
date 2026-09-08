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
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::entity::common::InfoId;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::model::website_media_category::{MediaCategorySaveDTO, MediaCategorySaveRequest, MediaCategoryUpdateRequest};
use crate::modules::website::service::website_media_category_service;

/// 新增媒体分类
pub async fn add(state: web::Data<AppState>, _req: HttpRequest, item: web::Json<MediaCategorySaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    if payload.category_name.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "分类名称不能为空", "local")));
    }
    let form_data = MediaCategorySaveDTO::from(payload);
    let result = website_media_category_service::insert(db, form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("添加成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}

/// 删除媒体分类
pub async fn delete(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = website_media_category_service::delete_by_id(db, item.id.unwrap_or_default()).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("删除成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除失败", "local")))
    }
}

/// 更新媒体分类
pub async fn update_by_id(state: web::Data<AppState>, _req: HttpRequest, id: web::Path<i64>, item: web::Json<MediaCategoryUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    let category_id = Some(id.into_inner());
    if payload.category_name.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "分类名称不能为空", "local")));
    }
    let mut form_data = MediaCategorySaveDTO::from(payload);
    form_data.id = category_id;
    let result = website_media_category_service::update_by_id(db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("修改成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "修改失败", "local")))
    }
}

/// 查询媒体分类详情
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = website_media_category_service::get_by_detail(db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

/// 查询所有媒体分类（树形列表）
pub async fn get_by_list(state: web::Data<AppState>, _req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let result = website_media_category_service::select_all(db).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

/// 查询所有媒体分类（下拉选项）
pub async fn get_by_options(state: web::Data<AppState>, _req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let result = website_media_category_service::select_all_options(db).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

// ==================== 路由注册（单点维护）====================

/// 注册媒体分类模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 通过 `website_media_admin_controller::register` 中的 `.configure()` 嵌套注册，
/// 避免父级 `/website/media` scope 吞掉 `/website/media/category` 路由。
/// 最终URL前缀为 `/website/media/category`。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/category")
            // POST /website/media/category/add - 新建媒体分类
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("website:media:category:add")),
            )
            // DELETE /website/media/category/delete/{id} - 删除媒体分类
            .route(
                "/delete/{id}",
                web::delete()
                    .to(delete)
                    .wrap(require_permission("website:media:category:delete")),
            )
            // PUT /website/media/category/update/{id} - 修改媒体分类
            .route(
                "/update/{id}",
                web::put()
                    .to(update_by_id)
                    .wrap(require_permission("website:media:category:update")),
            )
            // GET /website/media/category/detail/{id} - 媒体分类详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("website:media:category:view")),
            )
            // GET /website/media/category/list - 媒体分类列表（树形）
            .route(
                "/list",
                web::get()
                    .to(get_by_list)
                    .wrap(require_permission("website:media:category:list")),
            )
            // GET /website/media/category/all - 所有媒体分类（下拉选项）
            .route(
                "/all",
                web::get()
                    .to(get_by_options)
                    .wrap(require_permission("website:media:category:list")),
            ),
    );
}
