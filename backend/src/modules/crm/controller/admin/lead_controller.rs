use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::crm::model::lead::{LeadDetailVO, LeadListQuery, LeadListVO, LeadSaveRequest, LeadStatusUpdateQuery, LeadUpdateRequest};
use crate::modules::crm::service::lead_service;

#[post("/lead/save")]
#[protect("crm:lead:save")]
pub async fn lead_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<LeadSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.company_name.as_ref().map_or(true, |name| name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "公司名称不能为空", "local")));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = lead_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/lead/update")]
#[protect("crm:lead:update")]
pub async fn lead_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<LeadUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "线索ID不能为空", "local")));
    }

    if form_data.company_name.as_ref().map_or(true, |name| name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "公司名称不能为空", "local")));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = lead_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/lead/bath_delete")]
#[protect("crm:lead:delete")]
pub async fn bath_delete_lead(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到删除的线索ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = lead_service::batch_delete_by_ids(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

#[get("/lead/info")]
#[protect("crm:lead:info")]
pub async fn lead_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "线索ID不能为空", "local"));
    }

    match lead_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[get("/lead/list")]
#[protect("crm:lead:list")]
pub async fn lead_list(state: web::Data<AppState>, query: web::Query<LeadListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    match lead_service::list(&db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[get("/lead-pool/list")]
#[protect("crm:lead-pool:list")]
pub async fn lead_pool_list(state: web::Data<AppState>, query: web::Query<LeadListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    match lead_service::list(&db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[get("/lead-pool/info")]
#[protect("crm:lead-pool:info")]
pub async fn lead_pool_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "线索ID不能为空", "local"));
    }

    match lead_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[delete("/lead-pool/bath_delete")]
#[protect("crm:lead-pool:delete")]
pub async fn bath_delete_lead_pool(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "鏈幏鍙栧埌鍒犻櫎鐨勭嚎绱D", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))   
        .collect();

    let result = lead_service::batch_delete_by_ids(&db, &filtered_ids).await;   
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

#[put("/lead/update-status")]
#[protect("crm:lead:update")]
pub async fn lead_update_status(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<LeadStatusUpdateQuery>) -> HttpResponse {
    let db = &state.db;
    let query = form_data.0;

    if query.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "绾跨储ID涓嶈兘涓虹┖", "local"));
    }

    if query.status.is_none() || query.status.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "鐘舵€佷笉鑳戒负绌�", "local"));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = lead_service::update_status(&db, query.id.unwrap(), query.status.unwrap().as_str(), Some(jwt_token.id.unwrap_or_default())).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

#[put("/lead/add-to-pool")]
#[protect("crm:lead:update")]
pub async fn lead_add_to_pool(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<InfoId>) -> HttpResponse {
    let db = &state.db;
    let query = form_data.0;

    if query.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "绾跨储ID涓嶈兘涓虹┖", "local"));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = lead_service::add_to_pool(&db, query.id.unwrap(), Some(jwt_token.id.unwrap_or_default())).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}