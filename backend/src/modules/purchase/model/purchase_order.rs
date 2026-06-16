//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::purchase::entity::purchase_order::{self, Entity as PurchaseOrder};
use crate::utils::string_utils::serialize_option_u64_to_string;
use sea_orm::prelude::{Decimal, DateTime};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, UpdateResult,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderSaveRequest {
    pub order_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub order_type: Option<String>,
    pub status: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub tax_amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub payment_status: Option<String>,
    pub payment_method: Option<String>,
    pub paid_amount: Option<Decimal>,
    pub delivery_address: Option<String>,
    pub expected_delivery_date: Option<DateTime>,
    pub remark: Option<String>,
}

impl From<PurchaseOrderSaveRequest> for PurchaseOrderSaveDTO {
    fn from(req: PurchaseOrderSaveRequest) -> Self {
        PurchaseOrderSaveDTO {
            id: None,
            order_no: req.order_no,
            supplier_id: req.supplier_id,
            order_type: req.order_type,
            status: req.status,
            amount: req.amount,
            currency: req.currency,
            tax_amount: req.tax_amount,
            total_amount: req.total_amount,
            payment_status: req.payment_status,
            payment_method: req.payment_method,
            paid_amount: req.paid_amount,
            delivery_address: req.delivery_address,
            expected_delivery_date: req.expected_delivery_date,
            remark: req.remark,
            created_by: None,
            updated_by: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderUpdateRequest {
    pub id: Option<i64>,
    pub order_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub order_type: Option<String>,
    pub status: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub tax_amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub payment_status: Option<String>,
    pub payment_method: Option<String>,
    pub paid_amount: Option<Decimal>,
    pub delivery_address: Option<String>,
    pub expected_delivery_date: Option<DateTime>,
    pub remark: Option<String>,
}

impl From<PurchaseOrderUpdateRequest> for PurchaseOrderSaveDTO {
    fn from(req: PurchaseOrderUpdateRequest) -> Self {
        PurchaseOrderSaveDTO {
            id: req.id,
            order_no: req.order_no,
            supplier_id: req.supplier_id,
            order_type: req.order_type,
            status: req.status,
            amount: req.amount,
            currency: req.currency,
            tax_amount: req.tax_amount,
            total_amount: req.total_amount,
            payment_status: req.payment_status,
            payment_method: req.payment_method,
            paid_amount: req.paid_amount,
            delivery_address: req.delivery_address,
            expected_delivery_date: req.expected_delivery_date,
            remark: req.remark,
            created_by: None,
            updated_by: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderSaveDTO {
    pub id: Option<i64>,
    pub order_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub order_type: Option<String>,
    pub status: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub tax_amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub payment_status: Option<String>,
    pub payment_method: Option<String>,
    pub paid_amount: Option<Decimal>,
    pub delivery_address: Option<String>,
    pub expected_delivery_date: Option<DateTime>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderListQuery {
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub status: Option<String>,
    pub supplier_id: Option<i64>,
    pub order_type: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub order_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub order_type: Option<String>,
    pub status: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub tax_amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub payment_status: Option<String>,
    pub payment_method: Option<String>,
    pub paid_amount: Option<Decimal>,
    pub delivery_address: Option<String>,
    pub expected_delivery_date: Option<DateTime>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub updated_at: Option<DateTime>,
}

impl From<purchase_order::Model> for PurchaseOrderDetailVO {
    fn from(model: purchase_order::Model) -> Self {
        PurchaseOrderDetailVO {
            id: Some(model.id),
            order_no: model.order_no,
            supplier_id: model.supplier_id,
            order_type: model.order_type,
            status: model.status,
            amount: model.amount,
            currency: model.currency,
            tax_amount: model.tax_amount,
            total_amount: model.total_amount,
            payment_status: model.payment_status,
            payment_method: model.payment_method,
            paid_amount: model.paid_amount,
            delivery_address: model.delivery_address,
            expected_delivery_date: model.expected_delivery_date,
            remark: model.remark,
            created_by: model.created_by,
            created_at: model.created_at,
            updated_by: model.updated_by,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub order_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub order_type: Option<String>,
    pub status: Option<String>,
    pub amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub payment_status: Option<String>,
    pub delivery_address: Option<String>,
    pub expected_delivery_date: Option<DateTime>,
    pub created_at: Option<DateTime>,
}

impl From<purchase_order::Model> for PurchaseOrderListVO {
    fn from(model: purchase_order::Model) -> Self {
        PurchaseOrderListVO {
            id: Some(model.id),
            order_no: model.order_no,
            supplier_id: model.supplier_id,
            order_type: model.order_type,
            status: model.status,
            amount: model.amount,
            total_amount: model.total_amount,
            payment_status: model.payment_status,
            delivery_address: model.delivery_address,
            expected_delivery_date: model.expected_delivery_date,
            created_at: model.created_at,
        }
    }
}

pub struct PurchaseOrderModel;

impl PurchaseOrderModel {
    pub async fn insert(db: &DatabaseConnection, req: &PurchaseOrderSaveDTO) -> std::result::Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = purchase_order::ActiveModel {
            order_type: Set(req.order_type.clone()),
            supplier_id: Set(req.supplier_id.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            tax_amount: Set(req.tax_amount.clone()),
            total_amount: Set(req.total_amount.clone()),
            delivery_address: Set(req.delivery_address.clone()),
            expected_delivery_date: Set(req.expected_delivery_date.clone()),
            remark: Set(req.remark.clone()),
            created_by: Set(req.created_by.clone()),
            created_at: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(now)),
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

    pub async fn update_by_id(db: &DatabaseConnection, id: &Option<i64>, req: &PurchaseOrderSaveDTO) -> std::result::Result<i64, DbErr> {
        let payload = purchase_order::ActiveModel {
            order_type: Set(req.order_type.clone()),
            supplier_id: Set(req.supplier_id.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            tax_amount: Set(req.tax_amount.clone()),
            total_amount: Set(req.total_amount.clone()),
            delivery_address: Set(req.delivery_address.clone()),
            expected_delivery_date: Set(req.expected_delivery_date.clone()),
            remark: Set(req.remark.clone()),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
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
        order_type: Option<String>,
    ) -> std::result::Result<(Vec<purchase_order::Model>, i64), DbErr> {
        let mut query = PurchaseOrder::find()
            .filter(purchase_order::Column::Deleted.eq(0));
        
        if let Some(k) = keywords {
            query = query.filter(purchase_order::Column::OrderNo.contains(k));
        }
        if let Some(s) = status {
            query = query.filter(purchase_order::Column::Status.eq(s));
        }
        if let Some(s) = supplier_id {
            query = query.filter(purchase_order::Column::SupplierId.eq(s));
        }
        if let Some(t) = order_type {
            query = query.filter(purchase_order::Column::OrderType.eq(t));
        }
        
        let paginator = query.order_by_desc(purchase_order::Column::CreatedAt).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    pub async fn select_count(
        db: &DatabaseConnection,
        keywords: Option<String>,
        status: Option<String>,
        supplier_id: Option<i64>,
        order_type: Option<String>,
    ) -> std::result::Result<i64, DbErr> {
        let mut query = PurchaseOrder::find()
            .filter(purchase_order::Column::Deleted.eq(0));
        
        if let Some(k) = keywords {
            query = query.filter(purchase_order::Column::OrderNo.contains(k));
        }
        if let Some(s) = status {
            query = query.filter(purchase_order::Column::Status.eq(s));
        }
        if let Some(s) = supplier_id {
            query = query.filter(purchase_order::Column::SupplierId.eq(s));
        }
        if let Some(t) = order_type {
            query = query.filter(purchase_order::Column::OrderType.eq(t));
        }
        
        query.count(db).await.map(|c| c as i64)
    }
}
