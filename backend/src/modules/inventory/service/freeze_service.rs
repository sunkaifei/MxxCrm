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
use crate::modules::inventory::entity::{stock, stock_freeze, stock_log};

/// 冻结库存
pub async fn freeze_stock(
    db: &DatabaseConnection,
    product_id: i64,
    warehouse_id: i64,
    quantity: Decimal,
    reason: Option<String>,
    freeze_by: i64,
) -> Result<()> {
    db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            let now = chrono::Local::now().naive_local();

            // 锁定库存行
            let existing = stock::Entity::find()
                .filter(stock::Column::ProductId.eq(product_id))
                .filter(stock::Column::WarehouseId.eq(warehouse_id))
                .filter(stock::Column::Deleted.eq(0))
                .lock_exclusive()
                .one(txn)
                .await?
                .ok_or_else(|| DbErr::Custom("库存记录不存在".into()))?;

            let available = existing.available_quantity.unwrap_or_default();
            if available < quantity {
                return Err(DbErr::Custom(format!("可用库存不足：可用 {}，需要冻结 {}", available, quantity)));
            }

            let frozen = existing.frozen_quantity.unwrap_or_default();

            let mut active: stock::ActiveModel = existing.into();
            active.available_quantity = Set(Some(available - quantity));
            active.frozen_quantity = Set(Some(frozen + quantity));
            active.update_time = Set(Some(now));
            active.update(txn).await?;

            // 写入冻结记录
            let freeze = stock_freeze::ActiveModel {
                product_id: Set(Some(product_id)),
                warehouse_id: Set(Some(warehouse_id)),
                freeze_quantity: Set(Some(quantity)),
                reason: Set(reason),
                status: Set(Some(0)), // 冻结中
                freeze_by: Set(Some(freeze_by)),
                freeze_time: Set(Some(now)),
                deleted: Set(Some(0)),
                create_time: Set(Some(now)),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            freeze.insert(txn).await?;

            // 写入库存流水（记录可用库存的变动）
            let available_before = available;
            let available_after = available - quantity;
            let log = stock_log::ActiveModel {
                product_id: Set(Some(product_id)),
                warehouse_id: Set(Some(warehouse_id)),
                change_type: Set(Some("freeze".to_string())),
                biz_type: Set(Some("freeze".to_string())),
                quantity_before: Set(Some(available_before)),
                change_quantity: Set(Some(-quantity)),
                quantity_after: Set(Some(available_after)),
                operator_id: Set(Some(freeze_by)),
                remark: Set(Some(format!("冻结库存 {}，冻结后可用库存 {}", quantity, available_after))),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            log.insert(txn).await?;

            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(())
}

/// 解冻库存
pub async fn unfreeze_stock(
    db: &DatabaseConnection,
    product_id: i64,
    warehouse_id: i64,
    quantity: Decimal,
    unfreeze_by: i64,
) -> Result<()> {
    db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            let now = chrono::Local::now().naive_local();

            // 锁定库存行
            let existing = stock::Entity::find()
                .filter(stock::Column::ProductId.eq(product_id))
                .filter(stock::Column::WarehouseId.eq(warehouse_id))
                .filter(stock::Column::Deleted.eq(0))
                .lock_exclusive()
                .one(txn)
                .await?
                .ok_or_else(|| DbErr::Custom("库存记录不存在".into()))?;

            let frozen = existing.frozen_quantity.unwrap_or_default();
            if frozen < quantity {
                return Err(DbErr::Custom(format!("冻结数量不足：已冻结 {}，需要解冻 {}", frozen, quantity)));
            }

            let available = existing.available_quantity.unwrap_or_default();

            let mut active: stock::ActiveModel = existing.into();
            active.available_quantity = Set(Some(available + quantity));
            active.frozen_quantity = Set(Some(frozen - quantity));
            active.update_time = Set(Some(now));
            active.update(txn).await?;

            // 逐条查找并更新冻结记录（按 freeze_time 排序，先进先解）
            let freeze_records = stock_freeze::Entity::find()
                .filter(stock_freeze::Column::ProductId.eq(product_id))
                .filter(stock_freeze::Column::WarehouseId.eq(warehouse_id))
                .filter(stock_freeze::Column::Status.eq(0))
                .filter(stock_freeze::Column::Deleted.eq(0))
                .order_by_asc(stock_freeze::Column::FreezeTime)
                .lock_exclusive()
                .all(txn)
                .await?;

            let mut remaining = quantity;
            for record in &freeze_records {
                if remaining <= Decimal::ZERO {
                    break;
                }
                let fq = record.freeze_quantity.unwrap_or_default();
                if fq <= remaining {
                    // 整条记录全部解冻
                    let mut active: stock_freeze::ActiveModel = record.clone().into();
                    active.status = Set(Some(1));
                    active.unfreeze_by = Set(Some(unfreeze_by));
                    active.unfreeze_time = Set(Some(now));
                    active.update_time = Set(Some(now));
                    active.update(txn).await?;
                    remaining -= fq;
                } else {
                    // 部分解冻：用 update_many 配合条件更新
                    stock_freeze::Entity::update_many()
                        .col_expr(stock_freeze::Column::FreezeQuantity, Expr::value(fq - remaining))
                        .col_expr(stock_freeze::Column::UpdateTime, Expr::value(now))
                        .filter(stock_freeze::Column::Id.eq(record.id))
                        .filter(stock_freeze::Column::Deleted.eq(0))
                        .exec(txn)
                        .await?;
                    // 插入一条解冻记录（剩余部分）
                    let unfreeze_rec = stock_freeze::ActiveModel {
                        product_id: Set(Some(product_id)),
                        warehouse_id: Set(Some(warehouse_id)),
                        freeze_quantity: Set(Some(remaining)),
                        reason: Set(Some("部分解冻".to_string())),
                        status: Set(Some(1)),
                        freeze_by: Set(record.freeze_by),
                        freeze_time: Set(record.freeze_time),
                        unfreeze_by: Set(Some(unfreeze_by)),
                        unfreeze_time: Set(Some(now)),
                        remark: Set(Some(format!("从冻结记录 {} 中部分解冻 {}", record.id, remaining))),
                        deleted: Set(Some(0)),
                        create_time: Set(Some(now)),
                        update_time: Set(Some(now)),
                        ..Default::default()
                    };
                    unfreeze_rec.insert(txn).await?;
                    remaining = Decimal::ZERO;
                }
            }

            if remaining > Decimal::ZERO {
                return Err(DbErr::Custom(format!("未找到足够的冻结记录，仍有 {} 无法解冻", remaining)));
            }

            // 写入库存流水（记录可用库存的变动）
            let available_before = available;
            let available_after = available + quantity;
            let log = stock_log::ActiveModel {
                product_id: Set(Some(product_id)),
                warehouse_id: Set(Some(warehouse_id)),
                change_type: Set(Some("unfreeze".to_string())),
                biz_type: Set(Some("unfreeze".to_string())),
                quantity_before: Set(Some(available_before)),
                change_quantity: Set(Some(quantity)),
                quantity_after: Set(Some(available_after)),
                operator_id: Set(Some(unfreeze_by)),
                remark: Set(Some(format!("解冻库存 {}，解冻后可用库存 {}", quantity, available_after))),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            log.insert(txn).await?;

            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(())
}