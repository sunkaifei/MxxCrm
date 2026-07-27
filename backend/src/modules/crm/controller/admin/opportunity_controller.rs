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
use crate::modules::crm::model::opportunity::{OpportunityDetailVO, OpportunityListQuery, OpportunityListVO, OpportunitySaveRequest, OpportunityUpdateRequest};
use crate::modules::crm::service::opportunity_service;

pub async fn opportunity_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<OpportunitySaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = opportunity_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

pub async fn opportunity_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<OpportunityUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "商机ID不能为空", "local")));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = opportunity_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

pub async fn bath_delete_opportunity(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到删除的商机ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = opportunity_service::batch_delete_by_ids(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

pub async fn opportunity_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "商机ID不能为空", "local"));
    }

    match opportunity_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn opportunity_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<OpportunityListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();

    match opportunity_service::list(&db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn opportunity_convert_to_quotation(state: web::Data<AppState>, req: HttpRequest, item: web::Json<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "商机ID不能为空", "local"));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();
    let opp_id = item.id.unwrap();

    match opportunity_service::convert_to_quotation(&db, opp_id, user_id).await {
        Ok(quotation_id) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(quotation_id, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /opportunity/convert_to_order - 商机直接转订单（简易流程模式 B）
pub async fn opportunity_convert_to_order(state: web::Data<AppState>, req: HttpRequest, item: web::Json<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "商机ID不能为空", "local"));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();
    let opp_id = item.id.unwrap();

    match opportunity_service::convert_to_order(&db, opp_id, user_id).await {
        Ok(order_id) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(order_id, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册商机模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(opportunity_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/opportunity")
            // POST /opportunity/save - 新建商机
            .route(
                "/save",
                web::post()
                    .to(opportunity_insert)
                    .wrap(require_permission("crm:opportunity:save")),
            )
            // PUT /opportunity/update - 修改商机
            .route(
                "/update",
                web::put()
                    .to(opportunity_update)
                    .wrap(require_permission("crm:opportunity:update")),
            )
            // DELETE /opportunity/bath_delete - 批量删除商机
            .route(
                "/bath_delete",
                web::delete()
                    .to(bath_delete_opportunity)
                    .wrap(require_permission("crm:opportunity:delete")),
            )
            // GET /opportunity/info - 商机详情
            .route(
                "/info",
                web::get()
                    .to(opportunity_info)
                    .wrap(require_permission("crm:opportunity:info")),
            )
            // GET /opportunity/list - 商机列表
            .route(
                "/list",
                web::get()
                    .to(opportunity_list)
                    .wrap(require_permission("crm:opportunity:list")),
            )
            // POST /opportunity/convert_to_quotation - 商机转报价单
            .route(
                "/convert_to_quotation",
                web::post()
                    .to(opportunity_convert_to_quotation)
                    .wrap(require_permission("crm:opportunity:update")),
            )
            // POST /opportunity/convert_to_order - 商机直接转订单（简易流程）
            .route(
                "/convert_to_order",
                web::post()
                    .to(opportunity_convert_to_order)
                    .wrap(require_permission("crm:opportunity:update")),
            ),
    );
}