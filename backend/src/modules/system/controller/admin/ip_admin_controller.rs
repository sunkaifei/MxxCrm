//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{web, HttpResponse};
use actix_web::web::Query;
use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
use crate::modules::system::model::ip_address::ListQuery;
use crate::modules::system::service::ip_address_service;

pub async fn ip_address_page(state: web::Data<AppState>, query: Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    ip_address_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

// ==================== 路由注册（单点维护）====================

/// 注册 IP 地址模块所有路由
///
/// 修改路径、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(ip_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ip")
            // GET /ip/list - IP 地址列表
            .route("/list", web::get().to(ip_address_page)),
    );
}