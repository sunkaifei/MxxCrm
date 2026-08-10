use actix_web::{web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::service::logistics_service;

/// 查询轨迹详情
pub async fn info(state: web::Data<AppState>, query: web::Query<serde_json::Value>) -> HttpResponse {
    let db = &state.db;
    let id = query.get("id").and_then(|v| v.as_i64());
    if id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "追踪记录ID不能为空", "local"));
    }
    match logistics_service::query_tracking(db, id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 按发货单查询轨迹
pub async fn by_shipment(state: web::Data<AppState>, query: web::Query<serde_json::Value>) -> HttpResponse {
    let db = &state.db;
    let shipment_id = query.get("shipmentId")
        .or_else(|| query.get("shipment_id"))
        .and_then(|v| v.as_i64());
    if shipment_id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "发货单ID不能为空", "local"));
    }
    match logistics_service::get_tracking_by_shipment(db, shipment_id.unwrap()).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 手动刷新轨迹
pub async fn refresh(
    state: web::Data<AppState>,
    form_data: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let id = form_data.get("id").and_then(|v| v.as_i64());
    if id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "追踪记录ID不能为空", "local"));
    }
    match logistics_service::poll_tracking(db, id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 创建追踪记录
pub async fn create(
    state: web::Data<AppState>,
    form_data: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let shipment_id = form_data.get("shipmentId")
        .or_else(|| form_data.get("shipment_id"))
        .and_then(|v| v.as_i64());
    let tracking_no = form_data.get("trackingNo")
        .or_else(|| form_data.get("tracking_no"))
        .and_then(|v| v.as_str());
    let company_code = form_data.get("companyCode")
        .or_else(|| form_data.get("company_code"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let company_name = form_data.get("companyName")
        .or_else(|| form_data.get("company_name"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if shipment_id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "发货单ID不能为空", "local"));
    }
    if tracking_no.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "物流单号不能为空", "local"));
    }

    match logistics_service::create_tracking(
        db,
        shipment_id.unwrap(),
        tracking_no.unwrap().to_string(),
        company_code,
        company_name,
    ).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 注册物流追踪模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/logistics")
            .route("/info", web::get().to(info).wrap(require_permission("sale:logistics:list")))
            .route("/by-shipment", web::get().to(by_shipment).wrap(require_permission("sale:logistics:list")))
            .route("/refresh", web::post().to(refresh).wrap(require_permission("sale:logistics:update")))
            .route("/create", web::post().to(create).wrap(require_permission("sale:logistics:save"))),
    );
}
