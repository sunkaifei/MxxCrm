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
use std::collections::{HashMap, HashSet};

/// 将 SKU 的 specs JSON（如 {"颜色":"红色","尺寸":"XL"}）格式化为文本，如 "颜色：红色/尺寸:XL"
/// 兼容历史脏数据：specs 为双重编码的 JSON 字符串时先解包；值被拼上"规格名："前缀（如 键"颜色"配值"尺码：红色"）时剥离前缀
pub(crate) fn format_sku_specs(specs: Option<&serde_json::Value>) -> Option<String> {
    let value = specs?;
    let obj = if value.is_object() {
        value.as_object()?.to_owned()
    } else if let Some(s) = value.as_str() {
        serde_json::from_str::<serde_json::Value>(s)
            .ok()?
            .as_object()?
            .to_owned()
    } else {
        return None;
    };
    let parts: Vec<String> = obj
        .iter()
        .map(|(k, val)| {
            let raw = val.as_str().unwrap_or("");
            let cleaned = raw
                .split_once('：')
                .filter(|(p, _)| obj.contains_key(*p))
                .map(|(_, r)| r)
                .unwrap_or(raw);
            format!("{}：{}", k, cleaned)
        })
        .collect();
    if parts.is_empty() { None } else { Some(parts.join("/")) }
}

/// 预警规则六级优先级匹配：SKU精确(product+warehouse+sku) > product+sku > product+warehouse > product > warehouse > 全局
/// key 约定与 rule_map 一致：(product_id, warehouse_id, sku_id)，0 表示不限定
fn match_alert_rule<'a>(
    rule_map: &'a HashMap<(i64, i64, i64), &'a alert_rule::Model>,
    pid: i64,
    wid: i64,
    sid: Option<i64>,
) -> Option<&'a alert_rule::Model> {
    let mut found: Option<&alert_rule::Model> = None;
    if let Some(sku_id) = sid {
        found = found.or_else(|| rule_map.get(&(pid, wid, sku_id)).copied());
        found = found.or_else(|| rule_map.get(&(pid, 0, sku_id)).copied());
    }
    found = found.or_else(|| rule_map.get(&(pid, wid, 0)).copied());
    found = found.or_else(|| rule_map.get(&(pid, 0, 0)).copied());
    found = found.or_else(|| rule_map.get(&(0, wid, 0)).copied());
    found = found.or_else(|| rule_map.get(&(0, 0, 0)).copied());
    found
}

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

    // 查询库存流水（最近50条，按 SKU 维度过滤：多规格行只展示本规格流水）
    let product_id = stock.product_id.unwrap_or(0);
    let warehouse_id = stock.warehouse_id.unwrap_or(0);
    let logs = stock_log::Entity::find()
        .filter(stock_log::Column::ProductId.eq(product_id))
        .filter(stock_log::Column::WarehouseId.eq(warehouse_id))
        .filter(stock_engine::sku_condition(stock.sku_id))
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
        let req_sku_id = req.sku_id;
        let req_min = req.alert_min_quantity;
        let req_max = req.alert_max_quantity;
        Box::pin(async move {
            // 查找现有库存记录（含 SKU 维度：None 匹配单规格行）
            let existing = stock::Entity::find()
                .filter(stock::Column::WarehouseId.eq(req_ware_id))
                .filter(stock::Column::ProductId.eq(req_prod_id))
                .filter(stock::Column::Deleted.eq(0))
                .filter(stock_engine::sku_condition(req_sku_id))
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
                        sku_id: Set(req_sku_id),
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
    warehouse_id: Option<i64>,
    alert_type: Option<String>,
    page_num: u64,
    page_size: u64,
) -> Result<StockWarningListData> {
    // 1. 加载所有启用的预警规则，按 (product_id, warehouse_id) 索引
    let rules = alert_rule::Entity::find()
        .filter(alert_rule::Column::Deleted.eq(0))
        .all(db)
        .await?;

    // 构建 stock 快速查找映射：key = (product_id, warehouse_id, sku_id)，0 表示不限定
    // 规则 sku_id=None 表示对该产品全部规格生效；规则查找时 SKU 精确规则优先于全规格规则
    let mut rule_map: HashMap<(i64, i64, i64), &alert_rule::Model> = HashMap::new();
    for r in &rules {
        let key = (r.product_id.unwrap_or(0), r.warehouse_id.unwrap_or(0), r.sku_id.unwrap_or(0));
        rule_map.entry(key).or_insert(r);
    }

    // 2. 查询库存记录
    let mut condition = Entity::find()
        .filter(Column::Deleted.eq(0));

    if let Some(wid) = warehouse_id {
        condition = condition.filter(Column::WarehouseId.eq(wid));
    }

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

    // 已有库存行集合（sku 用 0 表示无 SKU），供缺货补全时排除已遍历的行
    let existing_rows: HashSet<(i64, i64, i64)> = stocks
        .iter()
        .map(|s| {
            (
                s.product_id.unwrap_or(0),
                s.warehouse_id.unwrap_or(0),
                s.sku_id.filter(|&id| id > 0).unwrap_or(0),
            )
        })
        .collect();

    // 批量预加载产品/仓库/SKU，避免循环内逐条查询
    let product_ids: Vec<i64> = stocks.iter().filter_map(|s| s.product_id).collect();
    let warehouse_ids: Vec<i64> = stocks.iter().filter_map(|s| s.warehouse_id).collect();
    let sku_ids: Vec<i64> = stocks.iter().filter_map(|s| s.sku_id.filter(|&id| id > 0)).collect();
    let product_map: HashMap<i64, product_entity::Model> = if product_ids.is_empty() {
        HashMap::new()
    } else {
        product_entity::Entity::find()
            .filter(product_entity::Column::Id.is_in(product_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|p| (p.id, p))
            .collect()
    };
    let warehouse_map: HashMap<i64, warehouse_entity::Model> = if warehouse_ids.is_empty() {
        HashMap::new()
    } else {
        warehouse_entity::Entity::find()
            .filter(warehouse_entity::Column::Id.is_in(warehouse_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|w| (w.id, w))
            .collect()
    };
    let sku_map: HashMap<i64, sku_entity::Model> = if sku_ids.is_empty() {
        HashMap::new()
    } else {
        sku_entity::Entity::find()
            .filter(sku_entity::Column::Id.is_in(sku_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|s| (s.id, s))
            .collect()
    };

    let mut items = Vec::new();
    for s in stocks {
        let pid = s.product_id.unwrap_or(0);
        let wid = s.warehouse_id.unwrap_or(0);
        let sid = s.sku_id.filter(|&id| id > 0);
        let product = product_map.get(&pid);
        let wh = warehouse_map.get(&wid);
        let sku = sid.and_then(|id| sku_map.get(&id));
        let qty = s.quantity.unwrap_or_default();
        let available = s.available_quantity.unwrap_or_default();

        // 规格文本：多规格行取 SKU specs，单规格行回退产品规格描述
        let sku_spec_text = sku.as_ref().and_then(|sk| format_sku_specs(sk.specs.as_ref()));
        let spec_text = sku_spec_text
            .or_else(|| product.as_ref().and_then(|p| p.sku.clone()));

        // 规则匹配（六级优先级，见 match_alert_rule）
        let rule: Option<&alert_rule::Model> = match_alert_rule(&rule_map, pid, wid, sid);

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
                            sku_id: sid,
                            sku_code: sku.as_ref().and_then(|sk| sk.sku_code.clone()),
                            spec_text: spec_text.clone(),
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
                            sku_id: sid,
                            sku_code: sku.as_ref().and_then(|sk| sk.sku_code.clone()),
                            spec_text: spec_text.clone(),
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
                            sku_id: sid,
                            sku_code: sku.as_ref().and_then(|sk| sk.sku_code.clone()),
                            spec_text: spec_text.clone(),
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

    // 3. 缺货补全：从未入库的规格在 stock 表没有行，仅按库存行遍历会漏报 0 库存缺货。
    //    对明确指定了产品+仓库且启用低库存预警的规则，其覆盖范围内没有库存行的
    //    规格/产品按 0 库存参与预警（0 < 最低阈值即缺货）。
    let abs_rules: Vec<&alert_rule::Model> = rules
        .iter()
        .filter(|r| {
            r.product_id.is_some()
                && r.warehouse_id.is_some()
                && r.enable_low_alert.unwrap_or(false)
                && r.min_quantity.map(|m| m > Decimal::ZERO).unwrap_or(false)
        })
        .collect();

    if !abs_rules.is_empty() {
        let covered_pids: Vec<i64> = abs_rules.iter().map(|r| r.product_id.unwrap()).collect();
        let covered_products: HashMap<i64, product_entity::Model> = product_entity::Entity::find()
            .filter(product_entity::Column::Id.is_in(covered_pids.clone()))
            .all(db)
            .await?
            .into_iter()
            .map(|p| (p.id, p))
            .collect();
        let mut covered_skus: HashMap<i64, Vec<sku_entity::Model>> = HashMap::new();
        for s in sku_entity::Entity::find()
            .filter(sku_entity::Column::ProductId.is_in(covered_pids.clone()))
            .all(db)
            .await?
        {
            covered_skus.entry(s.product_id).or_default().push(s);
        }

        let mut seen: HashSet<(i64, i64, i64)> = HashSet::new();
        for r in &abs_rules {
            let pid = r.product_id.unwrap();
            let wid = r.warehouse_id.unwrap();
            // 覆盖的规格单元：指定 sku 仅该规格；未指定则产品全部规格（产品无规格记录时为产品级单元 0）
            let units: Vec<i64> = match r.sku_id {
                Some(sid) => vec![sid],
                None => covered_skus
                    .get(&pid)
                    .map(|list| list.iter().map(|s| s.id).collect())
                    .unwrap_or_else(|| vec![0]),
            };
            for sid0 in units {
                if !seen.insert((pid, wid, sid0)) || existing_rows.contains(&(pid, wid, sid0)) {
                    continue;
                }
                let sid = if sid0 > 0 { Some(sid0) } else { None };
                // 与库存行同一套六级匹配，保证阈值来源一致
                let rule = match_alert_rule(&rule_map, pid, wid, sid);
                let Some(rule) = rule else { continue };
                if !rule.enable_low_alert.unwrap_or(false) {
                    continue;
                }
                let Some(min) = rule.min_quantity.filter(|m| *m > Decimal::ZERO) else {
                    continue;
                };
                let product = covered_products.get(&pid);
                let sku = sid.and_then(|id| {
                    covered_skus
                        .get(&pid)
                        .and_then(|list| list.iter().find(|s| s.id == id))
                });
                let spec_text = sku
                    .and_then(|sk| format_sku_specs(sk.specs.as_ref()))
                    .or_else(|| product.and_then(|p| p.sku.clone()));
                // 无库存行视为 0 库存，0 < 最低阈值即缺货预警
                items.push(StockWarningVO {
                    id: None,
                    product_id: Some(pid),
                    product_name: product.and_then(|p| p.name.clone()),
                    product_code: product.and_then(|p| p.product_no.clone()),
                    warehouse_id: Some(wid),
                    warehouse_name: warehouse_map.get(&wid).and_then(|w| w.name.clone()),
                    sku_id: sid,
                    sku_code: sku.and_then(|sk| sk.sku_code.clone()),
                    spec_text,
                    quantity: Some(Decimal::ZERO),
                    available_quantity: Some(Decimal::ZERO),
                    alert_min_quantity: Some(min),
                    alert_max_quantity: None,
                    alert_type: Some("low_stock".to_string()),
                    last_inbound_time: None,
                    last_outbound_time: None,
                    obsolete_days: None,
                });
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
    sku_id: Option<i64>,
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
                        sku_id: Set(sku_id),
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
                sku_id,
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
    // 批量预加载产品/仓库/SKU，避免循环内逐条查询
    let product_ids: Vec<i64> = models.iter().filter_map(|s| s.product_id).collect();
    let warehouse_ids: Vec<i64> = models.iter().filter_map(|s| s.warehouse_id).collect();
    let sku_ids: Vec<i64> = models.iter().filter_map(|s| s.sku_id.filter(|&id| id > 0)).collect();
    let product_map: HashMap<i64, product_entity::Model> = if product_ids.is_empty() {
        HashMap::new()
    } else {
        product_entity::Entity::find()
            .filter(product_entity::Column::Id.is_in(product_ids))
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|p| (p.id, p))
            .collect()
    };
    let warehouse_map: HashMap<i64, warehouse_entity::Model> = if warehouse_ids.is_empty() {
        HashMap::new()
    } else {
        warehouse_entity::Entity::find()
            .filter(warehouse_entity::Column::Id.is_in(warehouse_ids))
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|w| (w.id, w))
            .collect()
    };
    let sku_map: HashMap<i64, sku_entity::Model> = if sku_ids.is_empty() {
        HashMap::new()
    } else {
        sku_entity::Entity::find()
            .filter(sku_entity::Column::Id.is_in(sku_ids))
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|s| (s.id, s))
            .collect()
    };

    let mut result: Vec<StockWarningVO> = Vec::with_capacity(models.len());
    for s in models {
        let product = product_map.get(&s.product_id.unwrap_or(0));
        let warehouse = warehouse_map.get(&s.warehouse_id.unwrap_or(0));
        let sid = s.sku_id.filter(|&id| id > 0);
        let sku = sid.and_then(|id| sku_map.get(&id));
        // 规格文本：多规格行取 SKU specs，单规格行回退产品规格描述
        let spec_text = sku.as_ref().and_then(|sk| format_sku_specs(sk.specs.as_ref()))
            .or_else(|| product.as_ref().and_then(|p| p.sku.clone()));

        result.push(StockWarningVO {
            id: Some(s.id),
            product_id: s.product_id,
            product_name: product.as_ref().and_then(|p| p.name.clone()),
            product_code: product.as_ref().and_then(|p| p.product_no.clone()),
            warehouse_id: s.warehouse_id,
            warehouse_name: warehouse.as_ref().and_then(|w| w.name.clone()),
            sku_id: sid,
            sku_code: sku.as_ref().and_then(|sk| sk.sku_code.clone()),
            spec_text,
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
