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
use crate::core::errors::error::Result;
use crate::core::kit::template::{get_template, get_template_a};
use crate::core::kit::global::AppState;
use crate::modules::website::service::{template_user_data_service, website_service};
use crate::utils::domain_utils::get_subdomain;

/// 服务报价页面 — 列出安装部署、二次开发、技术支持等配套服务参考报价
/// 有子域名时从数据库模板渲染（type_id=5），无子域名时使用默认文件模板
pub async fn price_index(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    match get_subdomain(&req) {
        Ok(domain_name) => {
            let site = website_service::find_by_domain(db, &domain_name).await?;
            // 获取报价页模板数据（type_id=5：专题，用于报价页）
            let template_data = template_user_data_service::find_latest_by_template_and_type(
                db, &site.template_id, &Some(5)
            ).await?;
            let ctx = context!(
                site_name => site.site_name.unwrap_or_default(),
                site_domain => site.domain.unwrap_or_default(),
                keywords => "MxxCRM,开源CRM,报价,服务价格,二次开发,安装部署,技术支持",
                description => "MxxCRM 开源免费，本页列出安装部署、二次开发、技术支持等服务项目的参考报价。",
            );
            let rendered = get_template_a(
                template_data.temptext.unwrap_or_default().as_str(), ctx
            )?;
            Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
        }
        Err(_) => {
            // 无子域名时使用默认文件模板
            let ctx = context!(
                site_name => "MxxCRM · 开源客户管理系统",
                keywords => "MxxCRM,开源CRM,报价,服务价格,二次开发,安装部署,技术支持",
                description => "MxxCRM 开源免费，本页列出安装部署、二次开发、技术支持等服务项目的参考报价。",
                site_domain => "mxxshop.com",
            );
            let rendered = get_template("default/price.html", ctx)?;
            Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
        }
    }
}
