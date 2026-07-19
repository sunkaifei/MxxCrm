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
use actix_web::{web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::response::MetaResp;
use crate::modules::system::model::system_log::{ListQuery, SystemLogModel};
use crate::modules::system::service::system_log_service;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

pub async fn get_by_page(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    system_log_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

/// 批量删除系统日志
pub async fn bath_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到删除的日志ID", "local"));
    }

    let ids = convert_vec_option_string_to_vec_u64(delete_item.ids.unwrap_or_default());
    let result = SystemLogModel::batch_delete_by_ids(&db, ids).await;

    match result {
        Ok(count) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::success(count, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册系统日志模块所有路由
///
/// 修改路径、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(system_log_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/logs")
            // GET /logs/list - 系统日志列表
            .route("/list", web::get().to(get_by_page))
            // DELETE /logs/bath_delete - 批量删除系统日志
            .route("/bath_delete", web::delete().to(bath_delete)),
    );
}
