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
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::permission_guard::require_permission;
use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::crm::model::lead::{LeadDetailVO, LeadListQuery, LeadListVO, LeadSaveRequest, LeadStatusUpdateQuery, LeadUpdateRequest};
use crate::modules::crm::service::lead_service;
use crate::modules::crm::service::lead_transfer_service;

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

pub async fn lead_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<LeadListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();

    match lead_service::list(&db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn lead_pool_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<LeadListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();

    match lead_service::list(&db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

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

pub async fn bath_delete_lead_pool(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
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

pub async fn lead_update_status(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<LeadStatusUpdateQuery>) -> HttpResponse {
    let db = &state.db;
    let query = form_data.0;

    if query.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "线索ID不能为空", "local"));
    }

    if query.status.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "状态不能为空", "local"));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = lead_service::update_status(&db, query.id.unwrap(), query.status.unwrap(), Some(jwt_token.id.unwrap_or_default())).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

pub async fn lead_add_to_pool(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<InfoId>) -> HttpResponse {
    let db = &state.db;
    let query = form_data.0;

    if query.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "线索ID不能为空", "local"));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = lead_service::add_to_pool(&db, query.id.unwrap(), Some(jwt_token.id.unwrap_or_default())).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

pub async fn lead_claim(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<InfoId>) -> HttpResponse {
    let db = &state.db;
    let query = form_data.0;

    if query.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "线索ID不能为空", "local"));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    match lead_service::claim(&db, query.id.unwrap(), jwt_token.id.unwrap_or_default()).await {
        Ok(customer_id) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(customer_id, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn lead_convert_to_customer(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<InfoId>) -> HttpResponse {
    let db = &state.db;
    let query = form_data.0;

    if query.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "线索ID不能为空", "local"));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    match lead_service::convert_to_customer(&db, query.id.unwrap(), jwt_token.id.unwrap_or_default()).await {
        Ok(customer_id) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(customer_id, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 线索转移 ====================

/// 预览线索转移影响范围
pub async fn lead_transfer_preview(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<crate::modules::crm::service::lead_transfer_service::LeadTransferPreviewRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let _jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match lead_transfer_service::preview_transfer(db, &form_data.0).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 执行线索转移
pub async fn lead_transfer(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<crate::modules::crm::service::lead_transfer_service::LeadTransferRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let operator_id = jwt_token.id.unwrap_or_default();
    let operator_name = jwt_token.username.clone();

    match lead_transfer_service::transfer_lead(
        db,
        &form_data.0,
        operator_id,
        operator_name,
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册线索模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(lead_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/lead")
            // POST /lead/save - 新建线索
            .route(
                "/save",
                web::post()
                    .to(lead_insert)
                    .wrap(require_permission("crm:lead:create")),
            )
            // PUT /lead/update - 修改线索
            .route(
                "/update",
                web::put()
                    .to(lead_update)
                    .wrap(require_permission("crm:lead:edit")),
            )
            // DELETE /lead/bath_delete - 批量删除线索
            .route(
                "/bath_delete",
                web::delete()
                    .to(bath_delete_lead)
                    .wrap(require_permission("crm:lead:delete")),
            )
            // GET /lead/info - 线索详情
            .route(
                "/info",
                web::get()
                    .to(lead_info)
                    .wrap(require_permission("crm:lead:info")),
            )
            // GET /lead/list - 线索列表
            .route(
                "/list",
                web::get()
                    .to(lead_list)
                    .wrap(require_permission("crm:lead:list")),
            )
            // PUT /lead/update-status - 更新线索状态
            .route(
                "/update-status",
                web::put()
                    .to(lead_update_status)
                    .wrap(require_permission("crm:lead:edit")),
            )
            // PUT /lead/add-to-pool - 退回公海
            .route(
                "/add-to-pool",
                web::put()
                    .to(lead_add_to_pool)
                    .wrap(require_permission("crm:lead:edit")),
            )
            // PUT /lead/claim - 领取线索
            .route(
                "/claim",
                web::put()
                    .to(lead_claim)
                    .wrap(require_permission("crm:lead:edit")),
            )
            // POST /lead/convert-to-customer - 线索转客户
            .route(
                "/convert-to-customer",
                web::post()
                    .to(lead_convert_to_customer)
                    .wrap(require_permission("crm:lead:edit")),
            )
            // POST /lead/transfer/preview - 预览线索转移影响范围
            .route(
                "/transfer/preview",
                web::post()
                    .to(lead_transfer_preview)
                    .wrap(require_permission("crm:lead:transfer")),
            )
            // POST /lead/transfer - 执行线索转移
            .route(
                "/transfer",
                web::post()
                    .to(lead_transfer)
                    .wrap(require_permission("crm:lead:transfer")),
            ),
    );
    cfg.service(
        web::scope("/lead-pool")
            // GET /lead-pool/list - 公海线索列表
            .route(
                "/list",
                web::get()
                    .to(lead_pool_list)
                    .wrap(require_permission("crm:lead-pool:list")),
            )
            // GET /lead-pool/info - 公海线索详情
            .route(
                "/info",
                web::get()
                    .to(lead_pool_info)
                    .wrap(require_permission("crm:lead-pool:info")),
            )
            // DELETE /lead-pool/bath_delete - 批量删除公海线索
            .route(
                "/bath_delete",
                web::delete()
                    .to(bath_delete_lead_pool)
                    .wrap(require_permission("crm:lead-pool:delete")),
            ),
    );
}