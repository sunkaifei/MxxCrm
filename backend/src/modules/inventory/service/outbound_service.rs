//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use sea_orm::sea_query::Expr;
use rust_decimal::Decimal;
use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::model::outbound::*;
use crate::modules::inventory::entity::{outbound, outbound_item, stock, doc_change_log};
use crate::modules::inventory::service::stock_engine;
use crate::modules::inventory::entity::warehouse;
use crate::modules::product::entity::product as product_entity;
use crate::modules::system::entity::admin;
use crate::modules::approval::model::approval::{ApprovalSubmitRequest, ApprovalProcessRequest, ApprovalCancelRequest};
use crate::modules::approval::service::approval_service::ApprovalService;

/// 读取出库审核开关（从 mxx_system_config 表，默认开启）
async fn is_audit_enabled() -> bool {
    crate::modules::system::service::config_service::find_value_by_key_from_db("outbound_audit_enabled")
        .await
        .unwrap_or_else(|| "1".to_string())
            == "1"
}

/// 读取出库审核模式（0=严格 1=宽松），默认严格（仅制单人可提交/编辑）
async fn is_audit_mode_relaxed() -> bool {
    crate::modules::system::service::config_service::find_value_by_key_from_db("outbound_audit_mode")
        .await
        .unwrap_or_else(|| "0".to_string())
            == "1"
}

/// 生成出库单号：CK + yyyyMMdd + 4位流水号
pub async fn generate_outbound_no(db: &DatabaseConnection) -> Result<String> {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let prefix = format!("CK{}", today);

    // 查询当天最大流水号
    let max_no = outbound::Entity::find()
        .filter(outbound::Column::OutboundNo.starts_with(&prefix))
        .order_by_desc(outbound::Column::OutboundNo)
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let seq = match max_no {
        Some(m) => {
            let no = m.outbound_no.unwrap_or_default();
            let seq_str = no.trim_start_matches(&prefix);
            seq_str.parse::<i32>().unwrap_or(0) + 1
        }
        None => 1,
    };

    Ok(format!("{}{:04}", prefix, seq))
}

/// 创建出库单
pub async fn create(
    db: &DatabaseConnection,
    req: &OutboundSaveRequest,
    created_by: i64,
) -> Result<i64> {
    // 审核开关关闭：直接创建为已完成状态并执行库存扣减（复用自动审核逻辑，内部含库存校验）
    if !is_audit_enabled().await {
        return create_and_auto_audit(db, req, created_by).await;
    }

    // 审核开启（默认）：创建为草稿状态，不执行库存变动
    let outbound_no = generate_outbound_no(db).await?;

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    // 1. 插入主表
    let outbound_id = insert(&txn, req, &outbound_no, created_by).await
        .map_err(|e| Error::from(e.to_string()))?;

    // 2. 插入明细
    for item in &req.items {
        let item_active = outbound_item::ActiveModel {
            outbound_id: Set(Some(outbound_id)),
            product_id: Set(Some(item.product_id)),
            product_sku: Set(item.product_sku.clone()),
            quantity: Set(Some(item.quantity)),
            batch_no: Set(item.batch_no.clone()),
            remark: Set(item.remark.clone()),
            deleted: Set(Some(0)),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        item_active.insert(&txn).await.map_err(|e| Error::from(e.to_string()))?;
    }

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(outbound_id)
}

/// 提交审核（草稿/已驳回 → 待审核）：调用审批引擎创建审批实例（制单员提交 → 库管审批）
pub async fn submit_audit(
    db: &DatabaseConnection,
    id: i64,
    operator_id: i64,
    operator_name: &str,
    cc_user_ids: Vec<i64>,
    cc_reason: Option<String>,
) -> Result<i64> {
    let model = outbound::Entity::find_by_id(id)
        .filter(outbound::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("出库单不存在"))?;

    let status = model.status.unwrap_or(0);
    if status != 0 && status != 4 {
        return Err(Error::from("只有草稿或已驳回状态的出库单才能提交审核"));
    }

    // 审核模式：严格模式下仅制单人（创建人）可提交；宽松模式下有权限角色可提交他人草稿
    // （无论审核开关是否开启都执行此校验，防止审核关闭时绕过权限控制）
    if !is_audit_mode_relaxed().await && model.created_by != Some(operator_id) {
        return Err(Error::from("严格模式下只能提交本人创建的出库单"));
    }

    // 审核开关关闭：直接自动完成（先转待审核，再完成审核出库）
    if !is_audit_enabled().await {
        update_status(db, id, 1, operator_id).await
            .map_err(|e| Error::from(e.to_string()))?;
        record_submitted_by(db, id, operator_id).await?;
        return do_complete_audit(db, id, operator_id).await;
    }

    // 提交人姓名（用于审批实例展示）
    let submitter_name = admin::Entity::find_by_id(operator_id)
        .one(db)
        .await
        .ok()
        .flatten()
        .and_then(|a| a.nick_name.or(a.user_name))
        .unwrap_or_else(|| operator_name.to_string());

    let outbound_no = model.outbound_no.clone().unwrap_or_else(|| format!("出库单#{}", id));
    let total_amount = model.total_amount.unwrap_or_default();

    // 调用审批引擎创建审批实例
    let submit_req = ApprovalSubmitRequest {
        flow_code: "outbound_approval".to_string(),
        business_type: "outbound".to_string(),
        business_id: id,
        business_title: Some(outbound_no.clone()),
        submitter_id: operator_id,
        submitter_name: Some(submitter_name),
        extra_data: Some(serde_json::json!({ "amount": total_amount })),
        cc_user_ids: if cc_user_ids.is_empty() { None } else { Some(cc_user_ids) },
        cc_reason,
    };
    let instance_id = ApprovalService::submit(db, &submit_req).await?;

    // 更新出库单状态为待审核(1) + 记录审批实例ID + 提交人
    let now = chrono::Local::now().naive_local();
    let result = outbound::Entity::update_many()
        .col_expr(outbound::Column::Status, Expr::value(1))
        .col_expr(outbound::Column::InstanceId, Expr::value(instance_id))
        .col_expr(outbound::Column::SubmittedBy, Expr::value(operator_id))
        .col_expr(outbound::Column::UpdatedBy, Expr::value(operator_id))
        .col_expr(outbound::Column::UpdateTime, Expr::value(now))
        .filter(outbound::Column::Id.eq(id))
        .filter(outbound::Column::Deleted.eq(0))
        .filter(outbound::Column::Status.is_in([0, 4]))
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if result.rows_affected == 0 {
        return Err(Error::from("出库单状态已变更，请刷新后重试"));
    }

    Ok(id)
}

/// 记录提交人（自动审核路径使用）
async fn record_submitted_by(db: &DatabaseConnection, id: i64, operator_id: i64) -> Result<()> {
    outbound::Entity::update_many()
        .col_expr(outbound::Column::SubmittedBy, Expr::value(operator_id))
        .filter(outbound::Column::Id.eq(id))
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 审核通过（审批引擎）：调用审批引擎通过，审批完成后执行库存出库
pub async fn audit(
    db: &DatabaseConnection,
    id: i64,
    audit_by: i64,
    audit_name: &str,
    comment: Option<String>,
) -> Result<i64> {
    // 查询出库单
    let outbound_order = outbound::Entity::find_by_id(id)
        .filter(outbound::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("出库单不存在"))?;

    // 状态机检查：仅待审核(1)可审核
    let status = outbound_order.status.unwrap_or(0);
    if status != 1 {
        return Err(Error::from(format!("出库单状态异常，当前状态：{}，仅待审核状态可审核", status)));
    }

    let instance_id = outbound_order.instance_id
        .ok_or_else(|| Error::from("审批实例不存在，请重新提交审核"))?;

    // 调用审批引擎处理（通过）
    let process_req = ApprovalProcessRequest {
        instance_id,
        action: 1,
        approver_id: audit_by,
        approver_name: Some(audit_name.to_string()),
        comment,
        ..Default::default()
    };
    ApprovalService::process(db, &process_req).await?;

    // 查询实例最新状态，判断审批是否完成
    let instance = ApprovalService::find_instance_by_id(db, instance_id)
        .await?
        .ok_or_else(|| Error::from("审批实例不存在"))?;

    if instance.status == 3 {
        // 审批完成：执行库存出库 + 状态置为已完成
        do_complete_audit(db, id, audit_by).await?;
    }
    // 多节点流转中：保持待审核(1)，等待后续审批人处理

    Ok(id)
}

/// 执行审核完成的核心逻辑：检查库存 + 扣减库存 + 写流水 + 状态置为已完成(3)
/// 事务内完成，防止并发重复审核
pub async fn do_complete_audit(
    db: &DatabaseConnection,
    id: i64,
    audit_by: i64,
) -> Result<i64> {
    // 1. 查询出库单
    let outbound_order = outbound::Entity::find_by_id(id)
        .filter(outbound::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("出库单不存在"))?;

    // 2. 查询出库明细
    let items = outbound_item::Entity::find()
        .filter(outbound_item::Column::OutboundId.eq(id))
        .filter(outbound_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if items.is_empty() {
        return Err(Error::from("出库单明细为空，无法审核"));
    }

    // 3. 事务执行
    let warehouse_id = outbound_order.warehouse_id.unwrap_or_default();
    let outbound_no = outbound_order.outbound_no.clone().unwrap_or_default();
    let outbound_type = outbound_order.outbound_type.clone().unwrap_or_default();

    // 3.1 库存校验：检查每个产品的可用库存是否充足
    for item in &items {
        let product_id = item.product_id.unwrap_or_default();
        let quantity = item.quantity.unwrap_or_default();
        let stock_record = stock::Entity::find()
            .filter(stock::Column::ProductId.eq(product_id))
            .filter(stock::Column::WarehouseId.eq(warehouse_id))
            .filter(stock::Column::Deleted.eq(0))
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        match stock_record {
            Some(s) => {
                let available = s.available_quantity.unwrap_or_default();
                if quantity > available {
                    return Err(Error::from(format!(
                        "产品[{}]库存不足，当前可用: {}，需出库: {}",
                        product_id, available, quantity
                    )));
                }
            }
            None => {
                return Err(Error::from(format!(
                    "产品[{}]在指定仓库无库存记录",
                    product_id
                )));
            }
        }
    }

    db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            for item in &items {
                let product_id = item.product_id.unwrap_or_default();
                let quantity = item.quantity.unwrap_or_default();

                // 检查可用库存并扣减（含 SELECT FOR UPDATE 行锁）
                stock_engine::decrease_stock(txn, product_id, warehouse_id, quantity).await?;

                // 写入库存流水
                stock_engine::write_stock_log(
                    txn,
                    product_id,
                    warehouse_id,
                    None,
                    "outbound",
                    &format!("{}_out", outbound_type),
                    Some(id),
                    Some(&outbound_no),
                    -quantity,
                    Some(audit_by),
                    None,
                ).await?;
            }

            // 更新出库单状态为已完成(3)，使用 status=1 条件防止并发重复审核
            let now = chrono::Local::now().naive_local();
            let result = outbound::Entity::update_many()
                .col_expr(outbound::Column::Status, Expr::value(3))
                .col_expr(outbound::Column::AuditBy, Expr::value(audit_by))
                .col_expr(outbound::Column::AuditTime, Expr::value(now))
                .col_expr(outbound::Column::UpdateTime, Expr::value(now))
                .filter(outbound::Column::Id.eq(id))
                .filter(outbound::Column::Status.eq(1))
                .filter(outbound::Column::Deleted.eq(0))
                .exec(txn)
                .await?;

            if result.rows_affected == 0 {
                return Err(DbErr::Custom("出库单已被其他用户审核，请刷新后重试".into()));
            }

            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(id)
}

/// 审核驳回（审批引擎）：调用审批引擎驳回，实例状态→4，出库单状态→已驳回(4)
pub async fn reject(
    db: &DatabaseConnection,
    id: i64,
    audit_by: i64,
    audit_name: &str,
    comment: Option<String>,
) -> Result<i64> {
    let outbound_order = outbound::Entity::find_by_id(id)
        .filter(outbound::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("出库单不存在"))?;

    let status = outbound_order.status.unwrap_or(0);
    if status != 1 {
        return Err(Error::from(format!("出库单状态异常，当前状态：{}，仅待审核状态可驳回", status)));
    }

    let instance_id = outbound_order.instance_id
        .ok_or_else(|| Error::from("审批实例不存在，请重新提交审核"))?;

    // 调用审批引擎处理（驳回），实例状态→4
    let process_req = ApprovalProcessRequest {
        instance_id,
        action: 2,
        approver_id: audit_by,
        approver_name: Some(audit_name.to_string()),
        comment,
        ..Default::default()
    };
    ApprovalService::process(db, &process_req).await?;

    // 出库单状态置为已驳回(4)
    update_status(db, id, 4, audit_by).await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 撤回审批（审批引擎：仅发起人可撤回，回写业务状态为草稿）
pub async fn withdraw(
    db: &DatabaseConnection,
    id: i64,
    operator_id: i64,
    operator_name: &str,
) -> Result<i64> {
    let outbound_order = outbound::Entity::find_by_id(id)
        .filter(outbound::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("出库单不存在"))?;

    let status = outbound_order.status.unwrap_or(0);
    if status != 1 {
        return Err(Error::from(format!("出库单状态异常，当前状态：{}，仅待审核状态可撤回", status)));
    }

    // 撤回仅限本人（提交人），审批人/超管不可代撤
    if outbound_order.submitted_by != Some(operator_id) {
        return Err(Error::from("只能撤回本人提交的出库单"));
    }

    let instance_id = outbound_order.instance_id
        .ok_or_else(|| Error::from("审批实例不存在，请重新提交审核"))?;

    // 调用审批引擎撤回（内部校验发起人身份，并回写业务状态为草稿0）
    let cancel_req = ApprovalCancelRequest {
        instance_id,
        cancel_reason: Some("发起人撤回审批".to_string()),
    };
    ApprovalService::cancel_instance(db, &cancel_req, operator_id, operator_name).await?;

    Ok(id)
}

/// 出库单列表查询
pub async fn get_list(
    db: &DatabaseConnection,
    query: &OutboundListQuery,
) -> Result<OutboundListVO> {
    let (models, total) = select_page(db, query).await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut list: Vec<OutboundListItem> = models.into_iter().map(|m| m.into()).collect();

    // 补充仓库名称和创建人名称
    for item in &mut list {
        if let Some(wid) = item.warehouse_id {
            if let Ok(Some(wh)) = warehouse::Entity::find_by_id(wid)
                .filter(warehouse::Column::Deleted.eq(0))
                .one(db).await
            {
                item.warehouse_name = wh.name;
            }
        }
        if let Some(cb) = item.created_by {
            if let Ok(Some(admin)) = admin::Entity::find_by_id(cb).one(db).await {
                item.created_by_name = admin.nick_name.or(admin.user_name);
            }
        }
    }

    Ok(OutboundListVO { list, total })
}

/// 获取出库单详情
pub async fn get_detail(
    db: &DatabaseConnection,
    id: i64,
) -> Result<serde_json::Value> {
    let main = find_by_id(db, id).await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("出库单不存在"))?;

    let items = outbound_item::Entity::find()
        .filter(outbound_item::Column::OutboundId.eq(id))
        .filter(outbound_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 查询仓库名称
    let warehouse_name = if let Some(wid) = main.warehouse_id {
        warehouse::Entity::find_by_id(wid)
            .one(db)
            .await
            .ok()
            .flatten()
            .and_then(|w| w.name)
    } else {
        None
    };

    // 查询创建人姓名
    let created_by_name = if let Some(uid) = main.created_by {
        admin::Entity::find_by_id(uid).one(db).await.ok().flatten()
            .and_then(|a| a.nick_name.or(a.user_name))
    } else {
        None
    };

    // 查询提交人姓名
    let submitted_by_name = if let Some(uid) = main.submitted_by {
        admin::Entity::find_by_id(uid).one(db).await.ok().flatten()
            .and_then(|a| a.nick_name.or(a.user_name))
    } else {
        None
    };

    // 查询审核人姓名
    let audit_by_name = if let Some(uid) = main.audit_by {
        admin::Entity::find_by_id(uid).one(db).await.ok().flatten()
            .and_then(|a| a.nick_name.or(a.user_name))
    } else {
        None
    };

    // 审批实例（详情抽屉展示流程图/审批日志/抄送人，需审批引擎能力）
    let instance = if let Some(iid) = main.instance_id {
        ApprovalService::find_instance_by_id(db, iid).await.ok().flatten()
    } else {
        None
    };

    // 构建扁平 camelCase 主表数据
    let detail = serde_json::json!({
        "id": main.id,
        "outboundNo": main.outbound_no,
        "outboundType": main.outbound_type,
        "sourceOrderId": main.source_order_id,
        "sourceOrderNo": main.source_order_no,
        "warehouseId": main.warehouse_id,
        "warehouseName": warehouse_name,
        "status": main.status,
        "instanceId": main.instance_id,
        "totalQuantity": main.total_quantity,
        "totalAmount": main.total_amount,
        "remark": main.remark,
        "auditBy": main.audit_by,
        "auditByName": audit_by_name,
        "auditTime": main.audit_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        "createdBy": main.created_by,
        "createdByName": created_by_name,
        "submittedBy": main.submitted_by,
        "submittedByName": submitted_by_name,
        "createTime": main.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        "updateTime": main.update_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
    });

    // 构建明细数组（camelCase + 补充产品信息）
    let mut items_json: Vec<serde_json::Value> = Vec::new();
    for item in &items {
        let (product_name, product_code, unit, spec) = if let Some(pid) = item.product_id {
            let product = product_entity::Entity::find_by_id(pid).one(db).await.ok().flatten();
            if let Some(p) = product {
                (
                    p.name.clone(),
                    p.product_no.clone(),
                    p.unit.clone(),
                    p.spec_type.clone(),
                )
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };

        items_json.push(serde_json::json!({
            "id": item.id,
            "outboundId": item.outbound_id,
            "productId": item.product_id,
            "productSku": item.product_sku,
            "productCode": product_code,
            "productName": product_name,
            "spec": spec,
            "unit": unit,
            "quantity": item.quantity,
            "batchNo": item.batch_no,
            "remark": item.remark,
        }));
    }

    Ok(serde_json::json!({ "detail": detail, "items": items_json, "instance": instance }))
}

/// 编辑出库单
/// - status=0（草稿）：直接编辑，无需原因
/// - status=3（已完成）：审核关闭时必须填写修改原因，事务内回滚旧库存 → 软删除旧明细 → 插入新明细 → 重新出库（含库存校验）→ 写调整流水 → 写修改日志
pub async fn update(
    db: &DatabaseConnection,
    id: i64,
    req: &OutboundSaveRequest,
    updated_by: i64,
    change_reason: Option<&str>,
) -> Result<i64> {
    // 查询出库单
    let outbound_order = outbound::Entity::find_by_id(id)
        .filter(outbound::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("出库单不存在"))?;

    let status = outbound_order.status.unwrap_or(0);

    // 草稿状态：直接编辑（现有逻辑）
    if status == 0 {
        // 审核模式：严格模式下仅制单人可编辑草稿；宽松模式下有权限角色可编辑他人草稿
        if !is_audit_mode_relaxed().await && outbound_order.created_by != Some(updated_by) {
            return Err(Error::from("严格模式下只能编辑本人创建的单据"));
        }

        db.transaction::<_, _, DbErr>(|txn| {
            let req = req.clone();
            Box::pin(async move {
                // 更新主表
                update_by_id(txn, id, &req, updated_by).await?;

                // 删除原明细（软删除）
                outbound_item::Entity::update_many()
                    .col_expr(outbound_item::Column::Deleted, Expr::value(1))
                    .filter(outbound_item::Column::OutboundId.eq(id))
                    .exec(txn)
                    .await?;

                // 插入新明细
                let now = chrono::Local::now().naive_local();
                for item in &req.items {
                    let item_active = outbound_item::ActiveModel {
                        outbound_id: Set(Some(id)),
                        product_id: Set(Some(item.product_id)),
                        product_sku: Set(item.product_sku.clone()),
                        quantity: Set(Some(item.quantity)),
                        batch_no: Set(item.batch_no.clone()),
                        remark: Set(item.remark.clone()),
                        deleted: Set(Some(0)),
                        create_time: Set(Some(now)),
                        ..Default::default()
                    };
                    item_active.insert(txn).await?;
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

        return Ok(id);
    }

    // 已完成状态：仅在审核关闭时允许修改
    if status == 3 {
        if is_audit_enabled().await {
            return Err(Error::from("审核模式下已完成的单据不可直接编辑"));
        }
        let reason = change_reason
            .ok_or_else(|| Error::from("修改已完成的出库单必须填写修改原因"))?
            .to_string();

        let warehouse_id = outbound_order.warehouse_id.unwrap_or_default();
        let outbound_no = outbound_order.outbound_no.clone().unwrap_or_default();
        let outbound_type = outbound_order.outbound_type.clone().unwrap_or_default();
        let before_main = serde_json::to_value(&outbound_order).unwrap_or_default();
        let req_clone = req.clone();

        db.transaction::<_, _, DbErr>(|txn| {
            let reason = reason.clone();
            let outbound_no = outbound_no.clone();
            let outbound_type = outbound_type.clone();
            let req_clone = req_clone.clone();
            let before_main = before_main.clone();
            Box::pin(async move {
                // 1. 查询旧明细 + 保存修改前快照
                let old_items = outbound_item::Entity::find()
                    .filter(outbound_item::Column::OutboundId.eq(id))
                    .filter(outbound_item::Column::Deleted.eq(0))
                    .all(txn)
                    .await?;
                let before_snapshot = serde_json::json!({
                    "main": before_main,
                    "items": serde_json::to_value(&old_items).unwrap_or_default(),
                });

                // 2. 回滚旧库存（出库的反操作：加回库存，按当前 avg_cost 还原成本）
                for old_item in &old_items {
                    let product_id = old_item.product_id.unwrap_or_default();
                    let quantity = old_item.quantity.unwrap_or_default();
                    if quantity != Decimal::ZERO {
                        // 取当前库存的 avg_cost 作为回填成本，保证成本不漂移
                        let avg_cost = stock::Entity::find()
                            .filter(stock::Column::ProductId.eq(product_id))
                            .filter(stock::Column::WarehouseId.eq(warehouse_id))
                            .filter(stock::Column::Deleted.eq(0))
                            .one(txn)
                            .await?
                            .and_then(|s| s.avg_cost);
                        stock_engine::increase_stock(txn, product_id, warehouse_id, quantity, avg_cost).await?;
                        stock_engine::write_stock_log(
                            txn,
                            product_id,
                            warehouse_id,
                            None,
                            "outbound",
                            "adjust_rollback",
                            Some(id),
                            Some(&outbound_no),
                            quantity,
                            Some(updated_by),
                            Some("修改已完成出库单-回滚旧库存"),
                        ).await?;
                    }
                }

                // 3. 软删除旧明细
                outbound_item::Entity::update_many()
                    .col_expr(outbound_item::Column::Deleted, Expr::value(1))
                    .filter(outbound_item::Column::OutboundId.eq(id))
                    .exec(txn)
                    .await?;

                // 4. 更新主表（已完成状态，带 status=3 过滤）
                let now = chrono::Local::now().naive_local();
                outbound::Entity::update_many()
                    .col_expr(outbound::Column::OutboundType, Expr::value(req_clone.outbound_type.clone()))
                    .col_expr(outbound::Column::WarehouseId, Expr::value(req_clone.warehouse_id))
                    .col_expr(outbound::Column::SourceOrderId, Expr::value(req_clone.source_order_id))
                    .col_expr(outbound::Column::SourceOrderNo, Expr::value(req_clone.source_order_no.clone()))
                    .col_expr(outbound::Column::TotalQuantity, Expr::value(req_clone.total_quantity))
                    .col_expr(outbound::Column::TotalAmount, Expr::value(req_clone.total_amount))
                    .col_expr(outbound::Column::Remark, Expr::value(req_clone.remark.clone()))
                    .col_expr(outbound::Column::UpdatedBy, Expr::value(updated_by))
                    .col_expr(outbound::Column::UpdateTime, Expr::value(now))
                    .col_expr(outbound::Column::LastChangeReason, Expr::value(reason.clone()))
                    .col_expr(outbound::Column::LastChangeBy, Expr::value(updated_by))
                    .col_expr(outbound::Column::LastChangeTime, Expr::value(now))
                    .filter(outbound::Column::Id.eq(id))
                    .filter(outbound::Column::Status.eq(3))
                    .filter(outbound::Column::Deleted.eq(0))
                    .exec(txn)
                    .await?;

                // 5. 插入新明细
                for item in &req_clone.items {
                    let item_active = outbound_item::ActiveModel {
                        outbound_id: Set(Some(id)),
                        product_id: Set(Some(item.product_id)),
                        product_sku: Set(item.product_sku.clone()),
                        quantity: Set(Some(item.quantity)),
                        batch_no: Set(item.batch_no.clone()),
                        remark: Set(item.remark.clone()),
                        deleted: Set(Some(0)),
                        create_time: Set(Some(now)),
                        ..Default::default()
                    };
                    item_active.insert(txn).await?;
                }

                // 6. 重新执行库存变动（扣减新库存，含可用库存校验）+ 写调整流水
                for item in &req_clone.items {
                    let product_id = item.product_id;
                    let quantity = item.quantity;
                    if quantity != Decimal::ZERO {
                        stock_engine::decrease_stock(txn, product_id, req_clone.warehouse_id, quantity).await?;
                        stock_engine::write_stock_log(
                            txn,
                            product_id,
                            req_clone.warehouse_id,
                            None,
                            "outbound",
                            &format!("{}_adjust", outbound_type),
                            Some(id),
                            Some(&outbound_no),
                            -quantity,
                            Some(updated_by),
                            Some("修改已完成出库单-重新出库"),
                        ).await?;
                    }
                }

                // 7. 写修改日志（doc_change_log）
                let after_snapshot = serde_json::json!({
                    "main": {
                        "outboundType": req_clone.outbound_type,
                        "warehouseId": req_clone.warehouse_id,
                        "totalQuantity": req_clone.total_quantity,
                        "totalAmount": req_clone.total_amount,
                        "remark": req_clone.remark,
                    },
                    "items": serde_json::to_value(&req_clone.items).unwrap_or_default(),
                });
                let operator_name = admin::Entity::find_by_id(updated_by)
                    .one(txn)
                    .await
                    .ok()
                    .flatten()
                    .and_then(|a| a.nick_name.or(a.user_name));

                let log_active = doc_change_log::ActiveModel {
                    doc_type: Set(Some("outbound".to_string())),
                    doc_id: Set(Some(id)),
                    doc_no: Set(Some(outbound_no.clone())),
                    action: Set(Some("update".to_string())),
                    change_reason: Set(Some(reason.clone())),
                    before_snapshot: Set(Some(before_snapshot)),
                    after_snapshot: Set(Some(after_snapshot)),
                    operator_id: Set(Some(updated_by)),
                    operator_name: Set(operator_name),
                    create_time: Set(Some(now)),
                    ..Default::default()
                };
                log_active.insert(txn).await?;

                Ok(())
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

        return Ok(id);
    }

    Err(Error::from("当前状态的出库单不可编辑"))
}

/// 批量删除出库单（仅草稿状态可删除）
pub async fn batch_delete(
    db: &DatabaseConnection,
    ids: &[i64],
) -> Result<i64> {
    crate::modules::inventory::model::outbound::batch_delete(db, ids)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 获取出库单打印数据（主表 + 明细 + 仓库 + 操作人）
pub async fn get_print_data(
    db: &DatabaseConnection,
    id: i64,
) -> Result<serde_json::Value> {
    let main = find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("出库单不存在".to_string()))?;

    let items = outbound_item::Entity::find()
        .filter(outbound_item::Column::OutboundId.eq(id))
        .filter(outbound_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let warehouse_name = if let Some(wid) = main.warehouse_id {
        warehouse::Entity::find_by_id(wid)
            .one(db)
            .await
            .ok()
            .flatten()
            .and_then(|w| w.name)
    } else {
        None
    };

    let creator_name = if let Some(cb) = main.created_by {
        admin::Entity::find_by_id(cb).one(db).await.ok().flatten()
            .and_then(|a| a.nick_name.or(a.user_name))
    } else {
        None
    };

    let auditor_name = if let Some(ab) = main.audit_by {
        admin::Entity::find_by_id(ab).one(db).await.ok().flatten()
            .and_then(|a| a.nick_name.or(a.user_name))
    } else {
        None
    };

    // 提交人姓名
    let submitter_name = if let Some(sb) = main.submitted_by {
        admin::Entity::find_by_id(sb).one(db).await.ok().flatten()
            .and_then(|a| a.nick_name.or(a.user_name))
    } else {
        None
    };

    // 构建明细（camelCase + 产品信息）
    let mut items_json: Vec<serde_json::Value> = Vec::new();
    for item in &items {
        let (product_name, product_code, unit, spec) = if let Some(pid) = item.product_id {
            let product = product_entity::Entity::find_by_id(pid).one(db).await.ok().flatten();
            if let Some(p) = product {
                (p.name.clone(), p.product_no.clone(), p.unit.clone(), p.spec_type.clone())
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };

        items_json.push(serde_json::json!({
            "productCode": product_code,
            "productName": product_name,
            "spec": spec,
            "unit": unit,
            "quantity": item.quantity,
            "batchNo": item.batch_no,
            "remark": item.remark,
        }));
    }

    Ok(serde_json::json!({
        "main": {
            "outboundNo": main.outbound_no,
            "outboundType": main.outbound_type,
            "status": main.status,
            "totalQuantity": main.total_quantity,
            "totalAmount": main.total_amount,
            "remark": main.remark,
            "warehouseName": warehouse_name,
            "createTime": main.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            "auditTime": main.audit_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        },
        "items": items_json,
        "warehouse": { "name": warehouse_name },
        "creator": { "nick_name": creator_name, "user_name": creator_name },
        "submitter": { "nick_name": submitter_name, "user_name": submitter_name },
        "auditor": { "nick_name": auditor_name, "user_name": auditor_name },
    }))
}

/// 导出出库单列表（CSV 字符串）
pub async fn export_list(db: &DatabaseConnection, query: &OutboundListQuery) -> Result<String> {
    let (models, _total) = select_page(db, query)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut csv = String::from("ID,出库单号,出库类型,源单号,仓库ID,状态,总数量,总金额,备注,创建时间\n");
    for m in models {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            m.id,
            m.outbound_no.unwrap_or_default(),
            m.outbound_type.unwrap_or_default(),
            m.source_order_no.unwrap_or_default(),
            m.warehouse_id.unwrap_or_default(),
            m.status.unwrap_or_default(),
            m.total_quantity.unwrap_or_default(),
            m.total_amount.unwrap_or_default(),
            m.remark.unwrap_or_default().replace(',', " "),
            m.create_time
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default(),
        ));
    }
    Ok(csv)
}

/// 导入出库单（解析 JSON 数组，逐条创建）
pub async fn import_list(
    db: &DatabaseConnection,
    items: Vec<OutboundSaveRequest>,
    created_by: i64,
) -> Result<i64> {
    let mut count = 0i64;
    for req in items {
        create(db, &req, created_by).await?;
        count += 1;
    }
    Ok(count)
}

/// 创建并自动审核（盘亏出库等场景使用，状态直接置为已完成=3）
pub async fn create_and_auto_audit(
    db: &DatabaseConnection,
    req: &OutboundSaveRequest,
    created_by: i64,
) -> Result<i64> {
    // 库存校验：检查每个产品的可用库存是否充足（自动审核会直接扣减库存）
    for item in &req.items {
        let stock_record = stock::Entity::find()
            .filter(stock::Column::ProductId.eq(item.product_id))
            .filter(stock::Column::WarehouseId.eq(req.warehouse_id))
            .filter(stock::Column::Deleted.eq(0))
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        match stock_record {
            Some(s) => {
                let available = s.available_quantity.unwrap_or_default();
                if item.quantity > available {
                    return Err(Error::from(format!(
                        "产品[{}]库存不足，当前可用: {}，需出库: {}",
                        item.product_id, available, item.quantity
                    )));
                }
            }
            None => {
                return Err(Error::from(format!(
                    "产品[{}]在指定仓库无库存记录",
                    item.product_id
                )));
            }
        }
    }

    let outbound_no = generate_outbound_no(db).await?;

    let result_id = db.transaction::<_, _, DbErr>(|txn| {
        let outbound_no = outbound_no.clone();
        let req = req.clone();
        Box::pin(async move {
            // 1. 插入主表（状态直接设为已完成=3）
            let now = chrono::Local::now().naive_local();
            let active = outbound::ActiveModel {
                outbound_no: Set(Some(outbound_no.clone())),
                outbound_type: Set(Some(req.outbound_type.clone())),
                source_order_id: Set(req.source_order_id),
                source_order_no: Set(req.source_order_no.clone()),
                warehouse_id: Set(Some(req.warehouse_id)),
                status: Set(Some(3)), // 已完成
                total_quantity: Set(req.total_quantity),
                total_amount: Set(req.total_amount),
                remark: Set(req.remark.clone()),
                audit_by: Set(Some(created_by)),
                audit_time: Set(Some(now)),
                deleted: Set(Some(0)),
                created_by: Set(Some(created_by)),
                updated_by: Set(Some(created_by)),
                create_time: Set(Some(now)),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            let result = active.insert(txn).await?;
            let outbound_id = result.id;

            // 2. 插入明细并扣减库存
            for item in &req.items {
                let item_active = outbound_item::ActiveModel {
                    outbound_id: Set(Some(outbound_id)),
                    product_id: Set(Some(item.product_id)),
                    product_sku: Set(item.product_sku.clone()),
                    quantity: Set(Some(item.quantity)),
                    batch_no: Set(item.batch_no.clone()),
                    remark: Set(item.remark.clone()),
                    deleted: Set(Some(0)),
                    create_time: Set(Some(now)),
                    ..Default::default()
                };
                item_active.insert(txn).await?;

                // 扣减库存
                stock_engine::decrease_stock(txn, item.product_id, req.warehouse_id, item.quantity).await?;

                // 写入流水
                stock_engine::write_stock_log(
                    txn,
                    item.product_id,
                    req.warehouse_id,
                    None,
                    "outbound",
                    &format!("{}_out", req.outbound_type),
                    Some(outbound_id),
                    Some(&outbound_no),
                    -item.quantity,
                    Some(created_by),
                    None,
                ).await?;
            }

            Ok(outbound_id)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(result_id)
}
