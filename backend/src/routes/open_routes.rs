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
use crate::modules::articles::controller::open::article_open_controller;
use crate::modules::articles::controller::open::comment_open_controller;
use crate::modules::finance::controller::open::wechat_notify_controller;
use crate::modules::system::controller::open::captcha_controller;
use crate::modules::website::controller::open::cms_open_controller;
use crate::modules::website::controller::open::index_open_controller;
use crate::modules::website::controller::open::price_open_controller;
use crate::modules::website::controller::open::leave_msg_open_controller;
use crate::modules::website::controller::open::website_user_open_controller;
use actix_files::Files;
use actix_web::{get, web, HttpRequest, HttpResponse};

use crate::core::kit::config;

/// 健康检查
#[get("/healthz")]
async fn healthz() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("ok"))
}

/// 管理后台前端：按配置的前缀（admin_prefix）提供内嵌前端资源
///
/// 前端以相对路径 base（VITE_BASE=./）构建，资源引用均为相对路径，因此可挂在
/// 任意自定义前缀下均由本 handler 按相对路径取回；路径无扩展名或命中 SPA 兜底
/// 时返回 index.html。必须注册在开放的 CMS 兜底路由（/{short_url}*）之前。
async fn serve_admin_frontend(req: HttpRequest) -> HttpResponse {
    let prefix = config::section::<String>("server", "admin_prefix", "/console".to_string());
    let prefix = prefix.trim().trim_matches('/');
    let path = req.path().trim_start_matches('/');

    // 去掉已匹配的前缀段，得到相对路径；无法剥离（即访问前缀根）时回退首页
    let rel = match path.strip_prefix(&format!("{}/", prefix)) {
        Some(rest) => rest.trim_start_matches('/').to_string(),
        None => "index.html".to_string(),
    };

    crate::serve_frontend_asset(&rel)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // 管理后台：按配置的自定义前缀提供内嵌前端（相对 base 构建）。
    // 请求经由 default_service 的 serve_frontend 兜底即可，但此处显式注册前缀路由，
    // 必须保证在下方开放的 CMS 泛化兜底路由（/{short_url}*）之前命中，避免被吞成栏目。
    let admin_prefix = config::section::<String>("server", "admin_prefix", "/console".to_string());
    let admin_prefix_clean = admin_prefix.trim().trim_matches('/').to_string();
    if !admin_prefix_clean.is_empty() {
        let root = format!("/{}", admin_prefix_clean);
        let root_path = format!("{}/{{path:.*}}", root);
        cfg.service(web::resource(root.as_str()).route(web::get().to(serve_admin_frontend)));
        cfg.service(web::resource(&root_path).route(web::get().to(serve_admin_frontend)));
    }

    cfg
        // 静态资源：templates / static 均在程序包外，按目录直接对外提供，方便在线编辑与切换模板
        .service(Files::new("/static/", "static/"))
        // CMS 首页（动态）：有子域名时查数据库模板渲染，无子域名时使用默认首页
        .service(web::resource("/").route(web::get().to(cms_open_controller::cms_index)))
        .service(web::resource("/index").route(web::get().to(cms_open_controller::cms_index)))
        .service(web::resource("/index/").route(web::get().to(cms_open_controller::cms_index)))
        .service(web::resource("/index.html").route(web::get().to(cms_open_controller::cms_index)))
        .service(web::resource("/index.html/").route(web::get().to(cms_open_controller::cms_index)))
        // 仅暴露公开文件目录（产品图片、用户头像、模板封面），关闭目录列表
        // 私有文件（合同/发票/报价单/回款凭证/通用附件）通过 /api/system/attachment/download/{id} 接口鉴权访问
        .service(Files::new("/upload/product/", "storage/upload/product/"))
        .service(Files::new("/upload/avatar/", "storage/upload/avatar/"))
        .service(Files::new("/upload/template/", "storage/upload/template/"))
        // 服务报价页
        .service(web::resource("/price").route(web::get().to(price_open_controller::price_index)))
        .service(web::resource("/price.html").route(web::get().to(price_open_controller::price_index)))
        // 验证码（图形/文字点选）：/api/open/captcha/*
        .service(
            web::scope("/api/open/captcha")
                .configure(captcha_controller::register),
        )
        // CMS 栏目页 + 文章详情
        .service(web::resource("/category/{short_url}").route(web::get().to(cms_open_controller::category_page)))
        .service(web::resource("/article/{short_url}").route(web::get().to(cms_open_controller::article_detail)))
        // CMS 产品列表页 + 产品详情页
        .service(web::resource("/product").route(web::get().to(cms_open_controller::product_list)))
        .service(web::resource("/product/").route(web::get().to(cms_open_controller::product_list)))
        .service(web::resource("/product/{short_url}").route(web::get().to(cms_open_controller::product_detail)))
        // CMS 搜索页
        .service(web::resource("/search").route(web::get().to(cms_open_controller::search)))
        // CMS 站点地图
        .service(web::resource("/sitemap").route(web::get().to(cms_open_controller::sitemap)))
        .service(web::resource("/sitemap.html").route(web::get().to(cms_open_controller::sitemap)))
        .service(web::resource("/sitemap.xml").route(web::get().to(cms_open_controller::sitemap_xml)))
        // robots.txt
        .service(web::resource("/robots.txt").route(web::get().to(cms_open_controller::robots_txt)))
        // CMS 自定义页面
        .service(web::resource("/page/{short_url}").route(web::get().to(cms_open_controller::custom_page)))
        // 文章列表（兼容旧路由）
        .service(article_open_controller::get_article_list)
        // 文章评论（公开接口：提交评论、按文章查询评论）
        .configure(comment_open_controller::register)
        // P-1.5：前台会员登录页 / 购物车页（SSR 骨架，数据由 cms.js 拉取）
        .service(web::resource("/user/login").route(web::get().to(cms_open_controller::user_login_page)))
        .service(web::resource("/cart").route(web::get().to(cms_open_controller::cart_page)))
        // 公开附件访问（附件URL统一方案 v1.1：仅 is_public=1，其余一律 404）
        .service(web::scope("/api/open/file").configure(crate::modules::upload::controller::open::attachment_open_controller::register))
        // 微信支付回调
        .service(web::scope("/api/finance")
            .service(wechat_notify_controller::wechat_notify))
        // 留言提交（前台访客公开接口）
        .service(web::resource("/api/open/leave_msg/submit").route(web::post().to(leave_msg_open_controller::submit)))
        // 前台用户注册/登录（公开接口）
        .service(website_user_open_controller::register)
        .service(website_user_open_controller::login)
        // 12-D：内容模型前台 URL（泛化路由必须最后注册，具体路由已全部在上方优先命中；
        // 占位符名必须是 short_url，与 QueryUrl 字段名一致，否则 Path<QueryUrl> 绑定为 None）
        .service(web::resource("/{short_url}").route(web::get().to(cms_open_controller::model_content_list)))
        .service(web::resource("/{short_url}/{content_id}").route(web::get().to(cms_open_controller::model_content_detail)))
    ;
}
