//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{delete, get, post, put, web, HttpResponse};

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::{MetaResp, ResultPage};
use crate::modules::user::model::user_platform::{ListQuery, PageWhere, UserPlatformListVO, UserPlatformModel, UserPlatformSaveRequest, UserPlatformUpdateRequest};
use crate::modules::user::model::user_platform::UserPlatformDetailVO;
use crate::modules::user::service::user_platform_service;

#[post("/user/platform/add")]
pub async fn save_platform(state: web::Data<AppState>, item: web::Json<UserPlatformSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let save_dto = item.0.into();
    let result = user_platform_service::insert(db, &save_dto).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/user/platform/delete")]
pub async fn batch_delete(state: web::Data<AppState>, query: web::Query<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let ids: Vec<i64> = query.ids
        .as_ref()
        .and_then(|vec| {
            vec.iter()
                .filter_map(|s| s.as_ref()?.parse::<i64>().ok())
                .collect::<Vec<i64>>()
                .into()
        })
        .unwrap_or_default();

    if ids.is_empty() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "请选择要删除的记录", "local")));
    }

    let result = user_platform_service::batch_delete(db, &ids).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/user/platform/update/{id}")]
pub async fn update_platform(state: web::Data<AppState>, id: web::Path<i64>, item: web::Json<UserPlatformUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let save_dto = item.into_inner().into();
    let result = user_platform_service::update_by_id(db, id.into_inner(), &save_dto).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[get("/user/platform/detail")]
pub async fn get_platform_detail(state: web::Data<AppState>, query: web::Query<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = query.id.unwrap_or_default();
    let result = UserPlatformModel::find_by_id(db, &id).await;
    match result {
        Ok(Some(model)) => {
            let vo: UserPlatformDetailVO = model.into();
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<UserPlatformDetailVO>::success(vo, "local")))
        }
        Ok(None) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "记录不存在", "local")))
        }
        Err(e) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")))
        }
    }
}

#[get("/user/platform/list")]
pub async fn get_platform_list(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let wheres = PageWhere {
        user_id: query.user_id,
        system_type: query.system_type.clone(),
        platform: query.platform.clone(),
    };

    let where_data = wheres.format();

    let (items, total) = UserPlatformModel::select_in_page(db, page, page_size, where_data).await?;
    let list: Vec<UserPlatformListVO> = items.into_iter().map(|model| model.into()).collect();
    let page_data = ResultPage::new(list, total, page, page_size);

    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local")))
}