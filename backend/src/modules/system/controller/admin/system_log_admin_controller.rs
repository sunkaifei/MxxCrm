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
use actix_web::{get, web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
use crate::modules::system::model::system_log::ListQuery;
use crate::modules::system::service::system_log_service;

#[get("/logs/list")]
pub async fn get_by_page(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    system_log_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}