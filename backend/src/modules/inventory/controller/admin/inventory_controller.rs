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
use crate::modules::inventory::model::stock::{
    InventoryDetailVO, InventoryListData, InventoryListQuery, SafetyStockRequest,
    StockWarningQuery,
};
use crate::modules::inventory::model::stock_log::StockLogListQuery;
use crate::modules::inventory::service::inventory_service;
use crate::modules::inventory::service::stock_log_service;
use crate::modules::inventory::service::freeze_service;
use crate::modules::inventory::model::inbound::InboundSaveRequest;
use crate::modules::inventory::service::inbound_service;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::permission_guard::require_permission;

/// 从 query_string 中提取指定 key 的值
fn q<'a>(qs: &'a str, key: &str) -> Option<&'a str> {
    qs.split('&')
        .find(|s| s.starts_with(&format!("{}=", key)))
        .and_then(|s| s.split('=').nth(1))
}

pub async fn inventory_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    let query = InventoryListQuery {
        page_num: query_str.split("&").find(|s| s.starts_with("page=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        page_size: query_str.split("&").find(|s| s.starts_with("pageSize=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        product_name: query_str.split("&").find(|s| s.starts_with("productName=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
        warehouse_id: query_str.split("&").find(|s| s.starts_with("warehouseId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        low_stock: query_str.split("&").find(|s| s.starts_with("lowStock=")).and_then(|s| s.split("=").nth(1).map(|s| s == "true")),
    };

    match inventory_service::get_list(db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn inventory_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match inventory_service::get_detail(db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 库存流水查询
pub async fn inventory_log(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    let start_time = query_str.split("&").find(|s| s.starts_with("startTime="))
        .and_then(|s| s.split("=").nth(1))
        .and_then(|v| chrono::NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d").map(|d| d.and_hms_opt(0, 0, 0).unwrap()))
            .ok());
    let end_time = query_str.split("&").find(|s| s.starts_with("endTime="))
        .and_then(|s| s.split("=").nth(1))
        .and_then(|v| chrono::NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d").map(|d| d.and_hms_opt(23, 59, 59).unwrap()))
            .ok());

    let query = StockLogListQuery {
        page_num: query_str.split("&").find(|s| s.starts_with("page=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<u64>().ok())).unwrap_or(1),
        page_size: query_str.split("&").find(|s| s.starts_with("pageSize=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<u64>().ok())).unwrap_or(20),
        product_id: query_str.split("&").find(|s| s.starts_with("productId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        warehouse_id: query_str.split("&").find(|s| s.starts_with("warehouseId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())),
        change_type: query_str.split("&").find(|s| s.starts_with("changeType=")).and_then(|s| s.split("=").nth(1).map(|s| s.to_string())),
        start_time,
        end_time,
    };

    match stock_log_service::get_list(db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 库存初始化
pub async fn inventory_initial(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let body = body.0;
    let created_by = jwt_token.id.unwrap_or_default();

    let mut form_data: InboundSaveRequest = serde_json::from_value(body)?;
    form_data.inbound_type = "initial".to_string();

    match inbound_service::create_and_auto_audit(db, &form_data, created_by).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 库存冻结
pub async fn inventory_freeze(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let freeze_by = jwt_token.id.unwrap_or_default();

    let product_id = body.get("productId").and_then(|v| v.as_i64()).unwrap_or(0);
    let warehouse_id = body.get("warehouseId").and_then(|v| v.as_i64()).unwrap_or(0);
    let quantity = body.get("quantity").and_then(|v| v.as_f64()).map(|v| rust_decimal::Decimal::try_from(v).unwrap_or_default()).unwrap_or_default();
    let reason = body.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());

    if product_id <= 0 || warehouse_id <= 0 || quantity <= rust_decimal::Decimal::ZERO {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "参数无效：productId、warehouseId、quantity 必填", "local")));
    }

    match freeze_service::freeze_stock(db, product_id, warehouse_id, quantity, reason, freeze_by).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("冻结成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 库存解冻
pub async fn inventory_unfreeze(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let unfreeze_by = jwt_token.id.unwrap_or_default();

    let product_id = body.get("productId").and_then(|v| v.as_i64()).unwrap_or(0);
    let warehouse_id = body.get("warehouseId").and_then(|v| v.as_i64()).unwrap_or(0);
    let quantity = body.get("quantity").and_then(|v| v.as_f64()).map(|v| rust_decimal::Decimal::try_from(v).unwrap_or_default()).unwrap_or_default();

    if product_id <= 0 || warehouse_id <= 0 || quantity <= rust_decimal::Decimal::ZERO {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "参数无效：productId、warehouseId、quantity 必填", "local")));
    }

    match freeze_service::unfreeze_stock(db, product_id, warehouse_id, quantity, unfreeze_by).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("解冻成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 设置仓库级安全库存
pub async fn set_safety_stock(
    state: web::Data<AppState>,
    body: web::Json<SafetyStockRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let req = body.0;

    match inventory_service::set_safety_stock(db, &req).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 解析库存预警查询参数
fn parse_warning_query(query_str: &str) -> StockWarningQuery {
    StockWarningQuery {
        page_num: q(query_str, "page").and_then(|s| s.parse().ok()).unwrap_or(1),
        page_size: q(query_str, "pageSize").and_then(|s| s.parse().ok()).unwrap_or(20),
        warehouse_id: q(query_str, "warehouseId").and_then(|s| s.parse().ok()),
        days: q(query_str, "days").and_then(|s| s.parse().ok()),
    }
}

/// 低库存预警列表
pub async fn low_stock_warning(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query = parse_warning_query(req.query_string());

    match inventory_service::get_low_stock_list(db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 高库存预警列表
pub async fn high_stock_warning(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query = parse_warning_query(req.query_string());

    match inventory_service::get_high_stock_list(db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 呆滞库存预警
pub async fn obsolete_stock_warning(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query = parse_warning_query(req.query_string());

    match inventory_service::get_obsolete_stock_list(db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 库存调整
pub async fn inventory_adjust(state: web::Data<AppState>, req: HttpRequest, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let operator_id = jwt_token.id.unwrap_or_default();

    let product_id = body.get("productId").and_then(|v| v.as_i64()).unwrap_or(0);
    let warehouse_id = body.get("warehouseId").and_then(|v| v.as_i64()).unwrap_or(0);
    let quantity = body.get("quantity").and_then(|v| v.as_f64()).map(|v| rust_decimal::Decimal::try_from(v).unwrap_or_default()).unwrap_or_default();
    let reason = body.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());

    if product_id <= 0 || warehouse_id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "参数无效：productId、warehouseId 必填", "local")));
    }
    if quantity < rust_decimal::Decimal::ZERO {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "调整后的库存数量不能为负数", "local")));
    }

    match inventory_service::adjust_stock(db, product_id, warehouse_id, quantity, operator_id, reason).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("调整成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 统一预警列表（前端预警页面使用）
pub async fn alert_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    let product_name = q(query_str, "productName").map(|s| s.to_string());
    let alert_type = q(query_str, "alertType").map(|s| s.to_string());
    let page = q(query_str, "page").and_then(|s| s.parse().ok()).unwrap_or(1);
    let page_size = q(query_str, "pageSize").and_then(|s| s.parse().ok()).unwrap_or(20);

    match inventory_service::get_alert_list(db, product_name, alert_type, page, page_size).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/inventory")
            .route("/list", web::get().to(inventory_list).wrap(require_permission("product:inventory:list")))
            .route("/info", web::get().to(inventory_info).wrap(require_permission("product:inventory:view")))
            .route("/log", web::get().to(inventory_log).wrap(require_permission("product:inventory:view")))
            .route("/initial", web::post().to(inventory_initial).wrap(require_permission("product:inbound:create")))
            .route("/freeze", web::post().to(inventory_freeze).wrap(require_permission("product:inventory:freeze")))
            .route("/unfreeze", web::post().to(inventory_unfreeze).wrap(require_permission("product:inventory:freeze")))
            .route("/set_safety_stock", web::put().to(set_safety_stock).wrap(require_permission("product:inventory:update")))
            .route("/adjust", web::post().to(inventory_adjust).wrap(require_permission("product:inventory:adjust")))
            .route("/warning/low", web::get().to(low_stock_warning).wrap(require_permission("product:inventory:list")))
            .route("/warning/high", web::get().to(high_stock_warning).wrap(require_permission("product:inventory:list")))
            .route("/warning/obsolete", web::get().to(obsolete_stock_warning).wrap(require_permission("product:inventory:list"))),
    );
    // 统一预警列表（由 alert_controller.rs 提供完整的预警规则 CRUD）
    cfg.service(
        web::scope("/alert")
            .route("/list", web::get().to(alert_list).wrap(require_permission("product:inventory:list"))),
    );
}