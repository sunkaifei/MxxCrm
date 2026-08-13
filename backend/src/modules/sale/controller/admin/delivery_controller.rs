//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 虚拟商品交付控制器
//!
//! ## 路由表
//!
//! | 方法   | 路径                          | 权限码                  | handler     | 说明              |
//! |--------|------------------------------|------------------------|-------------|------------------|
//! | GET    | /sale/delivery/list          | sale:delivery:list     | list        | 交付记录列表       |
//! | GET    | /sale/delivery/info          | sale:delivery:list     | info        | 交付详情（脱敏）   |
//! | POST   | /sale/delivery/save          | sale:delivery:save     | save        | 手动录入交付       |
//! | POST   | /sale/delivery/resend        | sale:delivery:save     | resend      | 重发通知           |
//! | PUT    | /sale/delivery/update        | sale:delivery:update   | update      | 修改状态（撤销等） |
//! | GET    | /sale/delivery/view-full     | sale:delivery:view     | view_full   | 查看完整内容       |
//!

use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::entity::common::InfoId;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::model::order_delivery::{DeliveryModel, DeliveryListQuery, DeliverySaveRequest};
use crate::modules::sale::service::delivery_service;

/// 交付记录列表
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<DeliveryListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    match delivery_service::get_list(db, &query).await {
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

/// 交付详情（脱敏版）
pub async fn info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "交付记录ID不能为空", "local"));
    }
    match delivery_service::get_detail(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 查看完整卡密（需 sale:delivery:view 二次权限）
pub async fn view_full(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "交付记录ID不能为空", "local"));
    }
    match delivery_service::view_full(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 手动录入交付记录
pub async fn save(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<DeliverySaveRequest>,
) -> HttpResponse {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match delivery_service::create(db, form_data.0, user_id).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 修改状态（如撤销）
pub async fn update(
    state: web::Data<AppState>,
    form_data: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let id = form_data.get("id").and_then(|v| v.as_i64());
    let status = form_data.get("status").and_then(|v| v.as_i64());
    if id.is_none() || status.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "id 和 status 不能为空", "local"));
    }
    match delivery_service::update_status(db, id.unwrap(), status.unwrap() as i32).await {
        Ok(rows) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(rows, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 重发通知
pub async fn resend(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "ID不能为空", "local"));
    }
    match delivery_service::resend_notification(db, item.id.unwrap()).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(serde_json::json!({"success": true}), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 批量删除交付记录
pub async fn delete(
    state: web::Data<AppState>,
    item: web::Json<Vec<i64>>,
) -> HttpResponse {
    let db = &state.db;
    let ids = item.into_inner();
    if ids.is_empty() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "请选择要删除的记录", "local"));
    }
    match DeliveryModel::batch_delete(db, &ids).await {
        Ok(count) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(count, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 注册交付模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/delivery")
            .route("/list", web::get().to(list).wrap(require_permission("sale:delivery:list")))
            .route("/info", web::get().to(info).wrap(require_permission("sale:delivery:list")))
            .route("/save", web::post().to(save).wrap(require_permission("sale:delivery:save")))
            .route("/update", web::put().to(update).wrap(require_permission("sale:delivery:update")))
            .route("/resend", web::post().to(resend).wrap(require_permission("sale:delivery:save")))
            .route("/view-full", web::get().to(view_full).wrap(require_permission("sale:delivery:view")))
            .route("/delete", web::delete().to(delete).wrap(require_permission("sale:delivery:delete"))),
    );
}
