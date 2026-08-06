//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::modules::inventory::entity::{stock, stock_log, warehouse};
use crate::modules::product::entity::product as product_entity;
use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 收发存报表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceiveSendReportQuery {
    pub warehouse_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// 收发存报表项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceiveSendStockVO {
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    /// 期初库存
    pub begin_quantity: Option<Decimal>,
    /// 本期入库
    pub inbound_quantity: Option<Decimal>,
    /// 本期出库
    pub outbound_quantity: Option<Decimal>,
    /// 期末库存
    pub end_quantity: Option<Decimal>,
}

/// 库存周转率报表项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TurnoverReportVO {
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    /// 期初库存
    pub begin_quantity: Option<Decimal>,
    /// 期末库存
    pub end_quantity: Option<Decimal>,
    /// 平均库存
    pub avg_quantity: Option<Decimal>,
    /// 出库数量
    pub outbound_quantity: Option<Decimal>,
    /// 周转率
    pub turnover_rate: Option<Decimal>,
}

/// 呆滞库存清单项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ObsoleteStockVO {
    pub id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    pub quantity: Option<Decimal>,
    pub avg_cost: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub last_inbound_time: Option<String>,
    pub last_outbound_time: Option<String>,
    pub obsolete_days: Option<i64>,
}

/// 库存成本报表项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CostReportVO {
    pub id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    pub quantity: Option<Decimal>,
    pub avg_cost: Option<Decimal>,
    pub last_in_cost: Option<Decimal>,
    pub total_cost: Option<Decimal>,
}

/// 解析日期字符串为 NaiveDateTime（失败返回 None）
fn parse_date(s: &Option<String>) -> Option<chrono::NaiveDateTime> {
    s.as_ref().and_then(|v| {
        chrono::NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d").map(|d| d.and_hms_opt(0, 0, 0).unwrap()))
            .ok()
    })
}

/// 收发存报表
pub async fn receive_send_stock_report(
    db: &DatabaseConnection,
    warehouse_id: Option<i64>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<ReceiveSendStockVO>> {
    let start = parse_date(&start_date);
    let end = parse_date(&end_date);

    // 1. 拉取相关库存记录（确定产品/仓库范围）
    let mut stock_q = stock::Entity::find().filter(stock::Column::Deleted.eq(0));
    if let Some(wid) = warehouse_id {
        stock_q = stock_q.filter(stock::Column::WarehouseId.eq(wid));
    }
    let stocks = stock_q.all(db).await?;

    // 2. 拉取区间内的流水
    let mut log_q = stock_log::Entity::find();
    if let Some(wid) = warehouse_id {
        log_q = log_q.filter(stock_log::Column::WarehouseId.eq(wid));
    }
    if let Some(s) = start {
        log_q = log_q.filter(stock_log::Column::CreateTime.gte(s));
    }
    if let Some(e) = end {
        log_q = log_q.filter(stock_log::Column::CreateTime.lte(e));
    }
    let logs = log_q.all(db).await?;

    // 3. 按 (product_id, warehouse_id) 聚合入库/出库
    let mut inbound_map: HashMap<(i64, i64), Decimal> = HashMap::new();
    let mut outbound_map: HashMap<(i64, i64), Decimal> = HashMap::new();
    for lg in &logs {
        let key = (lg.product_id.unwrap_or(0), lg.warehouse_id.unwrap_or(0));
        let change = lg.change_quantity.unwrap_or_default();
        if change >= Decimal::ZERO {
            *inbound_map.entry(key).or_insert_with(|| Decimal::ZERO) += change;
        } else {
            *outbound_map
                .entry(key)
                .or_insert_with(|| Decimal::ZERO) += change.abs();
        }
    }

    // 4. 期初 = 当前库存 - 本期入库 + 本期出库；期末 = 当前库存
    let mut result: Vec<ReceiveSendStockVO> = Vec::new();
    for s in stocks {
        let pid = s.product_id.unwrap_or(0);
        let wid = s.warehouse_id.unwrap_or(0);
        let end_qty = s.quantity.unwrap_or_default();
        let inbound = *inbound_map.get(&(pid, wid)).unwrap_or(&Decimal::ZERO);
        let outbound = *outbound_map.get(&(pid, wid)).unwrap_or(&Decimal::ZERO);
        let begin_qty = end_qty - inbound + outbound;

        let product = product_entity::Entity::find_by_id(pid).one(db).await.ok().flatten();
        let wh = warehouse::Entity::find_by_id(wid).one(db).await.ok().flatten();

        result.push(ReceiveSendStockVO {
            product_id: Some(pid),
            product_name: product.as_ref().and_then(|p| p.name.clone()),
            product_code: product.as_ref().and_then(|p| p.product_no.clone()),
            warehouse_id: Some(wid),
            warehouse_name: wh.as_ref().and_then(|w| w.name.clone()),
            begin_quantity: Some(begin_qty),
            inbound_quantity: Some(inbound),
            outbound_quantity: Some(outbound),
            end_quantity: Some(end_qty),
        });
    }
    Ok(result)
}

/// 库存周转率报表
pub async fn turnover_report(
    db: &DatabaseConnection,
    warehouse_id: Option<i64>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<TurnoverReportVO>> {
    let rs = receive_send_stock_report(db, warehouse_id, start_date, end_date).await?;
    let mut result: Vec<TurnoverReportVO> = Vec::with_capacity(rs.len());
    for item in rs {
        let begin = item.begin_quantity.unwrap_or_default();
        let end = item.end_quantity.unwrap_or_default();
        let outbound = item.outbound_quantity.unwrap_or_default();
        let avg_qty = (begin + end) / Decimal::from(2);
        let turnover_rate = if avg_qty > Decimal::ZERO {
            Some(outbound / avg_qty)
        } else {
            Some(Decimal::ZERO)
        };
        result.push(TurnoverReportVO {
            product_id: item.product_id,
            product_name: item.product_name,
            product_code: item.product_code,
            warehouse_id: item.warehouse_id,
            warehouse_name: item.warehouse_name,
            begin_quantity: Some(begin),
            end_quantity: Some(end),
            avg_quantity: Some(avg_qty),
            outbound_quantity: Some(outbound),
            turnover_rate,
        });
    }
    Ok(result)
}

/// 呆滞库存清单
pub async fn obsolete_stock_report(
    db: &DatabaseConnection,
    warehouse_id: Option<i64>,
    days: i32,
) -> Result<Vec<ObsoleteStockVO>> {
    let days = std::cmp::Ord::max(days, 1);
    let cutoff = chrono::Local::now().naive_local() - chrono::Duration::days(days as i64);

    let mut q = stock::Entity::find()
        .filter(stock::Column::Deleted.eq(0))
        .filter(stock::Column::Quantity.gt(Decimal::ZERO))
        .filter(
            stock::Column::LastOutboundTime
                .lt(cutoff)
                .or(stock::Column::LastOutboundTime.is_null()),
        );
    if let Some(wid) = warehouse_id {
        q = q.filter(stock::Column::WarehouseId.eq(wid));
    }
    let stocks = q
        .order_by_desc(stock::Column::LastInboundTime)
        .all(db)
        .await?;

    let now = chrono::Local::now().naive_local();
    let mut result: Vec<ObsoleteStockVO> = Vec::with_capacity(stocks.len());
    for s in stocks {
        let pid = s.product_id.unwrap_or(0);
        let wid = s.warehouse_id.unwrap_or(0);
        let product = product_entity::Entity::find_by_id(pid).one(db).await.ok().flatten();
        let wh = warehouse::Entity::find_by_id(wid).one(db).await.ok().flatten();

        let obsolete_days = s.last_outbound_time.map(|t| (now - t).num_days())
            .or_else(|| s.last_inbound_time.map(|t| (now - t).num_days()))
            .or(Some(days as i64));

        result.push(ObsoleteStockVO {
            id: Some(s.id),
            product_id: Some(pid),
            product_name: product.as_ref().and_then(|p| p.name.clone()),
            product_code: product.as_ref().and_then(|p| p.product_no.clone()),
            warehouse_id: Some(wid),
            warehouse_name: wh.as_ref().and_then(|w| w.name.clone()),
            quantity: s.quantity,
            avg_cost: s.avg_cost,
            total_cost: s.total_cost,
            last_inbound_time: s.last_inbound_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            last_outbound_time: s.last_outbound_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            obsolete_days,
        });
    }
    Ok(result)
}

/// 入库汇总报表
pub async fn inbound_summary_report(
    db: &DatabaseConnection,
    warehouse_id: Option<i64>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<ReceiveSendStockVO>> {
    use crate::modules::inventory::entity::inbound;
    use sea_orm::QueryFilter;

    let mut q = inbound::Entity::find()
        .filter(inbound::Column::Deleted.eq(0))
        .filter(inbound::Column::Status.eq(3)); // 已完成

    if let Some(wid) = warehouse_id {
        q = q.filter(inbound::Column::WarehouseId.eq(wid));
    }
    if let Some(dt) = parse_date(&start_date) {
        q = q.filter(inbound::Column::CreateTime.gte(dt));
    }
    if let Some(dt) = parse_date(&end_date) {
        q = q.filter(inbound::Column::CreateTime.lte(dt));
    }

    let list = q.all(db).await?;
    let mut result = Vec::new();
    for item in list {
        let wh = warehouse::Entity::find_by_id(item.warehouse_id.unwrap_or(0)).one(db).await.ok().flatten();
        result.push(ReceiveSendStockVO {
            product_id: None,
            product_name: None,
            product_code: None,
            warehouse_id: item.warehouse_id,
            warehouse_name: wh.as_ref().and_then(|w| w.name.clone()),
            begin_quantity: None,
            inbound_quantity: item.total_quantity,
            outbound_quantity: None,
            end_quantity: None,
        });
    }
    Ok(result)
}

/// 出库汇总报表
pub async fn outbound_summary_report(
    db: &DatabaseConnection,
    warehouse_id: Option<i64>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<ReceiveSendStockVO>> {
    use crate::modules::inventory::entity::outbound;
    use sea_orm::QueryFilter;

    let mut q = outbound::Entity::find()
        .filter(outbound::Column::Deleted.eq(0))
        .filter(outbound::Column::Status.eq(3)); // 已完成

    if let Some(wid) = warehouse_id {
        q = q.filter(outbound::Column::WarehouseId.eq(wid));
    }
    if let Some(dt) = parse_date(&start_date) {
        q = q.filter(outbound::Column::CreateTime.gte(dt));
    }
    if let Some(dt) = parse_date(&end_date) {
        q = q.filter(outbound::Column::CreateTime.lte(dt));
    }

    let list = q.all(db).await?;
    let mut result = Vec::new();
    for item in list {
        let wh = warehouse::Entity::find_by_id(item.warehouse_id.unwrap_or(0)).one(db).await.ok().flatten();
        result.push(ReceiveSendStockVO {
            product_id: None,
            product_name: None,
            product_code: None,
            warehouse_id: item.warehouse_id,
            warehouse_name: wh.as_ref().and_then(|w| w.name.clone()),
            begin_quantity: None,
            inbound_quantity: None,
            outbound_quantity: item.total_quantity,
            end_quantity: None,
        });
    }
    Ok(result)
}

/// 库存成本报表
pub async fn cost_report(db: &DatabaseConnection, warehouse_id: Option<i64>) -> Result<Vec<CostReportVO>> {
    let mut q = stock::Entity::find().filter(stock::Column::Deleted.eq(0));
    if let Some(wid) = warehouse_id {
        q = q.filter(stock::Column::WarehouseId.eq(wid));
    }
    let stocks = q
        .order_by_desc(stock::Column::TotalCost)
        .all(db)
        .await?;

    let mut result: Vec<CostReportVO> = Vec::with_capacity(stocks.len());
    for s in stocks {
        let pid = s.product_id.unwrap_or(0);
        let wid = s.warehouse_id.unwrap_or(0);
        let product = product_entity::Entity::find_by_id(pid).one(db).await.ok().flatten();
        let wh = warehouse::Entity::find_by_id(wid).one(db).await.ok().flatten();

        result.push(CostReportVO {
            id: Some(s.id),
            product_id: Some(pid),
            product_name: product.as_ref().and_then(|p| p.name.clone()),
            product_code: product.as_ref().and_then(|p| p.product_no.clone()),
            warehouse_id: Some(wid),
            warehouse_name: wh.as_ref().and_then(|w| w.name.clone()),
            quantity: s.quantity,
            avg_cost: s.avg_cost,
            last_in_cost: s.last_in_cost,
            total_cost: s.total_cost,
        });
    }
    Ok(result)
}
