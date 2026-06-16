//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::CONTEXT;
use actix_web::{get, HttpResponse};
use base64::{engine, Engine};
use captcha::filters::{Dots, Noise, Wave};
use captcha::Captcha;
use std::collections::HashMap;

use crate::core::web::response::MetaResp;

// 定义验证码路由处理函数
#[get("/pub/captcha/get")]
pub async fn get_captcha() -> Result<HttpResponse, Box<dyn std::error::Error>> {
    // 生成验证码图像
    let mut captcha = Captcha::new();
    captcha
        .add_chars(4)
        .apply_filter(Noise::new(0.1))
        .apply_filter(Wave::new(1.0, 10.0).horizontal())
        .view(160, 60)
        .apply_filter(Dots::new(4));
    let captcha_str = captcha.chars_as_string().to_lowercase();

    //用于验证码校验
    let uuid = uuid::Uuid::new_v4().to_string();
    //写入缓存里，该验证码缓存一天，未使用的验证码自动删除
    let result = CONTEXT.cache_service.set_string(&format!("captcha:cache_{}", uuid.as_str()), &captcha_str.as_str()).await;
    if result.is_err() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "创建验证码失败", "local")));
    }
    let png = captcha.as_png().unwrap_or_default();
    let base64_captcha = engine::general_purpose::STANDARD.encode(png);
    let mut hashmap = HashMap::new();
    hashmap.insert("captchaKey", uuid);
    hashmap.insert("captchaBase64", base64_captcha);
    // 返回验证码图像
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(hashmap, "local")))
}
