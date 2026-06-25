//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.!
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use actix_web::{delete, get, HttpResponse, post, put, web, HttpRequest};
use actix_web_grants::protect;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::system::model::dict_data::{DataListQuery, DictDataSaveDTO, DictDataSaveRequest, DictDataUpdateRequest};
use crate::modules::system::model::dict::{DictSaveDTO, DictSaveRequest, DictUpdateRequest, TypeListQuery};
use crate::modules::system::service::{admin_service, dict_service};

#[post("/dict/add")]
#[protect("system:dict:save")]
pub async fn save_dict(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<DictSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let dict_request = payload.0;
    if dict_request.dict_name.as_ref().map_or(true, |dict_name| dict_name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典名称不能为空", "local")));
    }
    if dict_service::find_by_name_unique(&db, &dict_request.dict_name,&None).await? {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典名称已存在", "local")));
    }
    if dict_request.dict_code.as_ref().map_or(true, |dict_code| dict_code.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典编码不能为空", "local")));
    }
    if dict_service::find_by_code_unique(&db, &dict_request.dict_code,&None).await? {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典编码已存在", "local")));
    }
    //获取用户信息
    let jwt_token:JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;
    let mut form_data = DictSaveDTO::from(dict_request);
    form_data.create_by = admin.user_name.clone();
    form_data.update_by = admin.user_name;
    match dict_service::insert(&db, &form_data).await {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local")))
        }
        Err(e) => {
            log::error!("添加字典出错：{:}",e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[post("/dict/data/save")]
pub async fn save_dict_data(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<DictDataSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let data_request = payload.0;
    if data_request.dict_label.as_ref().map_or(true, |dict_label| dict_label.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典标签名称不能为空", "local")));
    }
    if dict_service::find_data_by_label_unique(&db, &data_request.dict_code, &data_request.dict_label, &None).await? {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典标签名称已存在", "local")));
    }
    if data_request.dict_value.as_ref().map_or(true, |dict_value| dict_value.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典值不能为空", "local")));
    }
    //获取用户信息
    let jwt_token:JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;
    let mut form_data = DictDataSaveDTO::from(data_request);
    form_data.create_by = admin.user_name;
    match dict_service::insert_data(&db, &form_data).await {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local")))
        }
        Err(e) => {
            log::error!("添加字典出错：{:}",e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[delete("/dict/batch_delete")]
#[protect("system:dict:delete")]
pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let result = dict_service::batch_delete_by_ids(&db, ids_vec).await;
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
        }
    }else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

#[delete("/dict/data/batch_delete")]
pub async fn batch_delete_data(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let result = dict_service::batch_delete_data_by_ids(&db, ids_vec).await;
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
        }
    }else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

#[put("/dict/update/{id}")]
#[protect("system:dict:update")]
pub async fn update_dict(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, item: web::Json<DictUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.dict_name.as_ref().map_or(true, |dict_name| dict_name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典名称不能为空", "local")));
    }
    let dict_id = Some(id.into_inner());
    if dict_service::find_by_name_unique(&db, &item.dict_name, &dict_id).await? {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典名称已存在", "local")));
    }
    if item.dict_code.as_ref().map_or(true, |dict_code| dict_code.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典编码不能为空", "local")));
    }
    if dict_service::find_by_code_unique(&db, &item.dict_code, &dict_id).await? {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典编码已存在", "local")));
    }

    //获取用户信息
    let jwt_token:JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;

    let mut form_data = DictSaveDTO::from(item.0);
    form_data.id = dict_id;
    form_data.update_by = admin.user_name;
    match dict_service::update_by_id(&db, &form_data).await {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local")))
        }
        Err(e) => {
            log::error!("更新字典出错：{:}",e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

/// 更新字典数据
#[put("/dict/data/update/{id}")]
pub async fn update_dict_data(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, item: web::Json<DictDataUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let dict_data_id = Some(id.into_inner());
    if item.dict_label.as_ref().map_or(true, |dict_label| dict_label.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典标签名称不能为空", "local")));
    }
    if dict_service::find_data_by_label_unique(&db, &item.dict_code, &item.dict_label, &dict_data_id).await? {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典标签名称已存在", "local")));
    }
    if item.dict_value.as_ref().map_or(true, |dict_value| dict_value.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典值不能为空", "local")));
    }

    //获取用户信息
    let jwt_token:JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;

    let mut form_data = DictDataSaveDTO::from(item.0);
    form_data.id = dict_data_id;
    form_data.update_by = admin.user_name;
    match dict_service::update_data_by_id(&db, &form_data).await{
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local")))
        }
        Err(e) => {
            log::error!("更新字典出错：{:}",e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

/// 获取字典类型详情
#[get("/dict/{id}")]
#[protect("system:dict:view")]
pub async fn get_dict_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    match dict_service::get_by_id(&db, &item.id).await {
        Ok(dict_type) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(dict_type, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

/// 获取字典数据详情
#[get("/dict/data/{id}")]
#[protect("dict:data:detail:view")]
pub async fn get_dict_data_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    match dict_service::get_data_by_id(&db, &item.id).await {
        Ok(dict_data) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(dict_data, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

#[get("/dict/data/{dict_code}/options")]
pub async fn get_dict_data_list_by_code(state: web::Data<AppState>, dict_code: web::Path<String>) -> Result<HttpResponse> {
    let db = &state.db;
    let dict_code = dict_code.into_inner();
    if dict_code.is_empty() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "字典编码不能为空", "local")));
    }
    dict_service::get_dict_data_list_by_code(&db, &Some(dict_code)).await.map(|dict_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(dict_data, "local"))
    })
}

#[get("/dict/list")]
#[protect("system:dict:list")]
pub async fn get_dict_page(state: web::Data<AppState>, query: web::Query<TypeListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    dict_service::get_dict_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

#[get("/dict/data/list")]
#[protect("dict:data:list")]
pub async fn get_dict_data_list(state: web::Data<AppState>, query: web::Query<DataListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    dict_service::get_dict_data_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}
