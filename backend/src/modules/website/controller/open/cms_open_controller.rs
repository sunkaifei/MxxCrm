//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::http::header::ContentType;
use minijinja::context;
use crate::core::errors::error::{Error, Result};
use crate::core::kit::template::{get_template, get_template_a};
use crate::core::web::entity::common::QueryUrl;
use crate::core::kit::global::AppState;
use crate::modules::articles::entity::category;
use crate::modules::articles::model::article::{ArticleModel, QueryPageRequest};
use crate::modules::articles::model::category::CategoryModel;
use crate::modules::articles::service::article_service;
use crate::modules::articles::service::category_service;
use crate::modules::website::service::{template_user_data_service, website_service};
use crate::utils::domain_utils::get_subdomain;

/// CMS 首页 — 按域名动态渲染
/// 有子域名时查数据库模板渲染，无子域名时使用默认首页
pub async fn cms_index(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    match get_subdomain(&req) {
        Ok(domain_name) => {
            let site = website_service::find_by_domain(db, &domain_name).await?;
            let site_id = site.id.unwrap_or_default();
            // 获取站点导航栏目
            let categories = CategoryModel::find_all(db, site_id).await.unwrap_or_default();
            let nav_categories: Vec<category::Model> = categories.into_iter()
                .filter(|c| c.is_show == Some(1))
                .collect();
            // 获取首页模板数据
            let template_data = template_user_data_service::find_latest_by_template_and_type(
                db, &site.template_id, &Some(1)
            ).await?;
            let ctx = context!(
                site => &site,
                categories => &nav_categories,
            );
            let rendered = get_template_a(
                template_data.temptext.unwrap_or_default().as_str(), ctx
            )?;
            Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
        }
        Err(_) => {
            // 无子域名时使用默认首页
            let ctx = context!(
                site_id => 0i64,
                site_name => "MxxCRM · 开源客户管理系统",
                keywords => "MxxCRM,开源CRM,客户关系管理,客户管理软件,私域运营,全行业CRM",
                description => "MxxCRM 是一款通用型开源客户关系管理系统，覆盖「线索 → 客户 → 商机 → 订单 → 履约」全链路，不限行业、不限规模，支持私有化部署。",
                site_domain => "mxxshop.com",
            );
            let rendered = get_template("default/index.html", ctx)?;
            Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
        }
    }
}

/// 栏目页 — 根据短链接匹配栏目，支持封面模式和列表模式
pub async fn category_page(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<QueryUrl>
) -> Result<HttpResponse> {
    let db = &state.db;
    let domain_name = get_subdomain(&req)?;
    let site = website_service::find_by_domain(db, &domain_name).await?;
    let site_id = site.id.unwrap_or_default();
    let short_url = path.short_url.clone();

    // 根据短链接查找栏目
    let category = CategoryModel::find_by_short_url(db, short_url.unwrap_or_default()).await?
        .ok_or_else(|| Error::from("栏目不存在"))?;

    let category_id = category.id;

    // 判断页面模式
    if category.page_type == Some(1) && category.page_template_data_id.is_some() {
        // 封面模式：使用栏目关联的封面模板
        let template_data = template_user_data_service::get_by_detail(db, &category.page_template_data_id).await?;
        let children = CategoryModel::find_all(db, site_id).await.unwrap_or_default();
        let sub_categories: Vec<category::Model> = children.into_iter()
            .filter(|c| c.parent_id == Some(category_id))
            .collect();

        let ctx = context!(
            site => &site,
            category => &category,
            children => &sub_categories,
        );
        let template_text = template_data.temptext.unwrap_or_default();
        let rendered = get_template_a(template_text.as_str(), ctx)?;
        Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
    } else {
        // 列表模式：分页查询文章
        let query_data = QueryPageRequest {
            title: None,
            page_num: Some(1),
            page_size: Some(10),
            category_id: Option::from(category_id),
            website_id: Option::from(site_id),
            status: Some(2),
        };
        let article_page = article_service::get_by_page(db, query_data).await?;

        let ctx = context!(
            site => &site,
            category => &category,
            list => &article_page.items,
            total => &article_page.total,
            page => &1,
        );
        let template_data = template_user_data_service::find_latest_by_template_and_type(
            db, &site.template_id, &Some(2)
        ).await?;
        let rendered = get_template_a(template_data.temptext.unwrap_or_default().as_str(), ctx)?;
        Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
    }
}

/// 文章详情
pub async fn article_detail(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<QueryUrl>
) -> Result<HttpResponse> {
    let db = &state.db;
    let domain_name = get_subdomain(&req)?;
    let site = website_service::find_by_domain(db, &domain_name).await?;
    let short_url = path.short_url.clone();

    if let Some(article) = article_service::get_by_short_url(db, &site.id, &short_url).await? {
        let ctx = context!(
            site => &site,
            field => &article,
        );
        let template_data = template_user_data_service::find_latest_by_template_and_type(
            db, &site.template_id, &Some(3)
        ).await?;
        let rendered = get_template_a(template_data.temptext.unwrap_or_default().as_str(), ctx)?;
        Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
    } else {
        let ctx = context!(error => "查询出现错误，请联系管理员");
        let out = get_template("default/404.html", ctx)?;
        Ok(HttpResponse::Ok().content_type(ContentType::html()).body(out))
    }
}
