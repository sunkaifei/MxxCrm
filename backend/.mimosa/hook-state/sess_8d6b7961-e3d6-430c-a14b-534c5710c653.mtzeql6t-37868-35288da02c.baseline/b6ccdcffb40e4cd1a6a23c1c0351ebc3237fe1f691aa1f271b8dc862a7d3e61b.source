use actix_web::{web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::statistics::service::subscription_stats_service;

/// 月度经常性收入（MRR）
pub async fn mrr(state: web::Data<AppState>, query: web::Query<serde_json::Value>) -> HttpResponse {
    let db = &state.db;
    let year = query.get("year").and_then(|v| v.as_i64()).map(|v| v as i32);
    let month = query.get("month").and_then(|v| v.as_i64()).map(|v| v as u32);
    match subscription_stats_service::get_mrr(db, year, month).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 年度经常性收入（ARR）
pub async fn arr(state: web::Data<AppState>, query: web::Query<serde_json::Value>) -> HttpResponse {
    let db = &state.db;
    let year = query.get("year").and_then(|v| v.as_i64()).map(|v| v as i32);
    match subscription_stats_service::get_arr(db, year).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 流失率
pub async fn churn_rate(state: web::Data<AppState>, query: web::Query<serde_json::Value>) -> HttpResponse {
    let db = &state.db;
    let year = query.get("year").and_then(|v| v.as_i64()).map(|v| v as i32);
    let month = query.get("month").and_then(|v| v.as_i64()).map(|v| v as u32);
    match subscription_stats_service::get_churn_rate(db, year, month).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 续约率
pub async fn renewal_rate(state: web::Data<AppState>, query: web::Query<serde_json::Value>) -> HttpResponse {
    let db = &state.db;
    let year = query.get("year").and_then(|v| v.as_i64()).map(|v| v as i32);
    let month = query.get("month").and_then(|v| v.as_i64()).map(|v| v as u32);
    match subscription_stats_service::get_renewal_rate(db, year, month).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 订阅概览
pub async fn overview(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match subscription_stats_service::get_subscription_overview(db).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 订阅趋势
pub async fn trend(state: web::Data<AppState>, query: web::Query<serde_json::Value>) -> HttpResponse {
    let db = &state.db;
    let months = query.get("months").and_then(|v| v.as_i64());
    match subscription_stats_service::get_subscription_trend(db, months).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 注册订阅统计模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/statistics/subscription")
            .route("/mrr", web::get().to(mrr).wrap(require_permission("statistics:subscription:view")))
            .route("/arr", web::get().to(arr).wrap(require_permission("statistics:subscription:view")))
            .route("/churn-rate", web::get().to(churn_rate).wrap(require_permission("statistics:subscription:view")))
            .route("/renewal-rate", web::get().to(renewal_rate).wrap(require_permission("statistics:subscription:view")))
            .route("/overview", web::get().to(overview).wrap(require_permission("statistics:subscription:view")))
            .route("/trend", web::get().to(trend).wrap(require_permission("statistics:subscription:view"))),
    );
}
