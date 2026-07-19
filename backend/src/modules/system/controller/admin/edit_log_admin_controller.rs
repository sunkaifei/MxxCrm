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
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::system::model::edit_log::EditLogQuery;
use crate::modules::system::service::edit_log_service;
use actix_web::{web, HttpResponse};

/// 分页查询编辑日志（管理员/经理权限）
pub async fn edit_log_list(
    state: web::Data<AppState>,
    query: web::Query<EditLogQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.0;

    match edit_log_service::query_page(db, query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(
                MetaResp::success_with_page(page_data, "local", page, total),
            ))
        }
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（方案 C：单点维护）====================

/// 注册编辑日志模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(edit_log_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/edit-log")
            // GET /edit-log/list - 编辑日志列表
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/list",
                web::get()
                    .to(edit_log_list)
                    .wrap(require_permission("system:edit-log:view")),
            ),
    );
}
