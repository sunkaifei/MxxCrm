use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, ResultPage, MPACK};
use crate::modules::purchase::model::purchase_order::{
    PurchaseOrderListQuery, PurchaseOrderSaveRequest, PurchaseOrderUpdateRequest,
};
use crate::modules::purchase::service::purchase_order_service;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn purchase_order_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PurchaseOrderSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let result = purchase_order_service::insert(&db, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn purchase_order_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PurchaseOrderUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "采购单ID不能为空", "local")));
    }

    let result = purchase_order_service::update(&db, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn batch_delete_purchase_order(state: web::Data<AppState>, ids: web::Json<Vec<i64>>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = purchase_order_service::batch_delete(&db, &ids.0).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn purchase_order_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }
    
    match purchase_order_service::get_detail(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn purchase_order_list(state: web::Data<AppState>, query: web::Query<PurchaseOrderListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match purchase_order_service::get_list(&db, &query).await {
        Ok((list, total, _total_pages)) => {
            let page = query.page_num.unwrap_or(1);
            let page_size = query.page_size.unwrap_or(10);
            let result = ResultPage::new(list, total, page, page_size);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
        },
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn audit_purchase_order(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let po_id = path.into_inner();

    match purchase_order_service::audit_po(&db, po_id, get_current_user_id(&req)).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("操作成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn close_purchase_order(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let po_id = path.into_inner();

    match purchase_order_service::close_po(&db, po_id, get_current_user_id(&req)).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("操作成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn reject_purchase_order(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let po_id = path.into_inner();

    match purchase_order_service::reject_po(&db, po_id, get_current_user_id(&req), None).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("操作成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/purchase/order")
            .route("/save", web::post().to(purchase_order_insert).wrap(require_permission("purchase:order:save")))
            .route("/update", web::put().to(purchase_order_update).wrap(require_permission("purchase:order:update")))
            .route("/bath_delete", web::delete().to(batch_delete_purchase_order).wrap(require_permission("purchase:order:delete")))
            .route("/info", web::get().to(purchase_order_info).wrap(require_permission("purchase:order:view")))
            .route("/list", web::get().to(purchase_order_list).wrap(require_permission("purchase:order:list")))
            .route("/audit/{id}", web::put().to(audit_purchase_order).wrap(require_permission("purchase:order:audit")))
            .route("/close/{id}", web::put().to(close_purchase_order).wrap(require_permission("purchase:order:close")))
            .route("/reject/{id}", web::put().to(reject_purchase_order).wrap(require_permission("purchase:order:audit"))),
    );
}