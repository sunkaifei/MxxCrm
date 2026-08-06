//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::r#enum::currency_code_enum::CurrencyCode;
use crate::core::r#enum::purchase_status_enum::PurchaseStatus;
use crate::core::r#enum::payment_status_enum::PaymentStatus;
use crate::modules::product::entity::product::{self as product_entity, Entity as Product};
use crate::modules::purchase::entity::purchase_order::{self, Entity as PurchaseOrder};
use crate::modules::purchase::entity::purchase_order_item::{self, Entity as PurchaseOrderItem};
use crate::modules::purchase::model::purchase_order_item::PoItemDTO;
use crate::utils::string_utils::serialize_option_u64_to_string;
use sea_orm::prelude::{Decimal, DateTime, Date};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, UpdateResult,
};
use serde::{Deserialize, Serialize};

// ==================== DTO ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PoSaveItemDTO {
    pub pr_item_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub quantity: Option<Decimal>,
    pub received_quantity: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub expected_date: Option<Date>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderSaveRequest {
    pub purchase_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub purchase_date: Option<Date>,
    pub expected_date: Option<Date>,
    pub amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub notes: Option<String>,
    pub pr_id: Option<i64>,
    pub pr_no: Option<String>,
    pub department_id: Option<i64>,
    pub buyer_id: Option<i64>,
    pub total_quantity: Option<Decimal>,
    pub tax_total: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub freight_amount: Option<Decimal>,
    pub delivery_address: Option<String>,
    pub delivery_terms: Option<String>,
    pub payment_terms: Option<String>,
    pub items: Vec<PoSaveItemDTO>,
}

impl From<PurchaseOrderSaveRequest> for PurchaseOrderSaveDTO {
    fn from(req: PurchaseOrderSaveRequest) -> Self {
        let status = req.status.and_then(|s| serde_json::from_str::<PurchaseStatus>(&format!("\"{}\"", s)).ok());
        let payment_status = req.payment_status.and_then(|s| serde_json::from_str::<PaymentStatus>(&format!("\"{}\"", s)).ok());
        PurchaseOrderSaveDTO {
            id: None,
            purchase_no: req.purchase_no,
            supplier_id: req.supplier_id,
            purchase_date: req.purchase_date,
            expected_date: req.expected_date,
            amount: req.amount,
            currency: req.currency,
            status,
            payment_status,
            notes: req.notes,
            created_by: None,
            updated_by: None,
            pr_id: req.pr_id,
            pr_no: req.pr_no,
            department_id: req.department_id,
            buyer_id: req.buyer_id,
            total_quantity: req.total_quantity,
            tax_total: req.tax_total,
            discount_amount: req.discount_amount,
            freight_amount: req.freight_amount,
            delivery_address: req.delivery_address,
            delivery_terms: req.delivery_terms,
            payment_terms: req.payment_terms,
            items: req.items.into_iter().map(|i| PoItemDTO {
                po_id: None,
                pr_item_id: i.pr_item_id,
                product_id: i.product_id,
                product_name: i.product_name,
                product_sku: i.product_sku,
                spec: i.spec,
                unit: i.unit,
                quantity: i.quantity,
                received_quantity: i.received_quantity,
                unit_price: i.unit_price,
                amount: i.amount,
                tax_rate: i.tax_rate,
                tax_amount: i.tax_amount,
                expected_date: i.expected_date,
                remark: i.remark,
            }).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderUpdateRequest {
    pub id: Option<i64>,
    pub purchase_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub purchase_date: Option<Date>,
    pub expected_date: Option<Date>,
    pub amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub notes: Option<String>,
    pub pr_id: Option<i64>,
    pub pr_no: Option<String>,
    pub department_id: Option<i64>,
    pub buyer_id: Option<i64>,
    pub total_quantity: Option<Decimal>,
    pub tax_total: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub freight_amount: Option<Decimal>,
    pub delivery_address: Option<String>,
    pub delivery_terms: Option<String>,
    pub payment_terms: Option<String>,
    pub items: Vec<PoSaveItemDTO>,
}

impl From<PurchaseOrderUpdateRequest> for PurchaseOrderSaveDTO {
    fn from(req: PurchaseOrderUpdateRequest) -> Self {
        let status = req.status.and_then(|s| serde_json::from_str::<PurchaseStatus>(&format!("\"{}\"", s)).ok());
        let payment_status = req.payment_status.and_then(|s| serde_json::from_str::<PaymentStatus>(&format!("\"{}\"", s)).ok());
        PurchaseOrderSaveDTO {
            id: req.id,
            purchase_no: req.purchase_no,
            supplier_id: req.supplier_id,
            purchase_date: req.purchase_date,
            expected_date: req.expected_date,
            amount: req.amount,
            currency: req.currency,
            status,
            payment_status,
            notes: req.notes,
            created_by: None,
            updated_by: None,
            pr_id: req.pr_id,
            pr_no: req.pr_no,
            department_id: req.department_id,
            buyer_id: req.buyer_id,
            total_quantity: req.total_quantity,
            tax_total: req.tax_total,
            discount_amount: req.discount_amount,
            freight_amount: req.freight_amount,
            delivery_address: req.delivery_address,
            delivery_terms: req.delivery_terms,
            payment_terms: req.payment_terms,
            items: req.items.into_iter().map(|i| PoItemDTO {
                po_id: None,
                pr_item_id: i.pr_item_id,
                product_id: i.product_id,
                product_name: i.product_name,
                product_sku: i.product_sku,
                spec: i.spec,
                unit: i.unit,
                quantity: i.quantity,
                received_quantity: i.received_quantity,
                unit_price: i.unit_price,
                amount: i.amount,
                tax_rate: i.tax_rate,
                tax_amount: i.tax_amount,
                expected_date: i.expected_date,
                remark: i.remark,
            }).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderSaveDTO {
    pub id: Option<i64>,
    pub purchase_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub purchase_date: Option<Date>,
    pub expected_date: Option<Date>,
    pub amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub status: Option<PurchaseStatus>,
    pub payment_status: Option<PaymentStatus>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub pr_id: Option<i64>,
    pub pr_no: Option<String>,
    pub department_id: Option<i64>,
    pub buyer_id: Option<i64>,
    pub total_quantity: Option<Decimal>,
    pub tax_total: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub freight_amount: Option<Decimal>,
    pub delivery_address: Option<String>,
    pub delivery_terms: Option<String>,
    pub payment_terms: Option<String>,
    pub items: Vec<PoItemDTO>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub status: Option<String>,
    pub supplier_id: Option<i64>,
    pub brand_id: Option<i64>,
}

// ==================== VO ====================

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub purchase_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub purchase_date: Option<Date>,
    pub expected_date: Option<Date>,
    pub amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub update_time: Option<DateTime>,
    pub pr_id: Option<i64>,
    pub pr_no: Option<String>,
    pub department_id: Option<i64>,
    pub buyer_id: Option<i64>,
    pub total_quantity: Option<Decimal>,
    pub tax_total: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub freight_amount: Option<Decimal>,
    pub delivery_address: Option<String>,
    pub delivery_terms: Option<String>,
    pub payment_terms: Option<String>,
    pub audit_by: Option<i64>,
    pub audit_time: Option<DateTime>,
    pub items: Vec<crate::modules::purchase::model::purchase_order_item::PoItemVO>,
}

impl From<purchase_order::Model> for PurchaseOrderDetailVO {
    fn from(model: purchase_order::Model) -> Self {
        PurchaseOrderDetailVO {
            id: Some(model.id),
            purchase_no: model.purchase_no,
            supplier_id: model.supplier_id,
            purchase_date: model.purchase_date,
            expected_date: model.expected_date,
            amount: model.amount,
            currency: model.currency,
            status: model.status.map(|s| s.to_string()),
            payment_status: model.payment_status.map(|s| s.to_string()),
            notes: model.notes,
            created_by: model.created_by,
            create_time: model.create_time,
            updated_by: model.updated_by,
            update_time: model.update_time,
            pr_id: model.pr_id,
            pr_no: model.pr_no,
            department_id: model.department_id,
            buyer_id: model.buyer_id,
            total_quantity: model.total_quantity,
            tax_total: model.tax_total,
            discount_amount: model.discount_amount,
            freight_amount: model.freight_amount,
            delivery_address: model.delivery_address,
            delivery_terms: model.delivery_terms,
            payment_terms: model.payment_terms,
            audit_by: model.audit_by,
            audit_time: model.audit_time,
            items: vec![],
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub purchase_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub status: Option<String>,
    pub amount: Option<Decimal>,
    pub payment_status: Option<String>,
    pub create_time: Option<DateTime>,
}

impl From<purchase_order::Model> for PurchaseOrderListVO {
    fn from(model: purchase_order::Model) -> Self {
        PurchaseOrderListVO {
            id: Some(model.id),
            purchase_no: model.purchase_no,
            supplier_id: model.supplier_id,
            status: model.status.map(|s| s.to_string()),
            amount: model.amount,
            payment_status: model.payment_status.map(|s| s.to_string()),
            create_time: model.create_time,
        }
    }
}

// ==================== Model ====================

pub struct PurchaseOrderModel;

impl PurchaseOrderModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &PurchaseOrderSaveDTO) -> std::result::Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = purchase_order::ActiveModel {
            purchase_no: Set(req.purchase_no.clone()),
            supplier_id: Set(req.supplier_id.clone()),
            purchase_date: Set(req.purchase_date.clone()),
            expected_date: Set(req.expected_date.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            status: Set(req.status.clone()),
            payment_status: Set(req.payment_status.clone()),
            notes: Set(req.notes.clone()),
            created_by: Set(req.created_by.clone()),
            create_time: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(now)),
            pr_id: Set(req.pr_id),
            pr_no: Set(req.pr_no.clone()),
            department_id: Set(req.department_id),
            buyer_id: Set(req.buyer_id),
            total_quantity: Set(req.total_quantity),
            tax_total: Set(req.tax_total),
            discount_amount: Set(req.discount_amount),
            freight_amount: Set(req.freight_amount),
            delivery_address: Set(req.delivery_address.clone()),
            delivery_terms: Set(req.delivery_terms.clone()),
            payment_terms: Set(req.payment_terms.clone()),
            ..Default::default()
        };

        PurchaseOrder::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DatabaseConnection, ids: &Vec<i64>) -> std::result::Result<i64, DbErr> {
        PurchaseOrder::update_many()
            .set(purchase_order::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(purchase_order::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: &Option<i64>, req: &PurchaseOrderSaveDTO) -> std::result::Result<i64, DbErr> {
        let payload = purchase_order::ActiveModel {
            purchase_no: Set(req.purchase_no.clone()),
            supplier_id: Set(req.supplier_id.clone()),
            purchase_date: Set(req.purchase_date.clone()),
            expected_date: Set(req.expected_date.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            status: Set(req.status.clone()),
            payment_status: Set(req.payment_status.clone()),
            notes: Set(req.notes.clone()),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            pr_id: Set(req.pr_id),
            pr_no: Set(req.pr_no.clone()),
            department_id: Set(req.department_id),
            buyer_id: Set(req.buyer_id),
            total_quantity: Set(req.total_quantity),
            tax_total: Set(req.tax_total),
            discount_amount: Set(req.discount_amount),
            freight_amount: Set(req.freight_amount),
            delivery_address: Set(req.delivery_address.clone()),
            delivery_terms: Set(req.delivery_terms.clone()),
            payment_terms: Set(req.payment_terms.clone()),
            ..Default::default()
        };
        
        let update_result: UpdateResult = PurchaseOrder::update_many()
            .set(payload)
            .filter(purchase_order::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> std::result::Result<Option<purchase_order::Model>, DbErr> {
        PurchaseOrder::find_by_id(id)
            .filter(purchase_order::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn select_in_page(
        db: &DatabaseConnection,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        status: Option<String>,
        supplier_id: Option<i64>,
        brand_id: Option<i64>,
    ) -> std::result::Result<(Vec<purchase_order::Model>, i64), DbErr> {
        let mut query = PurchaseOrder::find()
            .filter(purchase_order::Column::Deleted.eq(0));
        
        if let Some(k) = keywords {
            query = query.filter(purchase_order::Column::PurchaseNo.contains(k));
        }
        if let Some(s) = status {
            if let Ok(status_enum) = serde_json::from_str::<PurchaseStatus>(&format!("\"{}\"", s)) {
                query = query.filter(purchase_order::Column::Status.eq(status_enum));
            }
        }
        if let Some(s) = supplier_id {
            query = query.filter(purchase_order::Column::SupplierId.eq(s));
        }
        if let Some(b) = brand_id {
            // 通过采购订单明细表关联产品表，按品牌筛选
            let product_ids: Vec<i64> = Product::find()
                .filter(product_entity::Column::BrandId.eq(b))
                .filter(product_entity::Column::Deleted.eq(0))
                .all(db)
                .await?
                .into_iter()
                .map(|p| p.id)
                .collect();
            
            let po_ids: Vec<i64> = PurchaseOrderItem::find()
                .filter(purchase_order_item::Column::ProductId.is_in(product_ids))
                .filter(purchase_order_item::Column::Deleted.eq(0))
                .all(db)
                .await?
                .into_iter()
                .filter_map(|item| item.po_id)
                .collect();
            
            query = query.filter(purchase_order::Column::Id.is_in(po_ids));
        }
        
        let paginator = query.order_by_desc(purchase_order::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    pub async fn select_count(
        db: &DatabaseConnection,
        keywords: Option<String>,
        status: Option<String>,
        supplier_id: Option<i64>,
        brand_id: Option<i64>,
    ) -> std::result::Result<i64, DbErr> {
        let mut query = PurchaseOrder::find()
            .filter(purchase_order::Column::Deleted.eq(0));
        
        if let Some(k) = keywords {
            query = query.filter(purchase_order::Column::PurchaseNo.contains(k));
        }
        if let Some(s) = status {
            if let Ok(status_enum) = serde_json::from_str::<PurchaseStatus>(&format!("\"{}\"", s)) {
                query = query.filter(purchase_order::Column::Status.eq(status_enum));
            }
        }
        if let Some(s) = supplier_id {
            query = query.filter(purchase_order::Column::SupplierId.eq(s));
        }
        if let Some(b) = brand_id {
            // 通过采购订单明细表关联产品表，按品牌筛选
            let product_ids: Vec<i64> = Product::find()
                .filter(product_entity::Column::BrandId.eq(b))
                .filter(product_entity::Column::Deleted.eq(0))
                .all(db)
                .await?
                .into_iter()
                .map(|p| p.id)
                .collect();
            
            let po_ids: Vec<i64> = PurchaseOrderItem::find()
                .filter(purchase_order_item::Column::ProductId.is_in(product_ids))
                .filter(purchase_order_item::Column::Deleted.eq(0))
                .all(db)
                .await?
                .into_iter()
                .filter_map(|item| item.po_id)
                .collect();
            
            query = query.filter(purchase_order::Column::Id.is_in(po_ids));
        }
        
        query.count(db).await.map(|c| c as i64)
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: &str, operator: i64) -> std::result::Result<(), DbErr> {
        let status_enum = serde_json::from_str::<PurchaseStatus>(&format!("\"{}\"", status))
            .unwrap_or(PurchaseStatus::Draft);
        let now = chrono::Local::now().naive_local();
        let mut active = purchase_order::ActiveModel {
            status: Set(Some(status_enum)),
            updated_by: Set(Some(operator)),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        // 如果是审核通过或驳回，记录审核信息
        if status == "audited" {
            active.audit_by = Set(Some(operator));
            active.audit_time = Set(Some(now));
        } else if status == "rejected" {
            active.audit_by = Set(Some(operator));
            active.audit_time = Set(Some(now));
        }

        PurchaseOrder::update_many()
            .set(active)
            .filter(purchase_order::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn find_by_ids<C: ConnectionTrait>(db: &C, ids: &[i64]) -> std::result::Result<Vec<purchase_order::Model>, DbErr> {
        PurchaseOrder::find()
            .filter(purchase_order::Column::Id.is_in(ids.to_vec()))
            .filter(purchase_order::Column::Deleted.eq(0))
            .all(db)
            .await
    }
}