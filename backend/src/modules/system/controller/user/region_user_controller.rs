//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{get, HttpResponse, web};
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
use crate::modules::system::service::region_service;

#[get("/region/tree")]
pub async fn get_region_tree(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    let result = region_service::get_region_tree_for_user(&db).await;
    match result {
        Ok(v) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))
        }
        Err(err) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &("查询行政区域树异常,".to_string() + &err.to_string()), "local"))
        }
    }
}