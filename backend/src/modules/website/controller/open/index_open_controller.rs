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
use crate::core::kit::template::get_template;
use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use minijinja::context;

/// 官网首页 — 展示开源 CRM 宣传页面
/// 固定使用默认站点信息，不依赖域名识别与数据库配置
pub async fn site_index() -> Result<HttpResponse> {
    let ctx = context!(
        site_id => 0i64,
        site_name => "MxxCRM · 开源客户管理系统",
        keywords => "MxxCRM,开源CRM,客户关系管理,客户管理软件,私域运营,全行业CRM",
        description => "MxxCRM 是一款通用型开源客户关系管理系统，覆盖「线索 → 客户 → 商机 → 订单 → 履约」全链路，不限行业、不限规模，支持私有化部署。",
        site_domain => "mxxshop.com",
    );

    // 首页模板为文件模板，位于 templates/default/index.html
    // templates / static 均在程序包外，按目录直接读取，方便随时编辑与切换
    let rendered = get_template("default/index.html", ctx)?;
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}
