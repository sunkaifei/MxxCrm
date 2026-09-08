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
use crate::modules::product::entity::brand::{self, Entity as Brand};
use crate::modules::product::entity::product::{self, Entity as Product};
use crate::modules::purchase::entity::purchase_order::{self, Entity as PurchaseOrder};
use crate::modules::purchase::entity::purchase_order_item::{self, Entity as PurchaseOrderItem};
use sea_orm::prelude::Decimal;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, QuerySelect};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReportSummaryVO {
    pub total_po_count: i64,
    pub total_amount: Decimal,
    pub total_tax: Decimal,
    pub total_discount: Decimal,
    pub total_freight: Decimal,
    pub draft_count: i64,
    pub pending_audit_count: i64,
    pub audited_count: i64,
    pub completed_count: i64,
    pub cancelled_count: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SupplierReportVO {
    pub supplier_id: i64,
    pub po_count: i64,
    pub total_amount: Decimal,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductReportVO {
    pub product_id: i64,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub total_quantity: Decimal,
    pub total_amount: Decimal,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepartmentReportVO {
    pub department_id: i64,
    pub po_count: i64,
    pub total_amount: Decimal,
}

/// 采购报表汇总
pub async fn summary(db: &DbConn, _start_date: Option<String>, _end_date: Option<String>) -> Result<ReportSummaryVO> {
    let all = PurchaseOrder::find()
        .filter(purchase_order::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let total_po_count = all.len() as i64;
    let mut total_amount = Decimal::new(0, 0);
    let mut total_tax = Decimal::new(0, 0);
    let mut total_discount = Decimal::new(0, 0);
    let mut total_freight = Decimal::new(0, 0);
    let mut draft_count = 0i64;
    let mut pending_audit_count = 0i64;
    let mut audited_count = 0i64;
    let mut completed_count = 0i64;
    let mut cancelled_count = 0i64;

    for po in &all {
        if let Some(amt) = po.amount {
            total_amount += amt;
        }
        if let Some(tax) = po.tax_total {
            total_tax += tax;
        }
        if let Some(disc) = po.discount_amount {
            total_discount += disc;
        }
        if let Some(freight) = po.freight_amount {
            total_freight += freight;
        }

        match po.status {
            Some(crate::core::r#enum::purchase_status_enum::PurchaseStatus::Draft) => draft_count += 1,
            Some(crate::core::r#enum::purchase_status_enum::PurchaseStatus::PendingAudit) => pending_audit_count += 1,
            Some(crate::core::r#enum::purchase_status_enum::PurchaseStatus::Audited) => audited_count += 1,
            Some(crate::core::r#enum::purchase_status_enum::PurchaseStatus::Completed) => completed_count += 1,
            Some(crate::core::r#enum::purchase_status_enum::PurchaseStatus::Cancelled) => cancelled_count += 1,
            _ => {}
        }
    }

    Ok(ReportSummaryVO {
        total_po_count,
        total_amount,
        total_tax,
        total_discount,
        total_freight,
        draft_count,
        pending_audit_count,
        audited_count,
        completed_count,
        cancelled_count,
    })
}

/// 按供应商统计
pub async fn by_supplier(db: &DbConn, _start_date: Option<String>, _end_date: Option<String>) -> Result<Vec<SupplierReportVO>> {
    let all = PurchaseOrder::find()
        .filter(purchase_order::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut map: std::collections::HashMap<i64, (i64, Decimal)> = std::collections::HashMap::new();
    for po in &all {
        if let Some(supplier_id) = po.supplier_id {
            let entry = map.entry(supplier_id).or_insert((0, Decimal::new(0, 0)));
            entry.0 += 1;
            if let Some(amt) = po.amount {
                entry.1 += amt;
            }
        }
    }

    let result = map.into_iter().map(|(supplier_id, (count, amount))| SupplierReportVO {
        supplier_id,
        po_count: count,
        total_amount: amount,
    }).collect();

    Ok(result)
}

/// 按产品统计
pub async fn by_product(db: &DbConn, _start_date: Option<String>, _end_date: Option<String>) -> Result<Vec<ProductReportVO>> {
    let items = PurchaseOrderItem::find()
        .filter(purchase_order_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut map: std::collections::HashMap<i64, (Decimal, Decimal, Option<String>, Option<String>)> = std::collections::HashMap::new();
    for item in &items {
        if let Some(product_id) = item.product_id {
            let entry = map.entry(product_id).or_insert((
                Decimal::new(0, 0),
                Decimal::new(0, 0),
                item.product_name.clone(),
                item.product_sku.clone(),
            ));
            if let Some(qty) = item.quantity {
                entry.0 += qty;
            }
            if let Some(amt) = item.amount {
                entry.1 += amt;
            }
            if entry.2.is_none() {
                entry.2 = item.product_name.clone();
            }
            if entry.3.is_none() {
                entry.3 = item.product_sku.clone();
            }
        }
    }

    let result = map.into_iter().map(|(product_id, (total_qty, total_amt, name, sku))| ProductReportVO {
        product_id,
        product_name: name,
        product_sku: sku,
        total_quantity: total_qty,
        total_amount: total_amt,
    }).collect();

    Ok(result)
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrandReportVO {
    pub brand_id: i64,
    pub brand_name: Option<String>,
    pub item_count: i64,
    pub total_amount: Decimal,
}

/// 按品牌统计
pub async fn by_brand(db: &DbConn, _start_date: Option<String>, _end_date: Option<String>) -> Result<Vec<BrandReportVO>> {
    // 获取所有采购订单明细
    let items = PurchaseOrderItem::find()
        .filter(purchase_order_item::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 获取所有产品的 brand_id
    let product_ids: Vec<i64> = items.iter()
        .filter_map(|item| item.product_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let products = Product::find()
        .filter(product::Column::Id.is_in(product_ids))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut product_brand_map: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    for p in &products {
        if let Some(brand_id) = p.brand_id {
            product_brand_map.insert(p.id, brand_id);
        }
    }

    // 获取品牌名称
    let brand_ids: Vec<i64> = product_brand_map.values().cloned().collect::<std::collections::HashSet<_>>().into_iter().collect();
    let brands = Brand::find()
        .filter(brand::Column::Id.is_in(brand_ids))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut brand_name_map: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    for b in &brands {
        if let Some(name) = &b.name {
            brand_name_map.insert(b.id, name.clone());
        }
    }

    // 按品牌汇总
    let mut map: std::collections::HashMap<i64, (i64, Decimal)> = std::collections::HashMap::new();
    for item in &items {
        if let Some(product_id) = item.product_id {
            if let Some(brand_id) = product_brand_map.get(&product_id) {
                let entry = map.entry(*brand_id).or_insert((0, Decimal::new(0, 0)));
                entry.0 += 1;
                if let Some(amt) = item.amount {
                    entry.1 += amt;
                }
            }
        }
    }

    let result = map.into_iter().map(|(brand_id, (count, amount))| BrandReportVO {
        brand_id,
        brand_name: brand_name_map.get(&brand_id).cloned(),
        item_count: count,
        total_amount: amount,
    }).collect();

    Ok(result)
}

/// 按部门统计
pub async fn by_department(db: &DbConn, _start_date: Option<String>, _end_date: Option<String>) -> Result<Vec<DepartmentReportVO>> {
    let all = PurchaseOrder::find()
        .filter(purchase_order::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut map: std::collections::HashMap<i64, (i64, Decimal)> = std::collections::HashMap::new();
    for po in &all {
        if let Some(dept_id) = po.department_id {
            let entry = map.entry(dept_id).or_insert((0, Decimal::new(0, 0)));
            entry.0 += 1;
            if let Some(amt) = po.amount {
                entry.1 += amt;
            }
        }
    }

    let result = map.into_iter().map(|(dept_id, (count, amount))| DepartmentReportVO {
        department_id: dept_id,
        po_count: count,
        total_amount: amount,
    }).collect();

    Ok(result)
}