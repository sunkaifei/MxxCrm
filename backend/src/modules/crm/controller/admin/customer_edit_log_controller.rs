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
use crate::core::web::response::MetaResp;
use crate::modules::crm::model::customer_edit_log::CustomerEditLogQuery;
use crate::modules::crm::service::customer_edit_log_service;
use actix_web::{get, web, HttpResponse};

/// 查询客户修改日志（分页）
#[get("/customer/edit-log")]
pub async fn customer_edit_log_list(
    state: web::Data<AppState>,
    query: web::Query<CustomerEditLogQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    match customer_edit_log_service::query_by_customer(db, q).await {
        Ok(page_data) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(page_data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}