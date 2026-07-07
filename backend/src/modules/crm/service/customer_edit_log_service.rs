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
use crate::modules::crm::entity::customer_edit_log;
use crate::modules::crm::entity::customer_edit_log::Entity as CustomerEditLog;
use crate::modules::crm::model::customer_edit_log::{
    CustomerEditLogQuery, CustomerEditLogVO, EditLogItem, get_field_label,
};
use sea_orm::{
    ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set, ConnectionTrait,
};
use sea_orm::prelude::Json;
use serde_json::json;

/// 记录客户修改日志
/// 对比 old_data 和 new_data（均为 serde_json::Value 对象），
/// 只记录有差异的字段，无差异时不写入
pub async fn log_update(
    db: &impl ConnectionTrait,
    customer_id: i64,
    editor_id: i64,
    editor_name: Option<String>,
    old_data: &serde_json::Value,
    new_data: &serde_json::Value,
) -> Result<()> {
    let changes = compare_changes(old_data, new_data);
    if changes.is_empty() {
        return Ok(()); // 无变化，不记录
    }

    let content_json = serde_json::to_value(&changes)
        .map_err(|e| Error::from(format!("序列化变更内容失败: {}", e)))?;

    let now = chrono::Local::now().naive_local();
    let am = customer_edit_log::ActiveModel {
        customer_id: Set(Some(customer_id)),
        editor_id: Set(Some(editor_id)),
        editor_name: Set(editor_name),
        content: Set(Some(content_json)),
        edit_time: Set(Some(now)),
        deleted: Set(Some(0)),
        ..Default::default()
    };

    CustomerEditLog::insert(am)
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("插入客户修改日志失败: {}", e)))?;

    Ok(())
}

/// 记录客户删除日志
pub async fn log_delete(
    db: &impl ConnectionTrait,
    customer_id: i64,
    editor_id: i64,
    editor_name: Option<String>,
    old_data: &serde_json::Value,
) -> Result<()> {
    // 删除时记录所有字段的原值作为一个变更项
    let items = vec![EditLogItem {
        field: "deleted".to_string(),
        field_label: "删除客户".to_string(),
        old: Some(format!("{:?}", old_data)),
        new: None,
    }];

    let content_json = serde_json::to_value(&items)
        .map_err(|e| Error::from(format!("序列化删除日志失败: {}", e)))?;

    let now = chrono::Local::now().naive_local();
    let am = customer_edit_log::ActiveModel {
        customer_id: Set(Some(customer_id)),
        editor_id: Set(Some(editor_id)),
        editor_name: Set(editor_name),
        content: Set(Some(content_json)),
        edit_time: Set(Some(now)),
        deleted: Set(Some(0)),
        ..Default::default()
    };

    CustomerEditLog::insert(am)
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("插入客户删除日志失败: {}", e)))?;

    Ok(())
}

/// 分页查询客户修改日志
pub async fn query_by_customer(
    db: &DbConn,
    query: CustomerEditLogQuery,
) -> Result<ResultPage<Vec<CustomerEditLogVO>>> {
    let customer_id = query.customer_id.ok_or_else(|| Error::from("客户ID不能为空"))?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1).min(100);

    let mut q = CustomerEditLog::find()
        .filter(customer_edit_log::Column::CustomerId.eq(customer_id))
        .filter(customer_edit_log::Column::Deleted.eq(0))
        .order_by_desc(customer_edit_log::Column::EditTime);

    let total = q.clone().count(db).await? as i64;
    let paginator = q.paginate(db, page_size as u64);
    let items: Vec<customer_edit_log::Model> = paginator.fetch_page((page - 1) as u64).await?;

    let list: Vec<CustomerEditLogVO> = items.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new(list, total, page, page_size))
}

/// 比较两个 JSON 对象，返回有差异的字段列表
fn compare_changes(old: &serde_json::Value, new: &serde_json::Value) -> Vec<EditLogItem> {
    // 系统内部字段及非客户信息字段，不记录到修改日志
    const IGNORED_FIELDS: &[&str] = &[
        "id", "deleted", "customer_no", "created_by", "create_time",
        "updated_by", "update_time",
        "total_deal_amount", "total_deal_count",
        "assigned_to",
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

        // 将 Option 值转为可比较的字符串表示
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