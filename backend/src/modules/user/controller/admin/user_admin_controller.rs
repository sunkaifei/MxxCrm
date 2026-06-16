//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

extern crate bcrypt;

use crate::core::errors::error::{Error, Result};
use actix_web::{delete, get, HttpRequest, HttpResponse, post, put, web};
use actix_web_grants::protect;
use bcrypt::{DEFAULT_COST, hash};

use crate::core::kit::global::AppState;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::user::model::user::{ListQuery, UserSaveDTO, UserSaveRequest, UserUpdateRequest};
use crate::modules::user::service::user_service;

// 添加用户信息
#[post("/admin/user/add")]
pub async fn save_user(state: web::Data<AppState>, item: web::Json<UserSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.username.as_ref().map_or(true, |username| username.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户名称不能为空", "local")));
    }
    if item.password.as_ref().map_or(true, |password| password.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "密码不能为空", "local")));
    }
    
    let hashed_password = hash(item.password.clone().unwrap_or_default(), DEFAULT_COST)
        .map_err(|_| Error::from("密码哈希失败"))?;
    
    let dto_data = UserSaveDTO {
        id: None,
        username: item.username.clone(),
        nickname: item.nickname.clone(),
        avatar: item.avatar.clone(),
        email: item.email.clone(),
        mobile: item.mobile.clone(),
        loginfailure: Some(0),
        lastlogintime: None,
        lastloginip: None,
        password: Some(hashed_password),
        salt: None,
        motto: item.motto.clone(),
        status: Some("1".to_string()),
    };
    
    let result = user_service::insert(db, &dto_data).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

// 删除用户信息
#[delete("/admin/user/batch_delete")]
pub async fn user_batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let result = user_service::batch_delete_by_ids(db, &ids_vec).await;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

// 更新用户信息
#[put("/admin/user/update/{id}")]
pub async fn user_update(state: web::Data<AppState>, id: web::Path<i64>, item: web::Json<UserUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    let user_id = id.into_inner();
    
    // 检查用户是否存在
    let user_result = user_service::get_by_detail(db, &Some(user_id)).await;
    if user_result.is_err() || user_result.as_ref().unwrap().id == Some(0) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户信息不存在", "local")));
    }
    
    let mut dto_data = UserSaveDTO {
        id: Some(user_id),
        username: item.username.clone(),
        nickname: item.nickname.clone(),
        avatar: item.avatar.clone(),
        email: item.email.clone(),
        mobile: item.mobile.clone(),
        loginfailure: None,
        lastlogintime: None,
        lastloginip: None,
        password: None,
        salt: None,
        motto: item.motto.clone(),
        status: item.status.clone(),
    };
    
    // 如果密码有更新，重新哈希
    if item.password.is_some() && !item.password.clone().unwrap_or_default().is_empty() {
        let hashed_password = hash(item.password.clone().unwrap_or_default(), DEFAULT_COST)
            .map_err(|_| Error::from("密码哈希失败"))?;
        dto_data.password = Some(hashed_password);
    }
    
    let result = user_service::update_by_id(db, &dto_data).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

// 查询用户详情
#[get("/admin/user/detail/{id}")]
pub async fn get_user_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户id不能为空", "local")));
    }
    
    let user_result = user_service::get_by_detail(db, &item.id).await;
    if user_result.is_err() || user_result.as_ref().unwrap().id == Some(0) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户不存在", "local")));
    }
    
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(user_result.unwrap(), "local")))
}

// 查询用户列表
#[get("/admin/user/list")]
#[protect("user:list:show")]
pub async fn user_list(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    user_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

// 更新用户状态
#[put("/admin/user/update-status")]
pub async fn update_user_status(state: web::Data<AppState>, item: web::Json<UserUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户id不能为空", "local")));
    }
    
    let dto_data = UserSaveDTO {
        id: item.id,
        username: None,
        nickname: None,
        avatar: None,
        email: None,
        mobile: None,
        loginfailure: None,
        lastlogintime: None,
        lastloginip: None,
        password: None,
        salt: None,
        motto: None,
        status: item.status.clone(),
    };
    
    let result = user_service::update_by_id(db, &dto_data).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

// 更新用户密码
#[put("/admin/user/update_password")]
pub async fn update_password(
    state: web::Data<AppState>,
    _req: HttpRequest,
    item: web::Json<UserUpdateRequest>
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户id不能为空", "local")));
    }

    if item.password.is_none() || item.password.clone().unwrap_or_default().is_empty() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "密码不能为空", "local")));
    }

    let user_result = user_service::get_by_detail(db, &item.id).await;
    if user_result.is_err() || user_result.as_ref().unwrap().id == Some(0) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户不存在", "local")));
    }

    let hashed_password = hash(item.password.clone().unwrap_or_default(), DEFAULT_COST)
        .map_err(|_| Error::from("密码哈希失败"))?;

    let dto_data = UserSaveDTO {
        id: item.id,
        username: None,
        nickname: None,
        avatar: None,
        email: None,
        mobile: None,
        loginfailure: None,
        lastlogintime: None,
        lastloginip: None,
        password: Some(hashed_password),
        salt: None,
        motto: None,
        status: None,
    };

    let result = user_service::update_by_id(db, &dto_data).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}
