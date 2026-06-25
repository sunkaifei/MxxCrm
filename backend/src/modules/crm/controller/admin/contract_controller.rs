use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::crm::model::contract::{ContractApprovalDetailVO, ContractApprovalRequest, ContractDetailVO, ContractListQuery, ContractListVO, ContractSaveDTO, ContractSaveRequest, ContractUpdateRequest};
use crate::modules::crm::service::contract_service;

#[post("/contract/save")]
#[protect("crm:contract:save")]
pub async fn contract_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ContractSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data: ContractSaveDTO = form_data.0.into();

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = contract_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/contract/update")]
#[protect("crm:contract:update")]
pub async fn contract_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ContractUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data: ContractSaveDTO = form_data.0.into();

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "合同ID不能为空", "local")));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = contract_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/contract/bath_delete")]
#[protect("crm:contract:delete")]
pub async fn bath_delete_contract(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到删除的合同ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = contract_service::batch_delete_by_ids(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

#[get("/contract/info")]
#[protect("crm:contract:info")]
pub async fn contract_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "合同ID不能为空", "local"));
    }

    match contract_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[get("/contract/list")]
#[protect("crm:contract:list")]
pub async fn contract_list(state: web::Data<AppState>, query: web::Query<ContractListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    match contract_service::list(&db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[post("/contract/submit")]
#[protect("crm:contract:submit")]
pub async fn contract_submit(state: web::Data<AppState>, req: HttpRequest, item: web::Json<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "合同ID不能为空", "local"));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    match contract_service::submit_contract(&db, item.id.unwrap(), jwt_token.id.unwrap_or_default(), &jwt_token.username.unwrap_or_default()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[post("/contract/approve")]
#[protect("crm:contract:approve")]
pub async fn contract_approve(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ContractApprovalRequest>) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    match contract_service::approve_contract(&db, &form_data, jwt_token.id.unwrap_or_default(), &jwt_token.username.unwrap_or_default()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[post("/contract/reject")]
#[protect("crm:contract:reject")]
pub async fn contract_reject(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ContractApprovalRequest>) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    match contract_service::reject_contract(&db, &form_data, jwt_token.id.unwrap_or_default(), &jwt_token.username.unwrap_or_default()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[get("/contract/approval-detail/{contract_id}")]
#[protect("crm:contract:list")]
pub async fn contract_approval_detail(state: web::Data<AppState>, path: web::Path<i64>) -> HttpResponse {
    let db = &state.db;
    let contract_id = path.into_inner();

    match contract_service::get_approval_detail(&db, contract_id).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}