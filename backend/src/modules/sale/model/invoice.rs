//! 销售发票模型层
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal, Date};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::sale::entity::{invoice, invoice::Entity as SaleInvoice};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

// ==================== 请求 DTO ====================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceSaveRequest {
    pub title: Option<String>,
    pub invoice_type: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub contract_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub tax_no: Option<String>,
    pub invoice_date: Option<Date>,
    pub due_date: Option<Date>,
    pub amount: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub buyer_name: Option<String>,
    pub buyer_tax_no: Option<String>,
    pub buyer_address: Option<String>,
    pub buyer_bank: Option<String>,
    pub remark: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub owner_user_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub dept_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub title: Option<String>,
    pub invoice_type: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub contract_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub tax_no: Option<String>,
    pub invoice_date: Option<Date>,
    pub due_date: Option<Date>,
    pub amount: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub buyer_name: Option<String>,
    pub buyer_tax_no: Option<String>,
    pub buyer_address: Option<String>,
    pub buyer_bank: Option<String>,
    pub remark: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub owner_user_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub dept_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub invoice_type: Option<i32>,
    pub status: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
}

// ==================== 内部 DTO ====================

#[derive(Debug, Clone)]
pub struct InvoiceSaveDTO {
    pub invoice_no: Option<String>,
    pub title: Option<String>,
    pub invoice_type: Option<i32>,
    pub contract_id: Option<i64>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub tax_no: Option<String>,
    pub invoice_date: Option<Date>,
    pub due_date: Option<Date>,
    pub amount: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub status: Option<i32>,
    pub buyer_name: Option<String>,
    pub buyer_tax_no: Option<String>,
    pub buyer_address: Option<String>,
    pub buyer_bank: Option<String>,
    pub remark: Option<String>,
    pub owner_user_id: Option<i64>,
    pub dept_id: Option<i64>,
    pub create_by: Option<String>,
    pub update_by: Option<String>,
}

// ==================== 响应 VO ====================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub invoice_no: Option<String>,
    pub title: Option<String>,
    pub invoice_type: Option<i32>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub invoice_date: Option<Date>,
    pub due_date: Option<Date>,
    pub amount: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub status: Option<i32>,
    pub create_time: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub invoice_no: Option<String>,
    pub title: Option<String>,
    pub invoice_type: Option<i32>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub contract_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub tax_no: Option<String>,
    pub invoice_date: Option<Date>,
    pub due_date: Option<Date>,
    pub amount: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub status: Option<i32>,
    pub buyer_name: Option<String>,
    pub buyer_tax_no: Option<String>,
    pub buyer_address: Option<String>,
    pub buyer_bank: Option<String>,
    pub remark: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub owner_user_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub dept_id: Option<i64>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

// ==================== From 转换 ====================

impl From<InvoiceSaveRequest> for InvoiceSaveDTO {
    fn from(req: InvoiceSaveRequest) -> Self {
        Self {
            invoice_no: None,
            title: req.title,
            invoice_type: req.invoice_type,
            contract_id: req.contract_id,
            order_id: req.order_id,
            customer_id: req.customer_id,
            customer_name: req.customer_name,
            tax_no: req.tax_no,
            invoice_date: req.invoice_date,
            due_date: req.due_date,
            amount: req.amount,
            tax_rate: req.tax_rate,
            tax_amount: req.tax_amount,
            currency: req.currency,
            status: None,
            buyer_name: req.buyer_name,
            buyer_tax_no: req.buyer_tax_no,
            buyer_address: req.buyer_address,
            buyer_bank: req.buyer_bank,
            remark: req.remark,
            owner_user_id: req.owner_user_id,
            dept_id: req.dept_id,
            create_by: None,
            update_by: None,
        }
    }
}

impl From<InvoiceUpdateRequest> for InvoiceSaveDTO {
    fn from(req: InvoiceUpdateRequest) -> Self {
        Self {
            invoice_no: None,
            title: req.title,
            invoice_type: req.invoice_type,
            contract_id: req.contract_id,
            order_id: req.order_id,
            customer_id: req.customer_id,
            customer_name: req.customer_name,
            tax_no: req.tax_no,
            invoice_date: req.invoice_date,
            due_date: req.due_date,
            amount: req.amount,
            tax_rate: req.tax_rate,
            tax_amount: req.tax_amount,
            currency: req.currency,
            status: None,
            buyer_name: req.buyer_name,
            buyer_tax_no: req.buyer_tax_no,
            buyer_address: req.buyer_address,
            buyer_bank: req.buyer_bank,
            remark: req.remark,
            owner_user_id: req.owner_user_id,
            dept_id: req.dept_id,
            create_by: None,
            update_by: None,
        }
    }
}

impl From<&invoice::Model> for InvoiceListVO {
    fn from(model: &invoice::Model) -> Self {
        Self {
            id: model.id.into(),
            invoice_no: model.invoice_no.clone(),
            title: model.title.clone(),
            invoice_type: model.invoice_type,
            customer_id: model.customer_id,
            customer_name: model.customer_name.clone(),
            invoice_date: model.invoice_date,
            due_date: model.due_date,
            amount: model.amount,
            tax_amount: model.tax_amount,
            currency: model.currency,
            status: model.status,
            create_time: model.create_time,
        }
    }
}

impl From<&invoice::Model> for InvoiceDetailVO {
    fn from(model: &invoice::Model) -> Self {
        Self {
            id: model.id.into(),
            invoice_no: model.invoice_no.clone(),
            title: model.title.clone(),
            invoice_type: model.invoice_type,
            contract_id: model.contract_id,
            order_id: model.order_id,
            customer_id: model.customer_id,
            customer_name: model.customer_name.clone(),
            tax_no: model.tax_no.clone(),
            invoice_date: model.invoice_date,
            due_date: model.due_date,
            amount: model.amount,
            tax_rate: model.tax_rate,
            tax_amount: model.tax_amount,
            currency: model.currency,
            status: model.status,
            buyer_name: model.buyer_name.clone(),
            buyer_tax_no: model.buyer_tax_no.clone(),
            buyer_address: model.buyer_address.clone(),
            buyer_bank: model.buyer_bank.clone(),
            remark: model.remark.clone(),
            owner_user_id: model.owner_user_id,
            dept_id: model.dept_id,
            create_by: model.create_by.clone(),
            create_time: model.create_time,
            update_by: model.update_by.clone(),
            update_time: model.update_time,
        }
    }
}

// ==================== 数据库操作方法 ====================

fn deserialize_option_string_to_u64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<i64>().map(Some).map_err(serde::de::Error::custom)
            }
        }
        None => Ok(None),
    }
}

pub struct InvoiceModel;

impl InvoiceModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &InvoiceSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = invoice::ActiveModel {
            invoice_no: Set(req.invoice_no.clone()),
            title: Set(req.title.clone()),
            invoice_type: Set(req.invoice_type.or(Some(1))),
            contract_id: Set(req.contract_id),
            order_id: Set(req.order_id),
            customer_id: Set(req.customer_id),
            customer_name: Set(req.customer_name.clone()),
            tax_no: Set(req.tax_no.clone()),
            invoice_date: Set(req.invoice_date),
            due_date: Set(req.due_date),
            amount: Set(req.amount.or(Some(Decimal::from(0)))),
            tax_rate: Set(req.tax_rate.or(Some(Decimal::from(0)))),
            tax_amount: Set(req.tax_amount.or(Some(Decimal::from(0)))),
            currency: Set(req.currency.or(Some(1))),
            status: Set(req.status.or(Some(1))),
            buyer_name: Set(req.buyer_name.clone()),
            buyer_tax_no: Set(req.buyer_tax_no.clone()),
            buyer_address: Set(req.buyer_address.clone()),
            buyer_bank: Set(req.buyer_bank.clone()),
            remark: Set(req.remark.clone()),
            owner_user_id: Set(req.owner_user_id),
            dept_id: Set(req.dept_id),
            create_by: Set(req.create_by.clone()),
            create_time: Set(Some(now)),
            update_by: Set(req.update_by.clone()),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        SaleInvoice::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: i64, req: &InvoiceSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = invoice::ActiveModel {
            update_time: Set(Some(now)),
            ..Default::default()
        };

        if let Some(v) = req.title.clone() { payload.title = Set(Some(v)); }
        if let Some(v) = req.invoice_type { payload.invoice_type = Set(Some(v)); }
        if let Some(v) = req.contract_id { payload.contract_id = Set(Some(v)); }
        if let Some(v) = req.order_id { payload.order_id = Set(Some(v)); }
        if let Some(v) = req.customer_id { payload.customer_id = Set(Some(v)); }
        if let Some(v) = req.customer_name.clone() { payload.customer_name = Set(Some(v)); }
        if let Some(v) = req.tax_no.clone() { payload.tax_no = Set(Some(v)); }
        if let Some(v) = req.invoice_date { payload.invoice_date = Set(Some(v)); }
        if let Some(v) = req.due_date { payload.due_date = Set(Some(v)); }
        if let Some(v) = req.amount { payload.amount = Set(Some(v)); }
        if let Some(v) = req.tax_rate { payload.tax_rate = Set(Some(v)); }
        if let Some(v) = req.tax_amount { payload.tax_amount = Set(Some(v)); }
        if let Some(v) = req.currency { payload.currency = Set(Some(v)); }
        if let Some(v) = req.status { payload.status = Set(Some(v)); }
        if let Some(v) = req.buyer_name.clone() { payload.buyer_name = Set(Some(v)); }
        if let Some(v) = req.buyer_tax_no.clone() { payload.buyer_tax_no = Set(Some(v)); }
        if let Some(v) = req.buyer_address.clone() { payload.buyer_address = Set(Some(v)); }
        if let Some(v) = req.buyer_bank.clone() { payload.buyer_bank = Set(Some(v)); }
        if let Some(v) = req.remark.clone() { payload.remark = Set(Some(v)); }
        if let Some(v) = req.owner_user_id { payload.owner_user_id = Set(Some(v)); }
        if let Some(v) = req.dept_id { payload.dept_id = Set(Some(v)); }
        if let Some(v) = req.update_by.clone() { payload.update_by = Set(Some(v)); }

        let result = SaleInvoice::update_many()
            .set(payload)
            .filter(invoice::Column::Id.eq(id))
            .filter(invoice::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn batch_delete_by_ids<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        SaleInvoice::update_many()
            .set(invoice::ActiveModel {
                deleted: Set(Some(1)),
                update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(invoice::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<invoice::Model>, DbErr> {
        SaleInvoice::find_by_id(id)
            .filter(invoice::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn get_max_invoice_no_today<C: ConnectionTrait>(db: &C, date_prefix: &str) -> Result<Option<i64>, DbErr> {
        use sea_orm::QuerySelect;
        use sea_orm::prelude::Expr;

        let pattern = format!("{}%", date_prefix);
        let result = SaleInvoice::find()
            .filter(invoice::Column::InvoiceNo.like(&pattern))
            .select_only()
            .column_as(Expr::expr(Expr::cust("MAX(CAST(SUBSTRING(invoice_no, 11) AS INTEGER))")), "max_seq")
            .into_tuple::<Option<i64>>()
            .one(db)
            .await?;

        Ok(result.flatten())
    }

    pub async fn select_in_page<C: ConnectionTrait>(
        db: &C,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        invoice_type: Option<i32>,
        status: Option<i32>,
        customer_id: Option<i64>,
    ) -> Result<(Vec<invoice::Model>, i64), DbErr> {
        let mut query = SaleInvoice::find()
            .filter(invoice::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            if !k.trim().is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(invoice::Column::InvoiceNo.contains(k.trim()))
                        .add(invoice::Column::CustomerName.contains(k.trim()))
                        .add(invoice::Column::Title.contains(k.trim())),
                );
            }
        }
        if let Some(t) = invoice_type {
            query = query.filter(invoice::Column::InvoiceType.eq(t));
        }
        if let Some(s) = status {
            query = query.filter(invoice::Column::Status.eq(s));
        }
        if let Some(c) = customer_id {
            query = query.filter(invoice::Column::CustomerId.eq(c));
        }

        let paginator = query.order_by_desc(invoice::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }
}
