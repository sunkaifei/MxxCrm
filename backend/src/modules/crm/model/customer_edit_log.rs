//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::prelude::DateTime;
use serde::{Deserialize, Serialize};
use crate::modules::crm::entity::customer_edit_log;

/// 单个字段的变更记录
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditLogItem {
    /// 字段名（如 phone, company_name）
    pub field: String,
    /// 字段中文标签（如 手机号, 公司名称）
    pub field_label: String,
    /// 修改前值
    pub old: Option<String>,
    /// 修改后值
    pub new: Option<String>,
}

/// 客户修改日志 VO（前端展示用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomerEditLogVO {
    pub id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 编辑人ID
    pub editor_id: Option<i64>,
    /// 编辑人姓名
    pub editor_name: Option<String>,
    /// 变更内容
    pub content: Option<Vec<EditLogItem>>,
    /// 编辑时间
    pub edit_time: Option<DateTime>,
}

impl From<customer_edit_log::Model> for CustomerEditLogVO {
    fn from(item: customer_edit_log::Model) -> Self {
        let content = item.content.as_ref().and_then(|j| {
            serde_json::from_value::<Vec<EditLogItem>>(j.clone()).ok()
        });
        CustomerEditLogVO {
            id: Option::from(item.id),
            customer_id: item.customer_id,
            editor_id: item.editor_id,
            editor_name: item.editor_name,
            content,
            edit_time: item.edit_time,
        }
    }
}

/// 客户编辑日志查询参数
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerEditLogQuery {
    pub customer_id: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 客户字段中文标签映射
/// 用于比较旧值/新值时生成 fieldLabel
pub const FIELD_LABELS: &[(&str, &str)] = &[
    ("company_name", "公司名称"),
    ("short_name", "公司简称"),
    ("country", "国家"),
    ("region", "地区/省份"),
    ("address", "详细地址"),
    ("website", "公司官网"),
    ("industry", "所属行业"),
    ("level", "客户等级"),
    ("source", "客户来源"),
    ("currency", "币种"),
    ("credit_limit", "信用额度"),
    ("credit_days", "信用天数"),
    ("assigned_to", "负责人"),
    ("cooperated_at", "合作日期"),
    ("birthday_month", "生日月份"),
    ("description", "描述/备注"),
    ("custom_fields", "自定义字段"),
];

/// 获取字段中文标签
pub fn get_field_label(field: &str) -> &str {
    FIELD_LABELS.iter()
        .find(|(key, _)| *key == field)
        .map(|(_, label)| *label)
        .unwrap_or(field)
}