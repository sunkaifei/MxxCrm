//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 发货单控制器（方案 C 试点）
//!
//! 路由表 + 权限码集中定义在本文件 `register` 函数中。
//! 修改路径、权限、HTTP 方法只需修改 `register` 函数一处。
//! Handler 函数纯净，无任何属性宏。
//!
//! ## 路由表
//!
//! | 方法   | 路径                    | 权限码              | handler | 说明       |
//! |--------|------------------------|--------------------|---------|-----------|
//! | GET    | /sale/shipment/list    | sale:shipment:list   | list    | 发货单列表 |
//! | GET    | /sale/shipment/info    | sale:shipment:list   | info    | 发货单详情 |
//! | POST   | /sale/shipment/save    | sale:shipment:create | save    | 新建发货单 |
//! | PUT    | /sale/shipment/update  | sale:shipment:edit   | update  | 修改发货单 |
//! | DELETE | /sale/shipment/delete  | sale:shipment:delete | delete  | 删除发货单 |
//! | POST   | /sale/shipment/sign    | sale:shipment:sign   | sign    | 签收发货单 |

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::InfoId;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::model::shipment::{ShipmentListQuery, ShipmentSaveRequest, ShipmentUpdateRequest};
use crate::modules::sale::service::shipment_service;
use crate::modules::system::entity::admin::Entity as Admin;
use crate::modules::system::model::edit_log::EditLogItem;
use crate::modules::system::service::edit_log_service::{
    self, BUSINESS_TYPE_SHIPMENT,
};
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::EntityTrait;
use serde_json::json;

/// 发货单字段中文标签（用于编辑日志展示）
const SHIPMENT_FIELD_LABELS: &[(&str, &str)] = &[
    ("shipmentNo", "发货单号"),
    ("shipmentDate", "发货日期"),
    ("logisticsCompany", "物流公司"),
    ("trackingNo", "物流单号"),
    ("shippingMethod", "配送方式"),
    ("receiverName", "收货人"),
    ("receiverPhone", "收货电话"),
    ("shippingAddress", "收货地址"),
    ("totalQuantity", "发货总数"),
    ("status", "发货状态"),
    ("remark", "备注"),
];

/// 发货单列表
pub async fn list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<ShipmentListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    match shipment_service::get_list(db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 发货单详情
pub async fn info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "发货单ID不能为空", "local"));
    }
    match shipment_service::get_detail(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 新建发货单
pub async fn save(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<ShipmentSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();
    let result = shipment_service::create(db, &form_data, user_id).await;

    // 业务成功后记录「创建」操作日志
    if let Ok(shipment_id) = result {
        let editor_name = match Admin::find_by_id(user_id).one(db).await { Ok(Some(admin)) => {
            admin.nick_name.or(admin.user_name)
        } _ => {
            None
        }};

        let items = form_data.items.unwrap_or_default();
        let item_summary = items.iter()
            .map(|i| format!("{}×{}", i.product_name.clone().unwrap_or_default(), i.quantity.unwrap_or(0)))
            .collect::<Vec<_>>()
            .join("，");
        let total_qty: i32 = items.iter().map(|i| i.quantity.unwrap_or(0)).sum();

        let log_items = vec![
            EditLogItem {
                field: "action".to_string(),
                field_label: "操作类型".to_string(),
                old: None,
                new: Some("新建发货单".to_string()),
            },
            EditLogItem {
                field: "totalQuantity".to_string(),
                field_label: "发货总数量".to_string(),
                old: None,
                new: Some(total_qty.to_string()),
            },
            EditLogItem {
                field: "items".to_string(),
                field_label: "发货明细".to_string(),
                old: None,
                new: Some(item_summary),
            },
        ];

        // 查询新建后的发货单号
        let business_no = if let Ok(detail) = shipment_service::get_detail(db, shipment_id).await {
            detail.shipment_no
        } else {
            None
        };

        let _ = edit_log_service::log_action(
            db,
            BUSINESS_TYPE_SHIPMENT,
            shipment_id,
            business_no,
            None,
            user_id,
            editor_name,
            log_items,
        ).await;
    }

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 修改发货单
pub async fn update(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<ShipmentUpdateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "发货单ID不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();
    let shipment_id = form_data.id.unwrap();

    // 先查 old 数据用于 diff
    let old_data = if let Ok(old_detail) = shipment_service::get_detail(db, shipment_id).await {
        serde_json::to_value(&old_detail).unwrap_or_default()
    } else {
        json!({})
    };

    let result = shipment_service::update(db, &form_data).await;

    // 业务成功后记录「修改」日志
    if result.is_ok() {
        let new_data = if let Ok(new_detail) = shipment_service::get_detail(db, shipment_id).await {
            serde_json::to_value(&new_detail).unwrap_or_default()
        } else {
            json!({})
        };

        let editor_name = match Admin::find_by_id(user_id).one(db).await { Ok(Some(admin)) => {
            admin.nick_name.or(admin.user_name)
        } _ => {
            None
        }};

        let business_no = old_data.get("shipmentNo").and_then(|v| v.as_str()).map(|s| s.to_string());

        let _ = edit_log_service::log_update(
            db,
            BUSINESS_TYPE_SHIPMENT,
            shipment_id,
            business_no,
            None,
            user_id,
            editor_name,
            &old_data,
            &new_data,
            SHIPMENT_FIELD_LABELS,
        ).await;
    }

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 删除发货单
pub async fn delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Query<InfoId>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "发货单ID不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();
    let shipment_id = item.id.unwrap();

    // 先查 old 数据用于记录删除内容
    let old_data = if let Ok(old_detail) = shipment_service::get_detail(db, shipment_id).await {
        serde_json::to_value(&old_detail).unwrap_or_default()
    } else {
        json!({})
    };

    let result = shipment_service::delete(db, shipment_id).await;

    // 业务成功后记录「删除」日志
    if result.is_ok() {
        let editor_name = match Admin::find_by_id(user_id).one(db).await { Ok(Some(admin)) => {
            admin.nick_name.or(admin.user_name)
        } _ => {
            None
        }};

        let business_no = old_data.get("shipmentNo").and_then(|v| v.as_str()).map(|s| s.to_string());
        let total_qty = old_data.get("totalQuantity").and_then(|v| v.as_i64()).unwrap_or(0);

        let log_items = vec![
            EditLogItem {
                field: "action".to_string(),
                field_label: "操作类型".to_string(),
                old: None,
                new: Some("删除发货单".to_string()),
            },
            EditLogItem {
                field: "shipmentNo".to_string(),
                field_label: "发货单号".to_string(),
                old: business_no.clone(),
                new: None,
            },
            EditLogItem {
                field: "totalQuantity".to_string(),
                field_label: "发货总数量".to_string(),
                old: Some(total_qty.to_string()),
                new: None,
            },
        ];

        let _ = edit_log_service::log_action(
            db,
            BUSINESS_TYPE_SHIPMENT,
            shipment_id,
            business_no,
            None,
            user_id,
            editor_name,
            log_items,
        ).await;
    }

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 签收发货单
pub async fn sign(state: web::Data<AppState>, item: web::Query<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "发货单ID不能为空", "local")));
    }
    let result = shipment_service::sign(db, item.id.unwrap()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

// ==================== 路由注册（单点维护）====================

/// 注册发货单模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(shipment_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/shipment")
            // GET /sale/shipment/list - 发货单列表
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/list",
                web::get()
                    .to(list)
                    .wrap(require_permission("sale:shipment:list")),
            )
            // GET /sale/shipment/info - 发货单详情
            .route(
                "/info",
                web::get()
                    .to(info)
                    .wrap(require_permission("sale:shipment:list")),
            )
            // POST /sale/shipment/save - 新建发货单
            .route(
                "/save",
                web::post()
                    .to(save)
                    .wrap(require_permission("sale:shipment:create")),
            )
            // PUT /sale/shipment/update - 修改发货单
            .route(
                "/update",
                web::put()
                    .to(update)
                    .wrap(require_permission("sale:shipment:edit")),
            )
            // DELETE /sale/shipment/delete - 删除发货单
            .route(
                "/delete",
                web::delete()
                    .to(delete)
                    .wrap(require_permission("sale:shipment:delete")),
            )
            // POST /sale/shipment/sign - 签收发货单
            .route(
                "/sign",
                web::post()
                    .to(sign)
                    .wrap(require_permission("sale:shipment:sign")),
            ),
    );
}
