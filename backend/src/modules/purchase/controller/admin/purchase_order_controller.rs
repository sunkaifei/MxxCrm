use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::response::MetaResp;
use crate::modules::purchase::model::purchase_order::{PurchaseOrderDetailVO, PurchaseOrderListQuery, PurchaseOrderListVO, PurchaseOrderSaveRequest, PurchaseOrderUpdateRequest};
use crate::modules::purchase::service::purchase_order_service;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

#[post("/order/save")]
#[protect("purchase:order:save")]
pub async fn purchase_order_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PurchaseOrderSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;
    
    let result = purchase_order_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/order/update")]
#[protect("purchase:order:update")]
pub async fn purchase_order_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PurchaseOrderUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;
    
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "采购单ID不能为空", "local")));
    }
    
    let result = purchase_order_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/order/bath_delete")]
#[protect("purchase:order:delete")]
pub async fn batch_delete_purchase_order(state: web::Data<AppState>, ids: web::Json<Vec<i64>>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = purchase_order_service::batch_delete(&db, &ids.0).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[get("/order/info")]
#[protect("purchase:order:view")]
pub async fn purchase_order_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }
    
    match purchase_order_service::get_detail(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/order/list")]
#[protect("purchase:order:view")]
pub async fn purchase_order_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();
    
    let query = PurchaseOrderListQuery {
        page_num: query_str.split("&").find(|s| s.starts_with("page=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        page_size: query_str.split("&").find(|s| s.starts_with("pageSize=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        keywords: query_str.split("&").find(|s| s.starts_with("keywords=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
        status: query_str.split("&").find(|s| s.starts_with("status=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
        supplier_id: query_str.split("&").find(|s| s.starts_with("supplierId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        order_type: query_str.split("&").find(|s| s.starts_with("orderType=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
    };
    
    match purchase_order_service::get_list(&db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}