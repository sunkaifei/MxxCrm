use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::purchase::model::supplier::{SupplierDetailVO, SupplierListQuery, SupplierListVO, SupplierSaveRequest, SupplierUpdateRequest};
use crate::modules::purchase::service::supplier_service;

#[post("/supplier/save")]
#[protect("purchase:supplier:save")]
pub async fn supplier_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<SupplierSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.company_name.as_ref().map_or(true, |name| name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "公司名称不能为空", "local")));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = supplier_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/supplier/update")]
#[protect("purchase:supplier:update")]
pub async fn supplier_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<SupplierUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "供应商ID不能为空", "local")));
    }

    if form_data.company_name.as_ref().map_or(true, |name| name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "公司名称不能为空", "local")));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = supplier_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/supplier/bath_delete")]
#[protect("purchase:supplier:delete")]
pub async fn bath_delete_supplier(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到删除的供应商ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = supplier_service::batch_delete_by_ids(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

#[get("/supplier/info")]
#[protect("purchase:supplier:info")]
pub async fn supplier_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "供应商ID不能为空", "local"));
    }

    match supplier_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[get("/supplier/list")]
#[protect("purchase:supplier:list")]
pub async fn supplier_list(state: web::Data<AppState>, query: web::Query<SupplierListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    match supplier_service::list(&db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}