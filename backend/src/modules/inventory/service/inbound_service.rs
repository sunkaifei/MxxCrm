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
use crate::modules::inventory::entity::{inbound, inbound_item};
use crate::modules::inventory::service::stock_engine;
use crate::modules::inventory::service::batch_service;
use crate::modules::inventory::entity::warehouse;
use crate::modules::product::entity::product as product_entity;
use crate::modules::system::entity::admin;

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

/// 获取入库单详情
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

    Ok(serde_json::json!({
        "main": main,
        "items": items,
    }))
}

/// 编辑入库单（仅草稿状态可编辑）
pub async fn update(
    db: &DatabaseConnection,
    id: i64,
    req: &InboundSaveRequest,
    updated_by: i64,
) -> Result<i64> {
    // 检查状态：仅草稿(0)可编辑
    let inbound_order = inbound::Entity::find_by_id(id)
        .filter(inbound::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("入库单不存在"))?;

    if inbound_order.status.unwrap_or(0) != 0 {
        return Err(Error::from("仅草稿状态的入库单可编辑"));
    }

    // 事务更新主表 + 删除原明细 + 插入新明细
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

    Ok(id)
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

/// 获取入库单打印数据（主表 + 明细 + 仓库 + 操作人）
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

    // 仓库信息
    let warehouse_info = if let Some(wid) = main.warehouse_id {
        warehouse::Entity::find_by_id(wid)
            .filter(warehouse::Column::Deleted.eq(0))
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
    } else {
        None
    };

    // 创建人信息
    let creator_info = if let Some(cb) = main.created_by {
        admin::Entity::find_by_id(cb).one(db).await.ok().flatten()
    } else {
        None
    };

    // 审核人信息
    let auditor_info = if let Some(ab) = main.audit_by {
        admin::Entity::find_by_id(ab).one(db).await.ok().flatten()
    } else {
        None
    };

    Ok(serde_json::json!({
        "main": main,
        "items": items,
        "warehouse": warehouse_info,
        "creator": creator_info.as_ref().map(|a| serde_json::json!({
            "id": a.id,
            "nick_name": a.nick_name,
            "user_name": a.user_name,
        })),
        "auditor": auditor_info.as_ref().map(|a| serde_json::json!({
            "id": a.id,
            "nick_name": a.nick_name,
            "user_name": a.user_name,
        })),
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