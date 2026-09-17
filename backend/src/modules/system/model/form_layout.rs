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

/// 布局保存请求（新建/更新一体，upsert 语义）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FormLayoutSaveRequest {
    /// 业务模块标识
    pub module: String,
    /// 布局类型：1=新建/编辑表单 2=详情页
    pub layout_type: i32,
    /// 布局 JSON（见方案文档 §4.3）
    pub layout_json: serde_json::Value,
    /// 前端持有的版本号；与服务端不一致返回 409 语义（防设计器互相覆盖）
    pub version: Option<i32>,
    /// 适用角色 key：空/缺省=默认布局
    pub role_key: Option<String>,
}

/// 布局查询响应
/// rename_all 双向：布局走 Redis/内存 JSON 缓存回读，serialize-only 会缓存命中全 None
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormLayoutVO {
    pub id: i64,
    pub module: String,
    pub layout_type: i32,
    /// 适用角色 key：None=默认布局
    pub role_key: Option<String>,
    pub layout_json: serde_json::Value,
    pub version: i32,
    pub status: i32,
    pub update_time: Option<String>,
}

/// 布局字段项（校验用中间结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutFieldItem {
    pub key: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub tab: Option<String>,
    #[serde(default = "default_span")]
    pub span: i32,
    #[serde(default)]
    pub sort: i32,
}

fn default_source() -> String {
    "field_def".to_string()
}

fn default_span() -> i32 {
    1
}

/// 布局 JSON 顶层结构（校验用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutJson {
    #[serde(default = "default_layout_version")]
    pub version: i32,
    #[serde(default)]
    pub tabs: Vec<LayoutTabItem>,
    pub fields: Vec<LayoutFieldItem>,
    #[serde(default = "default_unassigned_policy")]
    pub unassigned_policy: String,
}

fn default_layout_version() -> i32 {
    1
}

fn default_unassigned_policy() -> String {
    "append".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutTabItem {
    pub key: String,
    pub title: String,
}
