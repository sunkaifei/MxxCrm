//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 汇率管理控制器
//!
//! ## 路由表
//!
//! | 方法   | 路径                            | 权限码              | handler  | 说明                          |
//! |--------|--------------------------------|---------------------|----------|-------------------------------|
//! | POST   | /system/exchange-rate/sync     | system:config:save  | sync     | 手动同步ECB汇率                |
//! | GET    | /system/exchange-rate/latest   | system:config:list  | latest   | 查询最新汇率                   |
//! | GET    | /system/exchange-rate/convert  | system:config:list  | convert  | 金额转换                       |
//!

use actix_web::{web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::service::exchange_rate_service;

/// 最新汇率查询参数
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestRateQuery {
    pub from_currency: Option<String>,
    pub to_currency: Option<String>,
}

/// 金额转换查询参数
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertQuery {
    pub amount: Option<String>,
    pub from_currency: Option<String>,
    pub to_currency: Option<String>,
}

/// 手动同步 ECB 汇率
pub async fn sync(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match exchange_rate_service::fetch_and_save_rates(db).await {
        Ok(count) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(serde_json::json!({ "syncedCount": count }), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 查询最新汇率
pub async fn latest(state: web::Data<AppState>, query: web::Query<LatestRateQuery>) -> HttpResponse {
    let db = &state.db;
    let from = match &query.from_currency {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "源币种(from_currency)不能为空", "local")),
    };
    let to = match &query.to_currency {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "目标币种(to_currency)不能为空", "local")),
    };
    match exchange_rate_service::get_latest_rate(db, &from, &to).await {
        Ok(rate) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(serde_json::json!({
                "fromCurrency": from,
                "toCurrency": to,
                "rate": rate.to_string()
            }), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 金额转换
pub async fn convert(state: web::Data<AppState>, query: web::Query<ConvertQuery>) -> HttpResponse {
    let db = &state.db;
    let amount_str = match &query.amount {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "金额(amount)不能为空", "local")),
    };
    let amount = match rust_decimal::Decimal::from_str_exact(&amount_str) {
        Ok(d) => d,
        Err(e) => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &format!("金额格式错误: {}", e), "local")),
    };
    let from = match &query.from_currency {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "源币种(from_currency)不能为空", "local")),
    };
    let to = match &query.to_currency {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "目标币种(to_currency)不能为空", "local")),
    };
    match exchange_rate_service::convert_amount(db, amount, &from, &to).await {
        Ok(converted) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(serde_json::json!({
                "fromCurrency": from,
                "toCurrency": to,
                "originalAmount": amount.to_string(),
                "convertedAmount": converted.to_string()
            }), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 注册汇率管理模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/system/exchange-rate")
            .route("/sync", web::post().to(sync).wrap(require_permission("system:config:save")))
            .route("/latest", web::get().to(latest).wrap(require_permission("system:config:list")))
            .route("/convert", web::get().to(convert).wrap(require_permission("system:config:list"))),
    );
}
