//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_http::StatusCode;
use actix_web::{get, HttpResponse, routes, web};
use crate::core::errors::error::{Result};
use minijinja::context;
use crate::core::kit::template::get_template;
use crate::core::web::entity::common::QueryUrl;


#[routes]
#[get("/help/")]
#[get("/help")]
pub async fn get_service_index() -> Result<HttpResponse> {
    let ctx = context!(
        title => "企业列表",
    );
    let out = get_template("default/pc/service/index.html", ctx)?;
    return Ok(HttpResponse::build(StatusCode::OK).body(out))
}

#[routes]
#[get("/help/list/")]
#[get("/help/list")]
pub async fn get_service_list() -> Result<HttpResponse> {
    let ctx = context!(
        title => "企业列表",
    );
    let out = get_template("default/pc/service/list.html", ctx)?;
    return Ok(HttpResponse::build(StatusCode::OK).body(out))
}

#[routes]
#[get("/help/{short_url}/")]
#[get("/help/{short_url}")]
pub async fn get_service_detail(_path: web::Path<QueryUrl>) -> Result<HttpResponse> {
    let ctx = context!(
        title => "企业列表",
    );
    let out = get_template("default/pc/service/detail.html", ctx)?;
    return Ok(HttpResponse::build(StatusCode::OK).body(out))
}