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
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::crm::model::customer_edit_log::CustomerEditLogQuery;
use crate::modules::crm::service::customer_edit_log_service;
use actix_web::{web, HttpResponse};

/// 查询客户修改日志（分页）
pub async fn customer_edit_log_list(
    state: web::Data<AppState>,
    query: web::Query<CustomerEditLogQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    match customer_edit_log_service::query_by_customer(db, q).await {
        Ok(page_data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(page_data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册客户修改日志模块所有路由
///
/// 注意：本路由需注册在 `/customer` scope 内部（由 customer_controller::register 调用），
/// 否则会被 `/customer` scope 捕获导致 404。
/// 完整路径：/api/system/customer/edit-log
///
/// 注意：本接口无 `#[protect]` 权限校验，原代码即如此，保持不变。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/edit-log")
            .route(web::get().to(customer_edit_log_list)),
    );
}