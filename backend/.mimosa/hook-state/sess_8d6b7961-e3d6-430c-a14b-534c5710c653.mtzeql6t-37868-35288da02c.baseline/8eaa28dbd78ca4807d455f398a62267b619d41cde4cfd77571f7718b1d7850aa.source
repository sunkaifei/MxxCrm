//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use serde::{Deserialize, Serialize};

/// 用户偏好展示对象（序列化 camelCase）
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct AdminPreferenceVO {
    pub id: i64,
    pub admin_id: i64,
    pub pref_key: Option<String>,
    pub pref_value: Option<serde_json::Value>,
}

/// 快捷导航项（同时作为请求体元素，反序列化 camelCase）
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickNavItem {
    pub menu_id: i64,
    pub sort: i32,
}

/// 保存偏好请求（反序列化 camelCase）
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePreferenceRequest {
    pub pref_key: String,
    pub pref_value: serde_json::Value,
}
