//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::purchase::entity::purchase_supplier_product::{self, Entity as PurchaseSupplierProduct};
use sea_orm::prelude::{DateTime, Decimal};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};

// ==================== DTO ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SupplierProductDTO {
    pub id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub product_id: Option<i64>,
    pub lead_time_days: Option<i32>,
    pub moq: Option<Decimal>,
    pub supplier_sku: Option<String>,
    pub agreement_price: Option<Decimal>,
    pub is_preferred: Option<i32>,
    pub remark: Option<String>,
}

// ==================== Model ====================

pub struct SupplierProductModel;

impl SupplierProductModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &SupplierProductDTO) -> std::result::Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = purchase_supplier_product::ActiveModel {
            supplier_id: Set(req.supplier_id),
            product_id: Set(req.product_id),
            lead_time_days: Set(req.lead_time_days),
            moq: Set(req.moq.clone()),
            supplier_sku: Set(req.supplier_sku.clone()),
            agreement_price: Set(req.agreement_price.clone()),
            is_preferred: Set(req.is_preferred),
            remark: Set(req.remark.clone()),
            deleted: Set(Some(0)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        PurchaseSupplierProduct::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: i64, req: &SupplierProductDTO) -> std::result::Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = purchase_supplier_product::ActiveModel {
            supplier_id: Set(req.supplier_id),
            product_id: Set(req.product_id),
            lead_time_days: Set(req.lead_time_days),
            moq: Set(req.moq.clone()),
            supplier_sku: Set(req.supplier_sku.clone()),
            agreement_price: Set(req.agreement_price.clone()),
            is_preferred: Set(req.is_preferred),
            remark: Set(req.remark.clone()),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        PurchaseSupplierProduct::update_many()
            .set(payload)
            .filter(purchase_supplier_product::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> std::result::Result<Option<purchase_supplier_product::Model>, DbErr> {
        PurchaseSupplierProduct::find_by_id(id)
            .filter(purchase_supplier_product::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn find_by_supplier_id(db: &DatabaseConnection, supplier_id: i64) -> std::result::Result<Vec<purchase_supplier_product::Model>, DbErr> {
        PurchaseSupplierProduct::find()
            .filter(purchase_supplier_product::Column::SupplierId.eq(supplier_id))
            .filter(purchase_supplier_product::Column::Deleted.eq(0))
            .all(db)
            .await
    }

    pub async fn find_by_product_id(db: &DatabaseConnection, product_id: i64) -> std::result::Result<Vec<purchase_supplier_product::Model>, DbErr> {
        PurchaseSupplierProduct::find()
            .filter(purchase_supplier_product::Column::ProductId.eq(product_id))
            .filter(purchase_supplier_product::Column::Deleted.eq(0))
            .all(db)
            .await
    }

    pub async fn find_all(db: &DatabaseConnection) -> std::result::Result<Vec<purchase_supplier_product::Model>, DbErr> {
        PurchaseSupplierProduct::find()
            .filter(purchase_supplier_product::Column::Deleted.eq(0))
            .all(db)
            .await
    }

    pub async fn batch_delete_by_ids(db: &DatabaseConnection, ids: &Vec<i64>) -> std::result::Result<i64, DbErr> {
        PurchaseSupplierProduct::update_many()
            .set(purchase_supplier_product::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(purchase_supplier_product::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}