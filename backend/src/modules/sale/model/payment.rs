//! 销售回款模型层
//!
//! 版权所有，侵权必究！
//!

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::*;
use sea_orm::prelude::DateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::sale::entity::{payment, payment::Entity as SalePayment, order};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

// ==================== 请求 DTO ====================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSaveRequest {
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub contract_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub plan_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub payment_method: Option<i32>,
    pub payment_date: Option<NaiveDate>,
    pub payer: Option<String>,
    pub payer_account: Option<String>,
    pub bank_flow_no: Option<String>,
    pub attachment: Option<String>,
    pub remark: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub owner_user_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub dept_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub contract_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub plan_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub payment_method: Option<i32>,
    pub payment_date: Option<NaiveDate>,
    pub payer: Option<String>,
    pub payer_account: Option<String>,
    pub bank_flow_no: Option<String>,
    pub attachment: Option<String>,
    pub remark: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub owner_user_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub dept_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub payment_no: Option<String>,
    pub order_no: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub contract_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    pub status: Option<i32>,
    pub payment_method: Option<i32>,
}

// ==================== 内部 DTO ====================

#[derive(Debug, Clone)]
pub struct PaymentSaveDTO {
    pub payment_no: Option<String>,
    pub contract_id: Option<i64>,
    pub order_id: Option<i64>,
    pub plan_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub payment_method: Option<i32>,
    pub payment_date: Option<NaiveDate>,
    pub payer: Option<String>,
    pub payer_account: Option<String>,
    pub bank_flow_no: Option<String>,
    pub attachment: Option<String>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub owner_user_id: Option<i64>,
    pub dept_id: Option<i64>,
    pub create_by: Option<String>,
    pub update_by: Option<String>,
}

// ==================== 响应 VO ====================

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PaymentListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub payment_no: Option<String>,
    pub contract_no: Option<String>,
    pub order_no: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub payment_method: Option<i32>,
    pub payment_date: Option<NaiveDate>,
    pub status: Option<i32>,
    pub create_time: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub payment_no: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub contract_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub plan_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub payment_method: Option<i32>,
    pub payment_date: Option<NaiveDate>,
    pub payer: Option<String>,
    pub payer_account: Option<String>,
    pub bank_flow_no: Option<String>,
    pub attachment: Option<String>,
    pub status: Option<i32>,
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

impl From<PaymentSaveRequest> for PaymentSaveDTO {
    fn from(req: PaymentSaveRequest) -> Self {
        Self {
            payment_no: None,
            contract_id: req.contract_id,
            order_id: req.order_id,
            plan_id: req.plan_id,
            customer_id: req.customer_id,
            customer_name: req.customer_name,
            amount: req.amount,
            currency: req.currency,
            payment_method: req.payment_method,
            payment_date: req.payment_date,
            payer: req.payer,
            payer_account: req.payer_account,
            bank_flow_no: req.bank_flow_no,
            attachment: req.attachment,
            status: None,
            remark: req.remark,
            owner_user_id: req.owner_user_id,
            dept_id: req.dept_id,
            create_by: None,
            update_by: None,
        }
    }
}

impl From<PaymentUpdateRequest> for PaymentSaveDTO {
    fn from(req: PaymentUpdateRequest) -> Self {
        Self {
            payment_no: None,
            contract_id: req.contract_id,
            order_id: req.order_id,
            plan_id: req.plan_id,
            customer_id: req.customer_id,
            customer_name: req.customer_name,
            amount: req.amount,
            currency: req.currency,
            payment_method: req.payment_method,
            payment_date: req.payment_date,
            payer: req.payer,
            payer_account: req.payer_account,
            bank_flow_no: req.bank_flow_no,
            attachment: req.attachment,
            status: None,
            remark: req.remark,
            owner_user_id: req.owner_user_id,
            dept_id: req.dept_id,
            create_by: None,
            update_by: None,
        }
    }
}

impl From<&payment::Model> for PaymentListVO {
    fn from(model: &payment::Model) -> Self {
        Self {
            id: model.id.into(),
            payment_no: model.payment_no.clone(),
            contract_no: None,
            order_no: None,
            customer_id: model.customer_id,
            customer_name: model.customer_name.clone(),
            amount: model.amount,
            currency: model.currency,
            payment_method: model.payment_method,
            payment_date: model.payment_date,
            status: model.status,
            create_time: model.create_time,
        }
    }
}

impl From<&payment::Model> for PaymentDetailVO {
    fn from(model: &payment::Model) -> Self {
        Self {
            id: model.id.into(),
            payment_no: model.payment_no.clone(),
            contract_id: model.contract_id,
            order_id: model.order_id,
            plan_id: model.plan_id,
            customer_id: model.customer_id,
            customer_name: model.customer_name.clone(),
            amount: model.amount,
            currency: model.currency,
            payment_method: model.payment_method,
            payment_date: model.payment_date,
            payer: model.payer.clone(),
            payer_account: model.payer_account.clone(),
            bank_flow_no: model.bank_flow_no.clone(),
            attachment: model.attachment.clone(),
            status: model.status,
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

pub struct PaymentModel;

impl PaymentModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &PaymentSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = payment::ActiveModel {
            payment_no: Set(req.payment_no.clone()),
            contract_id: Set(req.contract_id),
            order_id: Set(req.order_id),
            plan_id: Set(req.plan_id),
            customer_id: Set(req.customer_id),
            customer_name: Set(req.customer_name.clone()),
            amount: Set(req.amount.or(Some(Decimal::from(0)))),
            currency: Set(req.currency.or(Some(1))),
            payment_method: Set(req.payment_method.or(Some(1))),
            payment_date: Set(req.payment_date),
            payer: Set(req.payer.clone()),
            payer_account: Set(req.payer_account.clone()),
            bank_flow_no: Set(req.bank_flow_no.clone()),
            attachment: Set(req.attachment.clone()),
            status: Set(req.status.or(Some(1))),
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
        SalePayment::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: i64, req: &PaymentSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = payment::ActiveModel {
            update_time: Set(Some(now)),
            ..Default::default()
        };

        if let Some(v) = req.contract_id { payload.contract_id = Set(Some(v)); }
        if let Some(v) = req.order_id { payload.order_id = Set(Some(v)); }
        if let Some(v) = req.plan_id { payload.plan_id = Set(Some(v)); }
        if let Some(v) = req.customer_id { payload.customer_id = Set(Some(v)); }
        if let Some(v) = req.customer_name.clone() { payload.customer_name = Set(Some(v)); }
        if let Some(v) = req.amount { payload.amount = Set(Some(v)); }
        if let Some(v) = req.currency { payload.currency = Set(Some(v)); }
        if let Some(v) = req.payment_method { payload.payment_method = Set(Some(v)); }
        if let Some(v) = req.payment_date { payload.payment_date = Set(Some(v)); }
        if let Some(v) = req.payer.clone() { payload.payer = Set(Some(v)); }
        if let Some(v) = req.payer_account.clone() { payload.payer_account = Set(Some(v)); }
        if let Some(v) = req.bank_flow_no.clone() { payload.bank_flow_no = Set(Some(v)); }
        if let Some(v) = req.attachment.clone() { payload.attachment = Set(Some(v)); }
        if let Some(v) = req.status { payload.status = Set(Some(v)); }
        if let Some(v) = req.remark.clone() { payload.remark = Set(Some(v)); }
        if let Some(v) = req.owner_user_id { payload.owner_user_id = Set(Some(v)); }
        if let Some(v) = req.dept_id { payload.dept_id = Set(Some(v)); }
        if let Some(v) = req.update_by.clone() { payload.update_by = Set(Some(v)); }

        let result = SalePayment::update_many()
            .set(payload)
            .filter(payment::Column::Id.eq(id))
            .filter(payment::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn batch_delete_by_ids<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        SalePayment::update_many()
            .set(payment::ActiveModel {
                deleted: Set(Some(1)),
                update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(payment::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<payment::Model>, DbErr> {
        SalePayment::find_by_id(id)
            .filter(payment::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn get_max_payment_no_today<C: ConnectionTrait>(db: &C, date_prefix: &str) -> Result<Option<i64>, DbErr> {
        use sea_orm::QuerySelect;
        use sea_orm::prelude::Expr;

        let pattern = format!("{}%", date_prefix);
        let result = SalePayment::find()
            .filter(payment::Column::PaymentNo.like(&pattern))
            .select_only()
            .column_as(Expr::expr(Expr::cust("MAX(CAST(SUBSTRING(payment_no, 11) AS INTEGER))")), "max_seq")
            .into_tuple::<Option<i64>>()
            .one(db)
            .await?;

        Ok(result.flatten())
    }

    pub async fn select_in_page<C: ConnectionTrait>(
        db: &C,
        page: i64,
        per_page: i64,
        payment_no: Option<String>,
        order_no: Option<String>,
        contract_id: Option<i64>,
        customer_id: Option<i64>,
        status: Option<i32>,
        payment_method: Option<i32>,
    ) -> Result<(Vec<payment::Model>, i64), DbErr> {
        let mut query = SalePayment::find()
            .filter(payment::Column::Deleted.eq(0));

        if let Some(pno) = payment_no {
            if !pno.trim().is_empty() {
                query = query.filter(payment::Column::PaymentNo.contains(pno.trim()));
            }
        }
        if let Some(ono) = order_no {
            if !ono.trim().is_empty() {
                let order_ids = order::Entity::find()
                    .filter(order::Column::OrderNo.contains(ono.trim()))
                    .filter(order::Column::Deleted.eq(0))
                    .all(db)
                    .await?
                    .into_iter()
                    .map(|o| o.id)
                    .collect::<Vec<i64>>();
                if !order_ids.is_empty() {
                    query = query.filter(payment::Column::OrderId.is_in(order_ids));
                } else {
                    return Ok((vec![], 0));
                }
            }
        }
        if let Some(cid) = contract_id {
            query = query.filter(payment::Column::ContractId.eq(cid));
        }
        if let Some(cust_id) = customer_id {
            query = query.filter(payment::Column::CustomerId.eq(cust_id));
        }
        if let Some(s) = status {
            query = query.filter(payment::Column::Status.eq(s));
        }
        if let Some(pm) = payment_method {
            query = query.filter(payment::Column::PaymentMethod.eq(pm));
        }

        let paginator = query.order_by_desc(payment::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }

    pub async fn find_orders_by_ids<C: ConnectionTrait>(db: &C, ids: &[i64]) -> Result<Vec<order::Model>, DbErr> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        order::Entity::find()
            .filter(order::Column::Id.is_in(ids.to_vec()))
            .filter(order::Column::Deleted.eq(0))
            .all(db)
            .await
    }
}
