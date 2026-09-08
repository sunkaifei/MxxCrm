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
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::articles::model::comment::{CommentAdminUpdateRequest, CommentSaveRequest, ListQuery};
use crate::modules::articles::service::comment_service;
use crate::validate;

pub async fn add(
    state: web::Data<AppState>,
    req: web::Json<CommentSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = req.into_inner();

    validate!(payload.article_id.is_none(), "文章ID不能为空".to_string());
    validate!(payload.content.as_ref().map_or(true, |s| s.trim().is_empty()), "评论内容不能为空".to_string());

    let result = comment_service::insert(&db, &payload).await?;

    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("添加成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}

pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let result = comment_service::batch_delete_by_ids(&db, &ids_vec).await?;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// 审核评论
pub async fn audit(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    req: web::Json<CommentAdminUpdateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let comment_id = id.into_inner();
    let payload = req.into_inner();

    validate!(payload.status.is_none(), "审核状态不能为空".to_string());

    let result = comment_service::update_status(&db, comment_id, payload.status.unwrap_or_default()).await?;

    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("审核成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "审核失败，评论不存在", "local")))
    }
}

pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    match comment_service::get_by_detail(&db, &item.id).await {
        Ok(comment) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(comment, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

pub async fn get_by_page(state: web::Data<AppState>, _req: HttpRequest, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;

    comment_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

// ==================== 路由注册（单点维护）====================

/// 注册评论模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(comment_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/comment")
            // POST /comment/add - 新建评论
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("articles:comment:add")),
            )
            // DELETE /comment/batch_delete - 批量删除评论
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("articles:comment:delete")),
            )
            // PUT /comment/audit/{id} - 审核评论
            .route(
                "/audit/{id}",
                web::put()
                    .to(audit)
                    .wrap(require_permission("articles:comment:audit")),
            )
            // GET /comment/detail/{id} - 评论详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("articles:comment:view")),
            )
            // GET /comment/list - 评论列表
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("articles:comment:list")),
            ),
    );
}
