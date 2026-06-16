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
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::{MetaResp};
use crate::modules::website::model::website_links::{LinkSaveDTO, LinkSaveRequest, LinkUpdateRequest, ListQuery};
use crate::modules::website::service::{website_links_service};


#[post("/links/add")]
#[protect("website:links:add")]
pub async fn add_links(state: web::Data<AppState>, req: HttpRequest, item: web::Json<LinkSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.0;
    if payload.link_name.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "名称不能为空", "local")));
    }
    if payload.link_url.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "链接地址不能为空", "local")));
    }
    if website_links_service::find_by_link_url_unique(&db, &payload.link_url, &None).await?{
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "链接地址已存在", "local")));
    }
    let mut form_data = LinkSaveDTO::from(payload);
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    form_data.website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());
    let result = website_links_service::insert(&db, form_data).await?;

    if result > 0 {
        Ok(HttpResponse::Ok().json(("添加成功")))
    } else {
        Ok(HttpResponse::Ok().json(("添加失败".to_string())))
    }
}

#[delete("/links/batch_delete")]
#[protect("website:links:delete")]
pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        for id_opt in ids_vec.iter() {
            if let Some(id) = id_opt {
                if id == "1" {
                    return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "含有不能删除的超级管理员账户", "local")));
                }
            }
        }

        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let result = website_links_service::batch_delete_by_ids(&db, &ids_vec).await;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

#[put("/links/update/{id}")]
#[protect("website:links:update")]
pub async fn update_by_id(state: web::Data<AppState>, id: web::Path<i64>, item: web::Json<LinkUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.0;
    let link_id = Some(id.into_inner());
    if payload.link_name.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "名称不能为空", "local")));
    }
    if payload.link_url.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "链接地址不能为空", "local")));
    }
    if website_links_service::find_by_link_url_unique(&db, &payload.link_url, &link_id).await?{
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "链接地址已存在", "local")));
    }
    let mut form_data = LinkSaveDTO::from(payload);
    form_data.id = link_id;
    let result = website_links_service::update_by_id(&db, &form_data).await?;

    if result > 0 {
        Ok(HttpResponse::Ok().json(("更新成功")))
    } else {
        Ok(HttpResponse::Ok().json(("更新失败".to_string())))
    }
}
#[get("/links/detail/{id}")]
#[protect("website:links:find")]
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = website_links_service::get_by_detail(&db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

#[get("/links/list")]
#[protect("website:links:list")]
pub async fn get_by_page(state: web::Data<AppState>, req: HttpRequest, query: web::Query<ListQuery>,) -> Result<HttpResponse> {
    let db = &state.db;
    let mut form_data = query.0;
    let website_id = req.headers().get("website_id")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    form_data.website_id = Some(website_id);
    website_links_service::get_by_page(&db, form_data).await.map(|page_data| {
        HttpResponse::Ok().json((page_data))
    })
}
