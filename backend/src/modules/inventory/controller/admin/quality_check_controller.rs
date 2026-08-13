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
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::inventory::model::quality_check::{
    QualityCheckListQuery, QualityCheckResultRequest, QualityCheckSaveRequest,
};
use crate::modules::inventory::service::quality_check_service;
use actix_web::{web, HttpRequest, HttpResponse};

/// 从 query_string 中提取指定 key 的值
fn q<'a>(qs: &'a str, key: &str) -> Option<&'a str> {
    qs.split('&')
        .find(|s| s.starts_with(&format!("{}=", key)))
        .and_then(|s| s.split('=').nth(1))
}

/// 创建质检单
pub async fn quality_check_save(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<QualityCheckSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = body.0;

    let result =
        quality_check_service::create(db, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// 更新质检单
pub async fn quality_check_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<QualityCheckSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = body.0;
    let id = form_data.id.unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "质检单ID无效", "local")));
    }

    let result =
        quality_check_service::update(db, id, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// 录入质检结果
pub async fn quality_check_input(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
    body: web::Json<QualityCheckResultRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();
    if id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "质检单ID无效", "local")));
    }

    let result =
        quality_check_service::check(db, id, &body.0, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// 批量删除质检单
pub async fn quality_check_batch_delete(
    state: web::Data<AppState>,
    item: web::Json<BathDeleteIdRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let ids: Vec<i64> = item.0.ids.unwrap_or_default()
        .into_iter()
        .flatten()
        .filter_map(|s| s.parse().ok())
        .collect();
    if ids.is_empty() {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "请选择要删除的记录", "local")));
    }
    let result = quality_check_service::batch_delete(db, &ids).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// 质检单详情
pub async fn quality_check_info(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let id = q(req.query_string(), "id")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "质检单ID无效", "local")));
    }

    match quality_check_service::get_detail(db, id).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 质检单列表
pub async fn quality_check_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let qs = req.query_string();

    let query = QualityCheckListQuery {
        page_num: q(qs, "page").and_then(|s| s.parse().ok()).unwrap_or(1),
        page_size: q(qs, "pageSize").and_then(|s| s.parse().ok()).unwrap_or(20),
        check_no: q(qs, "checkNo").map(|s| s.to_string()),
        warehouse_id: q(qs, "warehouseId").and_then(|s| s.parse().ok()),
        product_id: q(qs, "productId").and_then(|s| s.parse().ok()),
        check_result: q(qs, "checkResult").and_then(|s| s.parse().ok()),
        status: q(qs, "status").and_then(|s| s.parse().ok()),
    };

    match quality_check_service::get_list(db, &query).await {
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
        web::scope("/inventory/quality_check")
            .route("/save", web::post().to(quality_check_save).wrap(require_permission("product:quality:save")))
            .route("/update", web::put().to(quality_check_update).wrap(require_permission("product:quality:save")))
            .route("/check/{id}", web::put().to(quality_check_input).wrap(require_permission("product:quality:save")))
            .route("/bath_delete", web::delete().to(quality_check_batch_delete).wrap(require_permission("product:quality:delete")))
            .route("/info", web::get().to(quality_check_info).wrap(require_permission("product:quality:list")))
            .route("/list", web::get().to(quality_check_list).wrap(require_permission("product:quality:list"))),
    );
}
