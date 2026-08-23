//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 Mxx 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 线索转移服务层
//! - preview_transfer：预览转移影响范围（不执行任何写操作）
//! - transfer_lead：单事务全量转移线索及关联数据的负责人字段
//!   - 线索/商机/跟进记录: assigned_to
//!   - 跟进记录: 不改 created_by（保留历史溯源）
//! - 转移成功后：发送站内信通知新负责人
//!
//! 说明：线索关联表较少（仅商机和跟进记录），且客户分配历史/修改日志表
//! 强绑定 customer_id 字段无法直接复用，故线索转移不记录历史日志，
//! 仅做事务更新负责人 + 站内信通知。
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::crm::entity::{
    followup, lead, lead::Entity as Lead, opportunity,
};
use crate::modules::crm::service::customer_transfer_service::handle_transfer_tags;
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use crate::utils::string_utils::{
    deserialize_string_or_num_vec_to_i64_vec, deserialize_string_to_u64,
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter,
    TransactionTrait, sea_query::Expr,
};
use serde::{Deserialize, Serialize};

/// 线索转移请求参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadTransferRequest {
    #[serde(deserialize_with = "deserialize_string_or_num_vec_to_i64_vec")]
    pub lead_ids: Vec<i64>,
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub to_user_id: Option<i64>,
    /// 交接原因（字典 dict_label 文本）
    pub transfer_reason: String,
    /// 备注（非必填）
    pub remark: Option<String>,
}

/// 线索转移预览请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadTransferPreviewRequest {
    #[serde(deserialize_with = "deserialize_string_or_num_vec_to_i64_vec")]
    pub lead_ids: Vec<i64>,
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub to_user_id: Option<i64>,
}

/// 线索转移影响范围统计
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct LeadTransferPreviewVO {
    /// 线索数量
    pub lead_count: i64,
    /// 跟进记录数量（source_type=1 线索跟进）
    pub followup_count: i64,
    /// 商机数量（通过 lead_id 关联）
    pub opportunity_count: i64,
    /// 受影响总数（不含线索本身）
    pub affected_total: i64,
}

/// 线索转移结果
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct LeadTransferResult {
    pub transferred_count: i64,
    pub affected_total: i64,
}

/// 预览线索转移影响范围
pub async fn preview_transfer(
    db: &DbConn,
    req: &LeadTransferPreviewRequest,
) -> Result<LeadTransferPreviewVO> {
    if req.lead_ids.is_empty() {
        return Err(Error::from("请选择要转移的线索"));
    }
    let to_user_id = req.to_user_id.unwrap_or(0);
    if to_user_id <= 0 {
        return Err(Error::from("请选择新负责人"));
    }

    let lead_ids = &req.lead_ids;

    // 1. 统计线索数量
    let lead_count = Lead::find()
        .filter(lead::Column::Id.is_in(lead_ids.clone()))
        .filter(lead::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("统计线索数量失败: {}", e)))? as i64;

    // 2. 统计跟进记录数量（source_type=1 为线索跟进）
    let followup_count = followup::Entity::find()
        .filter(followup::Column::LeadId.is_in(lead_ids.clone()))
        .filter(followup::Column::SourceType.eq(1i16))
        .filter(followup::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("统计跟进记录失败: {}", e)))? as i64;

    // 3. 统计商机数量（通过 lead_id 关联）
    let opportunity_count = opportunity::Entity::find()
        .filter(opportunity::Column::LeadId.is_in(lead_ids.clone()))
        .filter(opportunity::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("统计商机数量失败: {}", e)))? as i64;

    let affected_total = followup_count + opportunity_count;

    Ok(LeadTransferPreviewVO {
        lead_count,
        followup_count,
        opportunity_count,
        affected_total,
    })
}

/// 执行线索转移（单事务）
pub async fn transfer_lead(
    db: &DbConn,
    req: &LeadTransferRequest,
    operator_id: i64,
    operator_name: Option<String>,
) -> Result<LeadTransferResult> {
    if req.lead_ids.is_empty() {
        return Err(Error::from("请选择要转移的线索"));
    }
    let to_user_id = req.to_user_id.unwrap_or(0);
    if to_user_id <= 0 {
        return Err(Error::from("请选择新负责人"));
    }

    // 校验新负责人是否存在且启用
    let to_admin = Admin::find_by_id(to_user_id)
        .one(db)
        .await
        .map_err(|e| Error::from(format!("查询新负责人失败: {}", e)))?
        .ok_or_else(|| Error::from("新负责人不存在"))?;
    if to_admin.status.unwrap_or(0) != 1 {
        return Err(Error::from("新负责人已被禁用，无法转移"));
    }
    let to_user_name = to_admin
        .nick_name
        .clone()
        .or(to_admin.user_name.clone())
        .unwrap_or_else(|| to_user_id.to_string());

    // 1. 查询线索原负责人映射
    let leads = Lead::find()
        .filter(lead::Column::Id.is_in(&req.lead_ids))
        .filter(lead::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(format!("查询线索失败: {}", e)))?;

    if leads.is_empty() {
        return Err(Error::from("未找到有效的线索记录"));
    }

    let transferred_count = leads.len() as i64;

    // 2. 开启事务
    let txn = db.begin().await?;

    // 2.1 更新线索表 assigned_to
    lead::Entity::update_many()
        .col_expr(lead::Column::AssignedTo, Expr::value(Some(to_user_id)))
        .filter(lead::Column::Id.is_in(&req.lead_ids))
        .filter(lead::Column::Deleted.eq(0))
        .exec(&txn)
        .await
        .map_err(|e| Error::from(format!("更新线索负责人失败: {}", e)))?;

    // 2.2 更新跟进记录 assigned_to（按 lead_id 过滤，source_type=1 线索跟进）
    let followup_res = followup::Entity::update_many()
        .col_expr(followup::Column::AssignedTo, Expr::value(Some(to_user_id)))
        .filter(followup::Column::LeadId.is_in(&req.lead_ids))
        .filter(followup::Column::SourceType.eq(1i16))
        .filter(followup::Column::Deleted.eq(0))
        .exec(&txn)
        .await
        .map_err(|e| Error::from(format!("更新跟进记录负责人失败: {}", e)))?;
    let followup_count = followup_res.rows_affected as i64;

    // 2.3 更新商机 assigned_to（按 lead_id 过滤）
    let opportunity_res = opportunity::Entity::update_many()
        .col_expr(opportunity::Column::AssignedTo, Expr::value(Some(to_user_id)))
        .filter(opportunity::Column::LeadId.is_in(&req.lead_ids))
        .filter(opportunity::Column::Deleted.eq(0))
        .exec(&txn)
        .await
        .map_err(|e| Error::from(format!("更新商机负责人失败: {}", e)))?;
    let opportunity_count = opportunity_res.rows_affected as i64;

    let affected_total = followup_count + opportunity_count;

    // 2.4 处理原负责人的私有标签交接（离职场景）
    // - 标签标注的客户/线索负责人全部为交接人 → 标签转移给交接人
    // - 交叉/部分交接 → 转为公共标签由管理员接管
    let from_uids: Vec<i64> = leads
        .iter()
        .filter_map(|l| l.assigned_to)
        .filter(|&uid| uid != to_user_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    handle_transfer_tags(&txn, &from_uids, to_user_id).await?;

    // 3. 提交事务
    txn.commit().await?;

    // 4. 发送站内信通知新负责人（失败不影响转移结果）
    if let Err(e) = send_transfer_notice(
        db,
        to_user_id,
        &to_user_name,
        operator_id,
        &operator_name,
        transferred_count,
        affected_total,
        &req.transfer_reason,
        &req.remark,
    )
    .await
    {
        log::warn!("发送线索转移站内信失败: {}", e);
    }

    Ok(LeadTransferResult {
        transferred_count,
        affected_total,
    })
}

/// 发送线索转移站内信通知新负责人
async fn send_transfer_notice(
    db: &DbConn,
    to_user_id: i64,
    to_user_name: &str,
    operator_id: i64,
    operator_name: &Option<String>,
    lead_count: i64,
    affected_total: i64,
    transfer_reason: &str,
    remark: &Option<String>,
) -> Result<()> {
    use crate::modules::system::model::notice::{NoticeModel, NoticeSaveDTO};

    let operator = operator_name.clone().unwrap_or_else(|| "系统".to_string());
    let now = chrono::Local::now().naive_local();
    let title = format!("线索转移通知 - 您接收到 {} 条线索", lead_count);
    let content = format!(
        "<p>您好，{}：</p>\
         <p>您于 {} 接收到 <strong>{}</strong> 条线索，操作人：<strong>{}</strong>。</p>\
         <p>本次转移共影响 <strong>{}</strong> 条关联数据（含跟进记录/商机等）。</p>\
         <p>交接原因：<strong>{}</strong></p>\
         <p>备注：{}</p>\
         <p>请及时跟进这些线索的后续工作。</p>",
        to_user_name,
        now.format("%Y-%m-%d %H:%M:%S"),
        lead_count,
        operator,
        affected_total,
        transfer_reason,
        remark.clone().unwrap_or_else(|| "无".to_string())
    );

    let save_dto = NoticeSaveDTO {
        id: None,
        title: Some(title),
        content: Some(content),
        r#type: Some(4),                       // 4=系统消息
        level: Some("high".to_string()),       // 重要等级
        target_type: Some(2),                  // 2=指定用户
        target_user_ids: Some(to_user_id.to_string()),
        publisher_id: Some(operator_id),
        publish_status: Some(0),
        publish_time: Some(now),
        revoke_time: None,
        create_by: Some(operator_id),
        create_time: Some(now),
        update_by: Some(operator_id),
        update_time: Some(now),
    };

    let notice_id = NoticeModel::insert(db, &save_dto).await?;
    // 发布通知（创建 merge 记录 + WebSocket 推送）
    let _ = crate::modules::system::service::notice_service::update_by_id_publish(
        db,
        &Some(notice_id),
        &Some(operator_id),
    )
    .await;
    Ok(())
}
