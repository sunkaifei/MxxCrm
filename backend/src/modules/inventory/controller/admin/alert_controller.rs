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
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::inventory::model::alert::{AlertRuleListQuery, AlertRuleSaveRequest};
use crate::modules::inventory::service::alert_service;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::permission_guard::require_permission;

/// 预警规则列表
pub async fn rule_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    fn q<'a>(qs: &'a str, key: &str) -> Option<&'a str> {
        qs.split('&').find(|s| s.starts_with(&format!("{}=", key)))
            .and_then(|s| s.split('=').nth(1))
    }

    let query = AlertRuleListQuery {
        page_num: q(query_str, "page").and_then(|s| s.parse().ok()),
        page_size: q(query_str, "pageSize").and_then(|s| s.parse().ok()),
        product_id: q(query_str, "productId").and_then(|s| s.parse().ok()),
        warehouse_id: q(query_str, "warehouseId").and_then(|s| s.parse().ok()),
    };

    match alert_service::get_list(db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 预警规则详情
pub async fn rule_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id="))
        .and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok()))
        .unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match alert_service::get_detail(db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 创建预警规则
pub async fn rule_save(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let body = body.0;

    let form_data: AlertRuleSaveRequest = serde_json::from_value(body)?;

    let result = alert_service::create(db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 更新预警规则
pub async fn rule_update(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let body = body.0;

    let id = body.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    let form_data: AlertRuleSaveRequest = serde_json::from_value(body)?;

    let result = alert_service::update(db, id, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 删除预警规则
pub async fn rule_batch_delete(state: web::Data<AppState>, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let ids = body.get("ids").and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect::<Vec<i64>>())
        .unwrap_or_default();
    if ids.is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "请选择要删除的记录", "local")));
    }
    let result = alert_service::batch_delete(db, &ids).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/alert/rule")
            .route("/list", web::get().to(rule_list).wrap(require_permission("product:alert:list")))
            .route("/info", web::get().to(rule_info).wrap(require_permission("product:alert:list")))
            .route("/save", web::post().to(rule_save).wrap(require_permission("product:alert:edit")))
            .route("/update", web::put().to(rule_update).wrap(require_permission("product:alert:edit")))
            .route("/batch_delete", web::delete().to(rule_batch_delete).wrap(require_permission("product:alert:edit"))),
    );
}