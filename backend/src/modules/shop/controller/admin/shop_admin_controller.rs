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
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::{MetaResp};
use crate::modules::shop::model::shop::{ListQuery, ShopSaveRequest, ShopUpdateRequest};
use crate::modules::shop::service::shop_service;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

/// Save shop
#[post("/shop/save")]
#[protect("system:shop:save")]
pub async fn save_shop(state: web::Data<AppState>, _req: HttpRequest, item: web::Json<ShopSaveRequest>) -> HttpResponse {
    let db = &state.db;

    if item.store_name.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "搴楅摵鍚嶇О涓嶈兘涓虹┖", "local"));
    }

    let result = shop_service::save(db, &item.0.into()).await;
    match result {
        Ok(v) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::success(v, "local"))
        }
        Err(err) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local"))
        }
    }
}

/// Batch delete shops
#[delete("/shop/batch_delete")]
#[protect("system:shop:delete")]
pub async fn batch_delete_shop(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;

    if let Some(ids_vec) = item.ids.clone() {
        // Convert Vec<Option<String>> to Vec<i64>
        let ids: Vec<i64> = ids_vec.into_iter()
            .filter_map(|opt| opt)
            .filter_map(|s| s.parse::<i64>().ok())
            .collect();

        if ids.is_empty() {
            return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "鍒犻櫎鐨処D涓嶈兘涓虹┖", "local"));
        } else {
            let result = shop_service::batch_delete_by_ids(&db, &ids).await;
            match result {
                Ok(v) => {
                    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::success(v, "local"))
                }
                Err(err) => {
                    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local"))
                }
            }
        }
    } else {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "鍒犻櫎鐨処D涓嶈兘涓虹┖", "local"));
    }
}

/// Update shop
#[put("/shop/update/{id}")]
#[protect("system:shop:update")]
pub async fn update_shop(state: web::Data<AppState>, path: web::Path<i64>, _req: HttpRequest, item: web::Json<ShopUpdateRequest>) -> HttpResponse {
    let db = &state.db;
    let mut update_data = item.into_inner();

    // Set ID from path
    update_data.id = Some(path.into_inner());

    if update_data.store_name.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "搴楅摵鍚嶇О涓嶈兘涓虹┖", "local"));
    }

    let result = shop_service::update_by_id(&db, &update_data.into()).await;
    match result {
        Ok(v) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::success(v, "local"))
        }
        Err(err) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local"))
        }
    }
}

/// Get shop detail
#[get("/shop/detail/{id}")]
#[protect("system:shop:view")]
pub async fn get_shop_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;

    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID涓嶈兘涓虹┖", "local")));
    }

    match shop_service::get_by_detail(&db, &item.id).await {
        Ok(shop) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(shop, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

/// Get shop list (paginated)
#[get("/shop/list")]
#[protect("system:shop:list")]
pub async fn get_shop_list(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;

    shop_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}
