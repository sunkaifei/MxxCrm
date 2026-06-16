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
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::system::model::notice::{ListQuery, NoticeSaveDTO, NoticeSaveRequest, NoticeUpdateRequest};
use crate::modules::system::service::notice_service;
use crate::validate;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

#[post("/notice/add")]
#[protect("system:notice:add")]
pub async fn add_notice(state: web::Data<AppState>, req: HttpRequest, item: web::Json<NoticeSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;

    //获取当前用户id
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    let mut form_data = NoticeSaveDTO::from(item.into_inner());
    form_data.publish_status = Some(0);
    form_data.create_by = admin_token.id;
    form_data.update_by = admin_token.id;
    let result = notice_service::insert(db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success("添加成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}


#[delete("/notice/bath_delete")]
#[protect("system:notice:delete")]
pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let result = notice_service::batch_delete_by_ids(db, &ids_vec).await;
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
        }
    }else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

#[put("/notice/update/{id}")]
#[protect("system:notice:update")]
pub async fn update_by_id(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, item: web::Json<NoticeUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    let mut form_data = NoticeSaveDTO::from(item.into_inner());
    form_data.id = Some(id.into_inner());
    form_data.update_by = admin_token.id;

    let result = notice_service::update_by_id(&db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success("更新成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "更新失败", "local")))
    }
}

#[put("/notice/read-all")]
pub async fn user_read_all(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    let result = notice_service::update_by_read_all(&db, &admin_token.id).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(200, "已设置全部为阅读状态", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "阅读设置失败", "local")))
    }
}

#[put("/notice/{id}/revoke")]
#[protect("system:notice:revoke")]
pub async fn revoke_notice(state: web::Data<AppState>, req: HttpRequest, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    validate!(item.id.is_none(), t!("notice.index.id_empty", locale = "zh-CN").to_string());
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    let result= notice_service::update_by_id_revoke(&db, &item.id, &admin_token.id).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(200, "撤销成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "撤销失败", "local")))
    }
}

#[put("/notice/{id}/publish")]
#[protect("system:notice:publish")]
pub async fn publish_notice(state: web::Data<AppState>, req: HttpRequest, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    validate!(item.id.is_none(), t!("notice.index.id_empty", locale = "zh-CN").to_string());
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    let result=notice_service::update_by_id_publish(&db, &item.id, &admin_token.id).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(200, "发布成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "发布失败", "local")))
    }
}

#[get("/notice/detail/{id}")]
pub async fn get_by_detail(state: web::Data<AppState>, _req: HttpRequest, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    validate!(item.id.is_none(), t!("notice.index.id_empty", locale = "zh-CN").to_string());
    match notice_service::get_by_detail(&db, &item.id).await {
        Ok(Some(notice_vo)) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(notice_vo, "local")))
        }
        Ok(None) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "该公告信息不存在或者已删除", "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}


#[get("/notice/user/detail-{id}")]
pub async fn get_by_user_detail(state: web::Data<AppState>, req: HttpRequest, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    validate!(item.id.is_none(), t!("notice.index.id_empty", locale = "zh-CN").to_string());
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    match notice_service::get_by_user_detail(&db, &item.id, &admin_token.id).await {
        Ok(Some(notice_vo)) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(notice_vo, "local")))
        }
        Ok(None) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "该公告信息不存在或者已删除", "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}


#[get("/notice/my-page")]
pub async fn get_by_my_page(state: web::Data<AppState>, req: HttpRequest, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let mut query = query.into_inner();
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    query.user_id = admin_token.id;
    notice_service::get_by_my_page(&db, query).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

#[get("/notice/list")]
#[protect("system:notice:list")]
pub async fn get_by_page(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    notice_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}
