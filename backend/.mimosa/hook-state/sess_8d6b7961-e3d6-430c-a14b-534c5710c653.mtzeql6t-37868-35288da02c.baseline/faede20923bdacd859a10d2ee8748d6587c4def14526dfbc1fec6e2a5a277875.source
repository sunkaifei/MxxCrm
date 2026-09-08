//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::sale::entity::invoice_edit_log;
use sea_orm::prelude::DateTime;
use serde::{Deserialize, Serialize};

/// 单个字段的变更记录
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditLogItem {
    /// 字段名（如 amount, customer_name）
    pub field: String,
    /// 字段中文标签（如 发票金额, 客户名称）
    pub field_label: String,
    /// 修改前值
    pub old: Option<String>,
    /// 修改后值
    pub new: Option<String>,
}

/// 发票修改留痕 VO（前端"流转记录"展示用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceEditLogVO {
    pub id: Option<i64>,
    /// 发票ID
    pub invoice_id: Option<i64>,
    /// 编辑人ID
    pub editor_id: Option<i64>,
    /// 编辑人姓名
    pub editor_name: Option<String>,
    /// 变更内容（Vec<EditLogItem>）
    pub content: Option<serde_json::Value>,
    /// 编辑时间
    pub edit_time: Option<DateTime>,
    /// 关联审批实例ID
    pub instance_id: Option<i64>,
}

impl From<invoice_edit_log::Model> for InvoiceEditLogVO {
    fn from(item: invoice_edit_log::Model) -> Self {
        InvoiceEditLogVO {
            id: Option::from(item.id),
            invoice_id: item.invoice_id,
            editor_id: item.editor_id,
            editor_name: item.editor_name,
            content: item.content,
            edit_time: item.edit_time,
            instance_id: item.instance_id,
        }
    }
}

/// 发票字段中文标签映射（用于生成字段级 diff 的 fieldLabel）
pub const FIELD_LABELS: &[(&str, &str)] = &[
    ("title", "发票标题"),
    ("invoice_type", "发票类型"),
    ("contract_id", "关联合同"),
    ("order_id", "关联订单"),
    ("customer_id", "客户ID"),
    ("customer_name", "客户名称"),
    ("tax_no", "税号"),
    ("invoice_date", "开票日期"),
    ("due_date", "到期日"),
    ("amount", "发票金额"),
    ("tax_rate", "税率"),
    ("tax_amount", "税额"),
    ("currency", "币种"),
    ("buyer_name", "购买方名称"),
    ("buyer_tax_no", "购买方税号"),
    ("buyer_address", "购买方地址"),
    ("buyer_bank", "购买方开户行"),
    ("remark", "备注"),
    ("owner_user_id", "负责人"),
    ("dept_id", "所属部门"),
];

/// 获取字段中文标签
pub fn get_field_label(field: &str) -> &str {
    FIELD_LABELS
        .iter()
        .find(|(key, _)| *key == field)
        .map(|(_, label)| *label)
        .unwrap_or(field)
}
