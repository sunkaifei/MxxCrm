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
use crate::modules::inventory::model::inbound::*;
use crate::modules::inventory::model::batch::BatchCreateRequest;
use crate::modules::inventory::entity::{inbound, inbound_item, doc_change_log};
use crate::modules::inventory::service::stock_engine;
use crate::modules::inventory::service::batch_service;
use crate::modules::inventory::entity::warehouse;
use crate::modules::product::entity::product as product_entity;
use crate::modules::system::entity::admin;

/// 读取入库审核开关（从 mxx_system_config 表，默认开启）
async fn is_audit_enabled() -> bool {
    crate::modules::system::service::config_service::find_value_by_key_from_db("inbound_audit_enabled")
        .await
        .unwrap_or_else(|| "1".to_string())
            == "1"
}

/// 生成入库单号：RK + yyyyMMdd + 4位流水号
pub async fn generate_inbound_no(db: &DatabaseConnection) -> Result<String> {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let prefix = format!("RK{}", today);

    // 查询当天最大流水号
    let max_no = inbound::Entity::find()
        .filter(inbound::Column::InboundNo.starts_with(&prefix))
        .order_by_desc(inbound::Column::InboundNo)
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let seq = match max_no {
        Some(m) => {
            let no = m.inbound_no.unwrap_or_default();
            let seq_str = no.trim_start_matches(&prefix);
            seq_str.parse::<i32>().unwrap_or(0) + 1
        }
        None => 1,
    };

    Ok(format!("{}{:04}", prefix, seq))
}

/// 创建入库单
pub async fn create(
    db: &DatabaseConnection,
    req: &InboundSaveRequest,
    created_by: i64,
) -> Result<i64> {
    // 审核开关关闭：直接创建为已完成状态并执行库存增加（复用自动审核逻辑）
    if !is_audit_enabled().await {
        return create_and_auto_audit(db, req, created_by).await;
    }

    // 审核开启（默认）：创建为草稿状态，不执行库存变动
    let inbound_no = generate_inbound_no(db).await?;

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    // 1. 插入主表
    let inbound_id = insert(&txn, req, &inbound_no, created_by).await
        .map_err(|e| Error::from(e.to_string()))?;

    // 2. 插入明细
    for item in &req.items {
        let item_active = inbound_item::ActiveModel {
            inbound_id: Set(Some(inbound_id)),
            product_id: Set(Some(item.product_id)),
            product_sku: Set(item.product_sku.clone()),
            quantity: Set(Some(item.quantity)),
            unit_price: Set(item.unit_price),
            amount: Set(item.amount),
            batch_no: Set(item.batch_no.clone()),
            remark: Set(item.remark.clone()),
            deleted: Set(Some(0)),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        item_active.insert(&txn).await.map_err(|e| Error::from(e.to_string()))?;
    }

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(inbound_id)
}

/// 提交审核（草稿→待审核）
pub async fn submit_audit(
    db: &DatabaseConnection,
    id: i64,
) -> Result<i64> {
    let model = inbound::Entity::find_by_id(id)
        .filter(inbound::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("入库单不存在"))?;

    if model.status.unwrap_or(0) != 0 {
        return Err(Error::from("只有草稿状态才能提交审核"));
    }

    // 审核开关关闭：直接自动完成（先转待审核，再审核完成）
    if !is_audit_enabled().await {
        let operator = model.created_by.unwrap_or_default();
        update_status(db, id, 1, operator).await
            .map_err(|e| Error::from(e.to_string()))?;
        return audit(db, id, operator).await;
    }

    let mut active: inbound::ActiveModel = model.into();
    active.status = Set(Some(1)); // 草稿→待审核
    active.update(db).await.map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 审核入库单（核心：更新库存 + 写流水 + 更新状态，事务内完成）
pub async fn audit(
    db: &DatabaseConnection,
    id: i64,
    audit_by: i64,
) -> Result<i64> {
    // 1. 查询入库单
    let inbound_order = inbound::Entity::find_by_id(id)
        .filter(inbound::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("入库单不存在"))?;

    // 2. 状态机检查：仅待审核(1)可审核
    let status = inbound_order.status.unwrap_or(0);
    if status != 1 {
        return Err(Error::from(format!("入库单状态异常，当前状态：{}，仅待审核状态可审核", status)));
    }

    // 3. 查询入库明细
    let items = inbound_item::Entity::find()
        .filter(inbound_item::Column::InboundId.eq(id))
        .filter(inbound_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if items.is_empty() {
        return Err(Error::from("入库单明细为空，无法审核"));
    }

    // 4. 事务执行
    let warehouse_id = inbound_order.warehouse_id.unwrap_or_default();
    let inbound_no = inbound_order.inbound_no.clone().unwrap_or_default();
    let inbound_type = inbound_order.inbound_type.clone().unwrap_or_default();

    db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            for item in &items {
                let product_id = item.product_id.unwrap_or_default();
                let quantity = item.quantity.unwrap_or_default();
                let unit_price = item.unit_price;

                // 更新库存（使用 stock_engine）
                stock_engine::increase_stock(txn, product_id, warehouse_id, quantity, unit_price).await?;

                // 写入库存流水
                stock_engine::write_stock_log(
                    txn,
                    product_id,
                    warehouse_id,
                    None,
                    "inbound",
                    &format!("{}_in", inbound_type),
                    Some(id),
                    Some(&inbound_no),
                    quantity,
                    Some(audit_by),
                    None,
                ).await?;

                // 入库明细有 batch_no 时，自动创建批次记录
                if let Some(ref batch_no) = item.batch_no {
                    if !batch_no.is_empty() {
                        // 查询产品信息补充名称/SKU
                        let product_info = product_entity::Entity::find_by_id(product_id)
                            .one(txn)
                            .await
                            .ok()
                            .flatten();

                        let batch_req = BatchCreateRequest {
                            batch_no: batch_no.clone(),
                            product_id,
                            product_name: product_info.as_ref().and_then(|p| p.name.clone()),
                            product_sku: product_info.as_ref().and_then(|p| p.sku.clone()),
                            warehouse_id,
                            production_date: item.production_date,
                            expiry_date: item.expiry_date,
                            initial_quantity: quantity,
                            supplier_id: None,
                            inbound_id: Some(id),
                            remark: Some(format!("入库单 {} 自动创建", inbound_no)),
                        };
                        batch_service::create_batch_for_inbound(txn, &batch_req, audit_by)
                            .await
                            .map_err(|e| DbErr::Custom(e.to_string()))?;
                    }
                }
            }

            // 更新入库单状态为已完成(3)，使用 status=1 条件防止并发重复审核
            let now = chrono::Local::now().naive_local();
            let result = inbound::Entity::update_many()
                .col_expr(inbound::Column::Status, Expr::value(3))
                .col_expr(inbound::Column::AuditBy, Expr::value(audit_by))
                .col_expr(inbound::Column::AuditTime, Expr::value(now))
                .col_expr(inbound::Column::UpdateTime, Expr::value(now))
                .filter(inbound::Column::Id.eq(id))
                .filter(inbound::Column::Status.eq(1)) // 防止并发重复审核
                .filter(inbound::Column::Deleted.eq(0))
                .exec(txn)
                .await?;

            if result.rows_affected == 0 {
                return Err(DbErr::Custom("入库单已被其他用户审核，请刷新后重试".into()));
            }

            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(id)
}

/// 审核驳回
pub async fn reject(
    db: &DatabaseConnection,
    id: i64,
    audit_by: i64,
) -> Result<i64> {
    let inbound_order = inbound::Entity::find_by_id(id)
        .filter(inbound::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("入库单不存在"))?;

    let status = inbound_order.status.unwrap_or(0);
    if status != 1 {
        return Err(Error::from(format!("入库单状态异常，当前状态：{}，仅待审核状态可驳回", status)));
    }

    update_status(db, id, 0, audit_by).await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 入库单列表查询
pub async fn get_list(
    db: &DatabaseConnection,
    query: &InboundListQuery,
) -> Result<InboundListVO> {
    let (models, total) = select_page(db, query).await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut list: Vec<InboundListItem> = models.into_iter().map(|m| m.into()).collect();

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

    Ok(InboundListVO { list, total })
}

/// 创建并自动审核（库存初始化使用）
pub async fn create_and_auto_audit(
    db: &DatabaseConnection,
    req: &InboundSaveRequest,
    created_by: i64,
) -> Result<i64> {
    let inbound_no = generate_inbound_no(db).await?;

    let inbound_id: i64 = db.transaction::<_, _, DbErr>(|txn| {
        let inbound_no = inbound_no.clone();
        let req = req.clone();
        Box::pin(async move {
            // 1. 插入主表（状态直接设为已完成=3）
            let now = chrono::Local::now().naive_local();
            let active = inbound::ActiveModel {
                inbound_no: Set(Some(inbound_no.clone())),
                inbound_type: Set(Some(req.inbound_type.clone())),
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
            let inbound_id = result.id;

            // 2. 插入明细并更新库存
            for item in &req.items {
                let item_active = inbound_item::ActiveModel {
                    inbound_id: Set(Some(inbound_id)),
                    product_id: Set(Some(item.product_id)),
                    product_sku: Set(item.product_sku.clone()),
                    quantity: Set(Some(item.quantity)),
                    unit_price: Set(item.unit_price),
                    amount: Set(item.amount),
                    batch_no: Set(item.batch_no.clone()),
                    remark: Set(item.remark.clone()),
                    deleted: Set(Some(0)),
                    create_time: Set(Some(now)),
                    ..Default::default()
                };
                item_active.insert(txn).await?;

                // 更新库存
                stock_engine::increase_stock(txn, item.product_id, req.warehouse_id, item.quantity, item.unit_price).await?;

                // 写入流水
                stock_engine::write_stock_log(
                    txn,
                    item.product_id,
                    req.warehouse_id,
                    None,
                    "inbound",
                    "initial",
                    Some(inbound_id),
                    Some(&inbound_no),
                    item.quantity,
                    Some(created_by),
                    None,
                ).await?;
            }

            Ok(inbound_id)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(inbound_id)
}

/// 获取入库单详情（扁平 camelCase 结构，供前端直接使用）
pub async fn get_detail(
    db: &DatabaseConnection,
    id: i64,
) -> Result<serde_json::Value> {
    let main = find_by_id(db, id).await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("入库单不存在"))?;

    let items = inbound_item::Entity::find()
        .filter(inbound_item::Column::InboundId.eq(id))
        .filter(inbound_item::Column::Deleted.eq(0))
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

    // 查询审核人姓名
    let audit_by_name = if let Some(uid) = main.audit_by {
        admin::Entity::find_by_id(uid).one(db).await.ok().flatten()
            .and_then(|a| a.nick_name.or(a.user_name))
    } else {
        None
    };

    // 构建扁平 camelCase 主表数据
    let detail = serde_json::json!({
        "id": main.id,
        "inboundNo": main.inbound_no,
        "inboundType": main.inbound_type,
        "sourceOrderId": main.source_order_id,
        "sourceOrderNo": main.source_order_no,
        "warehouseId": main.warehouse_id,
        "warehouseName": warehouse_name,
        "status": main.status,
        "totalQuantity": main.total_quantity,
        "totalAmount": main.total_amount,
        "remark": main.remark,
        "auditBy": main.audit_by,
        "auditByName": audit_by_name,
        "auditTime": main.audit_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        "createdBy": main.created_by,
        "createdByName": created_by_name,
        "createTime": main.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        "updateTime": main.update_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
    });

    // 构建明细数组（camelCase + 补充产品信息）
    let mut items_json: Vec<serde_json::Value> = Vec::new();
    for item in &items {
        // 查产品信息
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
            "inboundId": item.inbound_id,
            "productId": item.product_id,
            "productSku": item.product_sku,
            "quantity": item.quantity,
            "unitPrice": item.unit_price,
            "totalPrice": item.amount,
            "batchNo": item.batch_no,
            "productionDate": item.production_date.map(|t| t.format("%Y-%m-%d").to_string()),
            "expiryDate": item.expiry_date.map(|t| t.format("%Y-%m-%d").to_string()),
            "remark": item.remark,
            "productName": product_name,
            "productCode": product_code,
            "unit": unit,
            "spec": spec,
        }));
    }

    Ok(serde_json::json!({
        "detail": detail,
        "items": items_json,
    }))
}

/// 编辑入库单
/// - status=0（草稿）：直接编辑，无需原因
/// - status=3（已完成）：审核关闭时必须填写修改原因，事务内回滚旧库存 → 软删除旧明细 → 插入新明细 → 重新入库 → 写调整流水 → 写修改日志
pub async fn update(
    db: &DatabaseConnection,
    id: i64,
    req: &InboundSaveRequest,
    updated_by: i64,
    change_reason: Option<&str>,
) -> Result<i64> {
    // 查询入库单
    let inbound_order = inbound::Entity::find_by_id(id)
        .filter(inbound::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("入库单不存在"))?;

    let status = inbound_order.status.unwrap_or(0);

    // 草稿状态：直接编辑（现有逻辑）
    if status == 0 {
        db.transaction::<_, _, DbErr>(|txn| {
            let req = req.clone();
            Box::pin(async move {
                // 更新主表
                update_by_id(txn, id, &req, updated_by).await?;

                // 删除原明细（软删除）
                inbound_item::Entity::update_many()
                    .col_expr(inbound_item::Column::Deleted, Expr::value(1))
                    .filter(inbound_item::Column::InboundId.eq(id))
                    .exec(txn)
                    .await?;

                // 插入新明细
                let now = chrono::Local::now().naive_local();
                for item in &req.items {
                    let item_active = inbound_item::ActiveModel {
                        inbound_id: Set(Some(id)),
                        product_id: Set(Some(item.product_id)),
                        product_sku: Set(item.product_sku.clone()),
                        quantity: Set(Some(item.quantity)),
                        unit_price: Set(item.unit_price),
                        amount: Set(item.amount),
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
            .ok_or_else(|| Error::from("修改已完成的入库单必须填写修改原因"))?
            .to_string();

        let warehouse_id = inbound_order.warehouse_id.unwrap_or_default();
        let inbound_no = inbound_order.inbound_no.clone().unwrap_or_default();
        let inbound_type = inbound_order.inbound_type.clone().unwrap_or_default();
        let before_main = serde_json::to_value(&inbound_order).unwrap_or_default();
        let req_clone = req.clone();

        db.transaction::<_, _, DbErr>(|txn| {
            let reason = reason.clone();
            let inbound_no = inbound_no.clone();
            let inbound_type = inbound_type.clone();
            let req_clone = req_clone.clone();
            let before_main = before_main.clone();
            Box::pin(async move {
                // 1. 查询旧明细 + 保存修改前快照
                let old_items = inbound_item::Entity::find()
                    .filter(inbound_item::Column::InboundId.eq(id))
                    .filter(inbound_item::Column::Deleted.eq(0))
                    .all(txn)
                    .await?;
                let before_snapshot = serde_json::json!({
                    "main": before_main,
                    "items": serde_json::to_value(&old_items).unwrap_or_default(),
                });

                // 2. 回滚旧库存（入库的反操作：减少库存）
                for old_item in &old_items {
                    let product_id = old_item.product_id.unwrap_or_default();
                    let quantity = old_item.quantity.unwrap_or_default();
                    if quantity != Decimal::ZERO {
                        stock_engine::decrease_stock(txn, product_id, warehouse_id, quantity).await?;
                        stock_engine::write_stock_log(
                            txn,
                            product_id,
                            warehouse_id,
                            None,
                            "inbound",
                            "adjust_rollback",
                            Some(id),
                            Some(&inbound_no),
                            -quantity,
                            Some(updated_by),
                            Some("修改已完成入库单-回滚旧库存"),
                        ).await?;
                    }
                }

                // 3. 软删除旧明细
                inbound_item::Entity::update_many()
                    .col_expr(inbound_item::Column::Deleted, Expr::value(1))
                    .filter(inbound_item::Column::InboundId.eq(id))
                    .exec(txn)
                    .await?;

                // 4. 更新主表（已完成状态，带 status=3 过滤）
                let now = chrono::Local::now().naive_local();
                inbound::Entity::update_many()
                    .col_expr(inbound::Column::InboundType, Expr::value(req_clone.inbound_type.clone()))
                    .col_expr(inbound::Column::WarehouseId, Expr::value(req_clone.warehouse_id))
                    .col_expr(inbound::Column::SourceOrderId, Expr::value(req_clone.source_order_id))
                    .col_expr(inbound::Column::SourceOrderNo, Expr::value(req_clone.source_order_no.clone()))
                    .col_expr(inbound::Column::TotalQuantity, Expr::value(req_clone.total_quantity))
                    .col_expr(inbound::Column::TotalAmount, Expr::value(req_clone.total_amount))
                    .col_expr(inbound::Column::Remark, Expr::value(req_clone.remark.clone()))
                    .col_expr(inbound::Column::UpdatedBy, Expr::value(updated_by))
                    .col_expr(inbound::Column::UpdateTime, Expr::value(now))
                    .col_expr(inbound::Column::LastChangeReason, Expr::value(reason.clone()))
                    .col_expr(inbound::Column::LastChangeBy, Expr::value(updated_by))
                    .col_expr(inbound::Column::LastChangeTime, Expr::value(now))
                    .filter(inbound::Column::Id.eq(id))
                    .filter(inbound::Column::Status.eq(3))
                    .filter(inbound::Column::Deleted.eq(0))
                    .exec(txn)
                    .await?;

                // 5. 插入新明细
                for item in &req_clone.items {
                    let item_active = inbound_item::ActiveModel {
                        inbound_id: Set(Some(id)),
                        product_id: Set(Some(item.product_id)),
                        product_sku: Set(item.product_sku.clone()),
                        quantity: Set(Some(item.quantity)),
                        unit_price: Set(item.unit_price),
                        amount: Set(item.amount),
                        batch_no: Set(item.batch_no.clone()),
                        remark: Set(item.remark.clone()),
                        deleted: Set(Some(0)),
                        create_time: Set(Some(now)),
                        ..Default::default()
                    };
                    item_active.insert(txn).await?;
                }

                // 6. 重新执行库存变动（增加新库存）+ 写调整流水
                for item in &req_clone.items {
                    let product_id = item.product_id;
                    let quantity = item.quantity;
                    if quantity != Decimal::ZERO {
                        stock_engine::increase_stock(txn, product_id, req_clone.warehouse_id, quantity, item.unit_price).await?;
                        stock_engine::write_stock_log(
                            txn,
                            product_id,
                            req_clone.warehouse_id,
                            None,
                            "inbound",
                            &format!("{}_adjust", inbound_type),
                            Some(id),
                            Some(&inbound_no),
                            quantity,
                            Some(updated_by),
                            Some("修改已完成入库单-重新入库"),
                        ).await?;
                    }
                }

                // 7. 写修改日志（doc_change_log）
                let after_snapshot = serde_json::json!({
                    "main": {
                        "inboundType": req_clone.inbound_type,
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
                    doc_type: Set(Some("inbound".to_string())),
                    doc_id: Set(Some(id)),
                    doc_no: Set(Some(inbound_no.clone())),
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

    Err(Error::from("当前状态的入库单不可编辑"))
}

/// 批量删除入库单（仅草稿状态可删除）
pub async fn batch_delete(
    db: &DatabaseConnection,
    ids: &[i64],
) -> Result<i64> {
    crate::modules::inventory::model::inbound::batch_delete(db, ids)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 获取入库单打印数据（复用 get_detail 扁平 camelCase 结构 + 仓库/操作人信息）
pub async fn get_print_data(
    db: &DatabaseConnection,
    id: i64,
) -> Result<serde_json::Value> {
    let main = find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("入库单不存在".to_string()))?;

    let items = inbound_item::Entity::find()
        .filter(inbound_item::Column::InboundId.eq(id))
        .filter(inbound_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 仓库名称
    let warehouse_name = if let Some(wid) = main.warehouse_id {
        warehouse::Entity::find_by_id(wid)
            .filter(warehouse::Column::Deleted.eq(0))
            .one(db)
            .await
            .ok()
            .flatten()
            .and_then(|w| w.name)
    } else {
        None
    };

    // 创建人姓名
    let creator_name = if let Some(cb) = main.created_by {
        admin::Entity::find_by_id(cb).one(db).await.ok().flatten()
            .and_then(|a| a.nick_name.or(a.user_name))
    } else {
        None
    };

    // 审核人姓名
    let auditor_name = if let Some(ab) = main.audit_by {
        admin::Entity::find_by_id(ab).one(db).await.ok().flatten()
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
            "unitPrice": item.unit_price,
            "totalPrice": item.amount,
            "remark": item.remark,
        }));
    }

    Ok(serde_json::json!({
        "main": {
            "inboundNo": main.inbound_no,
            "inboundType": main.inbound_type,
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
        "auditor": { "nick_name": auditor_name, "user_name": auditor_name },
    }))
}

/// 导出入库单列表（CSV 字符串）
pub async fn export_list(db: &DatabaseConnection, query: &InboundListQuery) -> Result<String> {
    let (models, _total) = select_page(db, query)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut csv = String::from("ID,入库单号,入库类型,源单号,仓库ID,状态,总数量,总金额,备注,创建时间\n");
    for m in models {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            m.id,
            m.inbound_no.unwrap_or_default(),
            m.inbound_type.unwrap_or_default(),
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

/// 导入入库单（解析 JSON 数组，逐条创建）
pub async fn import_list(
    db: &DatabaseConnection,
    items: Vec<InboundSaveRequest>,
    created_by: i64,
) -> Result<i64> {
    let mut count = 0i64;
    for req in items {
        create(db, &req, created_by).await?;
        count += 1;
    }
    Ok(count)
}