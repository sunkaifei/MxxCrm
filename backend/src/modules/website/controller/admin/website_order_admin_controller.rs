//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::model::website_order::{OrderListQuery, OrderUpdateRequest, ShipRequest};
use crate::modules::website::model::website_delivery::DeliveryListQuery;
use crate::modules::website::service::{website_delivery_service, website_order_service};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use actix_web::{web, HttpRequest, HttpResponse};

// ==================== 订单管理 ====================

/// GET /website_order/list - 订单列表
pub async fn order_list(
    state: web::Data<AppState>,
    query: web::Query<OrderListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match website_order_service::admin_order_list(db, query.into_inner()).await {
        Ok(page) => Ok(HttpResponse::Ok().json(page)),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /website_order/detail/{id} - 订单详情
pub async fn order_detail(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match website_order_service::admin_order_detail(db, id.into_inner()).await {
        Ok(vo) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(vo, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// PUT /website_order/update/{id} - 更新订单（卖家备注等）
pub async fn order_update(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<OrderUpdateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = website_order_service::admin_update_order(db, id.into_inner(), body.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// POST /website_order/ship/{id} - 订单发货
pub async fn order_ship(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
    body: web::Json<ShipRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let shipper_id = jwt_token.id.unwrap_or_default();
    let shipper_name = jwt_token.username.unwrap_or_else(|| "管理员".to_string());

    let mut ship_req: ShipRequest = body.into_inner();
    // 注入订单ID/订单号（也可由前端传）
    use crate::modules::website::model::website_delivery::DeliveryCreateRequest;
    let order_id = id.into_inner();
    let order = crate::modules::website::model::website_order::WebsiteOrderModel::find_by_id(db, order_id)
        .await
        .map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?
        .ok_or_else(|| crate::core::errors::error::Error::from("订单不存在"))?;
    let order_no = Some(order.order_no.clone());
    let create_req = DeliveryCreateRequest {
        order_id,
        order_no,
        delivery_no: ship_req.delivery_no.clone(),
        delivery_company: ship_req.delivery_company.clone(),
        delivery_type: ship_req.delivery_type,
        item_count: None,
        remark: ship_req.remark.take(),
    };
    let result = website_delivery_service::ship(db, create_req, shipper_id, shipper_name).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// DELETE /website_order/batch_delete - 批量删除订单
pub async fn order_batch_delete(
    state: web::Data<AppState>,
    item: web::Json<BathDeleteIdRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let result = website_order_service::admin_batch_delete_orders(db, ids).await;
        Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

// ==================== 发货单管理 ====================

/// GET /website_order/delivery/list - 发货单列表
pub async fn delivery_list(
    state: web::Data<AppState>,
    query: web::Query<DeliveryListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match website_delivery_service::admin_list(db, query.into_inner()).await {
        Ok(page) => Ok(HttpResponse::Ok().json(page)),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /website_order/delivery/order/{order_id} - 按订单查询发货单
pub async fn delivery_by_order(
    state: web::Data<AppState>,
    order_id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match website_delivery_service::find_by_order_id(db, order_id.into_inner()).await {
        Ok(list) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(list, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册 ====================

/// 注册网站订单/发货管理模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/website_order")
            .route("/list", web::get().to(order_list).wrap(require_permission("website:order:list")))
            .route("/detail/{id}", web::get().to(order_detail).wrap(require_permission("website:order:view")))
            .route("/update/{id}", web::put().to(order_update).wrap(require_permission("website:order:update")))
            .route("/ship/{id}", web::post().to(order_ship).wrap(require_permission("website:order:ship")))
            .route("/batch_delete", web::delete().to(order_batch_delete).wrap(require_permission("website:order:delete")))
            .route("/delivery/list", web::get().to(delivery_list).wrap(require_permission("website:delivery:list")))
            .route("/delivery/order/{order_id}", web::get().to(delivery_by_order).wrap(require_permission("website:delivery:view"))),
    );
}
