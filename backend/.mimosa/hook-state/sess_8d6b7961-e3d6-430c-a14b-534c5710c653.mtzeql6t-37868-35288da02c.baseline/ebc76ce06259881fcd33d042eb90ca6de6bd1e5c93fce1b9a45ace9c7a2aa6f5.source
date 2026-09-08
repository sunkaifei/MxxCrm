use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::purchase::model::purchase_receipt::{ReceiptListQuery, ReceiptSaveRequest};
use crate::modules::purchase::service::purchase_receipt_service;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::permission_guard::require_permission;

pub async fn receipt_save(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ReceiptSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let result = purchase_receipt_service::insert(&db, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn batch_delete_receipt(state: web::Data<AppState>, ids: web::Json<Vec<i64>>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = purchase_receipt_service::batch_delete(&db, &ids.0).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn receipt_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match purchase_receipt_service::get_info(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn receipt_list(state: web::Data<AppState>, query: web::Query<ReceiptListQuery>) -> Result<HttpResponse> {
    let db = &state.db;

    match purchase_receipt_service::get_list(&db, &query.into_inner()).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn receipt_to_inbound(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let body = body.0;
    let id = body.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let warehouse_id = body.get("warehouse_id").and_then(|v| v.as_i64()).unwrap_or(0);

    if id <= 0 || warehouse_id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "参数无效", "local")));
    }

    let result = purchase_receipt_service::to_inbound(&db, id, warehouse_id, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/purchase/receipt")
            .route("/save", web::post().to(receipt_save).wrap(require_permission("purchase:receipt:save")))
            .route("/bath_delete", web::delete().to(batch_delete_receipt).wrap(require_permission("purchase:receipt:delete")))
            .route("/info", web::get().to(receipt_info).wrap(require_permission("purchase:receipt:list")))
            .route("/list", web::get().to(receipt_list).wrap(require_permission("purchase:receipt:list")))
            .route("/to_inbound", web::post().to(receipt_to_inbound).wrap(require_permission("purchase:receipt:inbound"))),
    );
}