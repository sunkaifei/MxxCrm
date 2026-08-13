//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::entity::stock::{self, ActiveModel, Column, Entity};
use crate::modules::inventory::entity::stock_log;
use crate::modules::inventory::entity::alert_rule;
use crate::modules::inventory::model::stock::{
    InventoryDetailVO, InventoryListData, InventoryListQuery, InventoryListVO, SafetyStockRequest,
    StockWarningListData, StockWarningQuery, StockWarningVO,
};
use crate::modules::inventory::service::stock_engine;
use crate::modules::product::entity::product as product_entity;
use crate::modules::product::entity::sku as sku_entity;
use crate::modules::inventory::entity::warehouse as warehouse_entity;
use rust_decimal::Decimal;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use std::collections::HashMap;

pub async fn get_list(db: &DatabaseConnection, query: &InventoryListQuery) -> Result<InventoryListData> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let mut condition = Entity::find()
        .filter(Column::Deleted.eq(0));

    if let Some(warehouse_id) = query.warehouse_id {
        condition = condition.filter(Column::WarehouseId.eq(warehouse_id));
    }

    // 在 DB 层按产品名过滤：先查产品 ID 集合，再用 IN 过滤
    if let Some(ref name) = query.product_name {
        if !name.is_empty() {
            let product_ids: Vec<i64> = product_entity::Entity::find()
                .filter(product_entity::Column::Name.contains(name))
                .all(db)
                .await?
                .into_iter()
                .map(|p| p.id)
                .collect();
            if product_ids.is_empty() {
                return Ok(InventoryListData { total: 0, items: vec![] });
            }
            condition = condition.filter(Column::ProductId.is_in(product_ids));
        }
    }

    let paginator = condition.paginate(db, page_size as u64);
    let total = paginator.num_items().await?;
    let models = paginator.fetch_page((page_num - 1) as u64).await?;

    let mut result: Vec<InventoryListVO> = Vec::new();
    for stock in models {
        let product: Option<product_entity::Model> = product_entity::Entity::find_by_id(stock.product_id.unwrap_or(0))
            .one(db)
            .await
            .ok()
            .flatten();
        let warehouse: Option<warehouse_entity::Model> = warehouse_entity::Entity::find_by_id(stock.warehouse_id.unwrap_or(0))
            .one(db)
            .await
            .ok()
            .flatten();

        // 查询SKU信息
        let sku: Option<sku_entity::Model> = if let Some(sku_id) = stock.sku_id.filter(|&id| id > 0) {
            sku_entity::Entity::find_by_id(sku_id)
                .one(db)
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        // 格式化规格文本
        let spec_text = sku.as_ref().and_then(|s| {
            s.specs.as_ref().and_then(|v| {
                if v.is_object() {
                    let obj = v.as_object()?;
                    let parts: Vec<String> = obj.iter().map(|(k, v)| format!("{}:{}", k, v.as_str().unwrap_or(""))).collect();
                    if parts.is_empty() { None } else { Some(parts.join(" ")) }
                } else {
                    None
                }
            })
        });

        result.push(InventoryListVO {
            id: Some(stock.id),
            product_id: stock.product_id,
            product_name: product.as_ref().and_then(|p| p.name.clone()),
            product_code: product.as_ref().and_then(|p| p.product_no.clone()),
            warehouse_id: stock.warehouse_id,
            warehouse_name: warehouse.as_ref().and_then(|w| w.name.clone()),
            sku_id: stock.sku_id,
            sku_code: sku.as_ref().and_then(|s| s.sku_code.clone()),
            spec_text,
            quantity: stock.quantity,
            reserved_quantity: stock.reserved_quantity,
            available_quantity: stock.available_quantity,
            in_transit_quantity: stock.in_transit_quantity,
            frozen_quantity: stock.frozen_quantity,
            avg_cost: stock.avg_cost,
            total_cost: stock.total_cost,
            last_inbound_time: stock.last_inbound_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            last_outbound_time: stock.last_outbound_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: stock.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        });
    }

    Ok(InventoryListData { total: total as i64, items: result })
}

pub async fn get_detail(db: &DatabaseConnection, id: i64) -> Result<InventoryDetailVO> {
    let stock = Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| format!("库存不存在，ID: {}", id))?;

    let product: Option<product_entity::Model> = product_entity::Entity::find_by_id(stock.product_id.unwrap_or(0))
        .one(db)
        .await
        .ok()
        .flatten();
    let warehouse: Option<warehouse_entity::Model> = warehouse_entity::Entity::find_by_id(stock.warehouse_id.unwrap_or(0))
        .one(db)
        .await
        .ok()
        .flatten();

    // 查询库存流水（最近50条）
    let product_id = stock.product_id.unwrap_or(0);
    let warehouse_id = stock.warehouse_id.unwrap_or(0);
    let logs = stock_log::Entity::find()
        .filter(stock_log::Column::ProductId.eq(product_id))
        .filter(stock_log::Column::WarehouseId.eq(warehouse_id))
        .order_by_desc(stock_log::Column::CreateTime)
        .limit(50)
        .all(db)
        .await?;

    Ok(InventoryDetailVO {
        id: Some(stock.id),
        product_id: stock.product_id,
        product_name: product.as_ref().and_then(|p| p.name.clone()),
        product_code: product.as_ref().and_then(|p| p.product_no.clone()),
        spec: product.as_ref().and_then(|p| p.sku.clone()),
        unit: product.as_ref().and_then(|p| p.unit.clone()),
        warehouse_id: stock.warehouse_id,
        warehouse_name: warehouse.as_ref().and_then(|w| w.name.clone()),
        warehouse_code: warehouse.as_ref().and_then(|w| w.code.clone()),
        quantity: stock.quantity,
        reserved_quantity: stock.reserved_quantity,
        available_quantity: stock.available_quantity,
        in_transit_quantity: stock.in_transit_quantity,
        frozen_quantity: stock.frozen_quantity,
        avg_cost: stock.avg_cost,
        last_in_cost: stock.last_in_cost,
        total_cost: stock.total_cost,
        last_inbound_time: stock.last_inbound_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        last_outbound_time: stock.last_outbound_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        update_time: stock.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        logs,
    })
}

/// 设置仓库级安全库存（alert_min_quantity / alert_max_quantity）
pub async fn set_safety_stock(
    db: &DatabaseConnection,
    req: &SafetyStockRequest,
) -> Result<i64> {
    if req.warehouse_id <= 0 || req.product_id <= 0 {
        return Err(Error::from("参数无效：warehouseId、productId 必填"));
    }

    let now = chrono::Local::now().naive_local();

    db.transaction::<_, _, sea_orm::DbErr>(|txn| {
        let req_ware_id = req.warehouse_id;
        let req_prod_id = req.product_id;
        let req_min = req.alert_min_quantity;
        let req_max = req.alert_max_quantity;
        Box::pin(async move {
            // 查找现有库存记录
            let existing = stock::Entity::find()
                .filter(stock::Column::WarehouseId.eq(req_ware_id))
                .filter(stock::Column::ProductId.eq(req_prod_id))
                .filter(stock::Column::Deleted.eq(0))
                .one(txn)
                .await?;

            match existing {
                Some(s) => {
                    let mut active: ActiveModel = s.into();
                    active.alert_min_quantity = Set(req_min);
                    active.alert_max_quantity = Set(req_max);
                    active.update_time = Set(Some(now));
                    active.update(txn).await?;
                }
                None => {
                    // 库存记录不存在时，创建一条仅含安全库存设置的记录
                    let active = stock::ActiveModel {
                        product_id: Set(Some(req_prod_id)),
                        warehouse_id: Set(Some(req_ware_id)),
                        quantity: Set(Some(Decimal::ZERO)),
                        reserved_quantity: Set(Some(Decimal::ZERO)),
                        available_quantity: Set(Some(Decimal::ZERO)),
                        in_transit_quantity: Set(Some(Decimal::ZERO)),
                        frozen_quantity: Set(Some(Decimal::ZERO)),
                        alert_min_quantity: Set(req_min),
                        alert_max_quantity: Set(req_max),
                        deleted: Set(Some(0)),
                        create_time: Set(Some(now)),
                        update_time: Set(Some(now)),
                        ..Default::default()
                    };
                    active.insert(txn).await?;
                }
            }
            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(1)
}

/// 低库存预警列表：available_quantity < alert_min_quantity
pub async fn get_low_stock_list(
    db: &DatabaseConnection,
    query: &StockWarningQuery,
) -> Result<StockWarningListData> {
    let page_num = std::cmp::Ord::max(query.page_num, 1);
    let page_size = std::cmp::Ord::max(query.page_size, 1);

    let mut q = Entity::find()
        .filter(Column::Deleted.eq(0))
        .filter(Column::AlertMinQuantity.is_not_null())
        .filter(
            Expr::col(Column::AvailableQuantity).lt(Expr::col(Column::AlertMinQuantity)),
        );

    if let Some(wid) = query.warehouse_id {
        q = q.filter(Column::WarehouseId.eq(wid));
    }

    let paginator = q.clone().paginate(db, page_size);
    let total = paginator.num_items().await?;
    let models = paginator.fetch_page(page_num - 1).await?;

    let items = build_warning_vos(db, models).await;
    Ok(StockWarningListData { total, items })
}

/// 高库存预警列表：quantity > alert_max_quantity
pub async fn get_high_stock_list(
    db: &DatabaseConnection,
    query: &StockWarningQuery,
) -> Result<StockWarningListData> {
    let page_num = std::cmp::Ord::max(query.page_num, 1);
    let page_size = std::cmp::Ord::max(query.page_size, 1);

    let mut q = Entity::find()
        .filter(Column::Deleted.eq(0))
        .filter(Column::AlertMaxQuantity.is_not_null())
        .filter(Expr::col(Column::Quantity).gt(Expr::col(Column::AlertMaxQuantity)));

    if let Some(wid) = query.warehouse_id {
        q = q.filter(Column::WarehouseId.eq(wid));
    }

    let paginator = q.clone().paginate(db, page_size);
    let total = paginator.num_items().await?;
    let models = paginator.fetch_page(page_num - 1).await?;

    let items = build_warning_vos(db, models).await;
    Ok(StockWarningListData { total, items })
}

/// 呆滞库存预警：last_outbound_time 距今超过 days 天
pub async fn get_obsolete_stock_list(
    db: &DatabaseConnection,
    query: &StockWarningQuery,
) -> Result<StockWarningListData> {
    let page_num = std::cmp::Ord::max(query.page_num, 1);
    let page_size = std::cmp::Ord::max(query.page_size, 1);
    let days = std::cmp::Ord::max(query.days.unwrap_or(90), 1);

    let cutoff = chrono::Local::now().naive_local() - chrono::Duration::days(days as i64);

    let mut q = Entity::find()
        .filter(Column::Deleted.eq(0))
        .filter(
            Column::LastOutboundTime
                .lt(cutoff)
                .or(Column::LastOutboundTime.is_null()),
        )
        .filter(Column::Quantity.gt(Decimal::ZERO));

    if let Some(wid) = query.warehouse_id {
        q = q.filter(Column::WarehouseId.eq(wid));
    }

    let paginator = q.clone().paginate(db, page_size);
    let total = paginator.num_items().await?;
    let models = paginator.fetch_page(page_num - 1).await?;

    let now = chrono::Local::now().naive_local();
    let mut items = build_warning_vos(db, models).await;
    for vo in items.iter_mut() {
        if let Some(t) = &vo.last_outbound_time {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(t, "%Y-%m-%d %H:%M:%S") {
                vo.obsolete_days = Some((now - dt).num_days());
            }
        } else {
            // 从未出库，按入库时间或创建时间计算
            vo.obsolete_days = Some(days as i64);
        }
    }
    Ok(StockWarningListData { total, items })
}

/// 获取统一预警列表（联动 alert_rule 规则表）
pub async fn get_alert_list(
    db: &DatabaseConnection,
    product_name: Option<String>,
    alert_type: Option<String>,
    page_num: u64,
    page_size: u64,
) -> Result<StockWarningListData> {
    // 1. 加载所有启用的预警规则，按 (product_id, warehouse_id) 索引
    let rules = alert_rule::Entity::find()
        .filter(alert_rule::Column::Deleted.eq(0))
        .all(db)
        .await?;

    // 构建 stock 快速查找映射：key = (product_id, warehouse_id) → rule
    // 优先匹配精确规则（product+warehouse），其次匹配全局规则（product=None 或 warehouse=None）
    let mut exact_rules: HashMap<(i64, i64), &alert_rule::Model> = HashMap::new();
    let mut product_rules: HashMap<i64, &alert_rule::Model> = HashMap::new();
    let mut warehouse_rules: HashMap<i64, &alert_rule::Model> = HashMap::new();
    let mut global_rule: Option<&alert_rule::Model> = None;

    for r in &rules {
        match (r.product_id, r.warehouse_id) {
            (Some(pid), Some(wid)) => { exact_rules.insert((pid, wid), r); }
            (Some(pid), None) => { product_rules.entry(pid).or_insert(r); }
            (None, Some(wid)) => { warehouse_rules.entry(wid).or_insert(r); }
            (None, None) => { global_rule.get_or_insert(r); }
        }
    }

    // 2. 查询库存记录
    let mut condition = Entity::find()
        .filter(Column::Deleted.eq(0));

    if let Some(name) = &product_name {
        let product_ids: Vec<i64> = product_entity::Entity::find()
            .filter(product_entity::Column::Name.contains(name))
            .all(db)
            .await?
            .into_iter()
            .map(|p| p.id)
            .collect();
        if !product_ids.is_empty() {
            condition = condition.filter(Column::ProductId.is_in(product_ids));
        }
    }

    let stocks = condition.all(db).await?;
    let now = chrono::Local::now().naive_local();

    let mut items = Vec::new();
    for s in stocks {
        let pid = s.product_id.unwrap_or(0);
        let wid = s.warehouse_id.unwrap_or(0);
        let product = product_entity::Entity::find_by_id(pid).one(db).await.ok().flatten();
        let wh = warehouse_entity::Entity::find_by_id(wid).one(db).await.ok().flatten();
        let qty = s.quantity.unwrap_or_default();
        let available = s.available_quantity.unwrap_or_default();

        // 合并阈值：stock 表字段 + alert_rule（规则优先）
        let rule: Option<&alert_rule::Model> = exact_rules.get(&(pid, wid))
            .copied()
            .or_else(|| product_rules.get(&pid).copied())
            .or_else(|| warehouse_rules.get(&wid).copied())
            .or(global_rule);

        let (alert_min, enable_low) = match rule {
            Some(r) if r.enable_low_alert.unwrap_or(false) => (r.min_quantity.or(s.alert_min_quantity), true),
            _ => (s.alert_min_quantity, s.alert_min_quantity.is_some()),
        };
        let (alert_max, enable_high) = match rule {
            Some(r) if r.enable_high_alert.unwrap_or(false) => (r.max_quantity.or(s.alert_max_quantity), true),
            _ => (s.alert_max_quantity, s.alert_max_quantity.is_some()),
        };
        let stale_days_threshold = rule.and_then(|r| r.stale_days).unwrap_or(90);

        // 计算呆滞天数
        let stale_days = s.last_outbound_time
            .map(|dt| (now - dt).num_days())
            .or_else(|| s.last_inbound_time.map(|dt| (now - dt).num_days()));
        let enable_stale = match rule {
            Some(r) => r.enable_stale_alert.unwrap_or(false),
            None => true, // 无规则时默认启用
        };

        // 低库存预警
        if enable_low {
            if let Some(min) = alert_min {
                if available < min {
                    if alert_type.as_deref().map_or(true, |at| at == "low_stock") {
                        items.push(StockWarningVO {
                            id: Some(s.id),
                            product_id: Some(pid),
                            product_name: product.as_ref().and_then(|p| p.name.clone()),
                            product_code: product.as_ref().and_then(|p| p.product_no.clone()),
                            warehouse_id: Some(wid),
                            warehouse_name: wh.as_ref().and_then(|w| w.name.clone()),
                            quantity: Some(qty),
                            available_quantity: Some(available),
                            alert_min_quantity: Some(min),
                            alert_max_quantity: alert_max,
                            alert_type: Some("low_stock".to_string()),
                            last_inbound_time: s.last_inbound_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                            last_outbound_time: s.last_outbound_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                            obsolete_days: stale_days,
                        });
                        continue;
                    }
                }
            }
        }
        // 高库存预警
        if enable_high {
            if let Some(max) = alert_max {
                if qty > max {
                    if alert_type.as_deref().map_or(true, |at| at == "high_stock") {
                        items.push(StockWarningVO {
                            id: Some(s.id),
                            product_id: Some(pid),
                            product_name: product.as_ref().and_then(|p| p.name.clone()),
                            product_code: product.as_ref().and_then(|p| p.product_no.clone()),
                            warehouse_id: Some(wid),
                            warehouse_name: wh.as_ref().and_then(|w| w.name.clone()),
                            quantity: Some(qty),
                            available_quantity: Some(available),
                            alert_min_quantity: alert_min,
                            alert_max_quantity: Some(max),
                            alert_type: Some("high_stock".to_string()),
                            last_inbound_time: s.last_inbound_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                            last_outbound_time: s.last_outbound_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                            obsolete_days: stale_days,
                        });
                        continue;
                    }
                }
            }
        }
        // 呆滞预警
        if enable_stale {
            if let Some(days) = stale_days {
                if days >= stale_days_threshold as i64 {
                    if alert_type.as_deref().map_or(true, |at| at == "stale") {
                        items.push(StockWarningVO {
                            id: Some(s.id),
                            product_id: Some(pid),
                            product_name: product.as_ref().and_then(|p| p.name.clone()),
                            product_code: product.as_ref().and_then(|p| p.product_no.clone()),
                            warehouse_id: Some(wid),
                            warehouse_name: wh.as_ref().and_then(|w| w.name.clone()),
                            quantity: Some(qty),
                            available_quantity: Some(available),
                            alert_min_quantity: alert_min,
                            alert_max_quantity: alert_max,
                            alert_type: Some("stale".to_string()),
                            last_inbound_time: s.last_inbound_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                            last_outbound_time: s.last_outbound_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                            obsolete_days: Some(days),
                        });
                    }
                }
            }
        }
    }

    // 分页处理
    let total = items.len() as u64;
    let pn = if page_num < 1 { 1 } else { page_num };
    let ps = if page_size < 1 { 20 } else { page_size };
    let start = ((pn - 1) * ps) as usize;
    let end = std::cmp::min(start + ps as usize, items.len());
    let page_items = if start < items.len() { items[start..end].to_vec() } else { vec![] };

    Ok(StockWarningListData { total, items: page_items })
}

/// 库存调整：将指定产品在指定仓库的库存调整为指定数量
pub async fn adjust_stock(
    db: &DatabaseConnection,
    product_id: i64,
    warehouse_id: i64,
    new_quantity: Decimal,
    operator_id: i64,
    reason: Option<String>,
) -> Result<i64> {
    if product_id <= 0 || warehouse_id <= 0 {
        return Err(Error::from("参数无效：productId、warehouseId 必填"));
    }
    if new_quantity < Decimal::ZERO {
        return Err(Error::from("调整后的库存数量不能为负数"));
    }

    let now = chrono::Local::now().naive_local();

    db.transaction::<_, _, sea_orm::DbErr>(|txn| {
        let pid = product_id;
        let wid = warehouse_id;
        let new_qty = new_quantity;
        Box::pin(async move {
            let existing = stock::Entity::find()
                .filter(stock::Column::ProductId.eq(pid))
                .filter(stock::Column::WarehouseId.eq(wid))
                .filter(stock::Column::Deleted.eq(0))
                .lock_exclusive()
                .one(txn)
                .await?;

            let old_qty = existing.as_ref().map(|s| s.quantity.unwrap_or_default()).unwrap_or_default();
            let diff = new_qty - old_qty;

            match existing {
                Some(s) => {
                    let old_available = s.available_quantity.unwrap_or_default();
                    let new_available = old_available + diff;

                    let mut active: stock::ActiveModel = s.into();
                    active.quantity = Set(Some(new_qty));
                    active.available_quantity = Set(Some(new_available));
                    active.update_time = Set(Some(now));
                    active.update(txn).await?;
                }
                None => {
                    // 库存记录不存在时创建一条
                    let active = stock::ActiveModel {
                        product_id: Set(Some(pid)),
                        warehouse_id: Set(Some(wid)),
                        quantity: Set(Some(new_qty)),
                        reserved_quantity: Set(Some(Decimal::ZERO)),
                        available_quantity: Set(Some(new_qty)),
                        in_transit_quantity: Set(Some(Decimal::ZERO)),
                        frozen_quantity: Set(Some(Decimal::ZERO)),
                        deleted: Set(Some(0)),
                        create_time: Set(Some(now)),
                        update_time: Set(Some(now)),
                        ..Default::default()
                    };
                    active.insert(txn).await?;
                }
            }

            // 写入库存流水
            stock_engine::write_stock_log(
                txn,
                pid,
                wid,
                None,
                "adjust",
                "adjust",
                None,
                None,
                diff,
                Some(operator_id),
                reason.as_deref(),
            )
            .await?;

            Ok(1)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 将库存记录批量转换为预警 VO（补充产品/仓库名称）
async fn build_warning_vos(
    db: &DatabaseConnection,
    models: Vec<stock::Model>,
) -> Vec<StockWarningVO> {
    let mut result: Vec<StockWarningVO> = Vec::with_capacity(models.len());
    for s in models {
        let product: Option<product_entity::Model> =
            product_entity::Entity::find_by_id(s.product_id.unwrap_or(0))
                .one(db)
                .await
                .ok()
                .flatten();
        let warehouse: Option<warehouse_entity::Model> =
            warehouse_entity::Entity::find_by_id(s.warehouse_id.unwrap_or(0))
                .one(db)
                .await
                .ok()
                .flatten();

        result.push(StockWarningVO {
            id: Some(s.id),
            product_id: s.product_id,
            product_name: product.as_ref().and_then(|p| p.name.clone()),
            product_code: product.as_ref().and_then(|p| p.product_no.clone()),
            warehouse_id: s.warehouse_id,
            warehouse_name: warehouse.as_ref().and_then(|w| w.name.clone()),
            quantity: s.quantity,
            available_quantity: s.available_quantity,
            alert_min_quantity: s.alert_min_quantity,
            alert_max_quantity: s.alert_max_quantity,
            last_inbound_time: s
                .last_inbound_time
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            last_outbound_time: s
                .last_outbound_time
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            obsolete_days: None,
            alert_type: None,
        });
    }
    result
}
