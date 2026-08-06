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
use crate::core::web::base_controller::get_user;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::purchase::model::purchase_return::{PurchaseReturnListQuery, PurchaseReturnSaveRequest};
use crate::modules::purchase::service::purchase_return_service;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn save(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<PurchaseReturnSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;

    let result = purchase_return_service::insert(db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

pub async fn update(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<PurchaseReturnSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;

    let result = purchase_return_service::update(db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

pub async fn batch_delete(
    state: web::Data<AppState>,
    ids: web::Json<Vec<i64>>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = purchase_return_service::batch_delete(db, &ids.0).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

pub async fn info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req
        .query_string()
        .split("&")
        .find(|s| s.starts_with("id="))
        .and_then(|s| s.split("=").nth(1))
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    if id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match purchase_return_service::get_info(db, id).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<PurchaseReturnListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;

    match purchase_return_service::get_list(db, &query.into_inner()).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/purchase/return")
            .route(
                "/save",
                web::post()
                    .to(save)
                    .wrap(require_permission("purchase:return:save")),
            )
            .route(
                "/update",
                web::put()
                    .to(update)
                    .wrap(require_permission("purchase:return:update")),
            )
            .route(
                "/bath_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("purchase:return:delete")),
            )
            .route(
                "/info",
                web::get()
                    .to(info)
                    .wrap(require_permission("purchase:return:list")),
            )
            .route(
                "/list",
                web::get()
                    .to(list)
                    .wrap(require_permission("purchase:return:list")),
            ),
    );
}