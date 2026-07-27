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
use crate::modules::crm::entity::contact_edit_log;

/// 单个字段的变更记录
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditLogItem {
    /// 字段名（如 phone, mobile）
    pub field: String,
    /// 字段中文标签（如 手机号）
    pub field_label: String,
    /// 修改前值
    pub old: Option<String>,
    /// 修改后值
    pub new: Option<String>,
}

/// 联系人修改日志 VO（前端展示用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContactEditLogVO {
    pub id: Option<i64>,
    /// 联系人ID
    pub contact_id: Option<i64>,
    /// 编辑人ID
    pub editor_id: Option<i64>,
    /// 编辑人姓名
    pub editor_name: Option<String>,
    /// 变更内容
    pub content: Option<Vec<EditLogItem>>,
    /// 编辑时间
    pub edit_time: Option<DateTime>,
}

impl From<contact_edit_log::Model> for ContactEditLogVO {
    fn from(item: contact_edit_log::Model) -> Self {
        let content = item.content.as_ref().and_then(|j| {
            serde_json::from_value::<Vec<EditLogItem>>(j.clone()).ok()
        });
        ContactEditLogVO {
            id: Option::from(item.id),
            contact_id: item.contact_id,
            editor_id: item.editor_id,
            editor_name: item.editor_name,
            content,
            edit_time: item.edit_time,
        }
    }
}

/// 联系人编辑日志查询参数
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactEditLogQuery {
    pub contact_id: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 联系人字段中文标签映射
pub const FIELD_LABELS: &[(&str, &str)] = &[
    ("name", "姓名"),
    ("title", "职位/头衔"),
    ("email", "邮箱"),
    ("phone", "座机"),
    ("mobile", "手机"),
    ("whatsapp", "WhatsApp"),
    ("wechat", "微信"),
    ("qq", "QQ"),
    ("gender", "性别"),
    ("birthday", "生日"),
    ("notes", "备注"),
    ("is_primary", "首要联系人"),
    ("is_billing", "账单联系人"),
    ("is_shipping", "收货联系人"),
];

/// 获取字段中文标签
pub fn get_field_label(field: &str) -> &str {
    FIELD_LABELS.iter()
        .find(|(key, _)| *key == field)
        .map(|(_, label)| *label)
        .unwrap_or(field)
}
