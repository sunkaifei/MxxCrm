use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::purchase::model::purchase_requisition::{RequisitionListQuery, RequisitionSaveRequest};
use crate::modules::purchase::service::purchase_requisition_service;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::permission_guard::require_permission;

pub async fn requisition_save(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<RequisitionSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;

    let result = purchase_requisition_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn requisition_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<RequisitionSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "申请单ID不能为空", "local")));
    }

    let result = purchase_requisition_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    match result {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn requisition_submit(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let id = body.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    let result = purchase_requisition_service::submit_approval(&db, id, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn requisition_approve(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let id = body.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let comment = body.get("comment").and_then(|v| v.as_str().map(|s| s.to_string()));
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    let result = purchase_requisition_service::approve(&db, id, jwt_token.id.unwrap_or_default(), comment).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn requisition_reject(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let id = body.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let comment = body.get("comment").and_then(|v| v.as_str().map(|s| s.to_string()));
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    let result = purchase_requisition_service::reject(&db, id, jwt_token.id.unwrap_or_default(), comment).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn requisition_withdraw(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let id = body.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    let result = purchase_requisition_service::withdraw(&db, id, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn batch_delete_requisition(state: web::Data<AppState>, ids: web::Json<Vec<i64>>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = purchase_requisition_service::batch_delete(&db, &ids.0).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn requisition_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match purchase_requisition_service::get_info(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn requisition_list(state: web::Data<AppState>, query: web::Query<RequisitionListQuery>) -> Result<HttpResponse> {
    let db = &state.db;

    match purchase_requisition_service::get_list(&db, &query.into_inner()).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn requisition_approval_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let page_num = req.query_string().split("&").find(|s| s.starts_with("page=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(1);
    let page_size = req.query_string().split("&").find(|s| s.starts_with("pageSize=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(10);

    match purchase_requisition_service::get_my_approval_list(&db, jwt_token.id.unwrap_or_default(), page_num, page_size).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn requisition_convert(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let body = body.0;
    let pr_ids: Vec<i64> = body.get("pr_ids").and_then(|v| v.as_array()).map(|arr| {
        arr.iter().filter_map(|v| v.as_i64()).collect()
    }).unwrap_or_default();
    let supplier_id = body.get("supplier_id").and_then(|v| v.as_i64()).unwrap_or(0);

    if pr_ids.is_empty() || supplier_id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "参数无效", "local")));
    }

    let result = purchase_requisition_service::convert_to_po(&db, pr_ids, supplier_id, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/purchase/requisition")
            .route("/save", web::post().to(requisition_save).wrap(require_permission("purchase:requisition:save")))
            .route("/update", web::put().to(requisition_update).wrap(require_permission("purchase:requisition:update")))
            .route("/submit", web::post().to(requisition_submit).wrap(require_permission("purchase:requisition:save")))
            .route("/approve", web::post().to(requisition_approve).wrap(require_permission("purchase:requisition:approve")))
            .route("/reject", web::post().to(requisition_reject).wrap(require_permission("purchase:requisition:approve")))
            .route("/withdraw", web::post().to(requisition_withdraw).wrap(require_permission("purchase:requisition:save")))
            .route("/bath_delete", web::delete().to(batch_delete_requisition).wrap(require_permission("purchase:requisition:delete")))
            .route("/info", web::get().to(requisition_info).wrap(require_permission("purchase:requisition:view")))
            .route("/list", web::get().to(requisition_list).wrap(require_permission("purchase:requisition:list")))
            .route("/approval/list", web::get().to(requisition_approval_list).wrap(require_permission("purchase:requisition:list")))
            .route("/convert", web::post().to(requisition_convert).wrap(require_permission("purchase:requisition:convert"))),
    );
}