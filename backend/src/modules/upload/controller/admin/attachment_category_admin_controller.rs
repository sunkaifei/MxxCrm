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
use crate::core::web::response::MetaResp;
use actix_web::{delete, get, post, put, web, HttpResponse};
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::modules::upload::model::attachment_category::{AttachCategoryListVO, AttachCategorySaveRequest, AttachCategoryUpdateRequest, AttachmentCategoryModel, ListQuery, PageWhere};
use crate::modules::upload::service::{attachment_category_service};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use crate::validate;

#[post("/attachment/category/add")]
pub async fn save_category(state: web::Data<AppState>, item: web::Json<AttachCategorySaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    let result = attachment_category_service::save(&db, item).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
}

#[put("/attachment/category/update/{id}")]
pub async fn update_category(state: web::Data<AppState>, id: web::Path<i64>, item: web::Json<AttachCategoryUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let mut item = item.0;
    item.id = Some(id.into_inner());
    let result = attachment_category_service::update(&db, item).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
}

#[delete("/attachment/category/batch_delete")]
pub async fn batch_delete_by_ids(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let result = attachment_category_service::batch_delete_by_ids(&db, ids).await?;
        Ok(HttpResponse::Ok().json(&MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

#[get("/attachment/category/tree")]
pub async fn get_by_tree(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let tree_list = attachment_category_service::get_category_tree(&db, query.0).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(tree_list, "local")))
}

#[get("/attachment/category/detail/{id}")]
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    validate!(item.id.is_none(), t!("attachment.category.name_empty", locale = "zh-CN").to_string());
    let result = attachment_category_service::find_by_id(&db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

#[get("/attachment/category/list")]
pub async fn get_by_list(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.0;
    let search_where = PageWhere {
        name: Option::from(query.name),
    };
    let search_where = search_where.format();
    let list = AttachmentCategoryModel::find_all(&db, search_where).await?;
    
    let list_data: Vec<AttachCategoryListVO> = list.into_iter().map(|item| AttachCategoryListVO::from(item)).collect();
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(list_data, "local")))
}