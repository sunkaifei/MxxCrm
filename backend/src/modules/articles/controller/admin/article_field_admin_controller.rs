//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use actix_web::{web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::articles::model::article_field::{
    ArticleFieldListQuery, ArticleFieldSaveDTO, ArticleFieldValueBatchDTO,
};
use crate::modules::articles::service::article_field_service;

/// GET /article/field/list - 分页查询字段定义
pub async fn get_by_page(
    state: web::Data<AppState>,
    query: web::Query<ArticleFieldListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = article_field_service::get_by_page(db, query.into_inner()).await?;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// GET /article/field/detail/{id} - 字段定义详情
pub async fn get_by_id(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = article_field_service::get_by_id(db, id.into_inner()).await?;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// GET /article/field/by_category/{category_id} - 按栏目查询全部字段
/// 供文章编辑页动态表单使用
pub async fn get_by_category(
    state: web::Data<AppState>,
    category_id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = article_field_service::get_by_category(db, category_id.into_inner()).await?;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// POST /article/field/add - 新增字段定义
pub async fn add(
    state: web::Data<AppState>,
    req: web::Json<ArticleFieldSaveDTO>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = req.into_inner();
    if payload.field_name.trim().is_empty() {
        return Err(Error::from("字段名不能为空"));
    }
    let result = article_field_service::create(db, payload).await?;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::success_with_msg(
            result.to_string(),
            "新增成功",
            "local",
        )))
}

/// PUT /article/field/update/{id} - 更新字段定义
pub async fn update(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    req: web::Json<ArticleFieldSaveDTO>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = article_field_service::update(db, id.into_inner(), req.into_inner()).await?;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::success_with_msg(
            result.to_string(),
            "更新成功",
            "local",
        )))
}

/// DELETE /article/field/batch_delete - 批量删除字段定义
pub async fn batch_delete(
    state: web::Data<AppState>,
    item: web::Json<BathDeleteIdRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Err(Error::from("删除的ID不能为空"));
        }
        // 将 Vec<Option<String>> 转换为 Vec<i64>
        let ids: Vec<i64> = ids_vec
            .into_iter()
            .filter_map(|s| s.and_then(|v| v.parse::<i64>().ok()))
            .collect();
        if ids.is_empty() {
            return Err(Error::from("删除的ID不能为空"));
        }
        let result = article_field_service::batch_delete(db, ids).await?;
        Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Err(Error::from("删除的ID不能为空"))
    }
}

/// GET /article/field/values/{article_id} - 查询文章的自定义字段值
pub async fn get_article_values(
    state: web::Data<AppState>,
    article_id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = article_field_service::get_article_values(db, article_id.into_inner()).await?;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::success(result, "local")))
}

/// POST /article/field/save_values - 批量保存文章字段值
pub async fn save_article_values(
    state: web::Data<AppState>,
    req: web::Json<ArticleFieldValueBatchDTO>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = article_field_service::save_article_values(db, req.into_inner()).await?;
    Ok(HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(MetaResp::<String>::success_with_msg(
            result.to_string(),
            "保存成功",
            "local",
        )))
}

// ==================== 路由注册（单点维护）====================

/// 注册文章自定义字段模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/article/field")
            // GET /article/field/list - 分页查询字段定义
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("website:article:field:view")),
            )
            // GET /article/field/detail/{id} - 字段定义详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_id)
                    .wrap(require_permission("website:article:field:view")),
            )
            // GET /article/field/by_category/{category_id} - 按栏目查询全部字段
            .route(
                "/by_category/{category_id}",
                web::get()
                    .to(get_by_category)
                    .wrap(require_permission("website:article:field:view")),
            )
            // POST /article/field/add - 新增字段定义
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("website:article:field:add")),
            )
            // PUT /article/field/update/{id} - 更新字段定义
            .route(
                "/update/{id}",
                web::put()
                    .to(update)
                    .wrap(require_permission("website:article:field:update")),
            )
            // DELETE /article/field/batch_delete - 批量删除
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("website:article:field:delete")),
            )
            // GET /article/field/values/{article_id} - 查询文章字段值
            .route(
                "/values/{article_id}",
                web::get()
                    .to(get_article_values)
                    .wrap(require_permission("website:article:field:view")),
            )
            // POST /article/field/save_values - 批量保存文章字段值
            .route(
                "/save_values",
                web::post()
                    .to(save_article_values)
                    .wrap(require_permission("website:article:field:update")),
            ),
    );
}
