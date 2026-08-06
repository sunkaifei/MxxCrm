//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::purchase::entity::purchase_order_item::{self, Entity as PurchaseOrderItem};
use sea_orm::prelude::{Decimal, Date};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};

// ==================== DTO ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PoItemDTO {
    pub po_id: Option<i64>,
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

// ==================== VO ====================

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PoItemVO {
    pub id: Option<i64>,
    pub po_id: Option<i64>,
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

impl From<purchase_order_item::Model> for PoItemVO {
    fn from(model: purchase_order_item::Model) -> Self {
        PoItemVO {
            id: Some(model.id),
            po_id: model.po_id,
            pr_item_id: model.pr_item_id,
            product_id: model.product_id,
            product_name: model.product_name,
            product_sku: model.product_sku,
            spec: model.spec,
            unit: model.unit,
            quantity: model.quantity,
            received_quantity: model.received_quantity,
            unit_price: model.unit_price,
            amount: model.amount,
            tax_rate: model.tax_rate,
            tax_amount: model.tax_amount,
            expected_date: model.expected_date,
            remark: model.remark,
        }
    }
}

// ==================== Model ====================

pub struct PurchaseOrderItemModel;

impl PurchaseOrderItemModel {
    pub async fn batch_insert<C: ConnectionTrait>(
        db: &C,
        po_id: i64,
        items: &[PoItemDTO],
    ) -> Result<(), DbErr> {
        let now = chrono::Local::now().naive_local();
        for item in items {
            let active = purchase_order_item::ActiveModel {
                po_id: Set(Some(po_id)),
                pr_item_id: Set(item.pr_item_id),
                product_id: Set(item.product_id),
                product_name: Set(item.product_name.clone()),
                product_sku: Set(item.product_sku.clone()),
                spec: Set(item.spec.clone()),
                unit: Set(item.unit.clone()),
                quantity: Set(item.quantity),
                received_quantity: Set(item.received_quantity),
                unit_price: Set(item.unit_price),
                amount: Set(item.amount),
                tax_rate: Set(item.tax_rate),
                tax_amount: Set(item.tax_amount),
                expected_date: Set(item.expected_date),
                remark: Set(item.remark.clone()),
                deleted: Set(Some(0)),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            PurchaseOrderItem::insert(active).exec(db).await?;
        }
        Ok(())
    }

    pub async fn find_by_po_id<C: ConnectionTrait>(
        db: &C,
        po_id: i64,
    ) -> Result<Vec<purchase_order_item::Model>, DbErr> {
        PurchaseOrderItem::find()
            .filter(purchase_order_item::Column::PoId.eq(po_id))
            .filter(purchase_order_item::Column::Deleted.eq(0))
            .all(db)
            .await
    }

    pub async fn find_by_id<C: ConnectionTrait>(
        db: &C,
        id: i64,
    ) -> Result<Option<purchase_order_item::Model>, DbErr> {
        PurchaseOrderItem::find_by_id(id)
            .filter(purchase_order_item::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn delete_by_po_id<C: ConnectionTrait>(db: &C, po_id: i64) -> Result<u64, DbErr> {
        let result = PurchaseOrderItem::update_many()
            .set(purchase_order_item::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(purchase_order_item::Column::PoId.eq(po_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn update_received_quantity<C: ConnectionTrait>(
        db: &C,
        id: i64,
        quantity: Decimal,
    ) -> Result<(), DbErr> {
        PurchaseOrderItem::update_many()
            .set(purchase_order_item::ActiveModel {
                received_quantity: Set(Some(quantity)),
                ..Default::default()
            })
            .filter(purchase_order_item::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(())
    }
}