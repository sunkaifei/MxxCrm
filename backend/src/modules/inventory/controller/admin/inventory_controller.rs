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
use crate::modules::inventory::model::stock::{InventoryDetailVO, InventoryListData, InventoryListQuery};
use crate::modules::inventory::service::inventory_service;
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

#[get("/inventory/list")]
#[protect("product:inventory:list")]
pub async fn inventory_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    let query = InventoryListQuery {
        page_num: query_str.split("&").find(|s| s.starts_with("page=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        page_size: query_str.split("&").find(|s| s.starts_with("pageSize=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        product_name: query_str.split("&").find(|s| s.starts_with("productName=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
        warehouse_id: query_str.split("&").find(|s| s.starts_with("warehouseId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        low_stock: query_str.split("&").find(|s| s.starts_with("lowStock=")).and_then(|s| s.split("=").nth(1).map(|s| s == "true")),
    };

    match inventory_service::get_list(db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/inventory/info")]
#[protect("product:inventory:view")]
pub async fn inventory_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match inventory_service::get_detail(db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}
