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
use rust_decimal::Decimal;
use crate::modules::inventory::entity::{stock, stock_log};

/// 增加库存（入库审核时调用）
pub async fn increase_stock<C: ConnectionTrait>(
    db: &C,
    product_id: i64,
    warehouse_id: i64,
    quantity: Decimal,
    unit_price: Option<Decimal>,
) -> Result<(), DbErr> {
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
            let qty_before = s.quantity.unwrap_or_default();
            let qty_after = qty_before + quantity;
            let avail_before = s.available_quantity.unwrap_or_default();
            let avail_after = avail_before + quantity;
            let _reserved = s.reserved_quantity.unwrap_or_default();

            // 计算平均成本
            let old_cost = s.avg_cost.unwrap_or_default();
            let old_total_cost = old_cost * qty_before;
            let new_amount = unit_price.unwrap_or_default() * quantity;
            let new_total_cost = old_total_cost + new_amount;
            let new_avg_cost = if qty_after > Decimal::ZERO {
                new_total_cost / qty_after
            } else {
                Decimal::ZERO
            };

            let mut active: stock::ActiveModel = s.into();
            active.quantity = Set(Some(qty_after));
            active.available_quantity = Set(Some(avail_after));
            active.avg_cost = Set(Some(new_avg_cost));
            active.last_in_cost = Set(unit_price);
            active.total_cost = Set(Some(new_total_cost));
            active.last_inbound_time = Set(Some(now));
            active.update_time = Set(Some(now));
            active.update(db).await?;
        }
        None => {
            let avail = quantity - Decimal::ZERO;
            let active = stock::ActiveModel {
                product_id: Set(Some(product_id)),
                warehouse_id: Set(Some(warehouse_id)),
                quantity: Set(Some(quantity)),
                reserved_quantity: Set(Some(Decimal::ZERO)),
                available_quantity: Set(Some(avail)),
                in_transit_quantity: Set(Some(Decimal::ZERO)),
                frozen_quantity: Set(Some(Decimal::ZERO)),
                avg_cost: Set(unit_price),
                last_in_cost: Set(unit_price),
                total_cost: Set(unit_price.map(|p| p * quantity)),
                last_inbound_time: Set(Some(now)),
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

/// 扣减库存（出库审核时调用）
///
/// ## 拣货策略（FIFO 先入先出）
///
/// 当前阶段（仓储二期）按"产品维度"扣减总库存，暂未实现批次级拣货。
/// 第四期实现批次管理后，扣减顺序应遵循：
/// 1. 若有 `batch_no`，按 `production_date` / `expiry_date` 升序扣减（先到期先出）；
/// 2. 否则按 `last_inbound_time` 升序扣减（先入先出 FIFO）。
/// 3. 扣减时按批次逐条扣减直至满足出库数量。
///
/// 当前实现按明细直接扣减总库存（仅校验可用库存是否充足），
/// 批次号记录在出库明细 `batch_no` 字段中，由前端选择指定。
pub async fn decrease_stock<C: ConnectionTrait>(
    db: &C,
    product_id: i64,
    warehouse_id: i64,
    quantity: Decimal,
) -> Result<Decimal, DbErr> {
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
            let available = s.available_quantity.unwrap_or_default();
            if available < quantity {
                return Err(DbErr::Custom(format!("库存不足：可用 {}，需要 {}", available, quantity)));
            }

            let qty_before = s.quantity.unwrap_or_default();
            let qty_after = qty_before - quantity;
            let avail_after = available - quantity;
            let avg_cost = s.avg_cost.unwrap_or_default();
            let out_amount = avg_cost * quantity;
            let old_total_cost = s.total_cost.unwrap_or_default();

            let mut active: stock::ActiveModel = s.into();
            active.quantity = Set(Some(qty_after));
            active.available_quantity = Set(Some(avail_after));
            active.total_cost = Set(Some(old_total_cost - out_amount));
            active.last_outbound_time = Set(Some(now));
            active.update_time = Set(Some(now));
            active.update(db).await?;

            Ok(out_amount) // 返回出库成本
        }
        None => {
            Err(DbErr::Custom("库存记录不存在".into()))
        }
    }
}

/// 写入库存流水
pub async fn write_stock_log<C: ConnectionTrait>(
    db: &C,
    product_id: i64,
    warehouse_id: i64,
    warehouse_area_id: Option<i64>,
    change_type: &str,
    biz_type: &str,
    biz_id: Option<i64>,
    biz_no: Option<&str>,
    change_quantity: Decimal,
    operator_id: Option<i64>,
    remark: Option<&str>,
) -> Result<(), DbErr> {
    // 获取当前库存数量
    let current = stock::Entity::find()
        .filter(stock::Column::ProductId.eq(product_id))
        .filter(stock::Column::WarehouseId.eq(warehouse_id))
        .filter(stock::Column::Deleted.eq(0))
        .one(db)
        .await?;

    let quantity_after = current.as_ref()
        .and_then(|s| s.quantity)
        .unwrap_or_default();
    let quantity_before = quantity_after - change_quantity;

    let now = chrono::Local::now().naive_local();
    let active = stock_log::ActiveModel {
        product_id: Set(Some(product_id)),
        warehouse_id: Set(Some(warehouse_id)),
        warehouse_area_id: Set(warehouse_area_id),
        change_type: Set(Some(change_type.to_string())),
        biz_type: Set(Some(biz_type.to_string())),
        biz_id: Set(biz_id),
        biz_no: Set(biz_no.map(|s| s.to_string())),
        quantity_before: Set(Some(quantity_before)),
        change_quantity: Set(Some(change_quantity)),
        quantity_after: Set(Some(quantity_after)),
        operator_id: Set(operator_id),
        remark: Set(remark.map(|s| s.to_string())),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    active.insert(db).await?;
    Ok(())
}