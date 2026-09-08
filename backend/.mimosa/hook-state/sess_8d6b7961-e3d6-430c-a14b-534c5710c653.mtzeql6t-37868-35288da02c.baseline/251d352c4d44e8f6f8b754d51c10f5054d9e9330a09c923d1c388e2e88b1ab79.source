//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::purchase::entity::purchase_return::{self, Entity as PurchaseReturn};
use crate::modules::purchase::entity::purchase_return_item::{self, Entity as PurchaseReturnItem};
use sea_orm::prelude::{Decimal, DateTime};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};

// ==================== DTO ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReturnItemDTO {
    pub po_item_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub unit: Option<String>,
    pub return_quantity: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReturnSaveRequest {
    pub id: Option<i64>,
    pub receipt_id: Option<i64>,
    pub po_id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub return_date: Option<chrono::NaiveDate>,
    pub total_amount: Option<Decimal>,
    pub reason: Option<String>,
    pub remark: Option<String>,
    pub items: Vec<ReturnItemDTO>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReturnSaveDTO {
    pub id: Option<i64>,
    pub return_no: Option<String>,
    pub receipt_id: Option<i64>,
    pub po_id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub return_date: Option<chrono::NaiveDate>,
    pub total_amount: Option<Decimal>,
    pub reason: Option<String>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReturnListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub status: Option<i32>,
    pub supplier_id: Option<i64>,
    pub po_id: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReturnDetailVO {
    pub id: Option<i64>,
    pub return_no: Option<String>,
    pub receipt_id: Option<i64>,
    pub po_id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub return_date: Option<chrono::NaiveDate>,
    pub total_amount: Option<Decimal>,
    pub reason: Option<String>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub items: Vec<ReturnItemVO>,
}

impl From<purchase_return::Model> for PurchaseReturnDetailVO {
    fn from(model: purchase_return::Model) -> Self {
        PurchaseReturnDetailVO {
            id: Some(model.id),
            return_no: model.return_no,
            receipt_id: model.receipt_id,
            po_id: model.po_id,
            supplier_id: model.supplier_id,
            return_date: model.return_date,
            total_amount: model.total_amount,
            reason: model.reason,
            status: model.status,
            remark: model.remark,
            created_by: model.created_by,
            updated_by: model.updated_by,
            create_time: model.create_time,
            update_time: model.update_time,
            items: vec![],
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReturnItemVO {
    pub id: Option<i64>,
    pub return_id: Option<i64>,
    pub po_item_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub unit: Option<String>,
    pub return_quantity: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub reason: Option<String>,
}

impl From<purchase_return_item::Model> for ReturnItemVO {
    fn from(model: purchase_return_item::Model) -> Self {
        ReturnItemVO {
            id: Some(model.id),
            return_id: model.return_id,
            po_item_id: model.po_item_id,
            product_id: model.product_id,
            product_name: model.product_name,
            product_sku: model.product_sku,
            unit: model.unit,
            return_quantity: model.return_quantity,
            unit_price: model.unit_price,
            amount: model.amount,
            reason: model.reason,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReturnListVO {
    pub id: Option<i64>,
    pub return_no: Option<String>,
    pub po_id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub return_date: Option<chrono::NaiveDate>,
    pub total_amount: Option<Decimal>,
    pub status: Option<i32>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
}

impl From<purchase_return::Model> for PurchaseReturnListVO {
    fn from(model: purchase_return::Model) -> Self {
        PurchaseReturnListVO {
            id: Some(model.id),
            return_no: model.return_no,
            po_id: model.po_id,
            supplier_id: model.supplier_id,
            return_date: model.return_date,
            total_amount: model.total_amount,
            status: model.status,
            created_by: model.created_by,
            create_time: model.create_time,
        }
    }
}

// ==================== 数据库操作 ====================

pub struct PurchaseReturnModel;

impl PurchaseReturnModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &PurchaseReturnSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local();
        let payload = purchase_return::ActiveModel {
            return_no: Set(req.return_no.clone()),
            receipt_id: Set(req.receipt_id),
            po_id: Set(req.po_id),
            supplier_id: Set(req.supplier_id),
            return_date: Set(req.return_date),
            total_amount: Set(req.total_amount),
            reason: Set(req.reason.clone()),
            status: Set(req.status),
            remark: Set(req.remark.clone()),
            deleted: Set(Some(0)),
            created_by: Set(req.created_by),
            create_time: Set(Some(now)),
            updated_by: Set(req.updated_by),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        PurchaseReturn::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn update<C: ConnectionTrait>(db: &C, req: &PurchaseReturnSaveDTO) -> Result<(), DbErr> {
        let now = chrono::Local::now().naive_local();
        PurchaseReturn::update_many()
            .set(purchase_return::ActiveModel {
                receipt_id: Set(req.receipt_id),
                po_id: Set(req.po_id),
                supplier_id: Set(req.supplier_id),
                return_date: Set(req.return_date),
                total_amount: Set(req.total_amount),
                reason: Set(req.reason.clone()),
                remark: Set(req.remark.clone()),
                updated_by: Set(req.updated_by),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(purchase_return::Column::Id.eq(req.id.unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<purchase_return::Model>, DbErr> {
        PurchaseReturn::find_by_id(id)
            .filter(purchase_return::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn find_list<C: ConnectionTrait>(
        db: &C,
        query: &PurchaseReturnListQuery,
    ) -> Result<(Vec<purchase_return::Model>, u64), DbErr> {
        let page_num = query.page_num.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(10);

        let mut q = PurchaseReturn::find()
            .filter(purchase_return::Column::Deleted.eq(0));

        if let Some(ref kw) = query.keywords {
            q = q.filter(
                sea_orm::Condition::any()
                    .add(purchase_return::Column::ReturnNo.contains(kw)),
            );
        }
        if let Some(s) = query.status {
            q = q.filter(purchase_return::Column::Status.eq(s));
        }
        if let Some(s) = query.supplier_id {
            q = q.filter(purchase_return::Column::SupplierId.eq(s));
        }
        if let Some(p) = query.po_id {
            q = q.filter(purchase_return::Column::PoId.eq(p));
        }

        let paginator = q
            .order_by_desc(purchase_return::Column::CreateTime)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page((page_num - 1) as u64).await?;
        Ok((list, total))
    }

    pub async fn batch_delete<C: ConnectionTrait>(db: &C, ids: &[i64]) -> Result<u64, DbErr> {
        let result = PurchaseReturn::update_many()
            .set(purchase_return::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(purchase_return::Column::Id.is_in(ids.to_vec()))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }
}

pub struct PurchaseReturnItemModel;

impl PurchaseReturnItemModel {
    pub async fn batch_insert<C: ConnectionTrait>(
        db: &C,
        return_id: i64,
        items: &[ReturnItemDTO],
    ) -> Result<(), DbErr> {
        for item in items {
            let active = purchase_return_item::ActiveModel {
                return_id: Set(Some(return_id)),
                po_item_id: Set(item.po_item_id),
                product_id: Set(item.product_id),
                product_name: Set(item.product_name.clone()),
                product_sku: Set(item.product_sku.clone()),
                unit: Set(item.unit.clone()),
                return_quantity: Set(item.return_quantity),
                unit_price: Set(item.unit_price),
                amount: Set(item.amount),
                reason: Set(item.reason.clone()),
                deleted: Set(Some(0)),
                ..Default::default()
            };
            PurchaseReturnItem::insert(active).exec(db).await?;
        }
        Ok(())
    }

    pub async fn find_by_return_id<C: ConnectionTrait>(
        db: &C,
        return_id: i64,
    ) -> Result<Vec<purchase_return_item::Model>, DbErr> {
        PurchaseReturnItem::find()
            .filter(purchase_return_item::Column::ReturnId.eq(return_id))
            .filter(purchase_return_item::Column::Deleted.eq(0))
            .all(db)
            .await
    }

    pub async fn delete_by_return_id<C: ConnectionTrait>(db: &C, return_id: i64) -> Result<(), DbErr> {
        PurchaseReturnItem::update_many()
            .set(purchase_return_item::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(purchase_return_item::Column::ReturnId.eq(return_id))
            .exec(db)
            .await?;
        Ok(())
    }
}

/// 生成退货单号：TH{yyyyMMdd}{0001}
pub fn generate_return_no(seq: i32) -> String {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    format!("TH{}{:04}", today, seq)
}