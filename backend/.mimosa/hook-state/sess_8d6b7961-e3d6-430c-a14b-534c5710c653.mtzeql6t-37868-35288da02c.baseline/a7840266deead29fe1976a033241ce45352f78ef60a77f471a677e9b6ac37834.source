//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use serde::Serialize;
use crate::core::kit::global::Deserialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponseMeta {
    pub msg: String,
    pub total: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<serde_json::Value>,
    pub meta: Option<ApiResponseMeta>,
}

impl ApiResponse {
    pub fn success(msg: &str) -> Self {
        ApiResponse {
            code: 200,
            msg: msg.to_string(),
            data: None,
            meta: None,
        }
    }

    pub fn success_with_data<T: Serialize>(msg: &str, data: T) -> Self {
        ApiResponse {
            code: 200,
            msg: msg.to_string(),
            data: Some(serde_json::to_value(data).unwrap_or_default()),
            meta: None,
        }
    }

    pub fn error(code: i32, msg: String) -> Self {
        ApiResponse {
            code,
            msg,
            data: None,
            meta: None,
        }
    }

    pub fn success_with_pagination<T: Serialize>(msg: &str, data: T, total: i64, page: i64, page_size: i64) -> Self {
        ApiResponse {
            code: 200,
            msg: msg.to_string(),
            data: Some(serde_json::to_value(data).unwrap_or_default()),
            meta: Some(ApiResponseMeta {
                msg: msg.to_string(),
                total: Some(total),
                page: Some(page),
                page_size: Some(page_size),
            }),
        }
    }
}