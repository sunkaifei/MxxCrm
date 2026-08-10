//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 库位管理业务逻辑层
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::entity::bin_location::{self, Entity, Column};
use crate::modules::inventory::entity::stock_bin;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BinUtilizationVO {
    pub bin_id: i64,
    pub bin_code: Option<String>,
    pub bin_name: Option<String>,
    pub capacity: Option<Decimal>,
    pub used_capacity: Option<Decimal>,
    pub utilization_rate: Option<Decimal>,
    pub stock_count: i64,
}

/// 创建库位
pub async fn create_bin(
    db: &DbConn,
    warehouse_id: i64,
    area_id: Option<i64>,
    bin_code: String,
    bin_name: String,
    bin_type: Option<i32>,
    row: Option<i32>,
    col: Option<i32>,
    layer: Option<i32>,
    capacity: Option<Decimal>,
) -> Result<i64> {
    let now = chrono::Local::now().naive_local();

    let txn = db.begin().await?;
    let model = bin_location::ActiveModel {
        warehouse_id: Set(Some(warehouse_id)),
        area_id: Set(area_id),
        bin_code: Set(Some(bin_code)),
        bin_name: Set(Some(bin_name)),
        bin_type: Set(bin_type),
        row_no: Set(row),
        column_no: Set(col),
        layer_no: Set(layer),
        capacity: Set(capacity),
        used_capacity: Set(Some(Decimal::ZERO)),
        is_active: Set(Some(1)),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    let result = model.insert(&txn).await?;
    txn.commit().await?;

    Ok(result.id)
}

/// 按仓库查询库位
pub async fn get_bins_by_warehouse(db: &DbConn, warehouse_id: i64) -> Result<Vec<bin_location::Model>> {
    let list = Entity::find()
        .filter(Column::WarehouseId.eq(warehouse_id))
        .filter(Column::Deleted.eq(0))
        .order_by_asc(Column::SortOrder)
        .order_by_asc(Column::Id)
        .all(db)
        .await?;
    Ok(list)
}

/// 库存上架到库位
pub async fn assign_stock_to_bin(
    db: &DbConn,
    stock_id: i64,
    bin_id: i64,
    quantity: Decimal,
) -> Result<i64> {
    // 查询库位
    let bin = Entity::find_by_id(bin_id)
        .filter(Column::Deleted.eq(0))
        .filter(Column::IsActive.eq(1))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("库位不存在或已停用"))?;

    // 检查容量
    if let Some(cap) = bin.capacity {
        if cap > Decimal::ZERO {
            let used = bin.used_capacity.unwrap_or(Decimal::ZERO);
            if used + quantity > cap {
                return Err(Error::from(format!(
                    "库位容量不足：当前已用 {} + 本次 {} > 容量 {}",
                    used, quantity, cap
                )));
            }
        }
    }

    let now = chrono::Local::now().naive_local();
    let txn = db.begin().await?;

    // 写入 stock_bin
    let stock_bin_model = stock_bin::ActiveModel {
        stock_id: Set(Some(stock_id)),
        bin_location_id: Set(Some(bin_id)),
        warehouse_id: Set(bin.warehouse_id),
        quantity: Set(Some(quantity)),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    let result = stock_bin_model.insert(&txn).await?;

    // 更新库位已用容量
    let new_used = bin.used_capacity.unwrap_or(Decimal::ZERO) + quantity;
    bin_location::Entity::update_many()
        .col_expr(Column::UsedCapacity, sea_orm::sea_query::Expr::value(new_used))
        .col_expr(Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
        .filter(Column::Id.eq(bin_id))
        .filter(Column::Deleted.eq(0))
        .exec(&txn)
        .await
        .map_err(|e| Error::from(format!("更新库位容量失败: {}", e)))?;

    txn.commit().await?;

    Ok(result.id)
}

/// 库位利用率统计
pub async fn get_bin_utilization(db: &DbConn, warehouse_id: i64) -> Result<Vec<BinUtilizationVO>> {
    let bins = Entity::find()
        .filter(Column::WarehouseId.eq(warehouse_id))
        .filter(Column::Deleted.eq(0))
        .all(db)
        .await?;

    // 查询该仓库下所有 stock_bin 记录，统计每个库位的库存条目数
    let stock_bins = stock_bin::Entity::find()
        .filter(stock_bin::Column::WarehouseId.eq(warehouse_id))
        .filter(stock_bin::Column::Deleted.eq(0))
        .all(db)
        .await?;

    use std::collections::HashMap;
    let mut stock_count_map: HashMap<i64, i64> = HashMap::new();
    for sb in &stock_bins {
        if let Some(bin_id) = sb.bin_location_id {
            *stock_count_map.entry(bin_id).or_insert(0) += 1;
        }
    }

    let result: Vec<BinUtilizationVO> = bins.into_iter()
        .map(|bin| {
            let capacity = bin.capacity;
            let used = bin.used_capacity;
            let utilization_rate = match (&capacity, &used) {
                (Some(cap), Some(u)) if *cap > Decimal::ZERO => {
                    Some((*u / *cap * Decimal::from(100)).round_dp(2))
                }
                _ => None,
            };
            BinUtilizationVO {
                bin_id: bin.id,
                bin_code: bin.bin_code,
                bin_name: bin.bin_name,
                capacity,
                used_capacity: used,
                utilization_rate,
                stock_count: stock_count_map.get(&bin.id).copied().unwrap_or(0),
            }
        })
        .collect();

    Ok(result)
}
