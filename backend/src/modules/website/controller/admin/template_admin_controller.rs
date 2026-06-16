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
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::{MetaResp};
use crate::modules::website::model::template::{ListQuery, TemplateSaveDTO, TemplateSaveRequest, TemplateUpdateRequest};
use crate::modules::website::service::{website_service, template_service, template_user_data_service};
use crate::validate;

#[post("/template/add")]
#[protect("system:template:add")]
pub async fn add(state: web::Data<AppState>, req: HttpRequest, item: web::Json<TemplateSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    validate!(payload.name.is_none(), t!("website.template.name_empty", locale = "zh-CN").to_string());
    if template_service::find_by_name_unique(&db, &payload.name, &None).await? > 0 {
        validate!(true, t!("website.template.name_exists", locale = "zh-CN").to_string());
    }
    let mut form_data = TemplateSaveDTO::from(payload);

    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    form_data.user_id = admin_token.id;

    let result =  template_service::insert(&db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
    }else{
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "文章发布失败", "local")))
    }
}

#[delete("/template/batch_delete")]
#[protect("system:template:delete")]
pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let result = template_service::batch_delete_by_ids(&db, &ids_vec).await?;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

#[post("/template/update")]
#[protect("system:template:update")]
pub async fn update_by_id(state: web::Data<AppState>, req: HttpRequest, item: web::Json<TemplateUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    validate!(payload.id.is_none(), t!("website.template.id_empty", locale = "zh-CN").to_string());
    validate!(payload.name.is_none(), t!("website.template.name_empty", locale = "zh-CN").to_string());
    if template_service::find_by_name_unique(&db, &payload.name, &payload.id).await? > 0 {
        validate!(true, t!("website.template.name_exists", locale = "zh-CN").to_string());
    }
    let mut form_data = TemplateSaveDTO::from(payload);

    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    form_data.user_id = admin_token.id;

    let result = template_service::update_by_id(&db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
    }else{
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "文章修改失败", "local")))
    }
}

#[get("/template/detail/{id}")]
#[protect("system:template:view")]
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = template_service::get_by_detail(&db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

#[get("/template/common_options")]
pub async fn get_by_options(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = template_service::select_by_iscommon(db, &Some(1)).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

#[get("/template/list")]
#[protect("system:template:list")]
pub async fn get_by_page(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    template_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}
