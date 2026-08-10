//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 第三方接口统一配置 请求/响应 DTO
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use sea_orm::prelude::DateTime;

/// 保存配置请求
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationConfigSaveRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub category: Option<String>,
    pub integration_code: Option<String>,
    pub integration_name: Option<String>,
    pub config_json: Option<serde_json::Value>,
    pub api_base_url: Option<String>,
    pub enabled: Option<i32>,
    pub remark: Option<String>,
}

/// 配置详情 VO（敏感字段序列化时脱敏）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationConfigVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub category: Option<String>,
    pub integration_code: Option<String>,
    pub integration_name: Option<String>,
    pub config_json: Option<serde_json::Value>,
    pub api_base_url: Option<String>,
    pub enabled: Option<i32>,
    pub sort_order: Option<i32>,
    pub last_test_time: Option<DateTime>,
    pub last_test_result: Option<i32>,
    pub last_test_message: Option<String>,
    pub is_encrypted: Option<i32>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
    /// 测试状态名称（成功/失败/未测试）
    pub test_status_name: Option<String>,
}

/// 接口测试结果
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationTestResult {
    pub integration_code: String,
    pub success: bool,
    pub message: String,
}
