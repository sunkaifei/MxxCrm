//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!


use minijinja::{Error, Value};
use crate::utils::string_utils::filter_html_tags;

// 自定义过滤器函数
pub fn to_json_filter(value: &Value) -> Result<Value, Error> {
    // 处理输入的 value，假设输入是一个字符串
    let json_str = value.as_str().unwrap_or("");
    match serde_json::from_str::<Value>(json_str) {
        Ok(json_value) => {
            Ok(json_value)
        },
        Err(err) => { 
            log::error!("Error parsing JSON: {}", err);
            Ok(Value::from_safe_string("Json标签解析失败".to_string()))   
        },
    }
}


pub fn filter_html(name: &str) -> Result<Value, Error> {
    let new_content = if name.is_empty() {
        "格式化html错误".to_string()
    } else {
        filter_html_tags(Some(name.to_string()))
    };
    Ok(Value::from_safe_string(new_content))
}


/// 设置标签默认值
/// 
///
pub fn none_default(value: Value, other: Option<Value>) -> Value {
    if value.is_undefined() || value.is_none() {
        other.unwrap_or_else(|| Value::from(""))
    } else {
        value
    }
}