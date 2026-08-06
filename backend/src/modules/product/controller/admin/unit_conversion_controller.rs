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
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::product::model::unit_conversion::{
    UnitConversionListQuery, UnitConversionSaveRequest,
};
use crate::modules::product::service::unit_conversion_service;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn unit_conversion_save(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let body = body.0;
    let form_data: UnitConversionSaveRequest = serde_json::from_value(body)?;

    let result =
        unit_conversion_service::save(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

pub async fn unit_conversion_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let body = body.0;
    let form_data: UnitConversionSaveRequest = serde_json::from_value(body)?;
    let id = req
        .query_string()
        .split('&')
        .find(|s| s.starts_with("id="))
        .and_then(|s| s.split('=').nth(1))
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    let result =
        unit_conversion_service::update(&db, id, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

pub async fn unit_conversion_batch_delete(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let ids_str = req
        .query_string()
        .split('&')
        .find(|s| s.starts_with("ids="))
        .and_then(|s| s.split('=').nth(1))
        .map(|s| s.to_string())
        .unwrap_or_default();
    let ids: Vec<i64> = ids_str
        .split(',')
        .filter_map(|s| s.parse::<i64>().ok())
        .collect();
    if ids.is_empty() {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(
                400,
                "参数无效：ids 必填",
                "local",
            )));
    }

    let result = unit_conversion_service::batch_delete_ids(&db, &ids).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

pub async fn unit_conversion_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    let query = UnitConversionListQuery {
        page_num: query_str
            .split('&')
            .find(|s| s.starts_with("page="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1),
        page_size: query_str
            .split('&')
            .find(|s| s.starts_with("pageSize="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(20),
        product_id: query_str
            .split('&')
            .find(|s| s.starts_with("productId="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.parse::<i64>().ok()),
        from_unit: query_str
            .split('&')
            .find(|s| s.starts_with("fromUnit="))
            .and_then(|s| s.split('=').nth(1))
            .map(|s| s.to_string()),
        to_unit: query_str
            .split('&')
            .find(|s| s.starts_with("toUnit="))
            .and_then(|s| s.split('=').nth(1))
            .map(|s| s.to_string()),
        status: query_str
            .split('&')
            .find(|s| s.starts_with("status="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.parse::<i32>().ok()),
    };

    match unit_conversion_service::get_list(&db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn unit_conversion_list_by_product(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let product_id = req
        .query_string()
        .split('&')
        .find(|s| s.starts_with("productId="))
        .and_then(|s| s.split('=').nth(1))
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    if product_id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "产品ID无效", "local")));
    }

    match unit_conversion_service::list_by_product(&db, product_id).await {
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
        web::scope("/unit_conversion")
            .route(
                "/save",
                web::post()
                    .to(unit_conversion_save)
                    .wrap(require_permission("product:unit:save")),
            )
            .route(
                "/update",
                web::put()
                    .to(unit_conversion_update)
                    .wrap(require_permission("product:unit:save")),
            )
            .route(
                "/bath_delete",
                web::delete()
                    .to(unit_conversion_batch_delete)
                    .wrap(require_permission("product:unit:delete")),
            )
            .route(
                "/list",
                web::get()
                    .to(unit_conversion_list)
                    .wrap(require_permission("product:unit:list")),
            )
            .route(
                "/list_by_product",
                web::get()
                    .to(unit_conversion_list_by_product)
                    .wrap(require_permission("product:unit:list")),
            ),
    );
}
