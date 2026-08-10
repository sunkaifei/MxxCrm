use actix_web::{web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::service::online_payment_service;

/// 创建在线支付
pub async fn create(
    state: web::Data<AppState>,
    form_data: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let order_id = form_data.get("orderId")
        .or_else(|| form_data.get("order_id"))
        .and_then(|v| v.as_i64());
    let channel = form_data.get("channel")
        .and_then(|v| v.as_i64());
    let customer_id = form_data.get("customerId")
        .or_else(|| form_data.get("customer_id"))
        .and_then(|v| v.as_i64());

    if order_id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "订单ID不能为空", "local"));
    }
    if channel.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "支付渠道不能为空", "local"));
    }

    match online_payment_service::create_payment(db, order_id.unwrap(), channel.unwrap() as i32, customer_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 查询支付详情
pub async fn info(state: web::Data<AppState>, query: web::Query<serde_json::Value>) -> HttpResponse {
    let db = &state.db;
    let id = query.get("id").and_then(|v| v.as_i64());
    if id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "支付记录ID不能为空", "local"));
    }
    match online_payment_service::get_payment_info(db, id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 按订单查询支付记录
pub async fn by_order(state: web::Data<AppState>, query: web::Query<serde_json::Value>) -> HttpResponse {
    let db = &state.db;
    let order_id = query.get("orderId")
        .or_else(|| query.get("order_id"))
        .and_then(|v| v.as_i64());
    if order_id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "订单ID不能为空", "local"));
    }
    match online_payment_service::get_payment_by_order(db, order_id.unwrap()).await {
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

/// 主动查询支付状态
pub async fn query_status(
    state: web::Data<AppState>,
    form_data: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let payment_no = form_data.get("paymentNo")
        .or_else(|| form_data.get("payment_no"))
        .and_then(|v| v.as_str());
    if payment_no.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "支付流水号不能为空", "local"));
    }
    match online_payment_service::query_payment_status(db, payment_no.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 支付回调（开放接口，无权限校验）
pub async fn callback(
    state: web::Data<AppState>,
    form_data: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let payment_no = form_data.get("paymentNo")
        .or_else(|| form_data.get("payment_no"))
        .or_else(|| form_data.get("out_trade_no"))
        .and_then(|v| v.as_str());
    let channel_trade_no = form_data.get("channelTradeNo")
        .or_else(|| form_data.get("transaction_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let callback_data = form_data.get("rawData")
        .or_else(|| form_data.get("raw_data"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if payment_no.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "支付流水号不能为空", "local"));
    }
    match online_payment_service::handle_callback(
        db,
        payment_no.unwrap(),
        channel_trade_no,
        callback_data,
    ).await {
        Ok(success) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(success, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 注册在线支付模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/online-payment")
            .route("/create", web::post().to(create).wrap(require_permission("sale:online-payment:create")))
            .route("/info", web::get().to(info).wrap(require_permission("sale:online-payment:list")))
            .route("/by-order", web::get().to(by_order).wrap(require_permission("sale:online-payment:list")))
            .route("/query-status", web::post().to(query_status).wrap(require_permission("sale:online-payment:list")))
            .route("/callback", web::post().to(callback)),
    );
}
