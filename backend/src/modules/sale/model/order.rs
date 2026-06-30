//! 销售订单模型层
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal, Date};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::sale::entity::{order, order::Entity as SaleOrder, order_item, order_item::Entity as SaleOrderItem};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

// ==================== 请求 DTO ====================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderSaveRequest {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub contact_id: Option<i64>,
    pub contact_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub opportunity_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub quotation_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub contract_id: Option<i64>,
    pub order_date: Option<Date>,
    pub delivery_date: Option<Date>,
    pub order_type: Option<i32>,
    pub currency: Option<i32>,
    pub exchange_rate: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub shipping_fee: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub other_fee: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub payment_method: Option<i32>,
    pub payment_due_date: Option<Date>,
    pub shipping_method: Option<i32>,
    pub receiver_name: Option<String>,
    pub receiver_phone: Option<String>,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub remark: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub owner_user_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub dept_id: Option<i64>,
    pub items: Option<Vec<OrderItemSaveDTO>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub contact_id: Option<i64>,
    pub contact_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub opportunity_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub quotation_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub contract_id: Option<i64>,
    pub order_date: Option<Date>,
    pub delivery_date: Option<Date>,
    pub order_type: Option<i32>,
    pub currency: Option<i32>,
    pub exchange_rate: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub shipping_fee: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub other_fee: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub payment_method: Option<i32>,
    pub payment_due_date: Option<Date>,
    pub shipping_method: Option<i32>,
    pub receiver_name: Option<String>,
    pub receiver_phone: Option<String>,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub remark: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub owner_user_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub dept_id: Option<i64>,
    pub items: Option<Vec<OrderItemSaveDTO>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatusUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub order_status: Option<i32>,
    pub tracking_no: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderListQuery {
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub order_status: Option<i32>,
    pub payment_status: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub owner_user_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

// ==================== 内部 DTO ====================

#[derive(Debug, Clone)]
pub struct OrderSaveDTO {
    pub order_no: Option<String>,
    pub title: Option<String>,
    pub order_type: Option<i32>,
    pub order_status: Option<i32>,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub contact_id: Option<i64>,
    pub contact_name: Option<String>,
    pub opportunity_id: Option<i64>,
    pub quotation_id: Option<i64>,
    pub contract_id: Option<i64>,
    pub order_date: Option<Date>,
    pub delivery_date: Option<Date>,
    pub currency: Option<i32>,
    pub exchange_rate: Option<Decimal>,
    pub product_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub shipping_fee: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub other_fee: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub paid_amount: Option<Decimal>,
    pub unpaid_amount: Option<Decimal>,
    pub pay_status: Option<i32>,
    pub payment_method: Option<i32>,
    pub payment_due_date: Option<Date>,
    pub shipping_method: Option<i32>,
    pub tracking_no: Option<String>,
    pub shipping_time: Option<DateTime>,
    pub complete_time: Option<DateTime>,
    pub receiver_name: Option<String>,
    pub receiver_phone: Option<String>,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub remark: Option<String>,
    pub owner_user_id: Option<i64>,
    pub dept_id: Option<i64>,
    pub create_by: Option<i64>,
    pub update_by: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemSaveDTO {
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub sku: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub unit_id: Option<i64>,
    pub quantity: Option<i32>,
    pub unit_price: Option<Decimal>,
    pub discount_rate: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub delivery_date: Option<Date>,
    pub delivered_quantity: Option<Decimal>,
    pub remark: Option<String>,
    pub sort: Option<i32>,
}

// ==================== 响应 VO ====================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub order_no: Option<String>,
    pub title: Option<String>,
    pub order_type: Option<i32>,
    pub order_status: Option<i32>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub order_date: Option<Date>,
    pub delivery_date: Option<Date>,
    pub total_amount: Option<Decimal>,
    pub paid_amount: Option<Decimal>,
    pub unpaid_amount: Option<Decimal>,
    pub payment_status: Option<i32>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub owner_user_id: Option<i64>,
    pub create_time: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub order_no: Option<String>,
    pub title: Option<String>,
    pub order_type: Option<i32>,
    pub order_status: Option<i32>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub contact_id: Option<i64>,
    pub contact_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub opportunity_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub quotation_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub contract_id: Option<i64>,
    pub order_date: Option<Date>,
    pub delivery_date: Option<Date>,
    pub currency: Option<i32>,
    pub exchange_rate: Option<Decimal>,
    pub product_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub shipping_fee: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub other_fee: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub paid_amount: Option<Decimal>,
    pub unpaid_amount: Option<Decimal>,
    pub payment_status: Option<i32>,
    pub payment_method: Option<i32>,
    pub payment_due_date: Option<Date>,
    pub shipping_method: Option<i32>,
    pub tracking_no: Option<String>,
    pub shipping_time: Option<DateTime>,
    pub complete_time: Option<DateTime>,
    pub receiver_name: Option<String>,
    pub receiver_phone: Option<String>,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub remark: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub owner_user_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub dept_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub update_by: Option<i64>,
    pub update_time: Option<DateTime>,
    pub items: Vec<OrderItemVO>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub sku: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub unit_id: Option<i64>,
    pub quantity: Option<i32>,
    pub unit_price: Option<Decimal>,
    pub discount_rate: Option<Decimal>,
    pub discount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub delivery_date: Option<Date>,
    pub delivered_quantity: Option<Decimal>,
    pub remark: Option<String>,
    pub sort: Option<i32>,
}

// ==================== From 转换 ====================

impl From<OrderSaveRequest> for OrderSaveDTO {
    fn from(req: OrderSaveRequest) -> Self {
        Self {
            order_no: None,
            title: req.title,
            order_type: req.order_type,
            order_status: None,
            customer_id: req.customer_id,
            customer_name: req.customer_name,
            contact_id: req.contact_id,
            contact_name: req.contact_name,
            opportunity_id: req.opportunity_id,
            quotation_id: req.quotation_id,
            contract_id: req.contract_id,
            order_date: req.order_date,
            delivery_date: req.delivery_date,
            currency: req.currency,
            exchange_rate: req.exchange_rate,
            product_amount: None,
            discount_amount: req.discount_amount,
            shipping_fee: req.shipping_fee,
            tax_amount: req.tax_amount,
            other_fee: req.other_fee,
            total_amount: req.total_amount,
            paid_amount: None,
            unpaid_amount: None,
            pay_status: None,
            payment_method: req.payment_method,
            payment_due_date: req.payment_due_date,
            shipping_method: req.shipping_method,
            tracking_no: None,
            shipping_time: None,
            complete_time: None,
            receiver_name: req.receiver_name,
            receiver_phone: req.receiver_phone,
            shipping_address: req.shipping_address,
            billing_address: req.billing_address,
            remark: req.remark,
            owner_user_id: req.owner_user_id,
            dept_id: req.dept_id,
            create_by: None,
            update_by: None,
        }
    }
}

impl From<OrderUpdateRequest> for OrderSaveDTO {
    fn from(req: OrderUpdateRequest) -> Self {
        Self {
            order_no: None,
            title: req.title,
            order_type: req.order_type,
            order_status: None,
            customer_id: req.customer_id,
            customer_name: req.customer_name,
            contact_id: req.contact_id,
            contact_name: req.contact_name,
            opportunity_id: req.opportunity_id,
            quotation_id: req.quotation_id,
            contract_id: req.contract_id,
            order_date: req.order_date,
            delivery_date: req.delivery_date,
            currency: req.currency,
            exchange_rate: req.exchange_rate,
            product_amount: None,
            discount_amount: req.discount_amount,
            shipping_fee: req.shipping_fee,
            tax_amount: req.tax_amount,
            other_fee: req.other_fee,
            total_amount: req.total_amount,
            paid_amount: None,
            unpaid_amount: None,
            pay_status: None,
            payment_method: req.payment_method,
            payment_due_date: req.payment_due_date,
            shipping_method: req.shipping_method,
            tracking_no: None,
            shipping_time: None,
            complete_time: None,
            receiver_name: req.receiver_name,
            receiver_phone: req.receiver_phone,
            shipping_address: req.shipping_address,
            billing_address: req.billing_address,
            remark: req.remark,
            owner_user_id: req.owner_user_id,
            dept_id: req.dept_id,
            create_by: None,
            update_by: None,
        }
    }
}

impl From<&order::Model> for OrderListVO {
    fn from(model: &order::Model) -> Self {
        Self {
            id: model.id.into(),
            order_no: model.order_no.clone(),
            title: model.title.clone(),
            order_type: model.order_type,
            order_status: model.order_status,
            customer_id: model.customer_id,
            customer_name: model.customer_name.clone(),
            order_date: model.order_date,
            delivery_date: model.delivery_date,
            total_amount: model.total_amount,
            paid_amount: model.paid_amount,
            unpaid_amount: model.unpaid_amount,
            payment_status: model.pay_status,
            owner_user_id: model.owner_user_id,
            create_time: model.create_time,
        }
    }
}

impl From<(&order::Model, Vec<order_item::Model>)> for OrderDetailVO {
    fn from(data: (&order::Model, Vec<order_item::Model>)) -> Self {
        let (model, items) = data;
        Self {
            id: model.id.into(),
            order_no: model.order_no.clone(),
            title: model.title.clone(),
            order_type: model.order_type,
            order_status: model.order_status,
            customer_id: model.customer_id,
            customer_name: model.customer_name.clone(),
            contact_id: model.contact_id,
            contact_name: model.contact_name.clone(),
            opportunity_id: model.opportunity_id,
            quotation_id: model.quotation_id,
            contract_id: model.contract_id,
            order_date: model.order_date,
            delivery_date: model.delivery_date,
            currency: model.currency,
            exchange_rate: model.exchange_rate,
            product_amount: model.product_amount,
            discount_amount: model.discount_amount,
            shipping_fee: model.shipping_fee,
            tax_amount: model.tax_amount,
            other_fee: model.other_fee,
            total_amount: model.total_amount,
            paid_amount: model.paid_amount,
            unpaid_amount: model.unpaid_amount,
            payment_status: model.pay_status,
            payment_method: model.payment_method,
            payment_due_date: model.payment_due_date,
            shipping_method: model.shipping_method,
            tracking_no: model.tracking_no.clone(),
            shipping_time: model.shipping_time,
            complete_time: model.complete_time,
            receiver_name: model.receiver_name.clone(),
            receiver_phone: model.receiver_phone.clone(),
            shipping_address: model.shipping_address.clone(),
            billing_address: model.billing_address.clone(),
            remark: model.remark.clone(),
            owner_user_id: model.owner_user_id,
            dept_id: model.dept_id,
            create_by: model.create_by,
            create_time: model.create_time,
            update_by: model.update_by,
            update_time: model.update_time,
            items: items.iter().map(|i| i.into()).collect(),
        }
    }
}

impl From<&order_item::Model> for OrderItemVO {
    fn from(model: &order_item::Model) -> Self {
        Self {
            id: model.id.into(),
            order_id: model.order_id,
            product_id: model.product_id,
            product_name: model.product_name.clone(),
            product_code: model.product_code.clone(),
            sku: model.sku.clone(),
            spec: model.spec.clone(),
            unit: model.unit.clone(),
            unit_id: model.unit_id,
            quantity: model.quantity,
            unit_price: model.unit_price,
            discount_rate: model.discount_rate,
            discount: model.discount,
            discount_amount: model.discount_amount,
            tax_rate: model.tax_rate,
            tax_amount: model.tax_amount,
            amount: model.amount,
            total_amount: model.total_amount,
            delivery_date: model.delivery_date,
            delivered_quantity: model.delivered_quantity,
            remark: model.remark.clone(),
            sort: model.sort,
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

pub struct OrderModel;

impl OrderModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &OrderSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = order::ActiveModel {
            order_no: Set(req.order_no.clone()),
            title: Set(req.title.clone()),
            order_type: Set(req.order_type.or(Some(1))),
            order_status: Set(req.order_status.or(Some(1))),
            customer_id: Set(req.customer_id),
            customer_name: Set(req.customer_name.clone()),
            contact_id: Set(req.contact_id),
            contact_name: Set(req.contact_name.clone()),
            opportunity_id: Set(req.opportunity_id),
            quotation_id: Set(req.quotation_id),
            contract_id: Set(req.contract_id),
            order_date: Set(req.order_date),
            delivery_date: Set(req.delivery_date),
            currency: Set(req.currency.or(Some(1))),
            exchange_rate: Set(req.exchange_rate.or(Some(Decimal::from(1)))),
            product_amount: Set(req.product_amount.or(Some(Decimal::from(0)))),
            discount_amount: Set(req.discount_amount.or(Some(Decimal::from(0)))),
            shipping_fee: Set(req.shipping_fee.or(Some(Decimal::from(0)))),
            tax_amount: Set(req.tax_amount.or(Some(Decimal::from(0)))),
            other_fee: Set(req.other_fee.or(Some(Decimal::from(0)))),
            total_amount: Set(req.total_amount.or(Some(Decimal::from(0)))),
            paid_amount: Set(req.paid_amount.or(Some(Decimal::from(0)))),
            unpaid_amount: Set(req.unpaid_amount.or(Some(Decimal::from(0)))),
            pay_status: Set(req.pay_status.or(Some(1))),
            payment_method: Set(req.payment_method),
            payment_due_date: Set(req.payment_due_date),
            shipping_method: Set(req.shipping_method),
            tracking_no: Set(req.tracking_no.clone()),
            shipping_time: Set(req.shipping_time),
            complete_time: Set(req.complete_time),
            receiver_name: Set(req.receiver_name.clone()),
            receiver_phone: Set(req.receiver_phone.clone()),
            shipping_address: Set(req.shipping_address.clone()),
            billing_address: Set(req.billing_address.clone()),
            remark: Set(req.remark.clone()),
            owner_user_id: Set(req.owner_user_id),
            dept_id: Set(req.dept_id),
            create_by: Set(req.create_by),
            create_time: Set(Some(now)),
            update_by: Set(req.update_by),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        SaleOrder::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: i64, req: &OrderSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = order::ActiveModel {
            update_time: Set(Some(now)),
            ..Default::default()
        };

        if let Some(v) = req.title.clone() { payload.title = Set(Some(v)); }
        if let Some(v) = req.order_type { payload.order_type = Set(Some(v)); }
        if let Some(v) = req.customer_id { payload.customer_id = Set(Some(v)); }
        if let Some(v) = req.customer_name.clone() { payload.customer_name = Set(Some(v)); }
        if let Some(v) = req.contact_id { payload.contact_id = Set(Some(v)); }
        if let Some(v) = req.contact_name.clone() { payload.contact_name = Set(Some(v)); }
        if let Some(v) = req.opportunity_id { payload.opportunity_id = Set(Some(v)); }
        if let Some(v) = req.quotation_id { payload.quotation_id = Set(Some(v)); }
        if let Some(v) = req.contract_id { payload.contract_id = Set(Some(v)); }
        if let Some(v) = req.order_date { payload.order_date = Set(Some(v)); }
        if let Some(v) = req.delivery_date { payload.delivery_date = Set(Some(v)); }
        if let Some(v) = req.currency { payload.currency = Set(Some(v)); }
        if let Some(v) = req.exchange_rate { payload.exchange_rate = Set(Some(v)); }
        if let Some(v) = req.product_amount { payload.product_amount = Set(Some(v)); }
        if let Some(v) = req.discount_amount { payload.discount_amount = Set(Some(v)); }
        if let Some(v) = req.shipping_fee { payload.shipping_fee = Set(Some(v)); }
        if let Some(v) = req.tax_amount { payload.tax_amount = Set(Some(v)); }
        if let Some(v) = req.other_fee { payload.other_fee = Set(Some(v)); }
        if let Some(v) = req.total_amount { payload.total_amount = Set(Some(v)); }
        if let Some(v) = req.unpaid_amount { payload.unpaid_amount = Set(Some(v)); }
        if let Some(v) = req.payment_method { payload.payment_method = Set(Some(v)); }
        if let Some(v) = req.payment_due_date { payload.payment_due_date = Set(Some(v)); }
        if let Some(v) = req.shipping_method { payload.shipping_method = Set(Some(v)); }
        if let Some(v) = req.tracking_no.clone() { payload.tracking_no = Set(Some(v)); }
        if let Some(v) = req.receiver_name.clone() { payload.receiver_name = Set(Some(v)); }
        if let Some(v) = req.receiver_phone.clone() { payload.receiver_phone = Set(Some(v)); }
        if let Some(v) = req.shipping_address.clone() { payload.shipping_address = Set(Some(v)); }
        if let Some(v) = req.billing_address.clone() { payload.billing_address = Set(Some(v)); }
        if let Some(v) = req.remark.clone() { payload.remark = Set(Some(v)); }
        if let Some(v) = req.owner_user_id { payload.owner_user_id = Set(Some(v)); }
        if let Some(v) = req.dept_id { payload.dept_id = Set(Some(v)); }
        if let Some(v) = req.update_by { payload.update_by = Set(Some(v)); }

        let result = SaleOrder::update_many()
            .set(payload)
            .filter(order::Column::Id.eq(id))
            .filter(order::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, order_status: i32, tracking_no: Option<String>) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = order::ActiveModel {
            order_status: Set(Some(order_status)),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        if let Some(tn) = tracking_no {
            payload.tracking_no = Set(Some(tn));
        }

        if order_status == 5 {
            payload.shipping_time = Set(Some(now));
        }
        if order_status == 6 {
            payload.complete_time = Set(Some(now));
        }

        let result = SaleOrder::update_many()
            .set(payload)
            .filter(order::Column::Id.eq(id))
            .filter(order::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn batch_delete_by_ids<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        SaleOrder::update_many()
            .set(order::ActiveModel {
                deleted: Set(Some(1)),
                update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(order::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<order::Model>, DbErr> {
        SaleOrder::find_by_id(id)
            .filter(order::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn get_max_order_no_today<C: ConnectionTrait>(db: &C, date_prefix: &str) -> Result<Option<i64>, DbErr> {
        use sea_orm::QuerySelect;
        use sea_orm::prelude::Expr;

        let pattern = format!("{}%", date_prefix);
        let result = SaleOrder::find()
            .filter(order::Column::OrderNo.like(&pattern))
            .select_only()
            .column_as(Expr::expr(Expr::cust("MAX(CAST(SUBSTRING(order_no, 11) AS INTEGER))")), "max_seq")
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
        order_status: Option<i32>,
        payment_status: Option<i32>,
        customer_id: Option<i64>,
        owner_user_id: Option<i64>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<(Vec<order::Model>, i64), DbErr> {
        let mut query = SaleOrder::find()
            .filter(order::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            if !k.trim().is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(order::Column::OrderNo.contains(k.trim()))
                        .add(order::Column::CustomerName.contains(k.trim()))
                        .add(order::Column::Title.contains(k.trim())),
                );
            }
        }
        if let Some(s) = order_status {
            query = query.filter(order::Column::OrderStatus.eq(s));
        }
        if let Some(p) = payment_status {
            query = query.filter(order::Column::PayStatus.eq(p));
        }
        if let Some(c) = customer_id {
            query = query.filter(order::Column::CustomerId.eq(c));
        }
        if let Some(o) = owner_user_id {
            query = query.filter(order::Column::OwnerUserId.eq(o));
        }
        if let Some(sd) = start_date {
            query = query.filter(order::Column::OrderDate.gte(sd));
        }
        if let Some(ed) = end_date {
            query = query.filter(order::Column::OrderDate.lte(ed));
        }

        let paginator = query.order_by_desc(order::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }
}

pub struct OrderItemModel;

impl OrderItemModel {
    pub async fn insert_batch<C: ConnectionTrait>(db: &C, order_id: i64, items: &Vec<OrderItemSaveDTO>) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let hundred = Decimal::from(100);
        let models: Vec<order_item::ActiveModel> = items.iter().enumerate().map(|(idx, item)| {
            let qty = item.quantity.unwrap_or(1);
            let price = item.unit_price.unwrap_or(Decimal::from(0));
            let disc_rate = item.discount_rate.unwrap_or(Decimal::from(100));
            let tax = item.tax_rate.unwrap_or(Decimal::from(0));

            let gross = price * Decimal::from(qty);
            let disc_amt = gross * (hundred - disc_rate) / hundred;
            let tax_amt = (gross - disc_amt) * tax / hundred;
            let line_amt = gross - disc_amt + tax_amt;
            let total_amt = item.total_amount.unwrap_or(line_amt);

            order_item::ActiveModel {
                order_id: Set(Some(order_id)),
                product_id: Set(item.product_id),
                product_name: Set(item.product_name.clone()),
                product_code: Set(item.product_code.clone()),
                sku: Set(item.sku.clone()),
                spec: Set(item.spec.clone()),
                unit: Set(item.unit.clone()),
                unit_id: Set(item.unit_id),
                quantity: Set(Some(qty)),
                unit_price: Set(Some(price)),
                discount_rate: Set(Some(disc_rate)),
                discount: Set(item.discount_amount),
                discount_amount: Set(item.discount_amount.or(Some(disc_amt))),
                tax_rate: Set(Some(tax)),
                tax_amount: Set(Some(tax_amt)),
                amount: Set(Some(line_amt)),
                total_amount: Set(Some(total_amt)),
                delivery_date: Set(item.delivery_date),
                delivered_quantity: Set(item.delivered_quantity),
                remark: Set(item.remark.clone()),
                sort: Set(item.sort.or(Some(idx as i32))),
                create_time: Set(Some(now)),
                update_time: Set(Some(now)),
                deleted: Set(Some(0)),
                ..Default::default()
            }
        }).collect();

        if models.is_empty() {
            return Ok(0);
        }

        let result = SaleOrderItem::insert_many(models)
            .exec(db)
            .await?;
        Ok(result.last_insert_id.unwrap_or_default())
    }

    pub async fn delete_by_order_id<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<i64, DbErr> {
        let result = SaleOrderItem::update_many()
            .set(order_item::ActiveModel {
                deleted: Set(Some(1)),
                update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(order_item::Column::OrderId.eq(order_id))
            .filter(order_item::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_order_id<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<Vec<order_item::Model>, DbErr> {
        SaleOrderItem::find()
            .filter(order_item::Column::OrderId.eq(order_id))
            .filter(order_item::Column::Deleted.eq(0))
            .order_by_asc(order_item::Column::Sort)
            .all(db)
            .await
    }
}
