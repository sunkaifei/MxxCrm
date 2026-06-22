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
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::response::MetaResp;
use crate::modules::product::model::product::{ProductDetailVO, ProductListQuery, ProductListVO, ProductSaveRequest, ProductUpdateRequest};
use crate::modules::product::service::product_service;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

#[post("/product/product/save")]
#[protect("product:product:save")]
pub async fn product_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ProductSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;
    
    let result = product_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/product/product/update")]
#[protect("product:product:edit")]
pub async fn product_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ProductUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;
    
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "浜у搧ID涓嶈兘涓虹┖", "local")));
    }
    
    let result = product_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/product/product/batchDelete")]
#[protect("product:product:delete")]
pub async fn batch_delete_product(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    let ids: Vec<i64> = item.ids.unwrap_or_default()
        .iter()
        .filter_map(|id| id.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();
    let result = product_service::batch_delete(&db, &ids).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[get("/product/product/info")]
#[protect("product:product:view")]
pub async fn product_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID鏃犳晥", "local")));
    }
    
    match product_service::get_detail(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/product/product/list")]
#[protect("product:product:view")]
pub async fn product_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();
    
    let query = ProductListQuery {
        page_num: query_str.split("&").find(|s| s.starts_with("page=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        page_size: query_str.split("&").find(|s| s.starts_with("pageSize=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        keywords: query_str.split("&").find(|s| s.starts_with("keywords=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
        category_id: query_str.split("&").find(|s| s.starts_with("categoryId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        is_active: query_str.split("&").find(|s| s.starts_with("isActive=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<bool>().ok())),
    };
    
    match product_service::get_list(&db, &query).await {
        Ok((list, total, total_pages)) => {
            let page = query.page_num.unwrap_or(1);
            let page_size = query.page_size.unwrap_or(10);
            let result = crate::core::web::response::ResultPage::new(list, total, page, page_size);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
        },
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}