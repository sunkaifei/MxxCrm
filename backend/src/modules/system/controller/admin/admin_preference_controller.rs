//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::response::{MetaResp, MPACK};

use crate::modules::system::model::admin_preference::QuickNavItem;
use crate::modules::system::service::admin_preference_service;

/// GET /preference/quick-nav - 获取当前用户快捷导航配置
pub async fn get_quick_nav(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let db = &state.db;
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }
    match admin_preference_service::find_quick_nav(db, admin_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

/// PUT /preference/quick-nav - 保存当前用户快捷导航配置（请求体：Vec<QuickNavItem>）
pub async fn save_quick_nav(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<Vec<QuickNavItem>>,
) -> HttpResponse {
    let db = &state.db;
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }
    let items = payload.0;
    match admin_preference_service::save_quick_nav(db, admin_id, &items).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

/// GET /preference/sale-mode - 获取销售简易模式开关
pub async fn get_sale_mode(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let db = &state.db;
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }
    match admin_preference_service::find_sale_simple_mode(db, admin_id).await {
        Ok(enabled) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(enabled, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

/// PUT /preference/sale-mode - 保存销售简易模式开关（请求体：bool）
pub async fn save_sale_mode(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<bool>,
) -> HttpResponse {
    let db = &state.db;
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未登录", "local"));
    }
    let enabled = payload.0;
    match admin_preference_service::save_sale_simple_mode(db, admin_id, enabled).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/preference")
            .route("/quick-nav", web::get().to(get_quick_nav))
            .route("/quick-nav", web::put().to(save_quick_nav))
            .route("/sale-mode", web::get().to(get_sale_mode))
            .route("/sale-mode", web::put().to(save_sale_mode)),
    );
}
