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
use crate::modules::crm::model::contact_edit_log::ContactEditLogQuery;
use crate::modules::crm::service::contact_edit_log_service;
use actix_web::{web, HttpResponse};

/// 查询联系人修改日志（分页）
pub async fn contact_edit_log_list(
    state: web::Data<AppState>,
    query: web::Query<ContactEditLogQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    match contact_edit_log_service::query_by_contact(db, q).await {
        Ok(page_data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(page_data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册联系人修改日志模块路由
///
/// 注意：本路由需注册在 `/contact` scope 内部（由 contact_controller::register 调用），
/// 完整路径：/api/system/contact/edit-log
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/edit-log")
            .route(web::get().to(contact_edit_log_list)),
    );
}
