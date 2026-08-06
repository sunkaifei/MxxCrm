use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;

use crate::core::web::entity::common::{BathDeleteIdRequest, Id64};
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::purchase::model::supplier_brand::SupplierBrandDTO;
use crate::modules::purchase::service::supplier_brand_service;
use actix_web::{web, HttpResponse};

pub async fn supplier_brand_list_by_supplier(state: web::Data<AppState>, query: web::Query<Id64>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    if query.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "供应商ID不能为空", "local"));
    }

    match supplier_brand_service::list_by_supplier(&db, query.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn supplier_brand_list_by_brand(state: web::Data<AppState>, query: web::Query<Id64>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    if query.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "品牌ID不能为空", "local"));
    }

    match supplier_brand_service::list_by_brand(&db, query.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn supplier_brand_save(state: web::Data<AppState>, form_data: web::Json<SupplierBrandDTO>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let result = supplier_brand_service::insert(&db, &form_data).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn supplier_brand_bath_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到删除的供应商品牌关联ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = supplier_brand_service::batch_delete(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/purchase/supplier/brand")
            .route("/list_by_supplier", web::get().to(supplier_brand_list_by_supplier).wrap(require_permission("purchase:supplier:view")))
            .route("/list_by_brand", web::get().to(supplier_brand_list_by_brand).wrap(require_permission("purchase:supplier:list")))
            .route("/save", web::post().to(supplier_brand_save).wrap(require_permission("purchase:supplier:update")))
            .route("/bath_delete", web::delete().to(supplier_brand_bath_delete).wrap(require_permission("purchase:supplier:update"))),
    );
}