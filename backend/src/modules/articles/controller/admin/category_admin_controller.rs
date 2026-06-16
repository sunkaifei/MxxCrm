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
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::{BathDeleteIdRequest, BathIdRequest, InfoId};
use crate::core::web::response::{MetaResp};
use crate::modules::articles::model::category::{CategoryModel, CategoryPageDTO, CategoryPageRequest, CategorySaveDTO, CategorySaveRequest, CategoryUpdateRequest};
use crate::modules::articles::service::category_service;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

#[post("/category/save")]
#[protect("article:category:add")]
pub async fn save_category(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CategorySaveRequest>
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    let mut category_data = CategorySaveDTO::from(item);
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    category_data.website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());
    match category_service::save_category(&db, category_data).await {
        Ok(_) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(200, "保存成功", "local")))
        }
        Err(_err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "保存失败", "local")))
        }
    }
}


#[delete("/category/batch_delete")]
#[protect("article:category:delete")]
pub async fn batch_delete(state: web::Data<AppState>, req: HttpRequest, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let result = CategoryModel::batch_delete_by_ids(&db,&website_id.map(|s| s.parse::<i64>().unwrap_or_default()), ids).await?;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }

}

#[put("/category/update/{id}")]
#[protect("article:category:update")]
pub async fn update_category(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
    item: web::Json<CategoryUpdateRequest>
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    let mut category_data = CategorySaveDTO::from(item);
    category_data.id = Some(id.into_inner());
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    category_data.website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());

    let category_data = category_service::update_by_id(&db, category_data).await?;
    if category_data == 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "更新失败", "local")));
    }
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("修改成功".to_string(), "local")))
}

#[get("/category/Option")]
pub async fn category_option(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());

    match category_service::all_category_tree(&db, website_id.map(|s| s.parse::<i64>().unwrap_or_default()).unwrap_or_default()).await {
        Ok(router_list) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(router_list, "local")))
        }
        Err(_err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到文章分类列表", "local")))
        }
    }
}

#[get("/category/detail/{id}")]
#[protect("article:category:view")]
pub async fn get_category_detail(state: web::Data<AppState>, req: HttpRequest,item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = item.id;
    //log::info!("----------------find_by_id:{:?}", id);
    let _website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    let _website_id = _website_id.map(|s| s.parse::<i64>().unwrap_or_default());

    let result = category_service::find_by_id(db,&id).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

#[get("/category/list")]
#[protect("article:category:list")]
pub async fn category_list_tree(state: web::Data<AppState>, req: HttpRequest, item: web::Query<CategoryPageRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.0;
    let mut category_dto = CategoryPageDTO::from(payload);

    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());

    category_dto.website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());

    match category_service::select_all_list(&db,category_dto).await{
        Ok(router_list) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(router_list, "local")))
        }
        Err(_err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到文章分类列表", "local")))
        }
    }
}
