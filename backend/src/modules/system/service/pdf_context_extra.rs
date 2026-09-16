//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! P1-1：发货单/出库单/入库单/采购单/收款单的真实数据上下文构建
//! （对齐 `build_quotation_context` 模式：单据头 + 明细行 + 公司/往来单位/银行）

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::{json, Value};

use crate::core::errors::error::{Error, Result};
use crate::modules::company::entity::company_account::{self as account_entity, Entity as CompanyAccount};
use crate::modules::company::entity::company_info::{self as company_entity, Entity as CompanyInfo};
use crate::modules::crm::entity::customer::{self as customer_entity, Entity as Customer};
use crate::modules::finance::entity::payment;
use crate::modules::inventory::entity::{inbound, inbound_item, warehouse, outbound, outbound_item};
use crate::modules::purchase::entity::{purchase_order, purchase_order_item, supplier};
use crate::modules::sale::entity::{shipment, shipment_item};
use crate::modules::system::service::pdf_generator_service::{amount_to_chinese, currency_name};
use rust_decimal::Decimal;

fn d2s<T: ToString>(v: Option<T>) -> String {
    v.map(|x| x.to_string()).unwrap_or_default()
}

fn date_s(d: Option<chrono::NaiveDate>) -> String {
    d.map(|v| v.format("%Y-%m-%d").to_string()).unwrap_or_default()
}

/// 公司 + 默认银行账户（与 build_quotation_context 同口径）
async fn company_and_bank(
    db: &DatabaseConnection,
) -> (Value, Value) {
    let company = CompanyInfo::find()
        .filter(company_entity::Column::Deleted.eq(0))
        .one(db)
        .await
        .ok()
        .flatten();
    let bank = CompanyAccount::find()
        .filter(account_entity::Column::IsDefault.eq(1))
        .filter(account_entity::Column::Deleted.eq(0))
        .one(db)
        .await
        .ok()
        .flatten();
    let company_json = company
        .as_ref()
        .map(|c| {
            json!({
                "company_name": c.company_name,
                "credit_code": c.credit_code,
                "legal_person": c.legal_person,
                "legal_phone": c.legal_phone,
                "register_address": c.register_address,
                "contact_phone": c.contact_phone,
                "contact_email": c.contact_email,
                "logo_url": c.logo_url,
            })
        })
        .unwrap_or(Value::Null);
    let bank_json = bank
        .as_ref()
        .map(|b| {
            json!({
                "bank_name": b.bank_name,
                "account_name": b.account_name,
                "account_number": b.account_number,
            })
        })
        .unwrap_or(Value::Null);
    (company_json, bank_json)
}

async fn customer_context(db: &DatabaseConnection, customer_id: i64) -> Value {
    if customer_id <= 0 {
        return Value::Null;
    }
    Customer::find_by_id(customer_id)
        .filter(customer_entity::Column::Deleted.eq(0))
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|c| {
            json!({
                "company_name": c.company_name,
                "person_name": c.person_name,
                "address": c.address,
                "personal_mobile": c.personal_mobile,
            })
        })
        .unwrap_or(Value::Null)
}

async fn warehouse_name(db: &DatabaseConnection, warehouse_id: Option<i64>) -> String {
    if let Some(wid) = warehouse_id {
        if wid > 0 {
            if let Some(w) = warehouse::Entity::find_by_id(wid)
                .filter(warehouse::Column::Deleted.eq(0))
                .one(db)
                .await
                .ok()
                .flatten()
            {
                return w.name.unwrap_or_default();
            }
        }
    }
    String::new()
}

/// 发货单（shipment）
pub async fn build_shipment_context(db: &DatabaseConnection, doc_id: i64) -> Result<Value> {
    let head = shipment::Entity::find_by_id(doc_id)
        .filter(shipment::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("发货单不存在"))?;
    let items = shipment_item::Entity::find()
        .filter(shipment_item::Column::ShipmentId.eq(doc_id))
        .all(db)
        .await?;
    let items_json: Vec<Value> = items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            json!({
                "index": i + 1,
                "product_name": it.product_name,
                "quantity": d2s(it.quantity),
            })
        })
        .collect();
    let (company_json, bank_json) = company_and_bank(db).await;
    let customer_json =
        customer_context(db, head.customer_id.unwrap_or_default()).await;
    let head_json = json!({
        "shipment_no": head.shipment_no,
        "shipment_date": date_s(head.shipment_date),
        "logistics_company": head.logistics_company,
        "tracking_no": head.tracking_no,
        "receiver_name": head.receiver_name,
        "receiver_phone": head.receiver_phone,
        "shipping_address": head.shipping_address,
        "total_qty": head.total_quantity,
        "remark": head.remark,
    });
    Ok(json!({
        "shipment": head_json,
        "items": items_json,
        "customer": customer_json,
        "company": company_json,
        "bank_account": bank_json,
    }))
}

/// 出库单（outbound）
pub async fn build_outbound_context(db: &DatabaseConnection, doc_id: i64) -> Result<Value> {
    let head = outbound::Entity::find_by_id(doc_id)
        .filter(outbound::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("出库单不存在"))?;
    let items = outbound_item::Entity::find()
        .filter(outbound_item::Column::OutboundId.eq(doc_id))
        .all(db)
        .await?;
    let items_json: Vec<Value> = items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            json!({
                "index": i + 1,
                "product_sku": it.product_sku,
                "batch_no": it.batch_no,
                "quantity": d2s(it.quantity.map(|q| q.to_string())),
                "remark": it.remark,
            })
        })
        .collect();
    let (company_json, bank_json) = company_and_bank(db).await;
    let warehouse_name = warehouse_name(db, head.warehouse_id).await;
    let head_json = json!({
        "outbound_no": head.outbound_no,
        "outbound_type": head.outbound_type,
        "source_order_no": head.source_order_no,
        "warehouse_id": head.warehouse_id,
        "warehouse_name": warehouse_name,
        "total_qty": head.total_quantity,
        "remark": head.remark,
    });
    Ok(json!({
        "outbound": head_json,
        "items": items_json,
        "company": company_json,
        "bank_account": bank_json,
    }))
}

/// 入库单（inbound）
pub async fn build_inbound_context(db: &DatabaseConnection, doc_id: i64) -> Result<Value> {
    let head = inbound::Entity::find_by_id(doc_id)
        .filter(inbound::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("入库单不存在"))?;
    let items = inbound_item::Entity::find()
        .filter(inbound_item::Column::InboundId.eq(doc_id))
        .all(db)
        .await?;
    let items_json: Vec<Value> = items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            json!({
                "index": i + 1,
                "product_sku": it.product_sku,
                "quantity": d2s(it.quantity.map(|q| q.to_string())),
                "unit_price": d2s(it.unit_price.map(|q| q.to_string())),
                "amount": d2s(it.amount.map(|q| q.to_string())),
            })
        })
        .collect();
    let (company_json, bank_json) = company_and_bank(db).await;
    let total_amount = head.total_amount.unwrap_or_default();
    let warehouse_name = warehouse_name(db, head.warehouse_id).await;
    let head_json = json!({
        "inbound_no": head.inbound_no,
        "inbound_type": head.inbound_type,
        "source_order_no": head.source_order_no,
        "warehouse_id": head.warehouse_id,
        "warehouse_name": warehouse_name,
        "total_quantity": d2s(head.total_quantity.map(|q| q.to_string())),
        "total_amount": total_amount.to_string(),
        "remark": head.remark,
    });
    Ok(json!({
        "inbound": head_json,
        "items": items_json,
        "company": company_json,
        "bank_account": bank_json,
        "total_amount_cn": amount_to_chinese(&total_amount),
    }))
}

/// 采购单（purchase）
pub async fn build_purchase_context(db: &DatabaseConnection, doc_id: i64) -> Result<Value> {
    let head = purchase_order::Entity::find_by_id(doc_id)
        .filter(purchase_order::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("采购单不存在"))?;
    let items = purchase_order_item::Entity::find()
        .filter(purchase_order_item::Column::PoId.eq(doc_id))
        .all(db)
        .await?;
    let items_json: Vec<Value> = items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            json!({
                "index": i + 1,
                "product_name": it.product_name,
                "product_sku": it.product_sku,
                "spec": it.spec,
                "unit": it.unit,
                "quantity": d2s(it.quantity.map(|q| q.to_string())),
            })
        })
        .collect();
    let (company_json, bank_json) = company_and_bank(db).await;
    let supplier_json = if let Some(sid) = head.supplier_id {
        supplier::Entity::find_by_id(sid)
            .filter(supplier::Column::Deleted.eq(0))
            .one(db)
            .await?
            .map(|sp| {
                json!({
                    "supplier_name": sp.company_name,
                    "short_name": sp.short_name,
                    "bank_name": sp.bank_name,
                })
            })
            .unwrap_or(Value::Null)
    } else {
        Value::Null
    };
    let amount = head.amount.unwrap_or_default();
    let head_json = json!({
        "purchase_no": head.purchase_no,
        "purchase_date": date_s(head.purchase_date),
        "expected_date": date_s(head.expected_date),
        "amount": amount.to_string(),
        "amount_cn": amount_to_chinese(&amount),
        "notes": head.notes,
    });
    Ok(json!({
        "purchase": head_json,
        "items": items_json,
        "supplier": supplier_json,
        "company": company_json,
        "bank_account": bank_json,
    }))
}

/// 收款单（payment）
pub async fn build_payment_context(db: &DatabaseConnection, doc_id: i64) -> Result<Value> {
    let head = payment::Entity::find_by_id(doc_id)
        .filter(payment::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("收款单不存在"))?;
    let (company_json, bank_json) = company_and_bank(db).await;
    let amount = head.payment_amount;
    let head_json = json!({
        "payment_no": head.payment_no,
        "purchase_order_no": head.purchase_order_no,
        "supplier_name": head.supplier_name,
        "payment_method": head.payment_method,
        "bank_account": head.bank_account,
        "payment_date": date_s(head.payment_date),
        "applicant_name": head.applicant_name,
        "amount": amount.to_string(),
        "amount_cn": amount_to_chinese(&amount),
        "remark": head.remark,
    });
    Ok(json!({
        "payment": head_json,
        "company": company_json,
        "bank_account": bank_json,
        "amount_cn": amount_to_chinese(&amount),
    }))
}

/// 出图上下文分发（供 pdf_generator_service::generate_pdf 调用）
pub async fn build_real_context(
    db: &DatabaseConnection,
    doc_type: &str,
    doc_id: i64,
) -> Result<Option<Value>> {
    match doc_type {
        "shipment" => Ok(Some(build_shipment_context(db, doc_id).await?)),
        "outbound" => Ok(Some(build_outbound_context(db, doc_id).await?)),
        "inbound" => Ok(Some(build_inbound_context(db, doc_id).await?)),
        "purchase" => Ok(Some(build_purchase_context(db, doc_id).await?)),
        "payment" => Ok(Some(build_payment_context(db, doc_id).await?)),
        _ => Ok(None),
    }
}

// 保持 currency_name 在本模块可见性（采购单币种后续扩展用）
#[allow(dead_code)]
fn _currency_ref(c: i32) -> String {
    currency_name(c)
}
