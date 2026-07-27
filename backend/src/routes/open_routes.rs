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
use crate::modules::finance::controller::open::wechat_notify_controller;
use crate::modules::system::controller::open::captcha_controller;
use crate::modules::website::controller::open::cms_open_controller;
use crate::modules::website::controller::open::index_open_controller;
use crate::modules::website::controller::open::price_open_controller;
use actix_files::Files;
use actix_web::{get, web, HttpResponse};

/// 健康检查
#[get("/healthz")]
async fn healthz() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("ok"))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // 静态资源：templates / static 均在程序包外，按目录直接对外提供，方便在线编辑与切换模板
        .service(Files::new("/static/", "static/"))
        // CMS 首页（动态）：有子域名时查数据库模板渲染，无子域名时使用默认首页
        .service(web::resource("/").route(web::get().to(cms_open_controller::cms_index)))
        .service(web::resource("/index").route(web::get().to(cms_open_controller::cms_index)))
        .service(web::resource("/index/").route(web::get().to(cms_open_controller::cms_index)))
        .service(web::resource("/index.html").route(web::get().to(cms_open_controller::cms_index)))
        .service(web::resource("/index.html/").route(web::get().to(cms_open_controller::cms_index)))
        // 仅暴露公开文件目录（产品图片、用户头像），关闭目录列表
        // 私有文件（合同/发票/报价单/回款凭证/通用附件）通过 /api/system/attachment/download/{id} 接口鉴权访问
        .service(Files::new("/upload/product/", "storage/upload/product/"))
        .service(Files::new("/upload/avatar/", "storage/upload/avatar/"))
        // 服务报价页
        .service(web::resource("/price").route(web::get().to(price_open_controller::price_index)))
        .service(web::resource("/price.html").route(web::get().to(price_open_controller::price_index)))
        // 验证码
        .service(captcha_controller::get_captcha)
        // CMS 栏目页 + 文章详情
        .service(web::resource("/category/{short_url}").route(web::get().to(cms_open_controller::category_page)))
        .service(web::resource("/article/{short_url}").route(web::get().to(cms_open_controller::article_detail)))
        // 文章列表（兼容旧路由）
        .service(article_open_controller::get_article_list)
        // 微信支付回调
        .service(web::scope("/api/finance")
            .service(wechat_notify_controller::wechat_notify))
    ;
}
