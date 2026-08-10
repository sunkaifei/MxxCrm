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
use crate::modules::inventory::entity::{transfer, transfer_item, stock, stock_log, warehouse};
use crate::modules::inventory::model::transfer::*;
use crate::modules::inventory::service::stock_engine;
use crate::modules::system::entity::admin;

/// 生成调拨单号：DB + yyyyMMdd + 4位流水号
pub async fn generate_transfer_no(db: &DatabaseConnection) -> Result<String> {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let prefix = format!("DB{}", today);

    let max_no = transfer::Entity::find()
        .filter(transfer::Column::TransferNo.starts_with(&prefix))
        .order_by_desc(transfer::Column::TransferNo)
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let seq = match max_no {
        Some(m) => {
            let no = m.transfer_no.unwrap_or_default();
            let seq_str = no.trim_start_matches(&prefix);
            seq_str.parse::<i32>().unwrap_or(0) + 1
        }
        None => 1,
    };

    Ok(format!("{}{:04}", prefix, seq))
}

/// 创建调拨单
pub async fn create(
    db: &DatabaseConnection,
    req: &TransferSaveRequest,
    created_by: i64,
) -> Result<i64> {
    if req.from_warehouse_id == req.to_warehouse_id {
        return Err(Error::from("源仓库和目标仓库不能相同".to_string()));
    }

    // 库存校验：检查源仓库每个产品的可用库存是否充足
    for item in &req.items {
        let stock_record = stock::Entity::find()
            .filter(stock::Column::ProductId.eq(item.product_id))
            .filter(stock::Column::WarehouseId.eq(req.from_warehouse_id))
            .filter(stock::Column::Deleted.eq(0))
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        match stock_record {
            Some(s) => {
                let available = s.available_quantity.unwrap_or_default();
                if item.quantity > available {
                    return Err(Error::from(format!(
                        "产品[{}]库存不足，当前可用: {}，需调拨: {}",
                        item.product_id, available, item.quantity
                    )));
                }
            }
            None => {
                return Err(Error::from(format!(
                    "产品[{}]在源仓库无库存记录",
                    item.product_id
                )));
            }
        }
    }

    let transfer_no = generate_transfer_no(db).await?;
    let total_quantity: Decimal = req.items.iter()
        .map(|i| i.quantity)
        .sum();

    let transfer_id = db.transaction::<_, _, DbErr>(|txn| {
        let transfer_no = transfer_no.clone();
        let req = req.clone();
        let total_quantity = total_quantity;
        Box::pin(async move {
            // 1. 插入主表
            let id = insert_main(txn, &transfer_no, &req, total_quantity, created_by).await?;

            // 2. 插入明细
            let now = chrono::Local::now().naive_local();
            for item in &req.items {
                let item_active = transfer_item::ActiveModel {
                    transfer_id: Set(Some(id)),
                    product_id: Set(Some(item.product_id)),
                    product_name: Set(item.product_name.clone()),
                    product_sku: Set(item.product_sku.clone()),
                    quantity: Set(Some(item.quantity)),
                    remark: Set(item.remark.clone()),
                    deleted: Set(Some(0)),
                    create_time: Set(Some(now)),
                    update_time: Set(Some(now)),
                    ..Default::default()
                };
                item_active.insert(txn).await?;
            }

            Ok(id)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(transfer_id)
}

/// 编辑调拨单（仅草稿状态可编辑）
pub async fn update(
    db: &DatabaseConnection,
    id: i64,
    req: &TransferSaveRequest,
    updated_by: i64,
) -> Result<i64> {
    if req.from_warehouse_id == req.to_warehouse_id {
        return Err(Error::from("源仓库和目标仓库不能相同".to_string()));
    }

    let order = transfer::Entity::find_by_id(id)
        .filter(transfer::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("调拨单不存在".to_string()))?;

    if order.status.unwrap_or(0) != 0 {
        return Err(Error::from("仅草稿状态的调拨单可编辑".to_string()));
    }

    let total_quantity: Decimal = req.items.iter()
        .map(|i| i.quantity)
        .sum();

    db.transaction::<_, _, DbErr>(|txn| {
        let req = req.clone();
        let total_quantity = total_quantity;
        Box::pin(async move {
            // 1. 更新主表
            update_main(txn, id, &req, total_quantity, updated_by).await?;

            // 2. 软删除原明细
            transfer_item::Entity::update_many()
                .col_expr(transfer_item::Column::Deleted, Expr::value(1))
                .filter(transfer_item::Column::TransferId.eq(id))
                .exec(txn)
                .await?;

            // 3. 插入新明细
            let now = chrono::Local::now().naive_local();
            for item in &req.items {
                let item_active = transfer_item::ActiveModel {
                    transfer_id: Set(Some(id)),
                    product_id: Set(Some(item.product_id)),
                    product_name: Set(item.product_name.clone()),
                    product_sku: Set(item.product_sku.clone()),
                    quantity: Set(Some(item.quantity)),
                    remark: Set(item.remark.clone()),
                    deleted: Set(Some(0)),
                    create_time: Set(Some(now)),
                    update_time: Set(Some(now)),
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

/// 调拨出库（源仓库出库，在途库存增加）
pub async fn outbound(
    db: &DatabaseConnection,
    id: i64,
    updated_by: i64,
) -> Result<i64> {
    // 1. 查询调拨单
    let order = transfer::Entity::find_by_id(id)
        .filter(transfer::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("调拨单不存在".to_string()))?;

    if order.status.unwrap_or(0) != 0 {
        return Err(Error::from("仅草稿状态的调拨单可出库".to_string()));
    }

    // 2. 查询明细
    let items = transfer_item::Entity::find()
        .filter(transfer_item::Column::TransferId.eq(id))
        .filter(transfer_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if items.is_empty() {
        return Err(Error::from("调拨明细为空，无法出库".to_string()));
    }

    let from_warehouse_id = order.from_warehouse_id.unwrap_or_default();
    let to_warehouse_id = order.to_warehouse_id.unwrap_or_default();
    let transfer_no = order.transfer_no.clone().unwrap_or_default();

    // 3. 事务执行：扣减源仓库库存 + 增加目标仓库在途 + 写流水 + 更新状态
    db.transaction::<_, _, DbErr>(|txn| {
        let transfer_no = transfer_no.clone();
        let items = items.clone();
        Box::pin(async move {
            for item in &items {
                let product_id = item.product_id.unwrap_or_default();
                let quantity = item.quantity.unwrap_or_default();

                // 3.1 从源仓库扣减库存
                stock_engine::decrease_stock(txn, product_id, from_warehouse_id, quantity).await?;

                // 3.2 增加目标仓库的 in_transit_quantity
                increase_in_transit(txn, product_id, to_warehouse_id, quantity).await?;

                // 3.3 写入库存流水
                stock_engine::write_stock_log(
                    txn,
                    product_id,
                    from_warehouse_id,
                    None,
                    "transfer_out",
                    "transfer_out",
                    Some(id),
                    Some(&transfer_no),
                    -quantity,
                    Some(updated_by),
                    None,
                ).await?;
            }

            // 4. 更新调拨单状态为已出库(1)
            let now = chrono::Local::now().naive_local();
            transfer::Entity::update_many()
                .col_expr(transfer::Column::Status, Expr::value(1))
                .col_expr(transfer::Column::UpdatedBy, Expr::value(updated_by))
                .col_expr(transfer::Column::UpdateTime, Expr::value(now))
                .filter(transfer::Column::Id.eq(id))
                .filter(transfer::Column::Deleted.eq(0))
                .exec(txn)
                .await?;

            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(id)
}

/// 调拨入库（目标仓库入库，在途库存减少）
pub async fn inbound(
    db: &DatabaseConnection,
    id: i64,
    updated_by: i64,
) -> Result<i64> {
    // 1. 查询调拨单
    let order = transfer::Entity::find_by_id(id)
        .filter(transfer::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("调拨单不存在".to_string()))?;

    let status = order.status.unwrap_or(0);
    if status != 1 {
        return Err(Error::from("仅已出库状态的调拨单可入库".to_string()));
    }

    // 2. 查询明细
    let items = transfer_item::Entity::find()
        .filter(transfer_item::Column::TransferId.eq(id))
        .filter(transfer_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if items.is_empty() {
        return Err(Error::from("调拨明细为空，无法入库".to_string()));
    }

    let to_warehouse_id = order.to_warehouse_id.unwrap_or_default();
    let transfer_no = order.transfer_no.clone().unwrap_or_default();

    // 3. 事务执行：减少目标仓库在途 + 增加目标仓库库存 + 写流水 + 更新状态
    db.transaction::<_, _, DbErr>(|txn| {
        let transfer_no = transfer_no.clone();
        let items = items.clone();
        Box::pin(async move {
            for item in &items {
                let product_id = item.product_id.unwrap_or_default();
                let quantity = item.quantity.unwrap_or_default();

                // 3.1 减少目标仓库的 in_transit_quantity
                decrease_in_transit(txn, product_id, to_warehouse_id, quantity).await?;

                // 3.2 向目标仓库增加库存
                stock_engine::increase_stock(txn, product_id, to_warehouse_id, quantity, None).await?;

                // 3.3 写入库存流水
                stock_engine::write_stock_log(
                    txn,
                    product_id,
                    to_warehouse_id,
                    None,
                    "transfer_in",
                    "transfer_in",
                    Some(id),
                    Some(&transfer_no),
                    quantity,
                    Some(updated_by),
                    None,
                ).await?;
            }

            // 4. 更新调拨单状态为已完成(3)
            let now = chrono::Local::now().naive_local();
            transfer::Entity::update_many()
                .col_expr(transfer::Column::Status, Expr::value(3))
                .col_expr(transfer::Column::UpdatedBy, Expr::value(updated_by))
                .col_expr(transfer::Column::UpdateTime, Expr::value(now))
                .filter(transfer::Column::Id.eq(id))
                .filter(transfer::Column::Deleted.eq(0))
                .exec(txn)
                .await?;

            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(id)
}

/// 取消调拨
pub async fn cancel(
    db: &DatabaseConnection,
    id: i64,
    updated_by: i64,
) -> Result<i64> {
    let order = transfer::Entity::find_by_id(id)
        .filter(transfer::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("调拨单不存在".to_string()))?;

    let status = order.status.unwrap_or(0);
    if status == 3 || status == 4 {
        return Err(Error::from("已完成或已取消的调拨单不可取消".to_string()));
    }

    let now = chrono::Local::now().naive_local();
    let result = transfer::Entity::update_many()
        .col_expr(transfer::Column::Status, Expr::value(4))
        .col_expr(transfer::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(transfer::Column::UpdateTime, Expr::value(now))
        .filter(transfer::Column::Id.eq(id))
        .filter(transfer::Column::Deleted.eq(0))
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result.rows_affected as i64)
}

/// 批量删除（仅草稿状态可删除）
pub async fn batch_delete(
    db: &DatabaseConnection,
    ids: &[i64],
) -> Result<i64> {
    let ids_vec = ids.to_vec();
    db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            crate::modules::inventory::model::transfer::batch_delete(txn, &ids_vec).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 获取调拨单详情
pub async fn get_detail(
    db: &DatabaseConnection,
    id: i64,
) -> Result<serde_json::Value> {
    let main = transfer::Entity::find_by_id(id)
        .filter(transfer::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("调拨单不存在".to_string()))?;

    let items = transfer_item::Entity::find()
        .filter(transfer_item::Column::TransferId.eq(id))
        .filter(transfer_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(serde_json::json!({
        "main": main,
        "items": items,
    }))
}

/// 调拨单列表查询
pub async fn get_list(
    db: &DatabaseConnection,
    query: &TransferListQuery,
) -> Result<TransferListVO> {
    let (models, total) = select_page(db, query)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut items: Vec<TransferListItem> = models.into_iter().map(|m| m.into()).collect();

    // 补充仓库名称和创建人名称
    for item in &mut items {
        if let Some(wid) = item.from_warehouse_id {
            if let Ok(Some(wh)) = warehouse::Entity::find_by_id(wid)
                .filter(warehouse::Column::Deleted.eq(0))
                .one(db).await
            {
                item.from_warehouse_name = wh.name;
            }
        }
        if let Some(wid) = item.to_warehouse_id {
            if let Ok(Some(wh)) = warehouse::Entity::find_by_id(wid)
                .filter(warehouse::Column::Deleted.eq(0))
                .one(db).await
            {
                item.to_warehouse_name = wh.name;
            }
        }
        if let Some(cb) = item.created_by {
            if let Ok(Some(admin)) = admin::Entity::find_by_id(cb).one(db).await {
                item.created_by_name = admin.nick_name.or(admin.user_name);
            }
        }
    }

    Ok(TransferListVO { total: total as i64, items })
}

// ========== 内部辅助函数 ==========

async fn insert_main<C: ConnectionTrait>(
    db: &C,
    transfer_no: &str,
    req: &TransferSaveRequest,
    total_quantity: Decimal,
    created_by: i64,
) -> std::result::Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let active = transfer::ActiveModel {
        transfer_no: Set(Some(transfer_no.to_string())),
        from_warehouse_id: Set(Some(req.from_warehouse_id)),
        to_warehouse_id: Set(Some(req.to_warehouse_id)),
        status: Set(Some(0)),
        total_quantity: Set(Some(total_quantity)),
        remark: Set(req.remark.clone()),
        deleted: Set(Some(0)),
        created_by: Set(Some(created_by)),
        updated_by: Set(Some(created_by)),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let result = active.insert(db).await?;
    Ok(result.id)
}

async fn update_main<C: ConnectionTrait>(
    db: &C,
    id: i64,
    req: &TransferSaveRequest,
    total_quantity: Decimal,
    updated_by: i64,
) -> std::result::Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = transfer::Entity::update_many()
        .col_expr(transfer::Column::FromWarehouseId, Expr::value(req.from_warehouse_id))
        .col_expr(transfer::Column::ToWarehouseId, Expr::value(req.to_warehouse_id))
        .col_expr(transfer::Column::TotalQuantity, Expr::value(total_quantity))
        .col_expr(transfer::Column::Remark, Expr::value(req.remark.clone()))
        .col_expr(transfer::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(transfer::Column::UpdateTime, Expr::value(now))
        .filter(transfer::Column::Id.eq(id))
        .filter(transfer::Column::Deleted.eq(0))
        .filter(transfer::Column::Status.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

/// 增加目标仓库的在途库存
async fn increase_in_transit<C: ConnectionTrait>(
    db: &C,
    product_id: i64,
    warehouse_id: i64,
    quantity: Decimal,
) -> std::result::Result<(), DbErr> {
    let now = chrono::Local::now().naive_local();
    let existing = stock::Entity::find()
        .filter(stock::Column::ProductId.eq(product_id))
        .filter(stock::Column::WarehouseId.eq(warehouse_id))
        .filter(stock::Column::Deleted.eq(0))
        .lock_exclusive()
        .one(db)
        .await?;

    match existing {
        Some(s) => {
            let current = s.in_transit_quantity.unwrap_or_default();
            let mut active: stock::ActiveModel = s.into();
            active.in_transit_quantity = Set(Some(current + quantity));
            active.update_time = Set(Some(now));
            active.update(db).await?;
        }
        None => {
            // 目标仓库无库存记录，创建一条（仅含在途数量）
            let active = stock::ActiveModel {
                product_id: Set(Some(product_id)),
                warehouse_id: Set(Some(warehouse_id)),
                quantity: Set(Some(Decimal::ZERO)),
                reserved_quantity: Set(Some(Decimal::ZERO)),
                available_quantity: Set(Some(Decimal::ZERO)),
                in_transit_quantity: Set(Some(quantity)),
                frozen_quantity: Set(Some(Decimal::ZERO)),
                avg_cost: Set(Some(Decimal::ZERO)),
                last_in_cost: Set(None),
                total_cost: Set(Some(Decimal::ZERO)),
                deleted: Set(Some(0)),
                create_time: Set(Some(now)),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            active.insert(db).await?;
        }
    }
    Ok(())
}

/// 减少目标仓库的在途库存
async fn decrease_in_transit<C: ConnectionTrait>(
    db: &C,
    product_id: i64,
    warehouse_id: i64,
    quantity: Decimal,
) -> std::result::Result<(), DbErr> {
    let now = chrono::Local::now().naive_local();
    let existing = stock::Entity::find()
        .filter(stock::Column::ProductId.eq(product_id))
        .filter(stock::Column::WarehouseId.eq(warehouse_id))
        .filter(stock::Column::Deleted.eq(0))
        .lock_exclusive()
        .one(db)
        .await?;

    match existing {
        Some(s) => {
            let current = s.in_transit_quantity.unwrap_or_default();
            if current < quantity {
                return Err(DbErr::Custom(format!(
                    "在途库存不足：当前 {}，需要 {}", current, quantity
                )));
            }
            let mut active: stock::ActiveModel = s.into();
            active.in_transit_quantity = Set(Some(current - quantity));
            active.update_time = Set(Some(now));
            active.update(db).await?;
        }
        None => {
            return Err(DbErr::Custom("目标仓库库存记录不存在，无法减少在途库存".into()));
        }
    }
    Ok(())
}
