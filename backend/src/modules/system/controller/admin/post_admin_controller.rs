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
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::system::model::post::{ListQuery, PostSaveRequest, PostUpdateRequest};
use crate::modules::system::service::post_service;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

#[post("/post/save")]
#[protect("system:post:save")]
pub async fn save_post(state: web::Data<AppState>, _req: HttpRequest, item: web::Json<PostSaveRequest>) -> HttpResponse {
    let db = &state.db;
    if item.post_name.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "岗位名称不能为空", "local"));
    }
    if post_service::find_by_post_name_unique(&db, &item.post_name, &None).await.unwrap_or_default() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "岗位名称已存在", "local"));
    }
    let result = post_service::save(&db, &item.0).await;
    match result {
        Ok(v) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::success(v, "local"))
        }
        Err(err) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local"))
        }
    }
}

#[delete("/post/bath_delete")]
#[protect("system:post:delete")]
pub async fn bath_delete_post(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local"))
        } else {
            let result = post_service::batch_delete_by_ids(&db, &ids_vec).await;
            match result {
                Ok(v) => {
                    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::success(v, "local"))
                }
                Err(err) => {
                    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local"))
                }
            }
        }
    }else {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local"))
    }
}

#[put("/post/update/{id}")]
#[protect("system:post:update")]
pub async fn update_post(state: web::Data<AppState>, _req: HttpRequest, id: web::Path<i64>, item: web::Json<PostUpdateRequest>) -> HttpResponse {
    let db = &state.db;
    let mut data = item.into_inner();
    let the_id = id.into_inner();
    if data.post_name.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "岗位名称不能为空", "local"));
    }
    if post_service::find_by_post_name_unique(&db, &data.post_name, &Some(the_id)).await.unwrap_or_default() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "岗位名称已存在", "local"));
    }
    data.id = Some(the_id);
    let result = post_service::update_by_id(&db, &data).await;
    match result {
        Ok(v) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::success(v, "local"))
        }
        Err(err) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local"))
        }
    }
}

#[get("/post/detail/{id}")]
#[protect("system:post:view")]
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    match post_service::get_by_detail(&db, &item.id).await {
        Ok(post) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(post, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

#[get("/post/list")]
#[protect("system:post:list")]
pub async fn get_by_page(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    post_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

#[get("/post/options")]
pub async fn post_options(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    post_service::get_post_options(db).await.map(|list_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(list_data, "local"))
    })
}