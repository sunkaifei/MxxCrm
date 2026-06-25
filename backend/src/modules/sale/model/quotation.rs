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
use crate::modules::sale::entity::{quotation, quotation::Entity as Quotation};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 报价单新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct QuotationSaveRequest {
    pub customer_id: Option<i64>,
    pub contact_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub title: Option<String>,
    pub items: Option<serde_json::Value>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub tax_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub grand_total: Option<Decimal>,
    pub valid_until: Option<Date>,
    pub status: Option<String>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub delivery_date: Option<Date>,
    pub port_of_loading: Option<String>,
    pub port_of_destination: Option<String>,
    pub bank_info: Option<String>,
    pub notes: Option<String>,
    pub assigned_to: Option<i64>,
}

impl From<QuotationSaveRequest> for QuotationSaveDTO {
    fn from(item: QuotationSaveRequest) -> Self {
        QuotationSaveDTO {
            id: None,
            quotation_no: None,
            customer_id: item.customer_id,
            contact_id: item.contact_id,
            opportunity_id: item.opportunity_id,
            title: item.title,
            items: item.items,
            total_amount: item.total_amount,
            currency: item.currency,
            tax_amount: item.tax_amount,
            discount_amount: item.discount_amount,
            grand_total: item.grand_total,
            valid_until: item.valid_until,
            status: item.status,
            payment_terms: item.payment_terms,
            delivery_terms: item.delivery_terms,
            delivery_date: item.delivery_date,
            port_of_loading: item.port_of_loading,
            port_of_destination: item.port_of_destination,
            bank_info: item.bank_info,
            notes: item.notes,
            assigned_to: item.assigned_to,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 报价单更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct QuotationUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub customer_id: Option<i64>,
    pub contact_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub title: Option<String>,
    pub items: Option<serde_json::Value>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub tax_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub grand_total: Option<Decimal>,
    pub valid_until: Option<Date>,
    pub status: Option<String>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub delivery_date: Option<Date>,
    pub port_of_loading: Option<String>,
    pub port_of_destination: Option<String>,
    pub bank_info: Option<String>,
    pub notes: Option<String>,
    pub assigned_to: Option<i64>,
}

impl From<QuotationUpdateRequest> for QuotationSaveDTO {
    fn from(item: QuotationUpdateRequest) -> Self {
        QuotationSaveDTO {
            id: item.id,
            quotation_no: None,
            customer_id: item.customer_id,
            contact_id: item.contact_id,
            opportunity_id: item.opportunity_id,
            title: item.title,
            items: item.items,
            total_amount: item.total_amount,
            currency: item.currency,
            tax_amount: item.tax_amount,
            discount_amount: item.discount_amount,
            grand_total: item.grand_total,
            valid_until: item.valid_until,
            status: item.status,
            payment_terms: item.payment_terms,
            delivery_terms: item.delivery_terms,
            delivery_date: item.delivery_date,
            port_of_loading: item.port_of_loading,
            port_of_destination: item.port_of_destination,
            bank_info: item.bank_info,
            notes: item.notes,
            assigned_to: item.assigned_to,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 报价单保存DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct QuotationSaveDTO {
    pub id: Option<i64>,
    pub quotation_no: Option<String>,
    pub customer_id: Option<i64>,
    pub contact_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub title: Option<String>,
    pub items: Option<serde_json::Value>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub tax_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub grand_total: Option<Decimal>,
    pub valid_until: Option<Date>,
    pub status: Option<String>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub delivery_date: Option<Date>,
    pub port_of_loading: Option<String>,
    pub port_of_destination: Option<String>,
    pub bank_info: Option<String>,
    pub notes: Option<String>,
    pub assigned_to: Option<i64>,
    pub deleted: Option<i32>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub update_time: Option<DateTime>,
}

/// 报价单详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct QuotationDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub quotation_no: Option<String>,
    pub customer_id: Option<i64>,
    pub contact_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub title: Option<String>,
    pub items: Option<serde_json::Value>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub tax_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub grand_total: Option<Decimal>,
    pub valid_until: Option<Date>,
    pub status: Option<String>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub delivery_date: Option<Date>,
    pub port_of_loading: Option<String>,
    pub port_of_destination: Option<String>,
    pub bank_info: Option<String>,
    pub notes: Option<String>,
    pub assigned_to: Option<i64>,
    pub create_time: Option<DateTime>,
}

impl From<quotation::Model> for QuotationDetailVO {
    fn from(item: quotation::Model) -> Self {
        QuotationDetailVO {
            id: Option::from(item.id),
            quotation_no: item.quotation_no,
            customer_id: item.customer_id,
            contact_id: item.contact_id,
            opportunity_id: item.opportunity_id,
            title: item.title,
            items: item.items,
            total_amount: item.total_amount,
            currency: item.currency,
            tax_amount: item.tax_amount,
            discount_amount: item.discount_amount,
            grand_total: item.grand_total,
            valid_until: item.valid_until,
            status: item.status,
            payment_terms: item.payment_terms,
            delivery_terms: item.delivery_terms,
            delivery_date: item.delivery_date,
            port_of_loading: item.port_of_loading,
            port_of_destination: item.port_of_destination,
            bank_info: item.bank_info,
            notes: item.notes,
            assigned_to: item.assigned_to,
            create_time: item.create_time,
        }
    }
}

/// 报价单列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct QuotationListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub quotation_no: Option<String>,
    pub customer_id: Option<i64>,
    pub title: Option<String>,
    pub grand_total: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub status: Option<String>,
    pub valid_until: Option<Date>,
    pub assigned_to: Option<i64>,
    pub create_time: Option<DateTime>,
}

impl From<quotation::Model> for QuotationListVO {
    fn from(item: quotation::Model) -> Self {
        QuotationListVO {
            id: Option::from(item.id),
            quotation_no: item.quotation_no,
            customer_id: item.customer_id,
            title: item.title,
            grand_total: item.grand_total,
            currency: item.currency,
            status: item.status,
            valid_until: item.valid_until,
            assigned_to: item.assigned_to,
            create_time: item.create_time,
        }
    }
}

/// 报价单列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuotationListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub customer_id: Option<i64>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// 报价单数据模型操作类
pub struct QuotationModel;

impl QuotationModel {
    pub async fn insert(db: &DbConn, req: &QuotationSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = quotation::ActiveModel {
            customer_id: Set(req.customer_id),
            contact_id: Set(req.contact_id),
            opportunity_id: Set(req.opportunity_id),
            title: Set(req.title.clone()),
            items: Set(req.items.clone()),
            total_amount: Set(req.total_amount),
            currency: Set(req.currency),
            tax_amount: Set(req.tax_amount),
            discount_amount: Set(req.discount_amount),
            grand_total: Set(req.grand_total),
            valid_until: Set(req.valid_until),
            status: Set(req.status.clone()),
            payment_terms: Set(req.payment_terms.clone()),
            delivery_terms: Set(req.delivery_terms.clone()),
            delivery_date: Set(req.delivery_date),
            port_of_loading: Set(req.port_of_loading.clone()),
            port_of_destination: Set(req.port_of_destination.clone()),
            bank_info: Set(req.bank_info.clone()),
            notes: Set(req.notes.clone()),
            assigned_to: Set(req.assigned_to),
            created_by: Set(req.created_by),
            create_time: Set(Some(now)),
            updated_by: Set(req.updated_by),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        Quotation::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Quotation::update_many()
            .set(quotation::ActiveModel { deleted: Set(Some(1)), ..Default::default() })
            .filter(quotation::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &QuotationSaveDTO) -> Result<i64, DbErr> {
        let payload = quotation::ActiveModel {
            customer_id: Set(req.customer_id),
            contact_id: Set(req.contact_id),
            opportunity_id: Set(req.opportunity_id),
            title: Set(req.title.clone()),
            items: Set(req.items.clone()),
            total_amount: Set(req.total_amount),
            currency: Set(req.currency),
            tax_amount: Set(req.tax_amount),
            discount_amount: Set(req.discount_amount),
            grand_total: Set(req.grand_total),
            valid_until: Set(req.valid_until),
            status: Set(req.status.clone()),
            payment_terms: Set(req.payment_terms.clone()),
            delivery_terms: Set(req.delivery_terms.clone()),
            delivery_date: Set(req.delivery_date),
            port_of_loading: Set(req.port_of_loading.clone()),
            port_of_destination: Set(req.port_of_destination.clone()),
            bank_info: Set(req.bank_info.clone()),
            notes: Set(req.notes.clone()),
            assigned_to: Set(req.assigned_to),
            updated_by: Set(req.updated_by),
            update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let result = Quotation::update_many()
            .set(payload)
            .filter(quotation::Column::Id.eq(id.unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_status(db: &DbConn, id: i64, status: &str) -> Result<i64, DbErr> {
        let result = Quotation::update_many()
            .set(quotation::ActiveModel {
                status: Set(Some(status.to_string())),
                update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(quotation::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<quotation::Model>, DbErr> {
        Quotation::find_by_id(id)
            .filter(quotation::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        customer_id: Option<i64>,
        status: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<(Vec<quotation::Model>, i64), DbErr> {
        let mut query = Quotation::find().filter(quotation::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(
                quotation::Column::QuotationNo.contains(k.clone())
                    .or(quotation::Column::Title.contains(k))
            );
        }
        if let Some(cid) = customer_id {
            query = query.filter(quotation::Column::CustomerId.eq(cid));
        }
        if let Some(s) = status {
            query = query.filter(quotation::Column::Status.eq(s));
        }
        if let Some(start) = start_date {
            query = query.filter(quotation::Column::CreateTime.gte(start));
        }
        if let Some(end) = end_date {
            query = query.filter(quotation::Column::CreateTime.lte(end));
        }

        let paginator = query.order_by_desc(quotation::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}