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
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::controller::admin::website_media_category_admin_controller;
use crate::modules::website::model::website_media::{ListQuery, MediaSaveDTO, MediaSaveRequest, MediaUpdateRequest};
use crate::modules::website::service::website_media_service;

/// 新增媒体
pub async fn add(state: web::Data<AppState>, _req: HttpRequest, item: web::Json<MediaSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    if payload.original_name.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "原始文件名不能为空", "local")));
    }
    if payload.storage_name.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "存储文件名不能为空", "local")));
    }
    let form_data = MediaSaveDTO::from(payload);
    let result = website_media_service::insert(db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("添加成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}

/// 批量删除
pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let result = website_media_service::batch_delete_by_ids(db, &ids_vec).await?;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// 更新媒体元数据
pub async fn update_by_id(state: web::Data<AppState>, _req: HttpRequest, id: web::Path<i64>, item: web::Json<MediaUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    let media_id = Some(id.into_inner());
    let mut form_data = MediaSaveDTO::from(payload);
    form_data.id = media_id;
    let result = website_media_service::update_by_id(db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("修改成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "修改失败", "local")))
    }
}

/// 查询详情
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = website_media_service::get_by_detail(db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

/// 分页查询列表
pub async fn get_by_page(state: web::Data<AppState>, _req: HttpRequest, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = query.into_inner();
    website_media_service::get_by_page(db, form_data).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

// ==================== 路由注册（单点维护）====================

/// 注册媒体模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(website_media_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/website/media")
            // POST /website/media/add - 新建媒体
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("website:media:add")),
            )
            // DELETE /website/media/batch_delete - 批量删除媒体
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("website:media:delete")),
            )
            // PUT /website/media/update/{id} - 修改媒体
            .route(
                "/update/{id}",
                web::put()
                    .to(update_by_id)
                    .wrap(require_permission("website:media:update")),
            )
            // GET /website/media/detail/{id} - 媒体详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("website:media:view")),
            )
            // GET /website/media/list - 媒体列表
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("website:media:list")),
            )
            // 媒体分类管理（注册在 /website/media scope 内，避免 /website/media scope 吞掉 /website/media/category 路由）
            .configure(website_media_category_admin_controller::register),
    );
}
