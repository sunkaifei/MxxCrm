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
use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::model::order_item::{OrderItemDetailVO, OrderItemListQuery, OrderItemListVO, OrderItemSaveRequest, OrderItemUpdateRequest};
use crate::modules::sale::service::order_item_service;

pub async fn order_item_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<OrderItemSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = order_item_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn order_item_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<OrderItemUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "订单明细ID不能为空", "local")));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = order_item_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn bath_delete_order_item(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到删除的订单明细ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = order_item_service::batch_delete_by_ids(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

pub async fn order_item_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "订单明细ID不能为空", "local"));
    }

    match order_item_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn order_item_list(state: web::Data<AppState>, query: web::Query<OrderItemListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    match order_item_service::list(&db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册订单明细模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(order_item_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/order-item")
            // POST /sale/order-item/save - 新建订单明细
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/save",
                web::post()
                    .to(order_item_insert)
                    .wrap(require_permission("sale:order:item:save")),
            )
            // PUT /sale/order-item/update - 修改订单明细
            .route(
                "/update",
                web::put()
                    .to(order_item_update)
                    .wrap(require_permission("sale:order:item:update")),
            )
            // DELETE /sale/order-item/bath_delete - 批量删除订单明细
            .route(
                "/bath_delete",
                web::delete()
                    .to(bath_delete_order_item)
                    .wrap(require_permission("sale:order:item:delete")),
            )
            // GET /sale/order-item/info - 订单明细详情
            .route(
                "/info",
                web::get()
                    .to(order_item_info)
                    .wrap(require_permission("sale:order:item:info")),
            )
            // GET /sale/order-item/list - 订单明细列表
            .route(
                "/list",
                web::get()
                    .to(order_item_list)
                    .wrap(require_permission("sale:order:item:list")),
            ),
    );
}