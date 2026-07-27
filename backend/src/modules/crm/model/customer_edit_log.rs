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
/// content 字段为通用 JSON：
/// - log_type=0/1: Vec<EditLogItem>（字段级变更）
/// - log_type=2: 转移日志对象（含原/新负责人、原因、受影响资源等）
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
    /// 变更内容（结构根据 log_type 不同而不同）
    pub content: Option<serde_json::Value>,
    /// 编辑时间
    pub edit_time: Option<DateTime>,
    /// 日志类型：0=基本信息, 1=财务信息, 2=客户转移
    pub log_type: Option<i32>,
}

impl From<customer_edit_log::Model> for CustomerEditLogVO {
    fn from(item: customer_edit_log::Model) -> Self {
        // 保留原始 JSON，由前端按 log_type 解析
        CustomerEditLogVO {
            id: Option::from(item.id),
            customer_id: item.customer_id,
            editor_id: item.editor_id,
            editor_name: item.editor_name,
            content: item.content,
            edit_time: item.edit_time,
            log_type: item.log_type,
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
    /// 日志类型：0=基本信息, 1=财务信息, 2=客户转移, None=全部
    pub log_type: Option<i32>,
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
    // 财务信息字段（camelCase）
    ("taxId", "纳税人识别号"),
    ("invoiceTitle", "发票抬头"),
    ("registeredAddress", "注册地址"),
    ("registeredPhone", "注册电话"),
    ("financePhone", "财务电话"),
    ("bankAccounts", "银行账户"),
];

/// 获取字段中文标签
pub fn get_field_label(field: &str) -> &str {
    FIELD_LABELS.iter()
        .find(|(key, _)| *key == field)
        .map(|(_, label)| *label)
        .unwrap_or(field)
}