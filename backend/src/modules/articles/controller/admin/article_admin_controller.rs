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
use crate::core::web::response::{MetaResp};
use crate::modules::articles::model::article::{ArticleModel, ArticlesSaveDTO, ArticlesSaveRequest, ArticlesUpdateRequest, QueryPageRequest, QueryTitleUnique};
use crate::modules::articles::service::article_service;
use crate::modules::articles::service::article_service::find_by_title_unique;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use crate::validate;

pub async fn save_article(state: web::Data<AppState>, req: HttpRequest, item: web::Json<ArticlesSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    validate!(payload.title.is_none(), t!("article.index.title_empty", locale = "zh-CN").to_string());
    validate!(payload.category_id.is_none(), t!("article.index.category_id_empty", locale = "zh-CN").to_string());
    validate!(payload.content.is_none(), t!("article.index.content_empty", locale = "zh-CN").to_string());


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
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "文章标题已存在", "local")));
    }

    let result = article_service::save_article(&db, itme_dto).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
    }else{
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "文章发布失败", "local")))
    }
}

pub async fn batch_delete(state: web::Data<AppState>, req: HttpRequest, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());
        let result = ArticleModel::batch_delete_by_ids(&db, website_id.unwrap_or_default(), ids).await?;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn update_article_detail(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, item: web::Json<ArticlesUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();

    let mut article_data = ArticlesSaveDTO::from(item);
    article_data.id = Some(id.into_inner());
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
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "文章标题已存在", "local")));
    }

    let result = article_service::update_by_id(&db, article_data).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))

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
                Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "文章不存在", "local")))
            }
            Some(article) => {
                Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(article, "local")))
            }
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}



pub async fn get_article_list(state: web::Data<AppState>, req: HttpRequest, item: web::Query<QueryPageRequest>) -> Result<HttpResponse> {
    let mut payload = item.0;
    let db = &state.db;
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    payload.website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());
    let result = article_service::get_by_page(&db, payload).await?;

    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
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
                    .wrap(require_permission("content:article:add")),
            )
            // DELETE /article/batch_delete - 批量删除文章
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("content:article:delete")),
            )
            // PUT /article/update/{id} - 修改文章
            .route(
                "/update/{id}",
                web::put()
                    .to(update_article_detail)
                    .wrap(require_permission("content:article:update")),
            )
            // GET /article/detail/{id} - 文章详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_article_detail)
                    .wrap(require_permission("content:article:view")),
            )
            // GET /article/list - 文章列表
            .route(
                "/list",
                web::get()
                    .to(get_article_list)
                    .wrap(require_permission("content:article:list")),
            ),
    );
}
