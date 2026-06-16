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
use crate::modules::system::controller::open::{captcha_controller, service_open_controller};
use actix_files::Files;
use actix_web::{get, web, HttpResponse};
use std::fs;
use std::io::Read;
use std::path::Path;

/// 首页
#[get("/")]
async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Turtle Market - 服务已启动</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; padding: 50px; background-color: #f5f5f5; }
        .container { max-width: 600px; margin: 0 auto; background: white; padding: 40px; border-radius: 10px; box-shadow: 0 0 20px rgba(0,0,0,0.1); }
        h1 { color: #2c3e50; }
        p { color: #7f8c8d; line-height: 1.6; }
        .api-list { text-align: left; margin-top: 30px; }
        .api-item { padding: 10px; border-bottom: 1px solid #eee; }
        .api-item:last-child { border-bottom: none; }
        .api-url { font-family: monospace; color: #3498db; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🐢 Turtle Market</h1>
        <p>服务已成功启动！</p>
        <p>这是一个乌龟养殖市场管理系统。</p>

        <div class="api-list">
            <h3>可用接口：</h3>
            <div class="api-item">📝 验证码：<span class="api-url">/pub/captcha/get</span></div>
            <div class="api-item">📄 文章列表：<span class="api-url">/list/{short_url}</span></div>
            <div class="api-item">📁 静态文件：<span class="api-url">/static/{filepath}</span></div>
            <div class="api-item">📤 上传文件：<span class="api-url">/upload/{filename}</span></div>
        </div>
    </div>
</body>
</html>
        "#))
}

/// 静态文件服务
#[get("/static/{filepath:.*}")]
async fn serve_static(filepath: web::Path<String>) -> Result<HttpResponse> {
    let path = Path::new("./static").join(filepath.into_inner());
    if path.is_file() {
        let mut file = fs::File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(HttpResponse::Ok().content_type("text/css").body(buf))
    } else {
        Ok(HttpResponse::NotFound().body("Custom 404 Not Found"))
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index)
        .service(serve_static)
        .service(Files::new("/upload/", "storage/upload/").show_files_listing())
         //首页
        //.service(service_open_controller::get_service_index)
        //.service(service_open_controller::get_service_detail)
        //.service(service_open_controller::get_service_list)
        .service(captcha_controller::get_captcha)
        //.service(statistics_open_controller::save_statistics_record)

        //.service(article_open_controller::get_by_short_url)
        .service(article_open_controller::get_article_list)
        //.wrap_fn(web_open::check)
        //.service(index_open_controller::site_index)

        // 微信支付回调
        .service(web::scope("/api/finance")
            .service(wechat_notify_controller::wechat_notify))
    ;
}