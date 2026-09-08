//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::sale::entity::invoice::Model as InvoiceModelType;
use crate::modules::sale::entity::invoice_edit_log;
use crate::modules::sale::entity::invoice_edit_log::Entity as InvoiceEditLog;
use crate::modules::sale::model::invoice::InvoiceUpdateRequest;
use crate::modules::sale::model::invoice_edit_log::{EditLogItem, InvoiceEditLogVO, get_field_label};
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde_json::json;

/// 记录发票修改日志
/// 对比 old_data 和 new_data（均为 serde_json::Value 对象），只记录有差异的字段，无差异时不写入。
/// instance_id 为发票当前的审批实例（驳回/撤回所对应的实例），保证修改可追溯到"驳回它的那次审批"。
pub async fn log_update(
    db: &impl ConnectionTrait,
    invoice_id: i64,
    editor_id: i64,
    editor_name: Option<String>,
    old_data: &serde_json::Value,
    new_data: &serde_json::Value,
    instance_id: Option<i64>,
) -> Result<()> {
    let changes = compare_changes(old_data, new_data);
    if changes.is_empty() {
        return Ok(()); // 无变化，不记录
    }

    let content_json = serde_json::to_value(&changes)
        .map_err(|e| Error::from(format!("序列化变更内容失败: {}", e)))?;

    let now = chrono::Local::now().naive_local();
    let am = invoice_edit_log::ActiveModel {
        invoice_id: Set(Some(invoice_id)),
        editor_id: Set(Some(editor_id)),
        editor_name: Set(editor_name),
        content: Set(Some(content_json)),
        edit_time: Set(Some(now)),
        instance_id: Set(instance_id),
        deleted: Set(Some(0)),
        ..Default::default()
    };

    InvoiceEditLog::insert(am)
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("插入发票修改日志失败: {}", e)))?;

    Ok(())
}

/// 按发票查询全部修改留痕（时间正序，供"流转记录"聚合展示）
pub async fn query_by_invoice(
    db: &impl ConnectionTrait,
    invoice_id: i64,
) -> Result<Vec<InvoiceEditLogVO>> {
    let items = InvoiceEditLog::find()
        .filter(invoice_edit_log::Column::InvoiceId.eq(invoice_id))
        .filter(invoice_edit_log::Column::Deleted.eq(0))
        .order_by_asc(invoice_edit_log::Column::EditTime)
        .all(db)
        .await
        .map_err(|e| Error::from(format!("查询发票修改日志失败: {}", e)))?;
    Ok(items.into_iter().map(|m| m.into()).collect())
}

/// 将发票实体模型转为 JSON 对象（仅含参与留痕比较的业务字段）
pub fn invoice_model_to_json(model: &InvoiceModelType) -> serde_json::Value {
    json!({
        "title": model.title.clone(),
        "invoice_type": model.invoice_type,
        "contract_id": model.contract_id,
        "order_id": model.order_id,
        "customer_id": model.customer_id,
        "customer_name": model.customer_name.clone(),
        "tax_no": model.tax_no.clone(),
        "invoice_date": model.invoice_date.map(|d| d.to_string()),
        "due_date": model.due_date.map(|d| d.to_string()),
        "amount": model.amount.map(|v| v.to_string()),
        "tax_rate": model.tax_rate.map(|v| v.to_string()),
        "tax_amount": model.tax_amount.map(|v| v.to_string()),
        "currency": model.currency,
        "buyer_name": model.buyer_name.clone(),
        "buyer_tax_no": model.buyer_tax_no.clone(),
        "buyer_address": model.buyer_address.clone(),
        "buyer_bank": model.buyer_bank.clone(),
        "remark": model.remark.clone(),
        "owner_user_id": model.owner_user_id,
        "dept_id": model.dept_id,
    })
}

/// 在 old_json 基础上应用本次更新请求传入的字段，得到 new_json（部分更新语义）
pub fn apply_invoice_update(
    old_json: &serde_json::Value,
    req: &InvoiceUpdateRequest,
) -> serde_json::Value {
    let mut new_json = old_json.clone();
    if let Some(v) = &req.title { new_json["title"] = json!(v); }
    if let Some(v) = req.invoice_type { new_json["invoice_type"] = json!(v); }
    if let Some(v) = req.contract_id { new_json["contract_id"] = json!(v); }
    if let Some(v) = req.order_id { new_json["order_id"] = json!(v); }
    if let Some(v) = req.customer_id { new_json["customer_id"] = json!(v); }
    if let Some(v) = &req.customer_name { new_json["customer_name"] = json!(v); }
    if let Some(v) = &req.tax_no { new_json["tax_no"] = json!(v); }
    if let Some(v) = req.invoice_date { new_json["invoice_date"] = json!(v.to_string()); }
    if let Some(v) = req.due_date { new_json["due_date"] = json!(v.to_string()); }
    if let Some(v) = req.amount { new_json["amount"] = json!(v.to_string()); }
    if let Some(v) = req.tax_rate { new_json["tax_rate"] = json!(v.to_string()); }
    if let Some(v) = req.tax_amount { new_json["tax_amount"] = json!(v.to_string()); }
    if let Some(v) = req.currency { new_json["currency"] = json!(v); }
    if let Some(v) = &req.buyer_name { new_json["buyer_name"] = json!(v); }
    if let Some(v) = &req.buyer_tax_no { new_json["buyer_tax_no"] = json!(v); }
    if let Some(v) = &req.buyer_address { new_json["buyer_address"] = json!(v); }
    if let Some(v) = &req.buyer_bank { new_json["buyer_bank"] = json!(v); }
    if let Some(v) = &req.remark { new_json["remark"] = json!(v); }
    if let Some(v) = req.owner_user_id { new_json["owner_user_id"] = json!(v); }
    if let Some(v) = req.dept_id { new_json["dept_id"] = json!(v); }
    new_json
}

/// 比较两个 JSON 对象，返回有差异的字段列表
/// 忽略系统内部字段与状态字段（status/approval_status/instance_id 等由引擎驱动，不属于"修改内容"）
fn compare_changes(old: &serde_json::Value, new: &serde_json::Value) -> Vec<EditLogItem> {
    const IGNORED_FIELDS: &[&str] = &[
        "id", "invoice_no", "deleted", "create_by", "create_time",
        "updated_by", "update_time", "status", "approval_status", "instance_id",
    ];
    let mut changes = Vec::new();

    let mut all_fields: Vec<String> = Vec::new();
    if let Some(obj) = old.as_object() {
        for key in obj.keys() {
            if !all_fields.contains(key) && !IGNORED_FIELDS.contains(&key.as_str()) {
                all_fields.push(key.clone());
            }
        }
    }
    if let Some(obj) = new.as_object() {
        for key in obj.keys() {
            if !all_fields.contains(key) && !IGNORED_FIELDS.contains(&key.as_str()) {
                all_fields.push(key.clone());
            }
        }
    }

    for field in all_fields {
        let old_val = val_to_string(old.get(&field));
        let new_val = val_to_string(new.get(&field));
        if old_val != new_val {
            changes.push(EditLogItem {
                field: field.clone(),
                field_label: get_field_label(&field).to_string(),
                old: old_val,
                new: new_val,
            });
        }
    }

    changes
}

/// 将 JSON Value 转为 Option<String>，null/None 都转为 None
fn val_to_string(val: Option<&serde_json::Value>) -> Option<String> {
    match val {
        None | Some(serde_json::Value::Null) => None,
        Some(v) => {
            let s = match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => v.to_string(),
            };
            if s.is_empty() { None } else { Some(s) }
        }
    }
}
