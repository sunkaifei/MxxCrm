//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{delete, get, HttpResponse, post, put, web};
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::system::model::region::{RegionSaveRequest, RegionUpdateRequest};
use crate::modules::system::service::region_service;

#[post("/region/save")]
pub async fn save_region(state: web::Data<AppState>, item: web::Json<RegionSaveRequest>) -> HttpResponse {
    let db = &state.db;
    let result = region_service::insert(&db, &item.0).await;
    match result {
        Ok(v) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))
        }
        Err(err) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &("保存菜单异常,".to_string() + &err.to_string()), "local"))
        }
    }
}

#[delete("/region/batch_delete")]
pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        let region_list = region_service::get_by_parent_id(&db, &ids_vec).await.unwrap_or_default();
        if region_list.len() > 0 { 
            return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的地区存在子级，请先删除子级", "local"))
        }
        let result = region_service::batch_delete_by_ids(&db, &ids_vec).await;
        match result {
            Ok(v) => {
                HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))
            }
            Err(err) => {
                HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &("删除菜单异常,".to_string() + &err.to_string()), "local"))
            }
        }
    } else {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local"))
    }
}

#[put("/region/update/{id}")]
pub async fn update_by_id(state: web::Data<AppState>, id: web::Path<i64>, item: web::Json<RegionUpdateRequest>) -> HttpResponse {
    let db = &state.db;
    let mut data = item.into_inner();
    data.id = Some(id.into_inner());
    let result = region_service::update_by_id(&db, &data).await;
    return match result {
        Ok(v) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))
        }
        Err(err) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &("更新菜单异常,".to_string() + &err.to_string()), "local"))
        }
    }
}

#[get("/region/detail/{id}")]
pub async fn get_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> HttpResponse {
    let db = &state.db;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "行政区域id不能为空", "local"));
    }
    let result = region_service::get_by_detail(&db, &item.id).await;
    return match result {
        Ok(v) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))
        }
        Err(err) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &("查询行政区域异常,".to_string() + &err.to_string()), "local"))

       }
   }
}
#[get("/region/treelist")]
pub async fn get_region_tree(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    let result = region_service::get_region_tree(&db).await;
    match result {
        Ok(v) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))
        }
        Err(err) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &("查询菜单异常,".to_string() + &err.to_string()), "local"))
        }
    }
}
