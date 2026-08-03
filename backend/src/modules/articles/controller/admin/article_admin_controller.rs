//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.!
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究!
//!



use crate::core::errors::error::{Error, Result};
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, BathIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::articles::model::article::{ArticleModel, ArticlesSaveDTO, ArticlesSaveRequest, ArticlesUpdateRequest, QueryPageRequest, QueryTitleUnique};
use crate::modules::articles::model::article_revision::{ArticleRevisionModel, RevisionListQuery};
use crate::modules::articles::service::article_label_service;
use crate::modules::articles::service::article_service;
use crate::modules::articles::service::article_service::find_by_title_unique;
use crate::core::kit::global::Deserialize;
use crate::core::kit::global::Serialize;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use crate::validate;

pub async fn save_article(state: web::Data<AppState>, req: HttpRequest, item: web::Json<ArticlesSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    validate!(payload.title.is_none(), t!("article.index.title_empty", locale = "zh-CN").to_string());
    validate!(payload.category_id.is_none(), t!("article.index.category_id_empty", locale = "zh-CN").to_string());
    validate!(payload.content.is_none(), t!("article.index.content_empty", locale = "zh-CN").to_string());

    // 提前取出标签ID，payload 后续会被消费转换
    let label_ids = payload.label_ids.clone();

    let mut itme_dto = ArticlesSaveDTO::from(payload);
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    itme_dto.user_id = admin_token.id;
    itme_dto.website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());
    itme_dto.istop = Some(0);
    itme_dto.isrecommend = Some(0);

    let unique = QueryTitleUnique{
        id: None,
        title: itme_dto.title.clone(),
        website_id: itme_dto.website_id.clone(),
    };
    let unique_num = find_by_title_unique(&db, &unique).await?;
    if unique_num {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "文章标题已存在", "local")));
    }

    let result = article_service::save_article(&db, itme_dto).await?;
    if result > 0 {
        // 文章保存成功后，同步设置标签关联（传入则更新，未传入则跳过）
        if let Some(ids) = label_ids {
            let _ = article_label_service::set_labels(&db, result, ids).await?;
        }
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))
    }else{
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "文章发布失败", "local")))
    }
}

pub async fn batch_delete(state: web::Data<AppState>, req: HttpRequest, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());
        let result = ArticleModel::batch_delete_by_ids(&db, website_id.unwrap_or_default(), ids).await?;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn update_article_detail(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, item: web::Json<ArticlesUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();

    // 提前取出标签ID，item 后续会被消费转换
    let label_ids = item.label_ids.clone();
    let article_id_value = id.into_inner();

    let mut article_data = ArticlesSaveDTO::from(item);
    article_data.id = Some(article_id_value);
    article_data.user_id = admin_token.id;

    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    article_data.website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());


    let unique = QueryTitleUnique{
        id: article_data.id,
        title: article_data.title.clone(),
        website_id: article_data.website_id.clone(),
    };
    let unique_num = find_by_title_unique(&db, &unique).await?;
    if unique_num {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "文章标题已存在", "local")));
    }

    let result = article_service::update_by_id(&db, article_data).await?;
    // 文章更新成功后，同步设置标签关联（传入则更新，未传入则跳过）
    if result > 0 {
        if let Some(ids) = label_ids {
            let _ = article_label_service::set_labels(&db, article_id_value, ids).await?;
        }
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))

}

pub async fn get_article_detail(state: web::Data<AppState>, req: HttpRequest,item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = item.id;
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    let website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());

    let result = ArticleModel::find_by_id(db,website_id.unwrap_or_default(),id.unwrap_or_default()).await;
    match result {
        Ok(article_op) => match article_op {
            None => {
                Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "文章不存在", "local")))
            }
            Some(article) => {
                Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(article, "local")))
            }
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}



pub async fn get_article_list(state: web::Data<AppState>, req: HttpRequest, item: web::Query<QueryPageRequest>) -> Result<HttpResponse> {
    let mut payload = item.0;
    let db = &state.db;
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    payload.website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());
    let result = article_service::get_by_page(&db, payload).await?;

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

// ==================== 文章修订历史 ====================

/// GET /article/revisions/{article_id} - 文章修订历史列表（分页）
pub async fn list_revisions(
    state: web::Data<AppState>,
    article_id: web::Path<i64>,
    query: web::Query<RevisionListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let article_id = article_id.into_inner();
    let page = query.page.unwrap_or(1) as i64;
    let page_size = query.page_size.unwrap_or(20) as i64;

    let result = article_service::list_revisions(db, article_id, page, page_size).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

/// GET /article/revision/detail/{id} - 修订记录详情
pub async fn get_revision_detail(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match article_service::get_revision_detail(db, id.into_inner()).await {
        Ok(vo) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(vo, "local")))
        }
        Err(e) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")))
        }
    }
}

/// POST /article/revision/restore/{id} - 恢复到指定修订版本
pub async fn restore_revision(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let admin_token: JWTToken = get_user(&req).unwrap_or_default();
    let editor_id = admin_token.id.unwrap_or(0);
    let editor_name = admin_token.username.unwrap_or_else(|| "".to_string());
    let revision_id = id.into_inner();

    // 先取出修订记录以获得 article_id
    let revision = match ArticleRevisionModel::find_by_id(db, revision_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "修订记录不存在", "local")));
        }
        Err(e) => {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")));
        }
    };

    let article_id = revision.article_id.unwrap_or(0);
    if article_id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "修订记录缺少文章ID", "local")));
    }

    match article_service::restore_revision(db, article_id, revision_id, editor_id, editor_name).await {
        Ok(rows) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(rows, "local")))
        }
        Err(e) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")))
        }
    }
}

// ==================== 文章-标签关联 ====================

/// GET /article/labels/{article_id} - 获取文章的标签ID列表
pub async fn get_article_labels(
    state: web::Data<AppState>,
    article_id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let article_id = article_id.into_inner();
    match article_label_service::get_labels_by_article(db, article_id).await {
        Ok(label_ids) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(label_ids, "local")))
        }
        Err(e) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")))
        }
    }
}

// ==================== G-1.9: 文章批量操作 ====================

/// 批量审核请求体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BatchAuditRequest {
    pub ids: Vec<String>,
    /// 状态：1待审核 2已发布 3草稿 4驳回
    pub status: i32,
}

/// 批量置顶请求体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BatchTopRequest {
    pub ids: Vec<String>,
    /// 1置顶 0取消置顶
    pub istop: i32,
}

/// 批量移动分类请求体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BatchMoveCategoryRequest {
    pub ids: Vec<String>,
    pub category_id: i64,
}

/// 批量设置推荐请求体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BatchRecommendRequest {
    pub ids: Vec<String>,
    /// 1推荐 0取消推荐
    pub isrecommend: i32,
}

/// PUT /article/batch_audit - 批量审核文章
pub async fn batch_audit(
    state: web::Data<AppState>,
    req: web::Json<BatchAuditRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = req.into_inner();
    let ids: Vec<i64> = payload.ids.iter().filter_map(|s| s.parse::<i64>().ok()).collect();
    let result = article_service::batch_audit(db, ids, payload.status).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(Ok(result))))
}

/// PUT /article/batch_top - 批量置顶/取消置顶
pub async fn batch_top(
    state: web::Data<AppState>,
    req: web::Json<BatchTopRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = req.into_inner();
    let ids: Vec<i64> = payload.ids.iter().filter_map(|s| s.parse::<i64>().ok()).collect();
    let result = article_service::batch_set_top(db, ids, payload.istop).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(Ok(result))))
}

/// PUT /article/batch_move_category - 批量移动分类
pub async fn batch_move_category(
    state: web::Data<AppState>,
    req: web::Json<BatchMoveCategoryRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = req.into_inner();
    let ids: Vec<i64> = payload.ids.iter().filter_map(|s| s.parse::<i64>().ok()).collect();
    let result = article_service::batch_move_category(db, ids, payload.category_id).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(Ok(result))))
}

/// PUT /article/batch_recommend - 批量设置推荐
pub async fn batch_recommend(
    state: web::Data<AppState>,
    req: web::Json<BatchRecommendRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = req.into_inner();
    let ids: Vec<i64> = payload.ids.iter().filter_map(|s| s.parse::<i64>().ok()).collect();
    let result = article_service::batch_set_recommend(db, ids, payload.isrecommend).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(Ok(result))))
}

// ==================== 路由注册（单点维护）====================

/// 注册文章模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(article_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/article")
            // POST /article/save - 新建文章
            .route(
                "/save",
                web::post()
                    .to(save_article)
                    .wrap(require_permission("website:article:add")),
            )
            // DELETE /article/batch_delete - 批量删除文章
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("website:article:delete")),
            )
            // PUT /article/update/{id} - 修改文章
            .route(
                "/update/{id}",
                web::put()
                    .to(update_article_detail)
                    .wrap(require_permission("website:article:update")),
            )
            // GET /article/detail/{id} - 文章详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_article_detail)
                    .wrap(require_permission("website:article:view")),
            )
            // GET /article/list - 文章列表
            .route(
                "/list",
                web::get()
                    .to(get_article_list)
                    .wrap(require_permission("website:article:list")),
            )
            // GET /article/labels/{article_id} - 获取文章的标签ID列表
            .route(
                "/labels/{article_id}",
                web::get()
                    .to(get_article_labels)
                    .wrap(require_permission("website:article:view")),
            )
            // GET /article/revisions/{article_id} - 文章修订历史列表
            .route(
                "/revisions/{article_id}",
                web::get()
                    .to(list_revisions)
                    .wrap(require_permission("website:article:revision")),
            )
            // GET /article/revision/detail/{id} - 修订记录详情
            .route(
                "/revision/detail/{id}",
                web::get()
                    .to(get_revision_detail)
                    .wrap(require_permission("website:article:revision")),
            )
            // POST /article/revision/restore/{id} - 恢复到指定修订版本
            .route(
                "/revision/restore/{id}",
                web::post()
                    .to(restore_revision)
                    .wrap(require_permission("website:article:revision_restore")),
            )
            // G-1.9: PUT /article/batch_audit - 批量审核文章
            .route(
                "/batch_audit",
                web::put()
                    .to(batch_audit)
                    .wrap(require_permission("website:article:update")),
            )
            // G-1.9: PUT /article/batch_top - 批量置顶/取消置顶
            .route(
                "/batch_top",
                web::put()
                    .to(batch_top)
                    .wrap(require_permission("website:article:update")),
            )
            // G-1.9: PUT /article/batch_move_category - 批量移动分类
            .route(
                "/batch_move_category",
                web::put()
                    .to(batch_move_category)
                    .wrap(require_permission("website:article:update")),
            )
            // G-1.9: PUT /article/batch_recommend - 批量设置推荐
            .route(
                "/batch_recommend",
                web::put()
                    .to(batch_recommend)
                    .wrap(require_permission("website:article:update")),
            ),
    );
}
