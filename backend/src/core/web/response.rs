//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.//!
//! https://www.mxxshop.com
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Error;
use serde::Serialize;

/// API全局MetaMessage返回根对象（MetaResp<T> + RespMeta 固定结构）
#[derive(Debug, Serialize)]
pub struct MetaResp<T>
where
    T: Serialize,
{
    /// 业务状态码，200成功
    pub code: i32,

    /// 接口提示信息
    pub msg: String,

    /// 业务主体数据
    pub data: Option<T>,

    /// 追踪、分页元数据
    pub meta: Option<RespMeta>,
}

/// 链路追踪分页元信息
#[derive(Debug, Serialize)]
pub struct RespMeta {
    /// 请求唯一追踪ID
    pub trace_id: String,
    /// 当前页码
    pub page: Option<u32>,
    /// 数据总条数
    pub total: Option<u32>,
}

impl<T> MetaResp<T>
where
    T: Serialize,
{
    fn to_wire(&self) -> Vec<u8> {
        rmp_serde::to_vec_named(self).unwrap_or_default()
    }

    /// 成功返回带业务数据，编码为二进制 wire 格式
    pub fn success(data: T, trace_id: &str) -> Vec<u8> {
        Self {
            code: 200,
            msg: "success".to_string(),
            data: Some(data),
            meta: Some(RespMeta {
                trace_id: trace_id.to_string(),
                page: None,
                total: None,
            }),
        }
        .to_wire()
    }

    /// 成功返回带分页信息
    pub fn success_with_page(data: T, trace_id: &str, page: u32, total: u32) -> Vec<u8> {
        Self {
            code: 200,
            msg: "success".to_string(),
            data: Some(data),
            meta: Some(RespMeta {
                trace_id: trace_id.to_string(),
                page: Some(page),
                total: Some(total),
            }),
        }
        .to_wire()
    }

    /// 业务错误返回，data置空
    pub fn fail(code: i32, msg: &str, trace_id: &str) -> Vec<u8> {
        Self {
            code,
            msg: msg.to_string(),
            data: None,
            meta: Some(RespMeta {
                trace_id: trace_id.to_string(),
                page: None,
                total: None,
            }),
        }
        .to_wire()
    }

    /// 成功返回带自定义消息
    pub fn success_with_msg(data: T, msg: &str, trace_id: &str) -> Vec<u8> {
        Self {
            code: 200,
            msg: msg.to_string(),
            data: Some(data),
            meta: Some(RespMeta {
                trace_id: trace_id.to_string(),
                page: None,
                total: None,
            }),
        }
        .to_wire()
    }

    /// 处理数据库操作结果
    pub fn handle_result(result: Result<i64, Error>) -> Vec<u8> {
        match result {
            Ok(data) => {
                if data > 0 {
                    Self::fail(200, "success", "local")
                } else {
                    Self::fail(400, "操作失败", "local")
                }
            }
            Err(e) => Self::fail(400, &e.to_string(), "local"),
        }
    }

    /// 错误返回带数据体
    pub fn error_with_data(code: i32, msg: &str, data: T, trace_id: &str) -> Vec<u8> {
        Self {
            code,
            msg: msg.to_string(),
            data: Some(data),
            meta: Some(RespMeta {
                trace_id: trace_id.to_string(),
                page: None,
                total: None,
            }),
        }
        .to_wire()
    }
}

/// 分页响应数据结构
#[derive(Debug, Serialize)]
pub struct ResultPage<T>
where
    T: Serialize,
{
    /// 列表数据项
    pub items: T,
    /// 数据总条数
    pub total: i64,
    /// 当前页码
    pub current_page: i64,
    /// 每页显示条数
    pub page_size: i64,
    /// 总页数
    pub total_pages: i64,
}

impl<T> ResultPage<T>
where
    T: Serialize,
{
    pub fn new(items: T, total: i64, current_page: i64, page_size: i64) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            (total + page_size - 1) / page_size
        };
        Self {
            items,
            total,
            current_page,
            page_size,
            total_pages,
        }
    }

    pub fn new_simple(items: T, total: i64) -> Self {
        Self {
            items,
            total,
            current_page: 1,
            page_size: 0,
            total_pages: 1,
        }
    }
}
