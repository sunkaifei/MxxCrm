//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{DbConn, TransactionTrait, EntityTrait, ColumnTrait, QueryFilter, Set};
use sea_orm::prelude::Date;

use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::entity::{stock, stock_snapshot, warehouse};
use crate::modules::inventory::model::stock_snapshot::{
    delete_by_date, select_page, StockSnapshotListItem, StockSnapshotListQuery,
    StockSnapshotListVO,
};
use crate::modules::product::entity::product as product_entity;

/// 生成每日库存快照
/// 查询所有库存记录，生成当日快照
pub async fn generate_daily_snapshot(db: &DbConn) -> Result<i64> {
    let today: Date = chrono::Local::now().naive_local().date();

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    // 删除当日已存在的快照（支持重跑）
    delete_by_date(&txn, today)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 查询所有未删除的库存记录
    let stocks = stock::Entity::find()
        .filter(stock::Column::Deleted.eq(0))
        .all(&txn)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 批量查询产品信息
    let product_ids: Vec<i64> = stocks
        .iter()
        .filter_map(|s| s.product_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let products = if product_ids.is_empty() {
        Vec::new()
    } else {
        product_entity::Entity::find()
            .filter(product_entity::Column::Id.is_in(product_ids))
            .all(&txn)
            .await
            .map_err(|e| Error::from(e.to_string()))?
    };
    let product_map: std::collections::HashMap<i64, product_entity::Model> = products
        .into_iter()
        .map(|p| (p.id, p))
        .collect();

    // 构建快照记录
    let now = chrono::Local::now().naive_local();
    let mut records: Vec<stock_snapshot::ActiveModel> = Vec::with_capacity(stocks.len());
    for s in stocks {
        let product_id = s.product_id.unwrap_or_default();
        let product_info = product_map.get(&product_id);
        let active = stock_snapshot::ActiveModel {
            snapshot_date: sea_orm::Set(Some(today)),
            warehouse_id: sea_orm::Set(s.warehouse_id),
            product_id: sea_orm::Set(s.product_id),
            product_name: sea_orm::Set(product_info.and_then(|p| p.name.clone())),
            product_sku: sea_orm::Set(product_info.and_then(|p| p.sku.clone())),
            quantity: sea_orm::Set(s.quantity),
            available_quantity: sea_orm::Set(s.available_quantity),
            frozen_quantity: sea_orm::Set(s.frozen_quantity),
            in_transit_quantity: sea_orm::Set(s.in_transit_quantity),
            avg_cost: sea_orm::Set(s.avg_cost),
            total_cost: sea_orm::Set(s.total_cost),
            deleted: sea_orm::Set(Some(0)),
            create_time: sea_orm::Set(Some(now)),
            ..Default::default()
        };
        records.push(active);
    }

    let count = if records.is_empty() {
        0i64
    } else {
        let result = stock_snapshot::Entity::insert_many(records)
            .exec(&txn)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        result.last_insert_id.unwrap_or(0)
    };

    txn.commit()
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(count)
}

/// 快照列表查询
pub async fn get_list(
    db: &DbConn,
    query: &StockSnapshotListQuery,
) -> Result<StockSnapshotListVO> {
    let (models, total) = select_page(db, query)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut list: Vec<StockSnapshotListItem> = models.into_iter().map(|m| m.into()).collect();

    // 补充仓库名称
    for item in &mut list {
        if let Some(wid) = item.warehouse_id {
            if let Ok(Some(wh)) = warehouse::Entity::find_by_id(wid)
                .filter(warehouse::Column::Deleted.eq(0))
                .one(db)
                .await
            {
                item.warehouse_name = wh.name;
            }
        }
    }

    Ok(StockSnapshotListVO { list, total })
}

/// 手动生成快照（供 controller 调用）
pub async fn generate(db: &DbConn) -> Result<i64> {
    generate_daily_snapshot(db).await
}
