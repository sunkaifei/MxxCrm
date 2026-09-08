//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! JSON 字段提取工具 —— 消除全项目 86+ 处重复的
//! `json.get("key").and_then(|v| v.as_str()).map(|s| s.to_string())` 样板
//!

use serde_json::Value;

/// 从 JSON 中提取字符串字段，返回 Option<String>
pub fn get_str(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// 从 JSON 中提取 i64 字段，兼容数字和字符串数字
pub fn get_i64(v: &Value, key: &str) -> Option<i64> {
    v.get(key)
        .and_then(|v| v.as_i64())
        .or_else(|| v.get(key).and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok())))
}

/// 从 JSON 中提取 i32 字段，兼容数字和字符串数字
pub fn get_i32(v: &Value, key: &str) -> Option<i32> {
    get_i64(v, key).map(|n| n as i32)
}

/// 从 JSON 中提取 f64 字段，兼容数字和字符串数字
pub fn get_f64(v: &Value, key: &str) -> Option<f64> {
    v.get(key)
        .and_then(|v| v.as_f64())
        .or_else(|| v.get(key).and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())))
}

/// 从 JSON 中提取 bool 字段
pub fn get_bool(v: &Value, key: &str) -> Option<bool> {
    v.get(key).and_then(|v| v.as_bool())
}
