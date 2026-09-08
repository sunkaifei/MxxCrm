use actix_web::{web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::crm::service::customer_360_service;

/// 完整 360 视图
pub async fn view(state: web::Data<AppState>, query: web::Query<serde_json::Value>) -> HttpResponse {
    let db = &state.db;
    let customer_id = query.get("customerId")
        .or_else(|| query.get("customer_id"))
        .and_then(|v| v.as_i64());
    if customer_id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "客户ID不能为空", "local"));
    }
    match customer_360_service::get_360_view(db, customer_id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 仅统计汇总
pub async fn summary(state: web::Data<AppState>, query: web::Query<serde_json::Value>) -> HttpResponse {
    let db = &state.db;
    let customer_id = query.get("customerId")
        .or_else(|| query.get("customer_id"))
        .and_then(|v| v.as_i64());
    if customer_id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "客户ID不能为空", "local"));
    }
    match customer_360_service::get_summary(db, customer_id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 注册客户 360 模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/crm/customer-360")
            .route("/view", web::get().to(view).wrap(require_permission("crm:customer:view")))
            .route("/summary", web::get().to(summary).wrap(require_permission("crm:customer:view"))),
    );
}
