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
use crate::modules::inventory::entity::{stocktake, stocktake_item, stock, warehouse};
use crate::modules::inventory::entity::{inbound, inbound_item, outbound, outbound_item};
use crate::modules::inventory::model::stocktake::*;
use crate::modules::inventory::service::stock_engine;
use crate::modules::system::entity::admin;

/// 生成盘点单号：PD + yyyyMMdd + 4位流水号
pub async fn generate_stocktake_no(db: &DatabaseConnection) -> Result<String> {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let prefix = format!("PD{}", today);

    let max_no = stocktake::Entity::find()
        .filter(stocktake::Column::StocktakeNo.starts_with(&prefix))
        .order_by_desc(stocktake::Column::StocktakeNo)
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let seq = match max_no {
        Some(m) => {
            let no = m.stocktake_no.unwrap_or_default();
            let seq_str = no.trim_start_matches(&prefix);
            seq_str.parse::<i32>().unwrap_or(0) + 1
        }
        None => 1,
    };

    Ok(format!("{}{:04}", prefix, seq))
}

/// 创建盘点单（自动查询当前库存填充 system_quantity）
pub async fn create(
    db: &DatabaseConnection,
    req: &StocktakeSaveRequest,
    created_by: i64,
) -> Result<i64> {
    let stocktake_no = generate_stocktake_no(db).await?;
    let stocktake_type = req.stocktake_type.clone().unwrap_or_else(|| "partial".to_string());
    let warehouse_id = req.warehouse_id;
    let total_items = req.items.len() as i32;

    let stocktake_id = db.transaction::<_, _, DbErr>(|txn| {
        let stocktake_no = stocktake_no.clone();
        let stocktake_type = stocktake_type.clone();
        let remark = req.remark.clone();
        let items = req.items.clone();
        Box::pin(async move {
            // 1. 插入主表
            let id = insert_main(txn, &stocktake_no, warehouse_id, &stocktake_type, remark.as_deref(), total_items, created_by).await?;

            // 2. 插入明细（查询当前库存填充 system_quantity）
            let now = chrono::Local::now().naive_local();
            for item in &items {
                // 查询当前库存
                let current_stock = stock::Entity::find()
                    .filter(stock::Column::ProductId.eq(item.product_id))
                    .filter(stock::Column::WarehouseId.eq(warehouse_id))
                    .filter(stock::Column::Deleted.eq(0))
                    .one(txn)
                    .await?;

                let system_qty = current_stock.as_ref()
                    .and_then(|s| s.quantity)
                    .unwrap_or_default();

                let item_active = stocktake_item::ActiveModel {
                    stocktake_id: Set(Some(id)),
                    product_id: Set(Some(item.product_id)),
                    product_name: Set(item.product_name.clone()),
                    product_sku: Set(item.product_sku.clone()),
                    system_quantity: Set(Some(system_qty)),
                    actual_quantity: Set(None),
                    difference: Set(Some(Decimal::ZERO)),
                    difference_type: Set(Some(0)),
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

    Ok(stocktake_id)
}

/// 编辑盘点单（仅草稿状态可编辑）
pub async fn update(
    db: &DatabaseConnection,
    id: i64,
    req: &StocktakeSaveRequest,
    updated_by: i64,
) -> Result<i64> {
    // 检查状态：仅草稿(0)可编辑
    let order = stocktake::Entity::find_by_id(id)
        .filter(stocktake::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("盘点单不存在".to_string()))?;

    if order.status.unwrap_or(0) != 0 {
        return Err(Error::from("仅草稿状态的盘点单可编辑".to_string()));
    }

    let stocktake_type = req.stocktake_type.clone().unwrap_or_else(|| "partial".to_string());
    let warehouse_id = req.warehouse_id;
    let total_items = req.items.len() as i32;

    db.transaction::<_, _, DbErr>(|txn| {
        let stocktake_type = stocktake_type.clone();
        let remark = req.remark.clone();
        let items = req.items.clone();
        Box::pin(async move {
            // 1. 更新主表
            update_main(txn, id, &stocktake_type, remark.as_deref(), total_items, updated_by).await?;

            // 2. 软删除原明细
            stocktake_item::Entity::update_many()
                .col_expr(stocktake_item::Column::Deleted, Expr::value(1))
                .filter(stocktake_item::Column::StocktakeId.eq(id))
                .exec(txn)
                .await?;

            // 3. 插入新明细
            let now = chrono::Local::now().naive_local();
            for item in &items {
                let current_stock = stock::Entity::find()
                    .filter(stock::Column::ProductId.eq(item.product_id))
                    .filter(stock::Column::WarehouseId.eq(warehouse_id))
                    .filter(stock::Column::Deleted.eq(0))
                    .one(txn)
                    .await?;

                let system_qty = current_stock.as_ref()
                    .and_then(|s| s.quantity)
                    .unwrap_or_default();

                let item_active = stocktake_item::ActiveModel {
                    stocktake_id: Set(Some(id)),
                    product_id: Set(Some(item.product_id)),
                    product_name: Set(item.product_name.clone()),
                    product_sku: Set(item.product_sku.clone()),
                    system_quantity: Set(Some(system_qty)),
                    actual_quantity: Set(None),
                    difference: Set(Some(Decimal::ZERO)),
                    difference_type: Set(Some(0)),
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

/// 提交盘点（草稿→盘点中）
pub async fn submit(
    db: &DatabaseConnection,
    id: i64,
    updated_by: i64,
) -> Result<i64> {
    let order = stocktake::Entity::find_by_id(id)
        .filter(stocktake::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("盘点单不存在".to_string()))?;

    if order.status.unwrap_or(0) != 0 {
        return Err(Error::from("仅草稿状态的盘点单可提交".to_string()));
    }

    update_status(db, id, 1, updated_by).await
}

/// 盘点录入（录入实盘数量，自动计算差异）
pub async fn input(
    db: &DatabaseConnection,
    id: i64,
    req: &StocktakeInputRequest,
    updated_by: i64,
) -> Result<i64> {
    let order = stocktake::Entity::find_by_id(id)
        .filter(stocktake::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("盘点单不存在".to_string()))?;

    let status = order.status.unwrap_or(0);
    if status != 0 && status != 1 {
        return Err(Error::from("仅草稿或盘点中状态可录入".to_string()));
    }

    let now = chrono::Local::now().naive_local();
    db.transaction::<_, _, DbErr>(|txn| {
        let items = req.items.clone();
        Box::pin(async move {
            for input_item in &items {
                // 查询明细
                let detail = stocktake_item::Entity::find_by_id(input_item.id)
                    .filter(stocktake_item::Column::Deleted.eq(0))
                    .one(txn)
                    .await?
                    .ok_or_else(|| DbErr::Custom(format!("盘点明细不存在，ID: {}", input_item.id)))?;

                let system_qty = detail.system_quantity.unwrap_or_default();
                let actual_qty = input_item.actual_quantity;
                let difference = actual_qty - system_qty;
                let diff_type = if difference > Decimal::ZERO {
                    1 // 盘盈
                } else if difference < Decimal::ZERO {
                    2 // 盘亏
                } else {
                    0 // 一致
                };

                stocktake_item::Entity::update_many()
                    .col_expr(stocktake_item::Column::ActualQuantity, Expr::value(actual_qty))
                    .col_expr(stocktake_item::Column::Difference, Expr::value(difference))
                    .col_expr(stocktake_item::Column::DifferenceType, Expr::value(diff_type))
                    .col_expr(stocktake_item::Column::Remark, Expr::value(input_item.remark.clone()))
                    .col_expr(stocktake_item::Column::UpdateTime, Expr::value(now))
                    .filter(stocktake_item::Column::Id.eq(input_item.id))
                    .filter(stocktake_item::Column::StocktakeId.eq(id))
                    .exec(txn)
                    .await?;
            }

            // 若当前是草稿状态，录入后自动转为盘点中
            if status == 0 {
                stocktake::Entity::update_many()
                    .col_expr(stocktake::Column::Status, Expr::value(1))
                    .col_expr(stocktake::Column::UpdatedBy, Expr::value(updated_by))
                    .col_expr(stocktake::Column::UpdateTime, Expr::value(now))
                    .filter(stocktake::Column::Id.eq(id))
                    .filter(stocktake::Column::Deleted.eq(0))
                    .exec(txn)
                    .await?;
            }

            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(id)
}

/// 完成盘点（盘点中→已完成，自动生成盘盈入库单/盘亏出库单，全部在同一事务内）
pub async fn complete(
    db: &DatabaseConnection,
    id: i64,
    updated_by: i64,
) -> Result<i64> {
    // 1. 查询盘点单
    let order = stocktake::Entity::find_by_id(id)
        .filter(stocktake::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("盘点单不存在".to_string()))?;

    if order.status.unwrap_or(0) != 1 {
        return Err(Error::from("仅盘点中状态的盘点单可完成".to_string()));
    }

    let warehouse_id = order.warehouse_id.unwrap_or_default();
    let stocktake_no = order.stocktake_no.clone().unwrap_or_default();

    // 2. 查询明细
    let items = stocktake_item::Entity::find()
        .filter(stocktake_item::Column::StocktakeId.eq(id))
        .filter(stocktake_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if items.is_empty() {
        return Err(Error::from("盘点明细为空，无法完成".to_string()));
    }

    // 3. 分类盘盈/盘亏明细（clone 出来避免借用冲突）
    let mut surplus_items: Vec<stocktake_item::Model> = Vec::new();
    let mut shortage_items: Vec<stocktake_item::Model> = Vec::new();
    let mut surplus_count = 0i32;
    let mut shortage_count = 0i32;

    for item in items {
        let diff = item.difference.unwrap_or_default();
        if diff > Decimal::ZERO {
            surplus_count += 1;
            surplus_items.push(item);
        } else if diff < Decimal::ZERO {
            shortage_count += 1;
            shortage_items.push(item);
        }
    }

    // 4. 全部在同一事务内执行：盘盈入库 + 盘亏出库 + 更新盘点单状态
    let stocktake_id = id;
    let stocktake_no_clone = stocktake_no.clone();
    let wid = warehouse_id;

    db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            let now = chrono::Local::now().naive_local();

            // 4a. 盘盈：直接创建入库单（已完成状态）并增加库存
            if !surplus_items.is_empty() {
                let mut total_qty = Decimal::ZERO;
                for item in &surplus_items {
                    total_qty += item.difference.unwrap_or_default();
                }

                // 生成入库单号
                let date_prefix = format!("RK{}", chrono::Local::now().format("%Y%m%d"));
                let max_inbound = inbound::Entity::find()
                    .filter(inbound::Column::InboundNo.starts_with(&date_prefix))
                    .order_by_desc(inbound::Column::InboundNo)
                    .one(txn).await?;
                let seq = max_inbound
                    .and_then(|m| m.inbound_no.as_deref()
                        .and_then(|s| s.get(date_prefix.len()..))
                        .and_then(|s| s.parse::<u32>().ok()))
                    .unwrap_or(0) + 1;
                let inbound_no = format!("{}{:04}", date_prefix, seq);

                let inbound_active = inbound::ActiveModel {
                    inbound_no: Set(Some(inbound_no.clone())),
                    inbound_type: Set(Some("check_surplus".to_string())),
                    source_order_id: Set(Some(stocktake_id)),
                    source_order_no: Set(Some(stocktake_no_clone.clone())),
                    warehouse_id: Set(Some(wid)),
                    status: Set(Some(3)),
                    total_quantity: Set(Some(total_qty)),
                    total_amount: Set(None),
                    remark: Set(Some(format!("盘盈入库，盘点单：{}", stocktake_no_clone))),
                    audit_by: Set(Some(updated_by)),
                    audit_time: Set(Some(now)),
                    deleted: Set(Some(0)),
                    created_by: Set(Some(updated_by)),
                    updated_by: Set(Some(updated_by)),
                    create_time: Set(Some(now)),
                    update_time: Set(Some(now)),
                    ..Default::default()
                };
                let inbound_result = inbound_active.insert(txn).await?;
                let inbound_id = inbound_result.id;

                for item in &surplus_items {
                    let product_id = item.product_id.unwrap_or_default();
                    let quantity = item.difference.unwrap_or_default();

                    let item_active = inbound_item::ActiveModel {
                        inbound_id: Set(Some(inbound_id)),
                        product_id: Set(Some(product_id)),
                        product_sku: Set(item.product_sku.clone()),
                        quantity: Set(Some(quantity)),
                        deleted: Set(Some(0)),
                        create_time: Set(Some(now)),
                        ..Default::default()
                    };
                    item_active.insert(txn).await?;

                    stock_engine::increase_stock(txn, product_id, wid, quantity, None).await?;
                    let surplus_remark = format!("盘盈入库，盘点单：{}", stocktake_no_clone);
                    stock_engine::write_stock_log(
                        txn, product_id, wid, None,
                        "inbound", "check_surplus",
                        Some(inbound_id), Some(&inbound_no),
                        quantity, Some(updated_by),
                        Some(&surplus_remark),
                    ).await?;
                }
            }

            // 4b. 盘亏：直接创建出库单（已完成状态）并扣减库存
            if !shortage_items.is_empty() {
                let mut total_qty = Decimal::ZERO;
                for item in &shortage_items {
                    total_qty += item.difference.unwrap_or_default().abs();
                }

                let date_prefix = format!("CK{}", chrono::Local::now().format("%Y%m%d"));
                let max_outbound = outbound::Entity::find()
                    .filter(outbound::Column::OutboundNo.starts_with(&date_prefix))
                    .order_by_desc(outbound::Column::OutboundNo)
                    .one(txn).await?;
                let seq = max_outbound
                    .and_then(|m| m.outbound_no.as_deref()
                        .and_then(|s| s.get(date_prefix.len()..))
                        .and_then(|s| s.parse::<u32>().ok()))
                    .unwrap_or(0) + 1;
                let outbound_no = format!("{}{:04}", date_prefix, seq);

                let outbound_active = outbound::ActiveModel {
                    outbound_no: Set(Some(outbound_no.clone())),
                    outbound_type: Set(Some("check_shortage".to_string())),
                    source_order_id: Set(Some(stocktake_id)),
                    source_order_no: Set(Some(stocktake_no_clone.clone())),
                    warehouse_id: Set(Some(wid)),
                    status: Set(Some(3)),
                    total_quantity: Set(Some(total_qty)),
                    total_amount: Set(None),
                    remark: Set(Some(format!("盘亏出库，盘点单：{}", stocktake_no_clone))),
                    audit_by: Set(Some(updated_by)),
                    audit_time: Set(Some(now)),
                    deleted: Set(Some(0)),
                    created_by: Set(Some(updated_by)),
                    updated_by: Set(Some(updated_by)),
                    create_time: Set(Some(now)),
                    update_time: Set(Some(now)),
                    ..Default::default()
                };
                let outbound_result = outbound_active.insert(txn).await?;
                let outbound_id = outbound_result.id;

                for item in &shortage_items {
                    let product_id = item.product_id.unwrap_or_default();
                    let quantity = item.difference.unwrap_or_default().abs();

                    let item_active = outbound_item::ActiveModel {
                        outbound_id: Set(Some(outbound_id)),
                        product_id: Set(Some(product_id)),
                        product_sku: Set(item.product_sku.clone()),
                        quantity: Set(Some(quantity)),
                        deleted: Set(Some(0)),
                        create_time: Set(Some(now)),
                        ..Default::default()
                    };
                    item_active.insert(txn).await?;

                    stock_engine::decrease_stock(txn, product_id, wid, quantity).await?;
                    let shortage_remark = format!("盘亏出库，盘点单：{}", stocktake_no_clone);
                    stock_engine::write_stock_log(
                        txn, product_id, wid, None,
                        "outbound", "check_shortage",
                        Some(outbound_id), Some(&outbound_no),
                        quantity, Some(updated_by),
                        Some(&shortage_remark),
                    ).await?;
                }
            }

            // 4c. 更新盘点单状态为已完成
            stocktake::Entity::update_many()
                .col_expr(stocktake::Column::Status, Expr::value(2))
                .col_expr(stocktake::Column::SurplusCount, Expr::value(surplus_count))
                .col_expr(stocktake::Column::ShortageCount, Expr::value(shortage_count))
                .col_expr(stocktake::Column::UpdatedBy, Expr::value(updated_by))
                .col_expr(stocktake::Column::UpdateTime, Expr::value(now))
                .filter(stocktake::Column::Id.eq(stocktake_id))
                .filter(stocktake::Column::Deleted.eq(0))
                .filter(stocktake::Column::Status.eq(1)) // 防止并发重复完成
                .exec(txn)
                .await?;

            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(id)
}

/// 取消盘点
pub async fn cancel(
    db: &DatabaseConnection,
    id: i64,
    updated_by: i64,
) -> Result<i64> {
    let order = stocktake::Entity::find_by_id(id)
        .filter(stocktake::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("盘点单不存在".to_string()))?;

    let status = order.status.unwrap_or(0);
    if status == 2 || status == 3 {
        return Err(Error::from("已完成或已取消的盘点单不可取消".to_string()));
    }

    update_status(db, id, 3, updated_by).await
}

/// 批量删除（仅草稿状态可删除）
pub async fn batch_delete(
    db: &DatabaseConnection,
    ids: &[i64],
) -> Result<i64> {
    let ids_vec = ids.to_vec();
    db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            crate::modules::inventory::model::stocktake::batch_delete(txn, &ids_vec).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 获取盘点单详情（主表 + 明细）
pub async fn get_detail(
    db: &DatabaseConnection,
    id: i64,
) -> Result<serde_json::Value> {
    let main = stocktake::Entity::find_by_id(id)
        .filter(stocktake::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("盘点单不存在".to_string()))?;

    let items = stocktake_item::Entity::find()
        .filter(stocktake_item::Column::StocktakeId.eq(id))
        .filter(stocktake_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(serde_json::json!({
        "main": main,
        "items": items,
    }))
}

/// 获取盘点明细列表
pub async fn get_items(
    db: &DatabaseConnection,
    stocktake_id: i64,
) -> Result<Vec<stocktake_item::Model>> {
    let items = stocktake_item::Entity::find()
        .filter(stocktake_item::Column::StocktakeId.eq(stocktake_id))
        .filter(stocktake_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(items)
}

/// 盘点单列表查询
pub async fn get_list(
    db: &DatabaseConnection,
    query: &StocktakeListQuery,
) -> Result<StocktakeListVO> {
    let (models, total) = select_page(db, query)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut items: Vec<StocktakeListItem> = models.into_iter().map(|m| m.into()).collect();

    // 补充仓库名称和创建人名称
    for item in &mut items {
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

    Ok(StocktakeListVO { total: total as i64, items })
}

// ========== 内部辅助函数 ==========

async fn insert_main<C: ConnectionTrait>(
    db: &C,
    stocktake_no: &str,
    warehouse_id: i64,
    stocktake_type: &str,
    remark: Option<&str>,
    total_items: i32,
    created_by: i64,
) -> std::result::Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let active = stocktake::ActiveModel {
        stocktake_no: Set(Some(stocktake_no.to_string())),
        warehouse_id: Set(Some(warehouse_id)),
        stocktake_type: Set(Some(stocktake_type.to_string())),
        status: Set(Some(0)),
        total_items: Set(Some(total_items)),
        surplus_count: Set(Some(0)),
        shortage_count: Set(Some(0)),
        remark: Set(remark.map(|s| s.to_string())),
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
    stocktake_type: &str,
    remark: Option<&str>,
    total_items: i32,
    updated_by: i64,
) -> std::result::Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = stocktake::Entity::update_many()
        .col_expr(stocktake::Column::StocktakeType, Expr::value(stocktake_type))
        .col_expr(stocktake::Column::Remark, Expr::value(remark.map(|s| s.to_string())))
        .col_expr(stocktake::Column::TotalItems, Expr::value(total_items))
        .col_expr(stocktake::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(stocktake::Column::UpdateTime, Expr::value(now))
        .filter(stocktake::Column::Id.eq(id))
        .filter(stocktake::Column::Deleted.eq(0))
        .filter(stocktake::Column::Status.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

async fn update_status<C: ConnectionTrait>(
    db: &C,
    id: i64,
    status: i32,
    updated_by: i64,
) -> Result<i64> {
    let now = chrono::Local::now().naive_local();
    let result = stocktake::Entity::update_many()
        .col_expr(stocktake::Column::Status, Expr::value(status))
        .col_expr(stocktake::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(stocktake::Column::UpdateTime, Expr::value(now))
        .filter(stocktake::Column::Id.eq(id))
        .filter(stocktake::Column::Deleted.eq(0))
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result.rows_affected as i64)
}
