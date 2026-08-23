//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 Mxx 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 客户转移服务层
//! - preview_transfer：预览转移影响范围（不执行任何写操作）
//! - transfer_customer：单事务全量转移客户及关联数据的负责人字段
//!   - 客户/商机/合同: assigned_to
//!   - 回款计划/报价单/订单/发票/回款: owner_user_id
//!   - 跟进记录: 不改 created_by（保留历史溯源）
//! - 转移成功后：写分配历史 + 客户修改日志 + 站内信通知新负责人
//!

use crate::core::errors::error::{Error, Result};
use crate::core::kit::global::Deserialize;
use crate::modules::crm::entity::{
    contract, contract_payment_plan, customer, customer::Entity as Customer, lead,
    lead::Entity as Lead, opportunity,
};
use crate::modules::crm::service::{
    assign_history_service, customer_edit_log_service,
    customer_edit_log_service::TransferAffected,
};
use crate::modules::sale::entity::{invoice, order, payment, quotation};
use crate::modules::system::entity::{admin::Entity as Admin, tag, tag_merge};
use crate::modules::system::entity::tag_merge::Entity as TagMerge;
use crate::utils::string_utils::{
    deserialize_string_or_num_vec_to_i64_vec, deserialize_string_to_u64,
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter,
    TransactionTrait, sea_query::Expr, Set,
};
use serde::Serialize;

/// 转移请求参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferRequest {
    #[serde(deserialize_with = "deserialize_string_or_num_vec_to_i64_vec")]
    pub customer_ids: Vec<i64>,
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub to_user_id: Option<i64>,
    /// 交接原因（字典 dict_label 文本）
    pub transfer_reason: String,
    /// 备注（非必填）
    pub remark: Option<String>,
}

/// 转移预览请求
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferPreviewRequest {
    #[serde(deserialize_with = "deserialize_string_or_num_vec_to_i64_vec")]
    pub customer_ids: Vec<i64>,
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub to_user_id: Option<i64>,
}

/// 转移影响范围统计
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransferPreviewVO {
    pub customer_count: i64,
    pub opportunity_count: i64,
    pub quotation_count: i64,
    pub order_count: i64,
    pub contract_count: i64,
    pub payment_plan_count: i64,
    pub payment_count: i64,
    pub invoice_count: i64,
    /// 受影响总数
    pub affected_total: i64,
}

/// 转移结果
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransferResult {
    pub transferred_count: i64,
    pub affected_total: i64,
}

/// 预览转移影响范围
pub async fn preview_transfer(
    db: &DbConn,
    req: &TransferPreviewRequest,
) -> Result<TransferPreviewVO> {
    if req.customer_ids.is_empty() {
        return Err(Error::from("请选择要转移的客户"));
    }
    let to_user_id = req.to_user_id.unwrap_or(0);
    if to_user_id <= 0 {
        return Err(Error::from("请选择新负责人"));
    }

    let customer_ids = &req.customer_ids;
    let customer_count = customer::Entity::find()
        .filter(customer::Column::Id.is_in(customer_ids.clone()))
        .filter(customer::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("统计客户失败: {}", e)))? as i64;

    let opportunity_count = opportunity::Entity::find()
        .filter(opportunity::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(opportunity::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("统计商机失败: {}", e)))? as i64;

    let quotation_count = quotation::Entity::find()
        .filter(quotation::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(quotation::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("统计报价单失败: {}", e)))? as i64;

    let order_count = order::Entity::find()
        .filter(order::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(order::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("统计订单失败: {}", e)))? as i64;

    let contract_count = contract::Entity::find()
        .filter(contract::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(contract::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("统计合同失败: {}", e)))? as i64;

    let payment_plan_count =
        count_payment_plan_by_customers(db, customer_ids).await?;

    let payment_count = payment::Entity::find()
        .filter(payment::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(payment::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("统计回款失败: {}", e)))? as i64;

    let invoice_count = invoice::Entity::find()
        .filter(invoice::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(invoice::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("统计发票失败: {}", e)))? as i64;

    let affected_total = customer_count
        + opportunity_count
        + quotation_count
        + order_count
        + contract_count
        + payment_plan_count
        + payment_count
        + invoice_count;

    Ok(TransferPreviewVO {
        customer_count,
        opportunity_count,
        quotation_count,
        order_count,
        contract_count,
        payment_plan_count,
        payment_count,
        invoice_count,
        affected_total,
    })
}

/// 执行客户转移（单事务）
pub async fn transfer_customer(
    db: &DbConn,
    req: &TransferRequest,
    operator_id: i64,
    operator_name: Option<String>,
) -> Result<TransferResult> {
    if req.customer_ids.is_empty() {
        return Err(Error::from("请选择要转移的客户"));
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

    // 1. 查询客户原负责人映射
    let customers = Customer::find()
        .filter(customer::Column::Id.is_in(&req.customer_ids))
        .filter(customer::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(format!("查询客户失败: {}", e)))?;

    if customers.is_empty() {
        return Err(Error::from("未找到有效的客户记录"));
    }

    // 构建 customer_id -> from_user_id 映射
    let from_user_map: std::collections::HashMap<i64, i64> = customers
        .iter()
        .filter_map(|c| c.assigned_to.map(|a| (c.id, a)))
        .collect();

    // 批量收集所有原负责人ID，用于查询姓名
    let from_user_ids: Vec<i64> = from_user_map.values().copied().collect();
    let from_name_map: std::collections::HashMap<i64, String> =
        crate::modules::system::service::admin_service::build_admin_name_map(db, from_user_ids).await;

    let customer_ids = &req.customer_ids;

    // 2. 开启事务
    let txn = db.begin().await?;

    // 2.1 更新客户表 assigned_to
    customer::Entity::update_many()
        .col_expr(customer::Column::AssignedTo, Expr::value(Some(to_user_id)))
        .filter(customer::Column::Id.is_in(customer_ids.clone()))
        .filter(customer::Column::Deleted.eq(0))
        .exec(&txn)
        .await
        .map_err(|e| Error::from(format!("更新客户负责人失败: {}", e)))?;

    // 2.2 更新商机 assigned_to
    opportunity::Entity::update_many()
        .col_expr(opportunity::Column::AssignedTo, Expr::value(Some(to_user_id)))
        .filter(opportunity::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(opportunity::Column::Deleted.eq(0))
        .exec(&txn)
        .await
        .map_err(|e| Error::from(format!("更新商机负责人失败: {}", e)))?;

    // 2.3 更新合同 assigned_to（同时收集合同ID供回款计划使用）
    contract::Entity::update_many()
        .col_expr(contract::Column::AssignedTo, Expr::value(Some(to_user_id)))
        .filter(contract::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(contract::Column::Deleted.eq(0))
        .exec(&txn)
        .await
        .map_err(|e| Error::from(format!("更新合同负责人失败: {}", e)))?;

    let contract_ids: Vec<i64> = contract::Entity::find()
        .filter(contract::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(contract::Column::Deleted.eq(0))
        .all(&txn)
        .await
        .map_err(|e| Error::from(format!("查询客户合同ID失败: {}", e)))?
        .into_iter()
        .map(|c| c.id)
        .collect();

    // 2.4 更新回款计划 owner_user_id（通过 contract_id 关联）
    if !contract_ids.is_empty() {
        contract_payment_plan::Entity::update_many()
            .col_expr(
                contract_payment_plan::Column::OwnerUserId,
                Expr::value(Some(to_user_id)),
            )
            .filter(contract_payment_plan::Column::ContractId.is_in(contract_ids.clone()))
            .filter(contract_payment_plan::Column::Deleted.eq(0))
            .exec(&txn)
            .await
            .map_err(|e| Error::from(format!("更新回款计划负责人失败: {}", e)))?;
    }

    // 2.5 更新报价单 owner_user_id
    quotation::Entity::update_many()
        .col_expr(quotation::Column::OwnerUserId, Expr::value(Some(to_user_id)))
        .filter(quotation::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(quotation::Column::Deleted.eq(0))
        .exec(&txn)
        .await
        .map_err(|e| Error::from(format!("更新报价单负责人失败: {}", e)))?;

    // 2.6 更新订单 owner_user_id
    order::Entity::update_many()
        .col_expr(order::Column::OwnerUserId, Expr::value(Some(to_user_id)))
        .filter(order::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(order::Column::Deleted.eq(0))
        .exec(&txn)
        .await
        .map_err(|e| Error::from(format!("更新订单负责人失败: {}", e)))?;

    // 2.7 更新发票 owner_user_id
    invoice::Entity::update_many()
        .col_expr(invoice::Column::OwnerUserId, Expr::value(Some(to_user_id)))
        .filter(invoice::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(invoice::Column::Deleted.eq(0))
        .exec(&txn)
        .await
        .map_err(|e| Error::from(format!("更新发票负责人失败: {}", e)))?;

    // 2.8 更新回款 owner_user_id
    payment::Entity::update_many()
        .col_expr(payment::Column::OwnerUserId, Expr::value(Some(to_user_id)))
        .filter(payment::Column::CustomerId.is_in(customer_ids.clone()))
        .filter(payment::Column::Deleted.eq(0))
        .exec(&txn)
        .await
        .map_err(|e| Error::from(format!("更新回款负责人失败: {}", e)))?;

    // 2.9 统计影响范围（用于日志）
    let affected = TransferAffected {
        opportunity_count: opportunity::Entity::find()
            .filter(opportunity::Column::CustomerId.is_in(customer_ids.clone()))
            .filter(opportunity::Column::Deleted.eq(0))
            .count(&txn)
            .await
            .map_err(|e| Error::from(format!("统计商机失败: {}", e)))? as i64,
        quotation_count: quotation::Entity::find()
            .filter(quotation::Column::CustomerId.is_in(customer_ids.clone()))
            .filter(quotation::Column::Deleted.eq(0))
            .count(&txn)
            .await
            .map_err(|e| Error::from(format!("统计报价单失败: {}", e)))? as i64,
        order_count: order::Entity::find()
            .filter(order::Column::CustomerId.is_in(customer_ids.clone()))
            .filter(order::Column::Deleted.eq(0))
            .count(&txn)
            .await
            .map_err(|e| Error::from(format!("统计订单失败: {}", e)))? as i64,
        contract_count: contract::Entity::find()
            .filter(contract::Column::CustomerId.is_in(customer_ids.clone()))
            .filter(contract::Column::Deleted.eq(0))
            .count(&txn)
            .await
            .map_err(|e| Error::from(format!("统计合同失败: {}", e)))? as i64,
        payment_plan_count: if contract_ids.is_empty() {
            0
        } else {
            contract_payment_plan::Entity::find()
                .filter(
                    contract_payment_plan::Column::ContractId.is_in(contract_ids.clone()),
                )
                .filter(contract_payment_plan::Column::Deleted.eq(0))
                .count(&txn)
                .await
                .map_err(|e| Error::from(format!("统计回款计划失败: {}", e)))? as i64
        },
        payment_count: payment::Entity::find()
            .filter(payment::Column::CustomerId.is_in(customer_ids.clone()))
            .filter(payment::Column::Deleted.eq(0))
            .count(&txn)
            .await
            .map_err(|e| Error::from(format!("统计回款失败: {}", e)))? as i64,
        invoice_count: invoice::Entity::find()
            .filter(invoice::Column::CustomerId.is_in(customer_ids.clone()))
            .filter(invoice::Column::Deleted.eq(0))
            .count(&txn)
            .await
            .map_err(|e| Error::from(format!("统计发票失败: {}", e)))? as i64,
    };

    // 2.10 处理原负责人的私有标签交接（离职场景）
    // - 标签标注的客户/线索负责人全部为交接人 → 标签转移给交接人
    // - 交叉/部分交接 → 转为公共标签由管理员接管
    let from_uids: Vec<i64> = from_user_map.values().copied().collect();
    handle_transfer_tags(&txn, &from_uids, to_user_id).await?;

    // 3. 关闭原负责人历史记录 + 新增转移历史记录
    for (customer_id, from_uid) in &from_user_map {
        assign_history_service::record_transfer(
            &txn,
            *customer_id,
            *from_uid,
            to_user_id,
            &req.transfer_reason,
            operator_id,
        )
        .await?;
    }

    // 4. 写入客户修改日志（log_type=2）
    for customer in &customers {
        let from_uid = from_user_map.get(&customer.id).copied().unwrap_or(0);
        let from_name = from_name_map
            .get(&from_uid)
            .cloned()
            .unwrap_or_else(|| "未知".to_string());
        customer_edit_log_service::log_transfer(
            &txn,
            customer.id,
            operator_id,
            operator_name.clone(),
            from_name,
            to_user_name.clone(),
            &req.transfer_reason,
            req.remark.clone(),
            &affected,
        )
        .await?;
    }

    // 5. 提交事务
    txn.commit().await?;

    // 6. 事务提交后：站内信通知新负责人（失败不影响转移结果）
    let customer_count = customers.len() as i64;
    let affected_total = affected.opportunity_count
        + affected.quotation_count
        + affected.order_count
        + affected.contract_count
        + affected.payment_plan_count
        + affected.payment_count
        + affected.invoice_count
        + customer_count;

    // 发送站内信通知（异步，失败仅记录日志）
    if let Err(e) = send_transfer_notice(
        db,
        to_user_id,
        &to_user_name,
        operator_id,
        &operator_name,
        customer_count,
        affected_total,
        &req.transfer_reason,
        &req.remark,
    )
    .await
    {
        log::warn!("[customer_transfer] 发送站内信通知失败: {}", e);
    }

    Ok(TransferResult {
        transferred_count: customer_count,
        affected_total,
    })
}

/// 发送转移成功站内信给新负责人
async fn send_transfer_notice(
    db: &DbConn,
    to_user_id: i64,
    to_user_name: &str,
    operator_id: i64,
    operator_name: &Option<String>,
    customer_count: i64,
    affected_total: i64,
    transfer_reason: &str,
    remark: &Option<String>,
) -> Result<()> {
    use crate::modules::system::model::notice::{NoticeModel, NoticeSaveDTO};

    let operator = operator_name.clone().unwrap_or_else(|| "系统".to_string());
    let now = chrono::Local::now().naive_local();
    let title = format!("客户转移通知 - 您接收到 {} 个客户", customer_count);
    let content = format!(
        "<p>您好，{}：</p>\
         <p>您于 {} 接收到 <strong>{}</strong> 个客户，操作人：<strong>{}</strong>。</p>\
         <p>本次转移共影响 <strong>{}</strong> 条关联数据（含商机/合同/订单/回款等）。</p>\
         <p>交接原因：<strong>{}</strong></p>\
         <p>备注：{}</p>\
         <p>请及时跟进这些客户的后续工作。</p>",
        to_user_name,
        now.format("%Y-%m-%d %H:%M:%S"),
        customer_count,
        operator,
        affected_total,
        transfer_reason,
        remark.clone().unwrap_or_else(|| "无".to_string())
    );

    let save_dto = NoticeSaveDTO {
        id: None,
        title: Some(title),
        content: Some(content),
        r#type: Some(4), // 4=系统消息
        level: Some("high".to_string()), // 重要等级
        target_type: Some(2), // 2=指定用户
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

/// 统计回款计划数量（通过 contract_id 关联客户）
/// 两步查询：先取客户的合同ID，再按合同ID统计回款计划
async fn count_payment_plan_by_customers(
    db: &impl ConnectionTrait,
    customer_ids: &[i64],
) -> Result<i64> {
    if customer_ids.is_empty() {
        return Ok(0);
    }
    let contract_ids: Vec<i64> = contract::Entity::find()
        .filter(contract::Column::CustomerId.is_in(customer_ids.to_vec()))
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(format!("查询客户合同ID失败: {}", e)))?
        .into_iter()
        .map(|c| c.id)
        .collect();

    if contract_ids.is_empty() {
        return Ok(0);
    }

    let count = contract_payment_plan::Entity::find()
        .filter(contract_payment_plan::Column::ContractId.is_in(contract_ids))
        .filter(contract_payment_plan::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("统计回款计划失败: {}", e)))? as i64;

    Ok(count)
}

/// 处理原负责人的私有标签交接（供客户转移/线索转移调用）
///
/// 规则（离职交接：单交接人转移 + 交叉转公共）：
/// - 标签标注的客户/线索当前负责人**全部**为交接人 → 标签所有权转移给交接人
/// - 标签无任何标注 → 直接转移给交接人（无交叉风险）
/// - 负责人集合包含多个不同负责人（交叉交接）或含无主记录 → 转为公共标签（is_global=1）由管理员接管
/// - 系统标签（is_global=1）不处理
pub async fn handle_transfer_tags(
    db: &impl ConnectionTrait,
    from_user_ids: &[i64],
    to_user_id: i64,
) -> Result<()> {
    if from_user_ids.is_empty() {
        return Ok(());
    }
    let tags = tag::Entity::find()
        .filter(tag::Column::Deleted.eq(0))
        .filter(tag::Column::CreatedBy.is_in(from_user_ids.to_vec()))
        .all(db)
        .await
        .map_err(|e| Error::from(format!("查询离职人员私有标签失败: {}", e)))?;
    if tags.is_empty() {
        return Ok(());
    }

    for t in tags {
        if t.is_global == Some(true) {
            continue; // 系统标签不参与交接
        }
        let tag_id = t.id;
        // 查询该标签标注的客户/线索
        let merges = TagMerge::find()
            .filter(tag_merge::Column::TagId.eq(tag_id))
            .filter(
                tag_merge::Column::EntityType.is_in(vec!["customer".to_string(), "lead".to_string()]),
            )
            .all(db)
            .await
            .map_err(|e| Error::from(format!("查询标签标注失败: {}", e)))?;

        // 收集当前负责人集合
        let mut owner_set: std::collections::HashSet<Option<i64>> = std::collections::HashSet::new();
        let customer_ids: Vec<i64> = merges
            .iter()
            .filter(|m| m.entity_type.as_deref() == Some("customer"))
            .filter_map(|m| m.entity_id)
            .collect();
        let lead_ids: Vec<i64> = merges
            .iter()
            .filter(|m| m.entity_type.as_deref() == Some("lead"))
            .filter_map(|m| m.entity_id)
            .collect();
        if !customer_ids.is_empty() {
            let owners = Customer::find()
                .filter(customer::Column::Id.is_in(customer_ids))
                .all(db)
                .await
                .map_err(|e| Error::from(format!("查询客户负责人失败: {}", e)))?;
            for c in owners {
                owner_set.insert(c.assigned_to);
            }
        }
        if !lead_ids.is_empty() {
            let owners = Lead::find()
                .filter(lead::Column::Id.is_in(lead_ids))
                .all(db)
                .await
                .map_err(|e| Error::from(format!("查询线索负责人失败: {}", e)))?;
            for l in owners {
                owner_set.insert(l.assigned_to);
            }
        }

        // 无标注 或 负责人全部为交接人 → 转移；否则（交叉/无主）→ 转公共
        let all_transferred = owner_set.is_empty()
            || (owner_set.len() == 1 && owner_set.contains(&Some(to_user_id)));
        let payload = if all_transferred {
            tag::ActiveModel {
                created_by: Set(Some(to_user_id)),
                update_time: Set(Option::from(chrono::Utc::now().naive_utc())),
                ..Default::default()
            }
        } else {
            tag::ActiveModel {
                is_global: Set(Some(true)),
                update_time: Set(Option::from(chrono::Utc::now().naive_utc())),
                ..Default::default()
            }
        };
        tag::Entity::update_many()
            .set(payload)
            .filter(tag::Column::Id.eq(tag_id))
            .exec(db)
            .await
            .map_err(|e| {
                Error::from(format!(
                    "处理标签【{}】交接失败: {}",
                    t.tag_name.unwrap_or_default(),
                    e
                ))
            })?;
    }
    Ok(())
}
