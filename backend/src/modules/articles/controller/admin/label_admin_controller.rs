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
use crate::modules::articles::model::label::{LabelSaveRequest, LabelUpdateRequest, ListQuery};
use crate::modules::articles::service::label_service;
use crate::validate;

pub async fn add(
    state: web::Data<AppState>,
    req: web::Json<LabelSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = req.into_inner();

    validate!(payload.title.is_none(), "标签名称不能为空".to_string());
    if label_service::find_by_title_unique(&db, &payload.title, &None).await? {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标签名称已存在", "local")));
    }

    if let Some(short_url) = &payload.short_url {
        let re = regex::Regex::new(r"^[a-zA-Z0-9\-_]{5,}$").unwrap();
        if !re.is_match(short_url) {
            validate!(true, "短网址不得小于5个字符且只能包含数字、字母、-和_".to_string());
        }
        
        if label_service::find_by_short_url_unique(&db, &Some(short_url.clone()), &None).await? {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "短网址已存在", "local")));
        }
    }

    let result = label_service::insert(&db, &payload).await?;

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

        let result = label_service::batch_delete_by_ids(&db, &ids_vec).await?;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn update_by_id(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    req: web::Json<LabelUpdateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = req.into_inner();
    let label_id = Some(id.into_inner());
    validate!(payload.title.is_none(), "标签名称不能为空".to_string());
    if label_service::find_by_title_unique(&db, &payload.title, &label_id).await? {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标签名称已存在", "local")));
    }
    
    validate!(payload.short_url.is_none(), "短网址不能为空".to_string());
    let re = regex::Regex::new(r"^[a-zA-Z0-9\-_]{5,}$").unwrap();
    if !re.is_match(payload.short_url.clone().unwrap_or_default().as_str()) {
        validate!(true, "短网址不得小于5个字符且只能包含数字、字母、-和_".to_string());
    }
    if label_service::find_by_short_url_unique(&db, &payload.short_url, &payload.id).await? {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "短网址已存在", "local")));
    }
    let result = label_service::update_by_id(&db, payload).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("修改成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "修改失败", "local")))
    }
}

pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    match label_service::get_by_detail(&db, &item.id).await {
        Ok(post) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(post, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

pub async fn get_by_page(state: web::Data<AppState>, _req: HttpRequest, query: web::Query<ListQuery>,) -> Result<HttpResponse> {
    let db = &state.db;

    label_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

// ==================== 路由注册（单点维护）====================

/// 注册标签模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(label_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/label")
            // POST /label/add - 新建标签
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("system:label:add")),
            )
            // DELETE /label/batch_delete - 批量删除标签
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("system:label:delete")),
            )
            // PUT /label/update/{id} - 修改标签
            .route(
                "/update/{id}",
                web::put()
                    .to(update_by_id)
                    .wrap(require_permission("system:label:update")),
            )
            // GET /label/detail/{id} - 标签详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("system:label:view")),
            )
            // GET /label/list - 标签列表
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("system:label:list")),
            ),
    );
}