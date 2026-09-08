//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{get, HttpRequest};
use crate::core::web::entity::common::QueryUrl;
use actix_web::{HttpResponse, web};
use actix_web::http::header::ContentType;
use minijinja::context;
use crate::core::kit::template::{get_template, get_template_a};
use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::modules::articles::model::article::{ArticleModel, QueryPageRequest};
use crate::modules::articles::service::article_service;
use crate::modules::website::model::website::SiteModel;
use crate::modules::website::service::{template_user_data_service, website_service};
use crate::utils::domain_utils::get_subdomain;


// 定义参数结构体
#[allow(dead_code)]
struct PathParams {
    short_url: String,
    p: Option<i64>,
}
#[get("/list/{short_url}")]
pub async fn get_article_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<QueryUrl>
) -> Result<HttpResponse> {
    let db = &state.db;
    // 获取路径参数
    log::info!("short_url================:{:?}", path.short_url);
    let domain_name = get_subdomain(&req)?;
    let site = website_service::find_by_domain(&db, &domain_name).await?;
    
    let query_data = QueryPageRequest{
        title: None,
        page_num: Some(1),
        page_size: Some(10),
        category_id: Some(1),
        website_id: site.id,
        status: Some(2),
    };
    let article = article_service::get_by_page(&db,query_data).await?;
    
    
    let ctx = context!(
        site => &site,
        list => &article.items,
    );
    let template_data = template_user_data_service::find_latest_by_template_and_type(db, &site.template_id, &Some(2)).await?;

    let rendered = get_template_a(template_data.temptext.unwrap_or_default().as_str(), ctx)?;
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}

#[get("/news/{short_url}")]
pub async fn get_by_short_url(state: web::Data<AppState>,req: HttpRequest, path: web::Path<QueryUrl>) -> Result<HttpResponse> {
    let db = &state.db;
    let domain_name = get_subdomain(&req)?;
    log::info!("get_by_short_url:{:?}", domain_name);
    let site = website_service::find_by_domain(&db, &domain_name).await?;
    let s_short_url = path.short_url.clone();
    if let Some(article) = article_service::get_by_short_url(db, &site.clone().id, &s_short_url).await? {
        let ctx= context!(
            field => &article,
        );
        let template_data = template_user_data_service::find_latest_by_template_and_type(db, &site.template_id, &Some(3)).await?;

        let rendered = get_template_a(template_data.temptext.unwrap_or_default().as_str(), ctx)?;
        Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
    } else {
        let ctx = context!(
            error => "查询出现错误，请联系管理员",
        );
        let out = get_template("default/404.html", ctx)?;
        Ok(HttpResponse::Ok().content_type(ContentType::html()).body(out))
    }
}