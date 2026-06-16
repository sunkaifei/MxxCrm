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
use crate::core::kit::template::{get_template, get_template_a};
use crate::modules::website::service::{website_service, template_data_service, template_service, template_user_data_service};
use crate::utils::domain_utils::get_subdomain;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpRequest, HttpResponse};
use minijinja::context;
use crate::utils::time_utils;

#[get(r#"/{path:(index|index\.html)}?/$"#)]
pub async fn site_index(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    log::info!("site_index================:{:?}", req);
    let domain_name = get_subdomain(&req)?;
    //log::info!("domain_name================:{:?}", domain_name);
    let site = website_service::find_by_domain(&db, &domain_name).await?;
    //log::info!("&site.template_id================:{:?}", &site.template_id);
    let ctx = context!(
        site_id => site.id,
        site_name => site.site_name,
        site_domain => site.domain,
        keywords => site.keywords,
        description => site.description,

    );

    let template_data = template_user_data_service::find_latest_by_template_and_type(db, &site.template_id, &Some(1)).await?;
    let rendered = get_template_a(template_data.temptext.unwrap_or_default().as_str(), ctx)?;
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}
