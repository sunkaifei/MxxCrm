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
use crate::core::web::permission_guard::require_permission;

use crate::core::web::entity::common::{BathDeleteIdRequest, Id64};
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::purchase::model::purchase_supplier_product::SupplierProductDTO;
use crate::modules::purchase::service::supplier_product_service;
use actix_web::{web, HttpResponse};

pub async fn supplier_product_list(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;

    match supplier_product_service::get_list(&db).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn supplier_product_list_by_supplier(state: web::Data<AppState>, query: web::Query<Id64>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    if query.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "供应商ID不能为空", "local"));
    }

    match supplier_product_service::list_by_supplier(&db, query.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn supplier_product_list_by_product(state: web::Data<AppState>, query: web::Query<Id64>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    if query.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "产品ID不能为空", "local"));
    }

    match supplier_product_service::list_by_product(&db, query.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn supplier_product_info(state: web::Data<AppState>, query: web::Query<Id64>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    if query.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local"));
    }

    match supplier_product_service::get_info(&db, query.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn supplier_product_save(state: web::Data<AppState>, form_data: web::Json<SupplierProductDTO>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let result = supplier_product_service::insert(&db, &form_data).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn supplier_product_update(state: web::Data<AppState>, form_data: web::Json<SupplierProductDTO>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let result = supplier_product_service::update(&db, &form_data).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn supplier_product_bath_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到删除的供应商产品关联ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = supplier_product_service::batch_delete(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/purchase/supplier_product")
            .route("/save", web::post().to(supplier_product_save).wrap(require_permission("purchase:supplier:save")))
            .route("/update", web::put().to(supplier_product_update).wrap(require_permission("purchase:supplier:save")))
            .route("/bath_delete", web::delete().to(supplier_product_bath_delete).wrap(require_permission("purchase:supplier:delete")))
            .route("/info", web::get().to(supplier_product_info).wrap(require_permission("purchase:supplier:list")))
            .route("/list", web::get().to(supplier_product_list).wrap(require_permission("purchase:supplier:list")))
            .route("/list_by_supplier", web::get().to(supplier_product_list_by_supplier).wrap(require_permission("purchase:supplier:list")))
            .route("/list_by_product", web::get().to(supplier_product_list_by_product).wrap(require_permission("purchase:supplier:list"))),
    );
}
