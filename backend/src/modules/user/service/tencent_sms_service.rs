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
use std::sync::Arc;
use crate::core::errors::error::{Error, Result};
use crate::config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TencentSmsConfig {
    pub secret_id: String,
    pub secret_key: String,
    pub app_id: String,
    pub sign_name: String,
    pub template_login: String,
    pub template_register: String,
    pub template_changepwd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TencentSmsResponse {
    pub response: TencentSmsInnerResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TencentSmsInnerResponse {
    pub request_id: String,
    pub send_status_set: Vec<TencentSmsSendStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TencentSmsSendStatus {
    pub phone_number: String,
    pub fee: i32,
    pub session_context: String,
    pub code: String,
    pub message: String,
    pub iso_country_code: String,
}

#[derive(Debug, Clone)]
pub struct TencentSmsService {
    config: Arc<TencentSmsConfig>,
}

impl TencentSmsService {
    pub fn new() -> Self {
        let config = TencentSmsConfig {
            secret_id: config::section("tencent_sms", "secret_id", "".to_string()),
            secret_key: config::section("tencent_sms", "secret_key", "".to_string()),
            app_id: config::section("tencent_sms", "app_id", "".to_string()),
            sign_name: config::section("tencent_sms", "sign_name", "".to_string()),
            template_login: config::section("tencent_sms", "template_login", "".to_string()),
            template_register: config::section("tencent_sms", "template_register", "".to_string()),
            template_changepwd: config::section("tencent_sms", "template_changepwd", "".to_string()),
        };
        TencentSmsService {
            config: Arc::new(config),
        }
    }

    pub async fn send_sms(&self, phone: &str, code: &str, scene: &str) -> Result<String> {
        let template_id = match scene {
            "login" => &self.config.template_login,
            "register" => &self.config.template_register,
            "changepwd" => &self.config.template_changepwd,
            _ => return Err(Error::from("无效的场景类型")),
        };

        if template_id.is_empty() {
            return Err(Error::from(format!("场景 {} 的模板ID未配置", scene)));
        }

        let request_id = format!("req_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        
        Ok(request_id)
    }

    pub fn is_success(&self, code: &str) -> bool {
        code == "Ok" || code == "0"
    }
}
