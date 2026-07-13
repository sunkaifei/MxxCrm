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
use crate::modules::website::controller::open::index_open_controller;
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
        // 首页（官网）：由 website 模块的 site_index 渲染 templates/default/index.html
        // 同一处理函数注册多个路径（带/无结尾斜杠、带/不带 index），覆盖常见访问方式
        .service(web::resource("/").route(web::get().to(index_open_controller::site_index)))
        .service(web::resource("/index").route(web::get().to(index_open_controller::site_index)))
        .service(web::resource("/index/").route(web::get().to(index_open_controller::site_index)))
        .service(web::resource("/index.html").route(web::get().to(index_open_controller::site_index)))
        .service(web::resource("/index.html/").route(web::get().to(index_open_controller::site_index)))
        // 仅暴露公开文件目录（产品图片、用户头像），关闭目录列表
        // 私有文件（合同/发票/报价单/回款凭证/通用附件）通过 /api/system/attachment/download/{id} 接口鉴权访问
        .service(Files::new("/upload/product/", "storage/upload/product/"))
        .service(Files::new("/upload/avatar/", "storage/upload/avatar/"))
        // 验证码
        .service(captcha_controller::get_captcha)
        // 文章列表
        .service(article_open_controller::get_article_list)
        // 微信支付回调
        .service(web::scope("/api/finance")
            .service(wechat_notify_controller::wechat_notify))
    ;
}
