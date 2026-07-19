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
/// log_type: 0=基本信息, 1=财务信息
pub async fn log_update(
    db: &impl ConnectionTrait,
    customer_id: i64,
    editor_id: i64,
    editor_name: Option<String>,
    old_data: &serde_json::Value,
    new_data: &serde_json::Value,
    log_type: Option<i32>,
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
        log_type: Set(log_type),
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
        .filter(customer_edit_log::Column::Deleted.eq(0));

    // 按日志类型筛选
    if let Some(lt) = query.log_type {
        q = q.filter(customer_edit_log::Column::LogType.eq(lt));
    }

    q = q.order_by_desc(customer_edit_log::Column::EditTime);

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
        // camelCase 版本（财务信息等使用）
        "customerId", "createdBy", "createTime", "updatedBy", "updateTime",
        "customer_id",
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

        // 银行账户数组字段特殊处理：展示账户摘要而非原始 JSON
        let (old_str, new_str) = if field == "bankAccounts" {
            (
                format_bank_accounts(old_val),
                format_bank_accounts(new_val),
            )
        } else {
            (val_to_string(old_val), val_to_string(new_val))
        };

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

/// 将银行账户数组格式化为可读摘要
/// 输入示例: [{"accountName":"张三","accountNumber":"1234","bankName":"工行","isDefault":true}]
/// 输出示例: "1个账户（默认: 工行-1234）"
/// 注意：bankAccounts 可能存储为 JSON 数组，也可能存储为 JSON 字符串（内含数组）
fn format_bank_accounts(val: Option<&serde_json::Value>) -> Option<String> {
    match val {
        None | Some(serde_json::Value::Null) => None,
        Some(v) => {
            // 如果是字符串，先解析成数组
            let arr_val: serde_json::Value = if let Some(s) = v.as_str() {
                serde_json::from_str::<serde_json::Value>(s).unwrap_or(serde_json::Value::Null)
            } else {
                v.clone()
            };
            let arr = match arr_val.as_array() {
                Some(a) if !a.is_empty() => a,
                _ => return None,
            };
            let count = arr.len();
            // 找默认账户
            let default_account = arr.iter().find(|a| {
                a.get("isDefault").and_then(|v| v.as_bool()).unwrap_or(false)
            }).or_else(|| arr.first());

            let summary = if let Some(acc) = default_account {
                let bank = acc.get("bankName").and_then(|v| v.as_str()).unwrap_or("");
                let acct_no = acc.get("accountNumber").and_then(|v| v.as_str()).unwrap_or("");
                let tail = if acct_no.len() >= 4 { &acct_no[acct_no.len()-4..] } else { acct_no };
                let name = acc.get("accountName").and_then(|v| v.as_str()).unwrap_or("");
                if !bank.is_empty() && !tail.is_empty() {
                    format!("{}个账户（默认: {}-{}）", count, bank, tail)
                } else if !name.is_empty() {
                    format!("{}个账户（默认: {}）", count, name)
                } else {
                    format!("{}个账户", count)
                }
            } else {
                format!("{}个账户", count)
            };
            Some(summary)
        }
    }
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