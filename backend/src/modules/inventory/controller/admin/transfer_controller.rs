//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{web, HttpResponse, HttpRequest, Result};
use rust_decimal::Decimal;
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::base_controller::get_current_user_id;
use crate::modules::inventory::service::transfer_service;
use crate::modules::inventory::model::transfer::*;
use crate::core::web::response::MPACK;
use crate::core::errors::error::Result as MyResult;

pub async fn transfer_save(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data: TransferSaveRequest = serde_json::from_value(body.0).unwrap_or(TransferSaveRequest {
        from_warehouse_id: 0,
        to_warehouse_id: 0,
        remark: None,
        items: vec![],
    });
    if form_data.from_warehouse_id <= 0 || form_data.to_warehouse_id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "仓库ID无效", "local")));
    }
    if form_data.items.is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "调拨明细不能为空", "local")));
    }
    let result = transfer_service::create(db, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn transfer_outbound(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }
    let result = transfer_service::outbound(db, id, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn transfer_inbound(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }
    let result = transfer_service::inbound(db, id, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn transfer_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }
    match transfer_service::get_detail(db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn transfer_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    let query = TransferListQuery {
        page_num: query_str.split("&").find(|s| s.starts_with("page=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse().ok())),
        page_size: query_str.split("&").find(|s| s.starts_with("pageSize=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse().ok())),
        transfer_no: query_str.split("&").find(|s| s.starts_with("transferNo=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
        from_warehouse_id: query_str.split("&").find(|s| s.starts_with("fromWarehouseId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse().ok())),
        to_warehouse_id: query_str.split("&").find(|s| s.starts_with("toWarehouseId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse().ok())),
        status: query_str.split("&").find(|s| s.starts_with("status=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse().ok())),
    };
    match transfer_service::get_list(&db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn transfer_batch_delete(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<String>().ok())).unwrap_or_default();
    let ids: Vec<i64> = id.split(',').filter_map(|s| s.parse::<i64>().ok()).collect();
    if ids.is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "参数无效", "local")));
    }
    let result = transfer_service::batch_delete(db, &ids).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/inventory/transfer")
            .route("/save", web::post().to(transfer_save).wrap(require_permission("product:transfer:create")))
            .route("/outbound/{id}", web::put().to(transfer_outbound).wrap(require_permission("product:transfer:audit")))
            .route("/inbound/{id}", web::put().to(transfer_inbound).wrap(require_permission("product:transfer:audit")))
            .route("/info", web::get().to(transfer_info).wrap(require_permission("product:transfer:list")))
            .route("/list", web::get().to(transfer_list).wrap(require_permission("product:transfer:list")))
            .route("/batch_delete", web::delete().to(transfer_batch_delete).wrap(require_permission("product:transfer:delete"))),
    );
}