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
use crate::modules::system::entity::edit_log;
use crate::modules::system::entity::edit_log::Entity as EditLog;
use crate::modules::system::model::edit_log::{EditLogItem, EditLogQuery, EditLogVO};
use sea_orm::{
    ColumnTrait, Condition, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
    ConnectionTrait,
};

/// 业务类型常量
pub const BUSINESS_TYPE_QUOTATION: i32 = 1;
pub const BUSINESS_TYPE_ORDER: i32 = 2;
pub const BUSINESS_TYPE_CONTRACT: i32 = 3;
pub const BUSINESS_TYPE_SHIPMENT: i32 = 4;

/// 记录修改日志
/// 对比 old_data 和 new_data，只记录有差异的字段，无差异时不写入
pub async fn log_update(
    db: &impl ConnectionTrait,
    business_type: i32,
    business_id: i64,
    business_no: Option<String>,
    business_title: Option<String>,
    editor_id: i64,
    editor_name: Option<String>,
    old_data: &serde_json::Value,
    new_data: &serde_json::Value,
    field_labels: &[(&str, &str)],
) -> Result<()> {
    let changes = compare_changes(old_data, new_data, field_labels);
    if changes.is_empty() {
        return Ok(());
    }

    let content_json = serde_json::to_value(&changes)
        .map_err(|e| Error::from(format!("序列化变更内容失败: {}", e)))?;

    let now = chrono::Local::now().naive_local();
    let am = edit_log::ActiveModel {
        business_type: Set(Some(business_type)),
        business_id: Set(Some(business_id)),
        business_no: Set(business_no),
        business_title: Set(business_title),
        editor_id: Set(Some(editor_id)),
        editor_name: Set(editor_name),
        content: Set(Some(content_json)),
        edit_time: Set(Some(now)),
        deleted: Set(Some(0)),
        ..Default::default()
    };

    EditLog::insert(am)
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("插入编辑日志失败: {}", e)))?;

    Ok(())
}

/// 记录单次操作日志（用于「创建」「删除」「签收」等无 old/new 对比的场景）
/// content_items 由调用方直接构造（如「新建发货单，共N件商品」）
pub async fn log_action(
    db: &impl ConnectionTrait,
    business_type: i32,
    business_id: i64,
    business_no: Option<String>,
    business_title: Option<String>,
    editor_id: i64,
    editor_name: Option<String>,
    content_items: Vec<EditLogItem>,
) -> Result<()> {
    if content_items.is_empty() {
        return Ok(());
    }

    let content_json = serde_json::to_value(&content_items)
        .map_err(|e| Error::from(format!("序列化操作日志失败: {}", e)))?;

    let now = chrono::Local::now().naive_local();
    let am = edit_log::ActiveModel {
        business_type: Set(Some(business_type)),
        business_id: Set(Some(business_id)),
        business_no: Set(business_no),
        business_title: Set(business_title),
        editor_id: Set(Some(editor_id)),
        editor_name: Set(editor_name),
        content: Set(Some(content_json)),
        edit_time: Set(Some(now)),
        deleted: Set(Some(0)),
        ..Default::default()
    };

    EditLog::insert(am)
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("插入操作日志失败: {}", e)))?;

    Ok(())
}

/// 分页查询编辑日志（管理员视图，可看所有）
pub async fn query_page(
    db: &DbConn,
    query: EditLogQuery,
) -> Result<ResultPage<Vec<EditLogVO>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1).min(100);

    let mut q = EditLog::find().filter(edit_log::Column::Deleted.eq(0));

    if let Some(bt) = query.business_type {
        q = q.filter(edit_log::Column::BusinessType.eq(bt));
    }
    if let Some(bid) = query.business_id {
        q = q.filter(edit_log::Column::BusinessId.eq(bid));
    }
    if let Some(eid) = query.editor_id {
        q = q.filter(edit_log::Column::EditorId.eq(eid));
    }
    if let Some(kw) = query.keyword.filter(|k| !k.trim().is_empty()) {
        let kw = format!("%{}%", kw.trim());
        q = q.filter(
            Condition::any()
                .add(edit_log::Column::BusinessNo.like(kw.as_str()))
                .add(edit_log::Column::BusinessTitle.like(kw.as_str()))
                .add(edit_log::Column::EditorName.like(kw.as_str())),
        );
    }

    q = q.order_by_desc(edit_log::Column::EditTime);

    let total = q.clone().count(db).await? as i64;
    let paginator = q.paginate(db, page_size as u64);
    let items: Vec<edit_log::Model> = paginator.fetch_page((page - 1) as u64).await?;

    let list: Vec<EditLogVO> = items.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new(list, total, page, page_size))
}

/// 比较两个 JSON 对象，返回有差异的字段列表
fn compare_changes(
    old: &serde_json::Value,
    new: &serde_json::Value,
    field_labels: &[(&str, &str)],
) -> Vec<EditLogItem> {
    // 系统内部字段，不记录到修改日志
    const IGNORED_FIELDS: &[&str] = &[
        "id", "deleted", "created_by", "create_time",
        "updated_by", "update_time", "create_by", "update_by",
        "createdAt", "updatedAt",
        "approval_status", "instance_id", "current_approval_stage",
        "next_approver_id", "approval_amount_limit",
        // camelCase
        "createTime", "updateTime", "createBy", "updateBy",
        "approvalStatus", "instanceId", "currentApprovalStage",
        "nextApproverId", "approvalAmountLimit",
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
        let old_val = old.get(&field);
        let new_val = new.get(&field);

        let old_str = val_to_string(old_val);
        let new_str = val_to_string(new_val);

        if old_str != new_str {
            changes.push(EditLogItem {
                field: field.clone(),
                field_label: get_field_label(&field, field_labels).to_string(),
                old: old_str,
                new: new_str,
            });
        }
    }

    changes
}

/// 获取字段中文标签
fn get_field_label<'a>(field: &'a str, labels: &'a [(&str, &str)]) -> &'a str {
    labels
        .iter()
        .find(|(key, _)| *key == field)
        .map(|(_, label)| *label)
        .unwrap_or(field)
}

/// 将 JSON Value 转为 Option<String>
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
