//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal, Date};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use crate::modules::sale::entity::{invoice, invoice::Entity as Invoice};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 发票新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct InvoiceSaveRequest {
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub invoice_type: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub status: Option<String>,
    pub issued_at: Option<Date>,
    pub due_date: Option<Date>,
    pub notes: Option<String>,
}

impl From<InvoiceSaveRequest> for InvoiceSaveDTO {
    fn from(item: InvoiceSaveRequest) -> Self {
        InvoiceSaveDTO {
            id: None,
            invoice_no: None,
            order_id: item.order_id,
            customer_id: item.customer_id,
            invoice_type: item.invoice_type,
            amount: item.amount,
            currency: item.currency,
            status: item.status,
            issued_at: item.issued_at,
            due_date: item.due_date,
            notes: item.notes,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
        }
    }
}

/// 发票更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct InvoiceUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub invoice_type: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub status: Option<String>,
    pub issued_at: Option<Date>,
    pub due_date: Option<Date>,
    pub notes: Option<String>,
}

impl From<InvoiceUpdateRequest> for InvoiceSaveDTO {
    fn from(item: InvoiceUpdateRequest) -> Self {
        InvoiceSaveDTO {
            id: item.id,
            invoice_no: None,
            order_id: item.order_id,
            customer_id: item.customer_id,
            invoice_type: item.invoice_type,
            amount: item.amount,
            currency: item.currency,
            status: item.status,
            issued_at: item.issued_at,
            due_date: item.due_date,
            notes: item.notes,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
        }
    }
}

/// 发票保存DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct InvoiceSaveDTO {
    pub id: Option<i64>,
    pub invoice_no: Option<String>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub invoice_type: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub status: Option<String>,
    pub issued_at: Option<Date>,
    pub due_date: Option<Date>,
    pub notes: Option<String>,
    pub deleted: Option<i32>,
    pub created_by: Option<i64>,
    pub created_at: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub updated_at: Option<DateTime>,
}

/// 发票详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct InvoiceDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub invoice_no: Option<String>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub invoice_type: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub status: Option<String>,
    pub issued_at: Option<Date>,
    pub due_date: Option<Date>,
    pub notes: Option<String>,
    pub created_at: Option<DateTime>,
}

impl From<invoice::Model> for InvoiceDetailVO {
    fn from(item: invoice::Model) -> Self {
        InvoiceDetailVO {
            id: Some(item.id),
            invoice_no: item.invoice_no,
            order_id: item.order_id,
            customer_id: item.customer_id,
            invoice_type: item.invoice_type,
            amount: item.amount,
            currency: item.currency,
            status: item.status,
            issued_at: item.issued_at,
            due_date: item.due_date,
            notes: item.notes,
            created_at: item.created_at,
        }
    }
}

/// 发票列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct InvoiceListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub invoice_no: Option<String>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub invoice_type: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub status: Option<String>,
    pub issued_at: Option<Date>,
    pub due_date: Option<Date>,
}

impl From<invoice::Model> for InvoiceListVO {
    fn from(item: invoice::Model) -> Self {
        InvoiceListVO {
            id: Some(item.id),
            invoice_no: item.invoice_no,
            order_id: item.order_id,
            customer_id: item.customer_id,
            invoice_type: item.invoice_type,
            amount: item.amount,
            currency: item.currency,
            status: item.status,
            issued_at: item.issued_at,
            due_date: item.due_date,
        }
    }
}

/// 发票列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub invoice_type: Option<String>,
    pub status: Option<String>,
    pub customer_id: Option<i64>,
}

/// 发票数据模型操作类
pub struct InvoiceModel;

impl InvoiceModel {
    pub async fn insert(db: &DbConn, req: &InvoiceSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = invoice::ActiveModel {
            order_id: Set(req.order_id),
            customer_id: Set(req.customer_id),
            invoice_type: Set(req.invoice_type.clone()),
            amount: Set(req.amount),
            currency: Set(req.currency),
            status: Set(req.status.clone()),
            issued_at: Set(req.issued_at),
            due_date: Set(req.due_date),
            notes: Set(req.notes.clone()),
            created_by: Set(req.created_by),
            created_at: Set(Some(now)),
            updated_by: Set(req.updated_by),
            updated_at: Set(Some(now)),
            ..Default::default()
        };
        Invoice::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Invoice::update_many()
            .set(invoice::ActiveModel { deleted: Set(Some(1)), ..Default::default() })
            .filter(invoice::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &InvoiceSaveDTO) -> Result<i64, DbErr> {
        let payload = invoice::ActiveModel {
            order_id: Set(req.order_id),
            customer_id: Set(req.customer_id),
            invoice_type: Set(req.invoice_type.clone()),
            amount: Set(req.amount),
            currency: Set(req.currency),
            status: Set(req.status.clone()),
            issued_at: Set(req.issued_at),
            due_date: Set(req.due_date),
            notes: Set(req.notes.clone()),
            updated_by: Set(req.updated_by),
            updated_at: Set(Some(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let result = Invoice::update_many()
            .set(payload)
            .filter(invoice::Column::Id.eq(id.unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<invoice::Model>, DbErr> {
        Invoice::find_by_id(id)
            .filter(invoice::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        invoice_type: Option<String>,
        status: Option<String>,
        customer_id: Option<i64>,
    ) -> Result<(Vec<invoice::Model>, i64), DbErr> {
        let mut query = Invoice::find().filter(invoice::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(invoice::Column::InvoiceNo.contains(k));
        }
        if let Some(t) = invoice_type {
            query = query.filter(invoice::Column::InvoiceType.eq(t));
        }
        if let Some(s) = status {
            query = query.filter(invoice::Column::Status.eq(s));
        }
        if let Some(cid) = customer_id {
            query = query.filter(invoice::Column::CustomerId.eq(cid));
        }

        let paginator = query.order_by_desc(invoice::Column::CreatedAt).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}