//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!


// 验证宏：检查条件并返回错误响应
#[macro_export]
macro_rules! validate {
    ($condition:expr, $message:expr) => {
        if $condition {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &$message, "local")));
        }
    };
}