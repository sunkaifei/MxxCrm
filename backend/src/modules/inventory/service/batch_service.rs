//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use rust_decimal::Decimal;
use sea_orm::{DbConn, TransactionTrait, EntityTrait, ColumnTrait, QueryFilter, QueryOrder};

use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::entity::{batch, warehouse};
use crate::modules::inventory::model::batch::{
    find_active_by_product, find_by_id, select_page, BatchCreateRequest, BatchListQuery, BatchListVO,
    BatchListItem,
};
use crate::modules::inventory::entity::stock_log;

/// 批次列表查询
pub async fn get_list(db: &DbConn, query: &BatchListQuery) -> Result<BatchListVO> {
    let (models, total) = select_page(db, query)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut list: Vec<BatchListItem> = models.into_iter().map(|m| m.into()).collect();

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

    Ok(BatchListVO { list, total })
}

/// 批次详情（含全链路流水）
pub async fn get_detail(db: &DbConn, id: i64) -> Result<serde_json::Value> {
    let batch_model = find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("批次不存在".to_string()))?;

    // 查询该批次的库存流水（通过 biz_no 关联批次号 或 biz_id 关联入库单）
    let mut logs_query = stock_log::Entity::find()
        .filter(stock_log::Column::ProductId.eq(batch_model.product_id.unwrap_or(0)))
        .filter(stock_log::Column::WarehouseId.eq(batch_model.warehouse_id.unwrap_or(0)));

    if let Some(ref batch_no) = batch_model.batch_no {
        logs_query = logs_query.filter(stock_log::Column::BizNo.contains(batch_no));
    }

    let logs = logs_query
        .order_by_desc(stock_log::Column::CreateTime)
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(serde_json::json!({
        "batch": batch_model,
        "logs": logs,
    }))
}

/// 批次追踪（查询某批次从入库到出库的全链路记录）
pub async fn trace(db: &DbConn, id: i64) -> Result<serde_json::Value> {
    let batch_model = find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("批次不存在".to_string()))?;

    let product_id = batch_model.product_id.unwrap_or(0);
    let warehouse_id = batch_model.warehouse_id.unwrap_or(0);
    let batch_no = batch_model.batch_no.clone().unwrap_or_default();

    // 全链路流水：该产品+仓库的所有变动记录
    let logs = stock_log::Entity::find()
        .filter(stock_log::Column::ProductId.eq(product_id))
        .filter(stock_log::Column::WarehouseId.eq(warehouse_id))
        .order_by_asc(stock_log::Column::CreateTime)
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(serde_json::json!({
        "batch": batch_model,
        "traceLogs": logs,
        "batchNo": batch_no,
    }))
}

/// 按产品查询有效批次
pub async fn list_by_product(db: &DbConn, product_id: i64) -> Result<Vec<batch::Model>> {
    find_active_by_product(db, product_id)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 内部接口：在入库审核时自动创建批次记录（事务内调用）
pub async fn create_batch_for_inbound<C: sea_orm::ConnectionTrait>(
    db: &C,
    req: &BatchCreateRequest,
    created_by: i64,
) -> Result<i64> {
    let id = crate::modules::inventory::model::batch::insert(db, req, created_by)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 创建批次（事务包裹，符合新模块强制规则）
pub async fn create(db: &DbConn, req: &BatchCreateRequest, created_by: i64) -> Result<i64> {
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let id = crate::modules::inventory::model::batch::insert(&txn, req, created_by)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    txn.commit()
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 出库时扣减批次数量（事务包裹）
pub async fn decrease_quantity_for_outbound(
    db: &DbConn,
    batch_id: i64,
    quantity: Decimal,
) -> Result<()> {
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    crate::modules::inventory::model::batch::decrease_quantity(&txn, batch_id, quantity)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    txn.commit()
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}
