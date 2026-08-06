//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 低库存自动采购建议服务
//!

use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};
use std::collections::HashMap;

use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::entity::{stock, warehouse};
use crate::modules::product::entity::product as product_entity;
use crate::modules::purchase::model::purchase_requisition::{
    RequisitionItemDTO, RequisitionSaveRequest,
};

/// 低库存建议明细项
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionItem {
    pub product_id: i64,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub unit: Option<String>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    pub available_quantity: Decimal,
    pub alert_min_quantity: Decimal,
    /// 建议采购数量（警戒线 - 可用 + 警戒线，即补到警戒线的两倍）
    pub suggest_quantity: Decimal,
}

/// 低库存建议结果
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionResult {
    pub items: Vec<SuggestionItem>,
    pub total: u64,
    /// 生成的采购申请单ID（若执行了生成）
    pub requisition_id: Option<i64>,
}

/// 扫描低库存产品（可用库存 < 最低警戒线）
pub async fn scan_low_stock(db: &DbConn) -> Result<Vec<SuggestionItem>> {
    // 查询所有未删除且设置了最低警戒线的库存记录
    let stocks = stock::Entity::find()
        .filter(stock::Column::Deleted.eq(0))
        .filter(stock::Column::AlertMinQuantity.gt(Decimal::ZERO))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 筛选可用库存低于警戒线的记录
    let low_stock_list: Vec<&stock::Model> = stocks
        .iter()
        .filter(|s| {
            let available = s.available_quantity.unwrap_or_default();
            let alert_min = s.alert_min_quantity.unwrap_or_default();
            available < alert_min
        })
        .collect();

    if low_stock_list.is_empty() {
        return Ok(Vec::new());
    }

    // 批量查询产品信息
    let product_ids: Vec<i64> = low_stock_list
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
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
    };
    let product_map: HashMap<i64, product_entity::Model> =
        products.into_iter().map(|p| (p.id, p)).collect();

    // 批量查询仓库名称
    let warehouse_ids: Vec<i64> = low_stock_list
        .iter()
        .filter_map(|s| s.warehouse_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let warehouses = if warehouse_ids.is_empty() {
        Vec::new()
    } else {
        warehouse::Entity::find()
            .filter(warehouse::Column::Id.is_in(warehouse_ids))
            .filter(warehouse::Column::Deleted.eq(0))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
    };
    let warehouse_map: HashMap<i64, warehouse::Model> =
        warehouses.into_iter().map(|w| (w.id, w)).collect();

    // 构建建议列表
    let mut items: Vec<SuggestionItem> = Vec::new();
    for s in low_stock_list {
        let product_id = s.product_id.unwrap_or_default();
        let product_info = product_map.get(&product_id);
        let available = s.available_quantity.unwrap_or_default();
        let alert_min = s.alert_min_quantity.unwrap_or_default();
        // 建议采购数量 = 警戒线 - 可用 + 警戒线（补到警戒线的两倍，留出缓冲）
        let suggest = if available < alert_min {
            (alert_min - available) + alert_min
        } else {
            Decimal::ZERO
        };

        items.push(SuggestionItem {
            product_id,
            product_name: product_info.and_then(|p| p.name.clone()),
            product_sku: product_info.and_then(|p| p.sku.clone()),
            unit: product_info.and_then(|p| p.unit.clone()),
            warehouse_id: s.warehouse_id,
            warehouse_name: s
                .warehouse_id
                .and_then(|wid| warehouse_map.get(&wid).and_then(|w| w.name.clone())),
            available_quantity: available,
            alert_min_quantity: alert_min,
            suggest_quantity: suggest,
        });
    }

    Ok(items)
}

/// 生成低库存采购建议（仅返回建议清单，不创建采购申请单）
pub async fn get_suggestions(db: &DbConn) -> Result<SuggestionResult> {
    let items = scan_low_stock(db).await?;
    let total = items.len() as u64;
    Ok(SuggestionResult {
        items,
        total,
        requisition_id: None,
    })
}

/// 自动生成采购申请单（基于低库存建议）
/// 扫描低库存产品，生成一张采购申请单
pub async fn generate_requisition(
    db: &DbConn,
    operator_id: i64,
) -> Result<SuggestionResult> {
    let items = scan_low_stock(db).await?;
    if items.is_empty() {
        return Ok(SuggestionResult {
            items: Vec::new(),
            total: 0,
            requisition_id: None,
        });
    }

    // 构建采购申请单请求
    let pr_items: Vec<RequisitionItemDTO> = items
        .iter()
        .map(|it| RequisitionItemDTO {
            product_id: Some(it.product_id),
            product_name: it.product_name.clone(),
            product_sku: it.product_sku.clone(),
            spec: None,
            unit: it.unit.clone(),
            quantity: Some(it.suggest_quantity),
            estimated_price: None,
            estimated_amount: None,
            remark: Some(format!(
                "低库存自动建议：可用 {} 低于警戒线 {}",
                it.available_quantity, it.alert_min_quantity
            )),
        })
        .collect();

    let today = chrono::Local::now().naive_local().date();
    let req = RequisitionSaveRequest {
        id: None,
        pr_type: Some("auto_low_stock".to_string()),
        title: Some(format!(
            "低库存自动采购建议-{}",
            chrono::Local::now().format("%Y%m%d")
        )),
        department_id: None,
        requester_id: Some(operator_id),
        expected_date: Some(today),
        urgency: Some("normal".to_string()),
        total_amount: None,
        currency: None,
        reason: Some("系统自动检测到部分产品库存低于警戒线，自动生成采购建议".to_string()),
        remark: Some("由库存系统自动生成".to_string()),
        items: pr_items,
    };

    let requisition_id =
        crate::modules::purchase::service::purchase_requisition_service::insert(db, &req, operator_id)
            .await?;

    let total = items.len() as u64;
    Ok(SuggestionResult {
        items,
        total,
        requisition_id: Some(requisition_id),
    })
}
