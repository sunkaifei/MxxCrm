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
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::model::order::{OrderApprovalDetailVO, OrderApprovalReq, OrderListQuery, OrderSaveRequest, OrderStatusUpdateRequest, OrderUpdateRequest};
use crate::modules::sale::service::order_service;
use crate::modules::system::service::edit_log_service;
use crate::modules::system::entity::admin::Entity as Admin;
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::EntityTrait;
use serde_json::json;

const ORDER_FIELD_LABELS: &[(&str, &str)] = &[
    ("title", "订单标题"),
    ("orderNo", "订单编号"),
    ("orderType", "订单类型"),
    ("customerName", "客户名称"),
    ("contactName", "联系人"),
    ("currency", "币种"),
    ("orderDate", "订单日期"),
    ("deliveryDate", "交货日期"),
    ("paymentMethod", "支付方式"),
    ("paymentDueDate", "付款期限"),
    ("shippingMethod", "发货方式"),
    ("receiverName", "收货人"),
    ("receiverPhone", "收货电话"),
    ("shippingAddress", "收货地址"),
    ("billingAddress", "账单地址"),
    ("remark", "备注"),
    ("productAmount", "商品金额"),
    ("discountAmount", "折扣金额"),
    ("shippingFee", "运费"),
    ("taxAmount", "税额"),
    ("otherFee", "其他费用"),
    ("totalAmount", "订单总额"),
    ("paidAmount", "已付金额"),
    ("unpaidAmount", "未付金额"),
];

pub async fn order_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<OrderSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();
    let result = order_service::insert(db, &form_data, user_id).await;

    if let Ok(order_id) = result {
        let new_data = if let Ok(new_detail) = order_service::get_detail(db, order_id).await {
            serde_json::to_value(&new_detail).unwrap_or_default()
        } else {
            json!({})
        };

        let editor_name = match Admin::find_by_id(user_id).one(db).await { Ok(Some(admin)) => {
            admin.nick_name.or(admin.user_name)
        } _ => {
            None
        }};

        let business_no = new_data.get("orderNo").and_then(|v| v.as_str()).map(|s| s.to_string());
        let business_title = new_data.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());

        let old_data = json!({});

        let _ = edit_log_service::log_update(
            db,
            2,
            order_id,
            business_no,
            business_title,
            user_id,
            editor_name,
            &old_data,
            &new_data,
            ORDER_FIELD_LABELS,
        ).await;
    }

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn order_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<OrderUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "订单ID不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();
    let order_id = form_data.id.unwrap();

    let old_data = if let Ok(old_detail) = order_service::get_detail(db, order_id).await {
        serde_json::to_value(&old_detail).unwrap_or_default()
    } else {
        json!({})
    };

    let result = order_service::update(db, &form_data, user_id).await;

    if result.is_ok() {
        let new_data = if let Ok(new_detail) = order_service::get_detail(db, order_id).await {
            serde_json::to_value(&new_detail).unwrap_or_default()
        } else {
            json!({})
        };

        let editor_name = match Admin::find_by_id(user_id).one(db).await { Ok(Some(admin)) => {
            admin.nick_name.or(admin.user_name)
        } _ => {
            None
        }};

        let business_no = old_data.get("orderNo").and_then(|v| v.as_str()).map(|s| s.to_string());
        let business_title = old_data.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());

        let _ = edit_log_service::log_update(
            db,
            2,
            order_id,
            business_no,
            business_title,
            user_id,
            editor_name,
            &old_data,
            &new_data,
            ORDER_FIELD_LABELS,
        ).await;
    }

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn order_update_status(state: web::Data<AppState>, form_data: web::Json<OrderStatusUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "订单ID不能为空", "local")));
    }
    if form_data.order_status.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "订单状态不能为空", "local")));
    }
    let result = order_service::update_status(db, &form_data).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn batch_delete_order(state: web::Data<AppState>, form_data: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = form_data.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids: Vec<i64> = ids_vec.into_iter().filter_map(|id| id.and_then(|s| s.parse().ok())).collect();
        let result = order_service::batch_delete(db, &ids).await;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn order_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "订单ID不能为空", "local"));
    }
    match order_service::get_detail(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn order_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<OrderListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    match order_service::get_list(db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ========== 订单审批 ==========

/// 提交审批
pub async fn order_submit(state: web::Data<AppState>, req: HttpRequest, item: web::Json<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "订单ID不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match order_service::submit_order(db, item.id.unwrap(), jwt_token.id.unwrap_or_default(), &jwt_token.username.unwrap_or_default()).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 审批通过
pub async fn order_approve(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<OrderApprovalReq>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match order_service::approve_order(db, form_data.order_id, jwt_token.id.unwrap_or_default(), &jwt_token.username.unwrap_or_default(), form_data.reason).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 驳回
pub async fn order_reject(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<OrderApprovalReq>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match order_service::reject_order(db, form_data.order_id, jwt_token.id.unwrap_or_default(), &jwt_token.username.unwrap_or_default(), form_data.reason).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 审批详情
pub async fn order_approval_detail(state: web::Data<AppState>, path: web::Path<i64>) -> HttpResponse {
    let db = &state.db;
    let order_id = path.into_inner();
    match order_service::get_approval_detail(db, order_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 从订单创建合同
pub async fn order_create_contract(state: web::Data<AppState>, req: HttpRequest, item: web::Json<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "订单ID不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match order_service::create_contract_from_order(db, item.id.unwrap(), jwt_token.id.unwrap_or_default()).await {
        Ok(contract_id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(contract_id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册订单模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(order_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/order")
            // POST /sale/order/save - 新建订单
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/save",
                web::post()
                    .to(order_insert)
                    .wrap(require_permission("sale:order:save")),
            )
            // PUT /sale/order/update - 修改订单
            .route(
                "/update",
                web::put()
                    .to(order_update)
                    .wrap(require_permission("sale:order:update")),
            )
            // PUT /sale/order/updateStatus - 修改订单状态
            .route(
                "/updateStatus",
                web::put()
                    .to(order_update_status)
                    .wrap(require_permission("sale:order:update")),
            )
            // POST /sale/order/batch-delete - 批量删除订单
            .route(
                "/batch-delete",
                web::post()
                    .to(batch_delete_order)
                    .wrap(require_permission("sale:order:delete")),
            )
            // GET /sale/order/info - 订单详情
            .route(
                "/info",
                web::get()
                    .to(order_info)
                    .wrap(require_permission("sale:order:list")),
            )
            // GET /sale/order/list - 订单列表
            .route(
                "/list",
                web::get()
                    .to(order_list)
                    .wrap(require_permission("sale:order:list")),
            )
            // POST /sale/order/submit - 提交审批
            .route(
                "/submit",
                web::post()
                    .to(order_submit)
                    .wrap(require_permission("sale:order:update")),
            )
            // POST /sale/order/approve - 审批通过
            .route(
                "/approve",
                web::post()
                    .to(order_approve)
                    .wrap(require_permission("sale:order:update")),
            )
            // POST /sale/order/reject - 驳回
            .route(
                "/reject",
                web::post()
                    .to(order_reject)
                    .wrap(require_permission("sale:order:update")),
            )
            // GET /sale/order/approval-detail/{order_id} - 审批详情
            .route(
                "/approval-detail/{order_id}",
                web::get()
                    .to(order_approval_detail)
                    .wrap(require_permission("sale:order:list")),
            )
            // POST /sale/order/create-contract - 从订单创建合同
            .route(
                "/create-contract",
                web::post()
                    .to(order_create_contract)
                    .wrap(require_permission("sale:order:update")),
            ),
    );
}
