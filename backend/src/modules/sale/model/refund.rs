//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 销售退货单模型层
//!

use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::sale::entity::{refund, refund::Entity as SaleRefund, refund_item, refund_item::Entity as SaleRefundItem, refund_payment, refund_payment::Entity as SaleRefundPayment};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

// ==================== 请求 DTO ====================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundSaveRequest {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub refund_type: Option<i16>,
    pub refund_reason: Option<String>,
    pub restocking_fee: Option<Decimal>,
    pub warehouse_id: Option<i64>,
    pub receiver: Option<String>,
    pub receiver_phone: Option<String>,
    pub receiver_address: Option<String>,
    pub remark: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub owner_user_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub dept_id: Option<i64>,
    pub items: Option<Vec<RefundItemSaveDTO>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub refund_type: Option<i16>,
    pub refund_reason: Option<String>,
    pub restocking_fee: Option<Decimal>,
    pub warehouse_id: Option<i64>,
    pub receiver: Option<String>,
    pub receiver_phone: Option<String>,
    pub receiver_address: Option<String>,
    pub remark: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub owner_user_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub dept_id: Option<i64>,
    pub items: Option<Vec<RefundItemSaveDTO>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundListQuery {
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub refund_status: Option<i16>,
    pub approval_status: Option<i16>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub owner_user_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub list_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundApprovalReq {
    pub refund_id: i64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundQualityCheckReq {
    pub refund_id: i64,
    pub quality_check_result: i16,
    pub quality_check_remark: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundReceiveReq {
    pub refund_id: i64,
    pub logistics_no: Option<String>,
    pub logistics_company: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundPaymentRequest {
    pub refund_id: i64,
    pub payment_method: Option<i32>,
    pub payment_amount: Option<Decimal>,
    pub payment_account: Option<String>,
    pub transaction_no: Option<String>,
    pub remark: Option<String>,
}

// ==================== 内部 DTO ====================

#[derive(Debug, Clone)]
pub struct RefundSaveDTO {
    pub refund_no: Option<String>,
    pub title: Option<String>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub refund_type: Option<i16>,
    pub refund_reason: Option<String>,
    pub refund_status: Option<i16>,
    pub approval_status: Option<i16>,
    pub total_amount: Option<Decimal>,
    pub restocking_fee: Option<Decimal>,
    pub refund_amount: Option<Decimal>,
    pub refunded_amount: Option<Decimal>,
    pub warehouse_id: Option<i64>,
    pub receiver: Option<String>,
    pub receiver_phone: Option<String>,
    pub receiver_address: Option<String>,
    pub remark: Option<String>,
    pub owner_user_id: Option<i64>,
    pub dept_id: Option<i64>,
    pub create_by: Option<i64>,
    pub update_by: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundItemSaveDTO {
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_item_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub refund_qty: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub refund_amount: Option<Decimal>,
}

// ==================== 响应 VO ====================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub refund_no: Option<String>,
    pub title: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    pub order_no: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub refund_type: Option<i16>,
    pub refund_status: Option<i16>,
    pub approval_status: Option<i16>,
    pub total_amount: Option<Decimal>,
    pub restocking_fee: Option<Decimal>,
    pub refund_amount: Option<Decimal>,
    pub refunded_amount: Option<Decimal>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub owner_user_id: Option<i64>,
    pub owner_user_name: Option<String>,
    pub create_time: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub refund_no: Option<String>,
    pub title: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    pub order_no: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub refund_type: Option<i16>,
    pub refund_reason: Option<String>,
    pub refund_status: Option<i16>,
    pub approval_status: Option<i16>,
    pub instance_id: Option<i64>,
    pub total_amount: Option<Decimal>,
    pub restocking_fee: Option<Decimal>,
    pub refund_amount: Option<Decimal>,
    pub refunded_amount: Option<Decimal>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub warehouse_id: Option<i64>,
    pub receiver: Option<String>,
    pub receiver_phone: Option<String>,
    pub receiver_address: Option<String>,
    pub logistics_no: Option<String>,
    pub logistics_company: Option<String>,
    pub quality_check_result: Option<i16>,
    pub quality_check_remark: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub owner_user_id: Option<i64>,
    pub owner_user_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub dept_id: Option<i64>,
    pub remark: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub update_by: Option<i64>,
    pub update_time: Option<DateTime>,
    pub items: Vec<RefundItemVO>,
    pub payments: Vec<RefundPaymentVO>,
    /// 提示信息（如已开票订单退货需红冲处理等）
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundItemVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub refund_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_item_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub refund_qty: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub refund_amount: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundPaymentVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub refund_id: Option<i64>,
    pub payment_no: Option<String>,
    pub payment_method: Option<i32>,
    pub payment_amount: Option<Decimal>,
    pub payment_time: Option<DateTime>,
    pub payment_account: Option<String>,
    pub transaction_no: Option<String>,
    pub remark: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
}

// ==================== From 转换 ====================

impl From<RefundSaveRequest> for RefundSaveDTO {
    fn from(req: RefundSaveRequest) -> Self {
        Self {
            refund_no: None,
            title: req.title,
            order_id: req.order_id,
            customer_id: req.customer_id,
            customer_name: req.customer_name,
            refund_type: req.refund_type,
            refund_reason: req.refund_reason,
            refund_status: None,
            approval_status: None,
            total_amount: None,
            restocking_fee: req.restocking_fee,
            refund_amount: None,
            refunded_amount: None,
            warehouse_id: req.warehouse_id,
            receiver: req.receiver,
            receiver_phone: req.receiver_phone,
            receiver_address: req.receiver_address,
            remark: req.remark,
            owner_user_id: req.owner_user_id,
            dept_id: req.dept_id,
            create_by: None,
            update_by: None,
        }
    }
}

impl From<RefundUpdateRequest> for RefundSaveDTO {
    fn from(req: RefundUpdateRequest) -> Self {
        Self {
            refund_no: None,
            title: req.title,
            order_id: req.order_id,
            customer_id: req.customer_id,
            customer_name: req.customer_name,
            refund_type: req.refund_type,
            refund_reason: req.refund_reason,
            refund_status: None,
            approval_status: None,
            total_amount: None,
            restocking_fee: req.restocking_fee,
            refund_amount: None,
            refunded_amount: None,
            warehouse_id: req.warehouse_id,
            receiver: req.receiver,
            receiver_phone: req.receiver_phone,
            receiver_address: req.receiver_address,
            remark: req.remark,
            owner_user_id: req.owner_user_id,
            dept_id: req.dept_id,
            create_by: None,
            update_by: None,
        }
    }
}

impl From<&refund::Model> for RefundListVO {
    fn from(model: &refund::Model) -> Self {
        Self {
            id: model.id.into(),
            refund_no: model.refund_no.clone(),
            title: model.title.clone(),
            order_id: model.order_id,
            order_no: None,
            customer_id: model.customer_id,
            customer_name: model.customer_name.clone(),
            refund_type: model.refund_type,
            refund_status: model.refund_status,
            approval_status: model.approval_status,
            total_amount: model.total_amount,
            restocking_fee: model.restocking_fee,
            refund_amount: model.refund_amount,
            refunded_amount: model.refunded_amount,
            owner_user_id: model.owner_user_id,
            owner_user_name: None,
            create_time: model.create_time,
        }
    }
}

impl From<&refund_item::Model> for RefundItemVO {
    fn from(model: &refund_item::Model) -> Self {
        Self {
            id: model.id.into(),
            refund_id: model.refund_id,
            order_item_id: model.order_item_id,
            product_id: model.product_id,
            product_name: model.product_name.clone(),
            spec: model.spec.clone(),
            unit: model.unit.clone(),
            refund_qty: model.refund_qty,
            unit_price: model.unit_price,
            refund_amount: model.refund_amount,
        }
    }
}

impl From<&refund_payment::Model> for RefundPaymentVO {
    fn from(model: &refund_payment::Model) -> Self {
        Self {
            id: model.id.into(),
            refund_id: model.refund_id,
            payment_no: model.payment_no.clone(),
            payment_method: model.payment_method,
            payment_amount: model.payment_amount,
            payment_time: model.payment_time,
            payment_account: model.payment_account.clone(),
            transaction_no: model.transaction_no.clone(),
            remark: model.remark.clone(),
            create_by: model.create_by,
            create_time: model.create_time,
        }
    }
}

// ==================== 数据库操作方法 ====================

fn deserialize_option_string_to_u64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    match Option::<Value>::deserialize(deserializer)? {
        Some(Value::String(s)) => {
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<i64>().map(Some).map_err(D::Error::custom)
            }
        }
        Some(Value::Number(n)) => Ok(n.as_i64()),
        Some(_) => Err(D::Error::custom("expected string or number")),
        None => Ok(None),
    }
}

pub struct RefundModel;

impl RefundModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &RefundSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = refund::ActiveModel {
            refund_no: Set(req.refund_no.clone()),
            title: Set(req.title.clone()),
            order_id: Set(req.order_id),
            customer_id: Set(req.customer_id),
            customer_name: Set(req.customer_name.clone()),
            refund_type: Set(req.refund_type.or(Some(2i16))),
            refund_reason: Set(req.refund_reason.clone()),
            refund_status: Set(req.refund_status.or(Some(1i16))),
            approval_status: Set(req.approval_status.or(Some(0i16))),
            instance_id: Set(None),
            total_amount: Set(req.total_amount.or(Some(Decimal::from(0)))),
            restocking_fee: Set(req.restocking_fee.or(Some(Decimal::from(0)))),
            refund_amount: Set(req.refund_amount.or(Some(Decimal::from(0)))),
            refunded_amount: Set(req.refunded_amount.or(Some(Decimal::from(0)))),
            warehouse_id: Set(req.warehouse_id),
            receiver: Set(req.receiver.clone()),
            receiver_phone: Set(req.receiver_phone.clone()),
            receiver_address: Set(req.receiver_address.clone()),
            logistics_no: Set(None),
            logistics_company: Set(None),
            quality_check_result: Set(Some(0i16)),
            quality_check_remark: Set(None),
            owner_user_id: Set(req.owner_user_id),
            dept_id: Set(req.dept_id),
            remark: Set(req.remark.clone()),
            create_by: Set(req.create_by),
            create_time: Set(Some(now)),
            update_by: Set(req.update_by),
            update_time: Set(Some(now)),
            deleted: Set(Some(0i16)),
            ..Default::default()
        };
        SaleRefund::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: i64, req: &RefundSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = refund::ActiveModel {
            update_time: Set(Some(now)),
            ..Default::default()
        };

        if let Some(v) = req.title.clone() { payload.title = Set(Some(v)); }
        if let Some(v) = req.order_id { payload.order_id = Set(Some(v)); }
        if let Some(v) = req.customer_id { payload.customer_id = Set(Some(v)); }
        if let Some(v) = req.customer_name.clone() { payload.customer_name = Set(Some(v)); }
        if let Some(v) = req.refund_type { payload.refund_type = Set(Some(v)); }
        if let Some(v) = req.refund_reason.clone() { payload.refund_reason = Set(Some(v)); }
        if let Some(v) = req.total_amount { payload.total_amount = Set(Some(v)); }
        if let Some(v) = req.restocking_fee { payload.restocking_fee = Set(Some(v)); }
        if let Some(v) = req.refund_amount { payload.refund_amount = Set(Some(v)); }
        if let Some(v) = req.warehouse_id { payload.warehouse_id = Set(Some(v)); }
        if let Some(v) = req.receiver.clone() { payload.receiver = Set(Some(v)); }
        if let Some(v) = req.receiver_phone.clone() { payload.receiver_phone = Set(Some(v)); }
        if let Some(v) = req.receiver_address.clone() { payload.receiver_address = Set(Some(v)); }
        if let Some(v) = req.remark.clone() { payload.remark = Set(Some(v)); }
        if let Some(v) = req.owner_user_id { payload.owner_user_id = Set(Some(v)); }
        if let Some(v) = req.dept_id { payload.dept_id = Set(Some(v)); }
        if let Some(v) = req.update_by { payload.update_by = Set(Some(v)); }

        let result = SaleRefund::update_many()
            .set(payload)
            .filter(refund::Column::Id.eq(id))
            .filter(refund::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, refund_status: i16) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result = SaleRefund::update_many()
            .set(refund::ActiveModel {
                refund_status: Set(Some(refund_status)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(refund::Column::Id.eq(id))
            .filter(refund::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_approval<C: ConnectionTrait>(db: &C, id: i64, approval_status: i16, instance_id: Option<i64>) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = refund::ActiveModel {
            approval_status: Set(Some(approval_status)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        if let Some(iid) = instance_id {
            payload.instance_id = Set(Some(iid));
        }
        let result = SaleRefund::update_many()
            .set(payload)
            .filter(refund::Column::Id.eq(id))
            .filter(refund::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_quality_check<C: ConnectionTrait>(db: &C, id: i64, result: i16, remark: Option<String>) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = refund::ActiveModel {
            quality_check_result: Set(Some(result)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        if let Some(r) = remark {
            payload.quality_check_remark = Set(Some(r));
        }
        let res = SaleRefund::update_many()
            .set(payload)
            .filter(refund::Column::Id.eq(id))
            .filter(refund::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(res.rows_affected as i64)
    }

    pub async fn update_logistics<C: ConnectionTrait>(db: &C, id: i64, logistics_no: Option<String>, logistics_company: Option<String>) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result = SaleRefund::update_many()
            .set(refund::ActiveModel {
                logistics_no: Set(logistics_no),
                logistics_company: Set(logistics_company),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(refund::Column::Id.eq(id))
            .filter(refund::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_payment_amount<C: ConnectionTrait>(db: &C, id: i64, refunded_amount: Decimal) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result = SaleRefund::update_many()
            .set(refund::ActiveModel {
                refunded_amount: Set(Some(refunded_amount)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(refund::Column::Id.eq(id))
            .filter(refund::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn batch_delete_by_ids<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        SaleRefund::update_many()
            .set(refund::ActiveModel {
                deleted: Set(Some(1i16)),
                update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(refund::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<refund::Model>, DbErr> {
        SaleRefund::find_by_id(id)
            .filter(refund::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 同客户同标题排重校验
    pub async fn find_by_customer_and_title<C: ConnectionTrait>(
        db: &C,
        customer_id: i64,
        title: &str,
        exclude_id: Option<i64>,
    ) -> Result<Option<refund::Model>, DbErr> {
        let mut query = SaleRefund::find()
            .filter(refund::Column::CustomerId.eq(customer_id))
            .filter(refund::Column::Title.eq(title))
            .filter(refund::Column::Deleted.eq(0));

        if let Some(id) = exclude_id {
            query = query.filter(refund::Column::Id.ne(id));
        }

        query.one(db).await
    }

    /// 同订单同标题排重校验（退货单针对同一订单不允许重复标题）
    pub async fn find_by_order_and_title<C: ConnectionTrait>(
        db: &C,
        order_id: i64,
        title: &str,
        exclude_id: Option<i64>,
    ) -> Result<Option<refund::Model>, DbErr> {
        let mut query = SaleRefund::find()
            .filter(refund::Column::OrderId.eq(order_id))
            .filter(refund::Column::Title.eq(title))
            .filter(refund::Column::Deleted.eq(0));

        if let Some(id) = exclude_id {
            query = query.filter(refund::Column::Id.ne(id));
        }

        query.one(db).await
    }

    pub async fn get_max_refund_no_today<C: ConnectionTrait>(db: &C, date_prefix: &str) -> Result<Option<i64>, DbErr> {
        use sea_orm::QuerySelect;
        use sea_orm::prelude::Expr;

        let pattern = format!("{}%", date_prefix);
        let result = SaleRefund::find()
            .filter(refund::Column::RefundNo.like(&pattern))
            .select_only()
            .column_as(Expr::expr(Expr::cust("MAX(CAST(SUBSTRING(refund_no, 11) AS BIGINT))")), "max_seq")
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
        refund_status: Option<i16>,
        approval_status: Option<i16>,
        customer_id: Option<i64>,
        order_id: Option<i64>,
        owner_user_id: Option<i64>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<(Vec<refund::Model>, i64), DbErr> {
        let mut query = SaleRefund::find()
            .filter(refund::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            if !k.trim().is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(refund::Column::RefundNo.contains(k.trim()))
                        .add(refund::Column::CustomerName.contains(k.trim()))
                        .add(refund::Column::Title.contains(k.trim())),
                );
            }
        }
        if let Some(s) = refund_status {
            query = query.filter(refund::Column::RefundStatus.eq(s));
        }
        if let Some(s) = approval_status {
            query = query.filter(refund::Column::ApprovalStatus.eq(s));
        }
        if let Some(c) = customer_id {
            query = query.filter(refund::Column::CustomerId.eq(c));
        }
        if let Some(o) = order_id {
            query = query.filter(refund::Column::OrderId.eq(o));
        }
        if let Some(o) = owner_user_id {
            query = query.filter(refund::Column::OwnerUserId.eq(o));
        }
        if let Some(sd) = start_date {
            if let Ok(d) = sd.parse::<chrono::NaiveDate>() {
                query = query.filter(refund::Column::CreateTime.gte(d.and_hms_opt(0, 0, 0).unwrap()));
            }
        }
        if let Some(ed) = end_date {
            if let Ok(d) = ed.parse::<chrono::NaiveDate>() {
                query = query.filter(refund::Column::CreateTime.lte(d.and_hms_opt(23, 59, 59).unwrap()));
            }
        }

        let paginator = query.order_by_desc(refund::Column::Id).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }

    pub async fn select_in_page_by_owner_user_ids<C: ConnectionTrait>(
        db: &C,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        refund_status: Option<i16>,
        approval_status: Option<i16>,
        customer_id: Option<i64>,
        order_id: Option<i64>,
        start_date: Option<String>,
        end_date: Option<String>,
        owner_user_ids: Option<Vec<i64>>,
    ) -> Result<(Vec<refund::Model>, i64), DbErr> {
        let mut query = SaleRefund::find()
            .filter(refund::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            if !k.trim().is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(refund::Column::RefundNo.contains(k.trim()))
                        .add(refund::Column::CustomerName.contains(k.trim()))
                        .add(refund::Column::Title.contains(k.trim())),
                );
            }
        }
        if let Some(s) = refund_status {
            query = query.filter(refund::Column::RefundStatus.eq(s));
        }
        if let Some(s) = approval_status {
            query = query.filter(refund::Column::ApprovalStatus.eq(s));
        }
        if let Some(c) = customer_id {
            query = query.filter(refund::Column::CustomerId.eq(c));
        }
        if let Some(o) = order_id {
            query = query.filter(refund::Column::OrderId.eq(o));
        }
        if let Some(sd) = start_date {
            if let Ok(d) = sd.parse::<chrono::NaiveDate>() {
                query = query.filter(refund::Column::CreateTime.gte(d.and_hms_opt(0, 0, 0).unwrap()));
            }
        }
        if let Some(ed) = end_date {
            if let Ok(d) = ed.parse::<chrono::NaiveDate>() {
                query = query.filter(refund::Column::CreateTime.lte(d.and_hms_opt(23, 59, 59).unwrap()));
            }
        }
        if let Some(ids) = owner_user_ids {
            if ids.is_empty() {
                return Ok((Vec::new(), 0));
            }
            query = query.filter(refund::Column::OwnerUserId.is_in(ids));
        }

        let paginator = query.order_by_desc(refund::Column::Id).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }
}

pub struct RefundItemModel;

impl RefundItemModel {
    pub async fn insert_batch<C: ConnectionTrait>(db: &C, refund_id: i64, items: &Vec<RefundItemSaveDTO>) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let models: Vec<refund_item::ActiveModel> = items.iter().map(|item| {
            let qty = item.refund_qty.unwrap_or(Decimal::from(0));
            let price = item.unit_price.unwrap_or(Decimal::from(0));
            let amt = item.refund_amount.unwrap_or(qty * price);

            refund_item::ActiveModel {
                refund_id: Set(Some(refund_id)),
                order_item_id: Set(item.order_item_id),
                product_id: Set(item.product_id),
                product_name: Set(item.product_name.clone()),
                spec: Set(item.spec.clone()),
                unit: Set(item.unit.clone()),
                refund_qty: Set(Some(qty)),
                unit_price: Set(Some(price)),
                refund_amount: Set(Some(amt)),
                create_time: Set(Some(now)),
                ..Default::default()
            }
        }).collect();

        if models.is_empty() {
            return Ok(0);
        }

        let result = SaleRefundItem::insert_many(models)
            .exec(db)
            .await?;
        Ok(result.last_insert_id.unwrap_or_default())
    }

    pub async fn delete_by_refund_id<C: ConnectionTrait>(db: &C, refund_id: i64) -> Result<i64, DbErr> {
        let result = SaleRefundItem::delete_many()
            .filter(refund_item::Column::RefundId.eq(refund_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_refund_id<C: ConnectionTrait>(db: &C, refund_id: i64) -> Result<Vec<refund_item::Model>, DbErr> {
        SaleRefundItem::find()
            .filter(refund_item::Column::RefundId.eq(refund_id))
            .all(db)
            .await
    }
}

pub struct RefundPaymentModel;

impl RefundPaymentModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, refund_id: i64, payment_no: String, req: &RefundPaymentRequest, create_by: i64) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = refund_payment::ActiveModel {
            refund_id: Set(Some(refund_id)),
            payment_no: Set(Some(payment_no)),
            payment_method: Set(req.payment_method),
            payment_amount: Set(req.payment_amount),
            payment_time: Set(Some(now)),
            payment_account: Set(req.payment_account.clone()),
            transaction_no: Set(req.transaction_no.clone()),
            remark: Set(req.remark.clone()),
            create_by: Set(Some(create_by)),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        SaleRefundPayment::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn find_by_refund_id<C: ConnectionTrait>(db: &C, refund_id: i64) -> Result<Vec<refund_payment::Model>, DbErr> {
        SaleRefundPayment::find()
            .filter(refund_payment::Column::RefundId.eq(refund_id))
            .order_by_desc(refund_payment::Column::Id)
            .all(db)
            .await
    }

    pub async fn sum_by_refund_id<C: ConnectionTrait>(db: &C, refund_id: i64) -> Result<Decimal, DbErr> {
        use sea_orm::prelude::Expr;
        let result = SaleRefundPayment::find()
            .filter(refund_payment::Column::RefundId.eq(refund_id))
            .select_only()
            .column_as(Expr::col(refund_payment::Column::PaymentAmount).sum(), "total")
            .into_tuple::<Option<Decimal>>()
            .one(db)
            .await?;
        Ok(result.flatten().unwrap_or(Decimal::from(0)))
    }

    pub async fn get_max_payment_no_today<C: ConnectionTrait>(db: &C, date_prefix: &str) -> Result<Option<i64>, DbErr> {
        use sea_orm::QuerySelect;
        use sea_orm::prelude::Expr;

        let pattern = format!("{}%", date_prefix);
        let result = SaleRefundPayment::find()
            .filter(refund_payment::Column::PaymentNo.like(&pattern))
            .select_only()
            .column_as(Expr::expr(Expr::cust("MAX(CAST(SUBSTRING(payment_no, 11) AS BIGINT))")), "max_seq")
            .into_tuple::<Option<i64>>()
            .one(db)
            .await?;

        Ok(result.flatten())
    }
}
