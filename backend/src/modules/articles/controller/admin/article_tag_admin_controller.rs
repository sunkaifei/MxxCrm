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
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::articles::model::article_tag::{ArticleTagSaveDTO, ArticleTagSaveRequest, ArticleTagUpdateRequest, ListQuery};
use crate::modules::articles::service::{article_tag_service};

/// 新增文章标签
pub async fn add(state: web::Data<AppState>, _req: HttpRequest, item: web::Json<ArticleTagSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    if payload.name.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标签名称不能为空", "local")));
    }
    let form_data = ArticleTagSaveDTO::from(payload);
    let result = article_tag_service::insert(db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("添加成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}

/// 批量删除文章标签
pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let result = article_tag_service::batch_delete_by_ids(db, &ids_vec).await?;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// 修改文章标签
pub async fn update_by_id(state: web::Data<AppState>, _req: HttpRequest, id: web::Path<i64>, item: web::Json<ArticleTagUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    let mut form_data = ArticleTagSaveDTO::from(payload);
    form_data.id = Some(id.into_inner());
    if form_data.name.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标签名称不能为空", "local")));
    }
    let result = article_tag_service::update_by_id(db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("修改成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "修改失败", "local")))
    }
}

/// 文章标签详情
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = article_tag_service::get_by_detail(db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

/// 文章标签分页列表
pub async fn get_by_page(state: web::Data<AppState>, _req: HttpRequest, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    article_tag_service::get_by_page(db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

/// 获取所有启用的文章标签
pub async fn get_all(state: web::Data<AppState>, _req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let result = article_tag_service::get_all(db).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

// ==================== 路由注册（单点维护）====================

/// 注册文章标签模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/article/tag")
            // POST /article/tag/add - 新增文章标签
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("website:article:tag:add")),
            )
            // DELETE /article/tag/batch_delete - 批量删除文章标签
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("website:article:tag:delete")),
            )
            // PUT /article/tag/update/{id} - 修改文章标签
            .route(
                "/update/{id}",
                web::put()
                    .to(update_by_id)
                    .wrap(require_permission("website:article:tag:update")),
            )
            // GET /article/tag/detail/{id} - 文章标签详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("website:article:tag:view")),
            )
            // GET /article/tag/list - 文章标签分页
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("website:article:tag:list")),
            )
            // GET /article/tag/all - 获取所有启用的文章标签
            .route(
                "/all",
                web::get()
                    .to(get_all)
                    .wrap(require_permission("website:article:tag:list")),
            ),
    );
}