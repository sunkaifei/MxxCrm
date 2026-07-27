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
use crate::core::web::response::ResultPage;
use crate::modules::crm::entity::contact_edit_log;
use crate::modules::crm::entity::contact_edit_log::Entity as ContactEditLog;
use crate::modules::crm::model::contact_edit_log::{
    ContactEditLogQuery, ContactEditLogVO, EditLogItem, get_field_label,
};
use sea_orm::{
    ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set, ConnectionTrait,
};
use sea_orm::prelude::Json;
use serde_json::json;

/// 记录联系人修改日志
/// 对比 old_data 和 new_data（均为 serde_json::Value 对象），
/// 只记录有差异的字段，无差异时不写入
pub async fn log_update(
    db: &impl ConnectionTrait,
    contact_id: i64,
    editor_id: i64,
    editor_name: Option<String>,
    old_data: &serde_json::Value,
    new_data: &serde_json::Value,
) -> Result<()> {
    let changes = compare_changes(old_data, new_data);
    if changes.is_empty() {
        return Ok(());
    }

    let content_json = serde_json::to_value(&changes)
        .map_err(|e| Error::from(format!("序列化变更内容失败: {}", e)))?;

    let now = chrono::Local::now().naive_local();
    let am = contact_edit_log::ActiveModel {
        contact_id: Set(Some(contact_id)),
        editor_id: Set(Some(editor_id)),
        editor_name: Set(editor_name),
        content: Set(Some(content_json)),
        edit_time: Set(Some(now)),
        deleted: Set(Some(0)),
        ..Default::default()
    };

    ContactEditLog::insert(am)
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("插入联系人修改日志失败: {}", e)))?;

    Ok(())
}

/// 分页查询联系人修改日志
pub async fn query_by_contact(
    db: &DbConn,
    query: ContactEditLogQuery,
) -> Result<ResultPage<Vec<ContactEditLogVO>>> {
    let contact_id = query.contact_id.ok_or_else(|| Error::from("联系人ID不能为空"))?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1).min(100);

    let q = ContactEditLog::find()
        .filter(contact_edit_log::Column::ContactId.eq(contact_id))
        .filter(contact_edit_log::Column::Deleted.eq(0))
        .order_by_desc(contact_edit_log::Column::EditTime);

    let total = q.clone().count(db).await? as i64;
    let paginator = q.paginate(db, page_size as u64);
    let items: Vec<contact_edit_log::Model> = paginator.fetch_page((page - 1) as u64).await?;

    let list: Vec<ContactEditLogVO> = items.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new(list, total, page, page_size))
}

/// 比较两个 JSON 对象，返回有差异的字段列表
fn compare_changes(old: &serde_json::Value, new: &serde_json::Value) -> Vec<EditLogItem> {
    // 系统内部字段，不记录到修改日志
    const IGNORED_FIELDS: &[&str] = &[
        "id", "deleted", "customer_id", "created_by", "create_time",
        "updated_by", "update_time",
        // camelCase 版本
        "customerId", "createdBy", "createTime", "updatedBy", "updateTime",
    ];
    let mut changes = Vec::new();

    // 合并所有字段名
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
        let old_val = old.get(&field);
        let new_val = new.get(&field);

        let old_str = val_to_string(old_val);
        let new_str = val_to_string(new_val);

        if old_str != new_str {
            changes.push(EditLogItem {
                field: field.clone(),
                field_label: get_field_label(&field).to_string(),
                old: old_str,
                new: new_str,
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

// 避免 unused import 警告（json 在未来扩展中可能使用）
#[allow(dead_code)]
fn _unused() -> serde_json::Value { json!({}) }
