//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::purchase::entity::purchase_receipt::{self, Entity as PurchaseReceipt};
use crate::modules::purchase::entity::purchase_receipt_item::{self, Entity as PurchaseReceiptItem};
use sea_orm::prelude::{Decimal, DateTime};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};

// ==================== DTO ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptItemDTO {
    pub po_item_id: Option<i64>,
    pub product_id: Option<i64>,
    pub quantity: Option<Decimal>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptSaveRequest {
    pub id: Option<i64>,
    pub po_id: Option<i64>,
    pub po_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub warehouse_id: Option<i64>,
    pub total_quantity: Option<Decimal>,
    pub remark: Option<String>,
    pub items: Vec<ReceiptItemDTO>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptSaveDTO {
    pub id: Option<i64>,
    pub receipt_no: Option<String>,
    pub po_id: Option<i64>,
    pub po_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub warehouse_id: Option<i64>,
    pub status: Option<i32>,
    pub total_quantity: Option<Decimal>,
    pub remark: Option<String>,
    pub inbound_id: Option<i64>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub status: Option<i32>,
    pub po_id: Option<i64>,
    pub supplier_id: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptDetailVO {
    pub id: Option<i64>,
    pub receipt_no: Option<String>,
    pub po_id: Option<i64>,
    pub po_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub warehouse_id: Option<i64>,
    pub status: Option<i32>,
    pub total_quantity: Option<Decimal>,
    pub remark: Option<String>,
    pub inbound_id: Option<i64>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub items: Vec<ReceiptItemVO>,
}

impl From<purchase_receipt::Model> for ReceiptDetailVO {
    fn from(model: purchase_receipt::Model) -> Self {
        ReceiptDetailVO {
            id: Some(model.id),
            receipt_no: model.receipt_no,
            po_id: model.po_id,
            po_no: model.po_no,
            supplier_id: model.supplier_id,
            warehouse_id: model.warehouse_id,
            status: model.status,
            total_quantity: model.total_quantity,
            remark: model.remark,
            inbound_id: model.inbound_id,
            created_by: model.created_by,
            create_time: model.create_time,
            update_time: model.update_time,
            items: vec![],
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptItemVO {
    pub id: Option<i64>,
    pub receipt_id: Option<i64>,
    pub po_item_id: Option<i64>,
    pub product_id: Option<i64>,
    pub quantity: Option<Decimal>,
    pub remark: Option<String>,
}

impl From<purchase_receipt_item::Model> for ReceiptItemVO {
    fn from(model: purchase_receipt_item::Model) -> Self {
        ReceiptItemVO {
            id: Some(model.id),
            receipt_id: model.receipt_id,
            po_item_id: model.po_item_id,
            product_id: model.product_id,
            quantity: model.quantity,
            remark: model.remark,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptListVO {
    pub id: Option<i64>,
    pub receipt_no: Option<String>,
    pub po_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub status: Option<i32>,
    pub total_quantity: Option<Decimal>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
}

impl From<purchase_receipt::Model> for ReceiptListVO {
    fn from(model: purchase_receipt::Model) -> Self {
        ReceiptListVO {
            id: Some(model.id),
            receipt_no: model.receipt_no,
            po_no: model.po_no,
            supplier_id: model.supplier_id,
            status: model.status,
            total_quantity: model.total_quantity,
            created_by: model.created_by,
            create_time: model.create_time,
        }
    }
}

// ==================== 数据库操作 ====================

pub struct ReceiptModel;

impl ReceiptModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &ReceiptSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local();
        let payload = purchase_receipt::ActiveModel {
            receipt_no: Set(req.receipt_no.clone()),
            po_id: Set(req.po_id),
            po_no: Set(req.po_no.clone()),
            supplier_id: Set(req.supplier_id),
            warehouse_id: Set(req.warehouse_id),
            status: Set(req.status),
            total_quantity: Set(req.total_quantity),
            remark: Set(req.remark.clone()),
            inbound_id: Set(req.inbound_id),
            deleted: Set(Some(0)),
            created_by: Set(req.created_by),
            create_time: Set(Some(now)),
            updated_by: Set(req.updated_by),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        PurchaseReceipt::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<purchase_receipt::Model>, DbErr> {
        PurchaseReceipt::find_by_id(id)
            .filter(purchase_receipt::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn batch_delete<C: ConnectionTrait>(db: &C, ids: &[i64]) -> Result<u64, DbErr> {
        let result = PurchaseReceipt::update_many()
            .set(purchase_receipt::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(purchase_receipt::Column::Id.is_in(ids.to_vec()))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i32, operator: i64) -> Result<(), DbErr> {
        let now = chrono::Local::now().naive_local();
        PurchaseReceipt::update_many()
            .set(purchase_receipt::ActiveModel {
                status: Set(Some(status)),
                updated_by: Set(Some(operator)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(purchase_receipt::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn update_inbound_id<C: ConnectionTrait>(db: &C, id: i64, inbound_id: i64) -> Result<(), DbErr> {
        PurchaseReceipt::update_many()
            .set(purchase_receipt::ActiveModel {
                inbound_id: Set(Some(inbound_id)),
                update_time: Set(Some(chrono::Local::now().naive_local())),
                ..Default::default()
            })
            .filter(purchase_receipt::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn find_list<C: ConnectionTrait>(
        db: &C,
        query: &ReceiptListQuery,
    ) -> Result<(Vec<purchase_receipt::Model>, u64), DbErr> {
        let page_num = query.page_num.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(10);

        let mut q = PurchaseReceipt::find()
            .filter(purchase_receipt::Column::Deleted.eq(0));

        if let Some(ref kw) = query.keywords {
            q = q.filter(
                sea_orm::Condition::any()
                    .add(purchase_receipt::Column::ReceiptNo.contains(kw))
                    .add(purchase_receipt::Column::PoNo.contains(kw)),
            );
        }
        if let Some(s) = query.status {
            q = q.filter(purchase_receipt::Column::Status.eq(s));
        }
        if let Some(p) = query.po_id {
            q = q.filter(purchase_receipt::Column::PoId.eq(p));
        }
        if let Some(s) = query.supplier_id {
            q = q.filter(purchase_receipt::Column::SupplierId.eq(s));
        }

        let paginator = q
            .order_by_desc(purchase_receipt::Column::CreateTime)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page((page_num - 1) as u64).await?;
        Ok((list, total))
    }
}

pub struct ReceiptItemModel;

impl ReceiptItemModel {
    pub async fn batch_insert<C: ConnectionTrait>(
        db: &C,
        receipt_id: i64,
        items: &[ReceiptItemDTO],
    ) -> Result<(), DbErr> {
        let now = chrono::Local::now().naive_local();
        for item in items {
            let active = purchase_receipt_item::ActiveModel {
                receipt_id: Set(Some(receipt_id)),
                po_item_id: Set(item.po_item_id),
                product_id: Set(item.product_id),
                quantity: Set(item.quantity),
                remark: Set(item.remark.clone()),
                deleted: Set(Some(0)),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            PurchaseReceiptItem::insert(active).exec(db).await?;
        }
        Ok(())
    }

    pub async fn find_by_receipt_id<C: ConnectionTrait>(
        db: &C,
        receipt_id: i64,
    ) -> Result<Vec<purchase_receipt_item::Model>, DbErr> {
        PurchaseReceiptItem::find()
            .filter(purchase_receipt_item::Column::ReceiptId.eq(receipt_id))
            .filter(purchase_receipt_item::Column::Deleted.eq(0))
            .all(db)
            .await
    }

    pub async fn delete_by_receipt_id<C: ConnectionTrait>(db: &C, receipt_id: i64) -> Result<(), DbErr> {
        PurchaseReceiptItem::update_many()
            .set(purchase_receipt_item::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(purchase_receipt_item::Column::ReceiptId.eq(receipt_id))
            .exec(db)
            .await?;
        Ok(())
    }
}

/// 生成收货单号：SH{yyyyMMdd}{0001}
pub fn generate_receipt_no(seq: i32) -> String {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    format!("SH{}{:04}", today, seq)
}