use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::sale::model::quotation::{QuotationListQuery, QuotationSaveRequest, QuotationUpdateRequest};
use crate::modules::sale::service::quotation_service;
use actix_web::{get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotationApprovalRequest {
    pub remark: Option<String>,
}

#[post("/sale/quotation/save")]
#[protect("sale:quotation:save")]
pub async fn quotation_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<QuotationSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default().to_string();
    let result = quotation_service::insert(db, &form_data, user_id).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/sale/quotation/update")]
#[protect("sale:quotation:edit")]
pub async fn quotation_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<QuotationUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "报价单ID不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default().to_string();
    let result = quotation_service::update(db, &form_data, user_id).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[post("/sale/quotation/batch-delete")]
#[protect("sale:quotation:delete")]
pub async fn bath_delete_quotation(state: web::Data<AppState>, form_data: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = form_data.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids: Vec<i64> = ids_vec.into_iter().filter_map(|id| id.and_then(|s| s.parse().ok())).collect();
        let result = quotation_service::batch_delete_by_ids(db, &ids).await;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

#[get("/sale/quotation/info")]
#[protect("sale:quotation:list")]
pub async fn quotation_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "报价单ID不能为空", "local"));
    }
    match quotation_service::find_by_id(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[get("/sale/quotation/list")]
#[protect("sale:quotation:list")]
pub async fn quotation_list(state: web::Data<AppState>, query: web::Query<QuotationListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    match quotation_service::list(db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[post("/sale/quotation/{id}/submit-approval")]
#[protect("sale:quotation:update")]
pub async fn quotation_submit_approval(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<InfoId>,
    form_data: web::Json<QuotationApprovalRequest>,
) -> HttpResponse {
    let db = &state.db;
    let id = path.id.unwrap_or_default();
    if id == 0 {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "报价单ID不能为空", "local"));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let operator_id = jwt_token.id.unwrap_or_default();
    let operator_name = jwt_token.username.unwrap_or_default();
    match quotation_service::submit_approval(db, id, operator_id, &operator_name, form_data.remark.clone()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[post("/sale/quotation/{id}/approve")]
#[protect("sale:quotation:approve")]
pub async fn quotation_approve(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<InfoId>,
    form_data: web::Json<QuotationApprovalRequest>,
) -> HttpResponse {
    let db = &state.db;
    let id = path.id.unwrap_or_default();
    if id == 0 {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "报价单ID不能为空", "local"));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let operator_id = jwt_token.id.unwrap_or_default();
    let operator_name = jwt_token.username.unwrap_or_default();
    match quotation_service::approve(db, id, operator_id, &operator_name, form_data.remark.clone()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[post("/sale/quotation/{id}/reject")]
#[protect("sale:quotation:approve")]
pub async fn quotation_reject(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<InfoId>,
    form_data: web::Json<QuotationApprovalRequest>,
) -> HttpResponse {
    let db = &state.db;
    let id = path.id.unwrap_or_default();
    if id == 0 {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "报价单ID不能为空", "local"));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let operator_id = jwt_token.id.unwrap_or_default();
    let operator_name = jwt_token.username.unwrap_or_default();
    match quotation_service::reject(db, id, operator_id, &operator_name, form_data.remark.clone()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[post("/sale/quotation/{id}/convert-order")]
#[protect("sale:quotation:update")]
pub async fn quotation_convert_order(state: web::Data<AppState>, req: HttpRequest, path: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let result = quotation_service::convert_to_order(db, path.id.unwrap_or_default(), jwt_token.id.unwrap_or_default().to_string()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}
