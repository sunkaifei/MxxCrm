//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! CRM 回收站模型：列表查询 / 行 VO / 还原与彻底删除请求体。
//! 设计规格见 docs/CRM数据删除与作废策略-规划方案.md 6.5。

use crate::core::kit::global::{Deserialize, Serialize};
use sea_orm::prelude::DateTime;

/// 回收站列表查询参数（module 为空查全部模块）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecycleListQuery {
    /// 数据模块：customer / opportunity / followup / contact / lead
    pub module: Option<String>,
    /// 页码
    pub page_num: Option<i64>,
    /// 每页条数
    pub page_size: Option<i64>,
    /// 标题关键词
    pub keywords: Option<String>,
}

/// 回收站列表行 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecycleItemVO {
    /// 业务数据 ID
    pub id: i64,
    /// 数据模块（英文标识）
    pub module: String,
    /// 数据模块（中文名，前端直接展示）
    pub module_label: String,
    /// 展示标题（各业务表主标题字段）
    pub title: String,
    /// 删除人 ID
    pub delete_by: Option<i64>,
    /// 删除人姓名（后端补齐，前端直接展示）
    pub delete_by_name: Option<String>,
    /// 业务数据创建时间
    pub create_time: Option<DateTime>,
    /// 删除时间（还原/过期判定依据）
    pub delete_time: Option<DateTime>,
}

/// 回收站操作请求体（还原 / 彻底删除共用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecycleActionRequest {
    /// 数据模块：customer / opportunity / followup / contact / lead
    pub module: String,
    /// 业务数据 ID
    pub id: i64,
}
