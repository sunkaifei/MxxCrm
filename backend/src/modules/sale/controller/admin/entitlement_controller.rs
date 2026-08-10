//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 服务权益控制器
//!
//! ## 路由表
//!
//! | 方法   | 路径                              | 权限码                       | handler      | 说明              |
//! |--------|----------------------------------|------------------------------|--------------|------------------|
//! | GET    | /sale/entitlement/list           | sale:entitlement:list        | list         | 权益列表          |
//! | GET    | /sale/entitlement/info           | sale:entitlement:list        | info         | 权益详情          |
//! | POST   | /sale/entitlement/save           | sale:entitlement:save        | save         | 新建权益（手动）  |
//! | PUT    | /sale/entitlement/update         | sale:entitlement:update      | update       | 修改状态          |
//! | POST   | /sale/entitlement/renew          | sale:entitlement:save        | renew        | 续约              |
//! | GET    | /sale/entitlement/by-customer    | sale:entitlement:list        | by_customer  | 按客户查询        |
//!

use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::InfoId;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::model::entitlement::{
    EntitlementListQuery, EntitlementRenewRequest, EntitlementSaveRequest,
};
use crate::modules::sale::service::entitlement_service;

/// 权益列表
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<EntitlementListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    match entitlement_service::get_list(db, &query).await {
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

/// 权益详情
pub async fn info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "权益ID不能为空", "local"));
    }
    // 复用 list 接口的查询
    let query = EntitlementListQuery {
        page_num: Some(1),
        page_size: Some(1),
        customer_id: None,
        order_id: None,
        status: None,
        entitlement_type: None,
    };
    let _ = query;
    match entitlement_service::get_list(db, &EntitlementListQuery {
        page_num: Some(1),
        page_size: Some(1),
        customer_id: None,
        order_id: None,
        status: None,
        entitlement_type: None,
    }).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(serde_json::json!({"id": item.id.unwrap()}), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 新建权益
pub async fn save(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<EntitlementSaveRequest>,
) -> HttpResponse {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();
    match entitlement_service::create(db, form_data.0, user_id).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 修改权益状态
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
    match entitlement_service::update_status(db, id.unwrap(), status.unwrap() as i32).await {
        Ok(rows) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(rows, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 续约
pub async fn renew(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<EntitlementRenewRequest>,
) -> HttpResponse {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();
    match entitlement_service::renew(db, form_data.0, user_id).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 按客户查询权益
pub async fn by_customer(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "客户ID不能为空", "local"));
    }
    match entitlement_service::find_by_customer(db, item.id.unwrap()).await {
        Ok(list) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(list, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 注册服务权益模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/entitlement")
            .route("/list", web::get().to(list).wrap(require_permission("sale:entitlement:list")))
            .route("/info", web::get().to(info).wrap(require_permission("sale:entitlement:list")))
            .route("/save", web::post().to(save).wrap(require_permission("sale:entitlement:save")))
            .route("/update", web::put().to(update).wrap(require_permission("sale:entitlement:update")))
            .route("/renew", web::post().to(renew).wrap(require_permission("sale:entitlement:save")))
            .route("/by-customer", web::get().to(by_customer).wrap(require_permission("sale:entitlement:list"))),
    );
}
