//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::utils::time_utils::time_difference;
use minijinja::value::Value;
use minijinja::Error;

pub fn format_time(name: &str) -> Result<Value, Error> {
    let new_time = if name.is_empty() {
        "时间格式错误".to_string()
    } else {
        time_difference(name)
    };
    Ok(Value::from_safe_string(new_time))
}