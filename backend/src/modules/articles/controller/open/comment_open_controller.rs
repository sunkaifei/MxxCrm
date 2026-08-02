//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{web, HttpResponse};
use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
use crate::modules::articles::model::comment::{CommentSaveRequest, ListQuery};
use crate::modules::articles::service::comment_service;
use crate::validate;

/// 提交评论（前台公开接口，写入 status=0 待审核）
///
/// POST /comment/submit
pub async fn submit(
    state: web::Data<AppState>,
    req: web::Json<CommentSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = req.into_inner();

    validate!(payload.article_id.is_none(), "文章ID不能为空".to_string());
    validate!(
        payload.content.as_ref().map_or(true, |s| s.trim().is_empty()),
        "评论内容不能为空".to_string()
    );

    let result = comment_service::insert(&db, &payload).await?;

    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("提交成功，待审核".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "提交失败", "local")))
    }
}

/// 按文章查询评论（前台公开接口，仅返回已审核 status=1 的评论）
///
/// GET /article/{article_id}/comments
pub async fn list_by_article(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let article_id = path.into_inner();
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let page_data = comment_service::get_by_article(&db, article_id, page, page_size).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local")))
}

// ==================== 路由注册（单点维护）====================

/// 注册评论模块公开路由
///
/// 调用方在 `open_routes.rs` 中通过 `cfg.configure(comment_open_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/comment")
            // POST /comment/submit - 提交评论
            .route("/submit", web::post().to(submit)),
    );
    // GET /article/{article_id}/comments - 按文章查询评论（路径以 /article 开头，独立于 /comment scope）
    cfg.service(
        web::resource("/article/{article_id}/comments").route(web::get().to(list_by_article)),
    );
}
