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
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::{MetaResp};
use crate::modules::website::model::template_user_data::{ListQuery, TemplateDataSaveDTO, TemplateDataSaveRequest, TemplateDataUpdateRequest};
use crate::modules::website::service::{website_service, template_user_data_service, template_service};
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::modules::website::model::template::MyListQuery;

#[post("/my_template/add")]
#[protect("my:template:add")]
pub async fn add(state: web::Data<AppState>, req: HttpRequest, item: web::Json<TemplateDataSaveRequest>) -> crate::core::errors::error::Result<HttpResponse> {
    let db = &state.db;

    let website_id = req.headers()
        .get("website_id")
        .and_then(|value| value.to_str().ok())
        .and_then(|id_str| id_str.parse::<i64>().ok());

    let website = website_service::find_by_id(db, &website_id).await?;
    let mut form_data = TemplateDataSaveDTO::from(item.into_inner());
    form_data.template_id = website.template_id;
    let result = template_user_data_service::insert(&db, &form_data).await?;
    Ok(HttpResponse::Ok().json((result)))
}

#[delete("/my_template/batch_delete")]
#[protect("my:template:delete")]
pub async fn batch_delete(state: web::Data<AppState>, _req: HttpRequest, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let result = template_user_data_service::batch_delete_by_ids(&db, &ids_vec).await?;
        Ok(HttpResponse::Ok().json(&MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

#[put("/my_template/update/{id}")]
#[protect("my:template:update")]
pub async fn update_by_id(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, item: web::Json<TemplateDataUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let mut form_data = TemplateDataSaveDTO::from(item.into_inner());
    form_data.id = Some(id.into_inner());
    let website_id = req.headers()
        .get("website_id")
        .and_then(|value| value.to_str().ok())
        .and_then(|id_str| id_str.parse::<i64>().ok());
    form_data.template_id = website_service::find_by_id(db, &website_id).await?.template_id;
    let result = template_user_data_service::update_by_id(&db, &form_data).await?;
    Ok(HttpResponse::Ok().json((result)))
}

#[get("/my_template/path_tree/{id}")]
pub async fn get_by_tree(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Path<InfoId>
) -> Result<HttpResponse> {
    let db = &state.db;

    // 尝试从查询参数中获取 website_id
    let mut website_id = query.id;

    // 如果查询参数中没有获取到，则从请求头中获取
    if website_id.is_none() {
        website_id = req.headers()
            .get("website_id")
            .and_then(|value| value.to_str().ok())
            .and_then(|id_str| id_str.parse::<i64>().ok());
    }

    // 验证 website_id 是否存在
    let website_id = website_id.ok_or_else(|| Error::from("website_id is required"))?;
    let site = website_service::find_by_id(db, &Some(website_id)).await?;
    template_user_data_service::find_by_template_id(&db, &site.template_id).await.map(|tree| {
        HttpResponse::Ok().json((tree))
    })
}


#[get("/my_template/detail/{id}")]
#[protect("my:template:view")]
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = template_user_data_service::get_by_detail(&db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

#[get("/my_template/list")]
#[protect("my:template:list")]
pub async fn get_by_page(state: web::Data<AppState>, req: HttpRequest, query: web::Query<ListQuery>,) -> Result<HttpResponse> {
    let db = &state.db;
    let mut form_data = query.0;
    let website_id = req.headers().get("website_id")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let site = website_service::find_by_id(db, &Some(website_id)).await?;
    form_data.template_id = site.template_id;
    form_data.website_id = site.id;
    template_user_data_service::get_by_page(&db, form_data).await.map(|page_data| {
        HttpResponse::Ok().json((page_data))
    })
}

/// 获取我的模版列表
#[get("/template/buy_list")]
#[protect("buy:template:list")]
pub async fn get_buy_by_page(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<MyListQuery>
) -> Result<HttpResponse> {
    let db = &state.db;

    let mut list_query = query.into_inner();
    //获取当前用户id
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    list_query.admin_id = admin_token.id;
    template_service::get_my_list_by_page(&db, list_query).await.map(|page_data| {
        HttpResponse::Ok().json((page_data))
    })
}