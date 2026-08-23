//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 销售发货单模型层
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use sea_orm::prelude::{DateTime, Date};
use sea_orm::sea_query::Expr;
use sea_orm::QuerySelect;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::sale::entity::{order, shipment, shipment::Entity as SaleShipment, shipment_item, shipment_item::Entity as SaleShipmentItem, order_item};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use rust_decimal::Decimal;

// ==================== 请求 DTO ====================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentSaveRequest {
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    pub shipment_date: Option<Date>,
    pub logistics_company: Option<String>,
    pub tracking_no: Option<String>,
    pub shipping_method: Option<i32>,
    pub receiver_name: Option<String>,
    pub receiver_phone: Option<String>,
    pub shipping_address: Option<String>,
    pub remark: Option<String>,
    pub items: Option<Vec<ShipmentItemSaveDTO>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    pub shipment_date: Option<Date>,
    pub logistics_company: Option<String>,
    pub tracking_no: Option<String>,
    pub shipping_method: Option<i32>,
    pub receiver_name: Option<String>,
    pub receiver_phone: Option<String>,
    pub shipping_address: Option<String>,
    pub remark: Option<String>,
    pub items: Option<Vec<ShipmentItemSaveDTO>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub status: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub contract_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub list_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentItemSaveDTO {
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_item_id: Option<i64>,
    pub product_name: Option<String>,
    pub quantity: Option<i32>,
}

// ==================== 内部 DTO ====================

#[derive(Debug, Clone)]
pub struct ShipmentSaveDTO {
    pub shipment_no: Option<String>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub shipment_date: Option<Date>,
    pub logistics_company: Option<String>,
    pub tracking_no: Option<String>,
    pub shipping_method: Option<i32>,
    pub receiver_name: Option<String>,
    pub receiver_phone: Option<String>,
    pub shipping_address: Option<String>,
    pub total_quantity: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
}

// ==================== 响应 VO ====================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub shipment_no: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub shipment_date: Option<Date>,
    pub logistics_company: Option<String>,
    pub tracking_no: Option<String>,
    pub shipping_method: Option<i32>,
    pub receiver_name: Option<String>,
    pub receiver_phone: Option<String>,
    pub shipping_address: Option<String>,
    pub total_quantity: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub shipment_no: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub shipment_date: Option<Date>,
    pub logistics_company: Option<String>,
    pub tracking_no: Option<String>,
    pub shipping_method: Option<i32>,
    pub receiver_name: Option<String>,
    pub receiver_phone: Option<String>,
    pub shipping_address: Option<String>,
    pub total_quantity: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub items: Vec<ShipmentItemVO>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentItemVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub shipment_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_item_id: Option<i64>,
    pub product_name: Option<String>,
    pub quantity: Option<i32>,
    pub create_time: Option<DateTime>,
}

// ==================== From 转换 ====================

impl From<ShipmentSaveRequest> for ShipmentSaveDTO {
    fn from(req: ShipmentSaveRequest) -> Self {
        Self {
            shipment_no: None,
            order_id: req.order_id,
            customer_id: req.customer_id,
            shipment_date: req.shipment_date,
            logistics_company: req.logistics_company,
            tracking_no: req.tracking_no,
            shipping_method: req.shipping_method,
            receiver_name: req.receiver_name,
            receiver_phone: req.receiver_phone,
            shipping_address: req.shipping_address,
            total_quantity: None,
            status: None,
            remark: req.remark,
            created_by: None,
        }
    }
}

impl From<ShipmentUpdateRequest> for ShipmentSaveDTO {
    fn from(req: ShipmentUpdateRequest) -> Self {
        Self {
            shipment_no: None,
            order_id: req.order_id,
            customer_id: req.customer_id,
            shipment_date: req.shipment_date,
            logistics_company: req.logistics_company,
            tracking_no: req.tracking_no,
            shipping_method: req.shipping_method,
            receiver_name: req.receiver_name,
            receiver_phone: req.receiver_phone,
            shipping_address: req.shipping_address,
            total_quantity: None,
            status: None,
            remark: req.remark,
            created_by: None,
        }
    }
}

impl From<&shipment::Model> for ShipmentListVO {
    fn from(model: &shipment::Model) -> Self {
        Self {
            id: model.id.into(),
            shipment_no: model.shipment_no.clone(),
            order_id: model.order_id,
            customer_id: model.customer_id,
            shipment_date: model.shipment_date,
            logistics_company: model.logistics_company.clone(),
            tracking_no: model.tracking_no.clone(),
            shipping_method: model.shipping_method,
            receiver_name: model.receiver_name.clone(),
            receiver_phone: model.receiver_phone.clone(),
            shipping_address: model.shipping_address.clone(),
            total_quantity: model.total_quantity,
            status: model.status,
            remark: model.remark.clone(),
            created_by: model.created_by,
            create_time: model.create_time,
        }
    }
}

impl From<(&shipment::Model, Vec<shipment_item::Model>)> for ShipmentDetailVO {
    fn from(data: (&shipment::Model, Vec<shipment_item::Model>)) -> Self {
        let (model, items) = data;
        Self {
            id: model.id.into(),
            shipment_no: model.shipment_no.clone(),
            order_id: model.order_id,
            customer_id: model.customer_id,
            shipment_date: model.shipment_date,
            logistics_company: model.logistics_company.clone(),
            tracking_no: model.tracking_no.clone(),
            shipping_method: model.shipping_method,
            receiver_name: model.receiver_name.clone(),
            receiver_phone: model.receiver_phone.clone(),
            shipping_address: model.shipping_address.clone(),
            total_quantity: model.total_quantity,
            status: model.status,
            remark: model.remark.clone(),
            created_by: model.created_by,
            create_time: model.create_time,
            update_time: model.update_time,
            items: items.iter().map(|i| i.into()).collect(),
        }
    }
}

impl From<&shipment_item::Model> for ShipmentItemVO {
    fn from(model: &shipment_item::Model) -> Self {
        Self {
            id: model.id.into(),
            shipment_id: model.shipment_id,
            order_item_id: model.order_item_id,
            product_name: model.product_name.clone(),
            quantity: model.quantity,
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
        Some(Value::Null) => Ok(None),
        Some(_) => Err(D::Error::custom("expected string or number")),
        None => Ok(None),
    }
}

pub struct ShipmentModel;

impl ShipmentModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &ShipmentSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = shipment::ActiveModel {
            shipment_no: Set(req.shipment_no.clone()),
            order_id: Set(req.order_id),
            customer_id: Set(req.customer_id),
            shipment_date: Set(req.shipment_date),
            logistics_company: Set(req.logistics_company.clone()),
            tracking_no: Set(req.tracking_no.clone()),
            shipping_method: Set(req.shipping_method),
            receiver_name: Set(req.receiver_name.clone()),
            receiver_phone: Set(req.receiver_phone.clone()),
            shipping_address: Set(req.shipping_address.clone()),
            total_quantity: Set(req.total_quantity.or(Some(0))),
            status: Set(req.status.or(Some(1))),
            remark: Set(req.remark.clone()),
            created_by: Set(req.created_by),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        SaleShipment::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: i64, req: &ShipmentSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = shipment::ActiveModel {
            update_time: Set(Some(now)),
            ..Default::default()
        };

        if let Some(v) = req.order_id { payload.order_id = Set(Some(v)); }
        if let Some(v) = req.customer_id { payload.customer_id = Set(Some(v)); }
        if let Some(v) = req.shipment_date { payload.shipment_date = Set(Some(v)); }
        if let Some(v) = req.logistics_company.clone() { payload.logistics_company = Set(Some(v)); }
        if let Some(v) = req.tracking_no.clone() { payload.tracking_no = Set(Some(v)); }
        if let Some(v) = req.shipping_method { payload.shipping_method = Set(Some(v)); }
        if let Some(v) = req.receiver_name.clone() { payload.receiver_name = Set(Some(v)); }
        if let Some(v) = req.receiver_phone.clone() { payload.receiver_phone = Set(Some(v)); }
        if let Some(v) = req.shipping_address.clone() { payload.shipping_address = Set(Some(v)); }
        if let Some(v) = req.total_quantity { payload.total_quantity = Set(Some(v)); }
        if let Some(v) = req.remark.clone() { payload.remark = Set(Some(v)); }

        let result = SaleShipment::update_many()
            .set(payload)
            .filter(shipment::Column::Id.eq(id))
            .filter(shipment::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i32) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result = SaleShipment::update_many()
            .set(shipment::ActiveModel {
                status: Set(Some(status)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(shipment::Column::Id.eq(id))
            .filter(shipment::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn soft_delete<C: ConnectionTrait>(db: &C, id: i64) -> Result<i64, DbErr> {
        let result = SaleShipment::update_many()
            .set(shipment::ActiveModel {
                deleted: Set(Some(1)),
                update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(shipment::Column::Id.eq(id))
            .filter(shipment::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<shipment::Model>, DbErr> {
        SaleShipment::find_by_id(id)
            .filter(shipment::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn find_by_order_id<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<Vec<shipment::Model>, DbErr> {
        SaleShipment::find()
            .filter(shipment::Column::OrderId.eq(order_id))
            .filter(shipment::Column::Deleted.eq(0))
            .order_by_desc(shipment::Column::CreateTime)
            .all(db)
            .await
    }

    pub async fn get_max_shipment_no_today<C: ConnectionTrait>(db: &C, date_prefix: &str) -> Result<Option<i64>, DbErr> {
        use sea_orm::QuerySelect;

        let pattern = format!("{}%", date_prefix);
        let result = SaleShipment::find()
            .filter(shipment::Column::ShipmentNo.like(&pattern))
            .select_only()
            .column_as(Expr::expr(Expr::cust("MAX(CAST(SUBSTRING(shipment_no, 11) AS BIGINT))")), "max_seq")
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
        status: Option<i32>,
        order_id: Option<i64>,
        customer_id: Option<i64>,
        contract_id: Option<i64>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<(Vec<shipment::Model>, i64), DbErr> {
        let mut query = SaleShipment::find()
            .filter(shipment::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            if !k.trim().is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(shipment::Column::ShipmentNo.contains(k.trim()))
                        .add(shipment::Column::ReceiverName.contains(k.trim()))
                        .add(shipment::Column::TrackingNo.contains(k.trim())),
                );
            }
        }
        if let Some(s) = status {
            query = query.filter(shipment::Column::Status.eq(s));
        }
        if let Some(o) = order_id {
            query = query.filter(shipment::Column::OrderId.eq(o));
        }
        if let Some(c) = customer_id {
            query = query.filter(shipment::Column::CustomerId.eq(c));
        }
        if let Some(cid) = contract_id {
            // 通过关联订单(contract_id)过滤该合同下的发货单
            let order_ids: Vec<i64> = order::Entity::find()
                .select_only()
                .column(order::Column::Id)
                .filter(order::Column::ContractId.eq(cid))
                .filter(order::Column::Deleted.eq(0))
                .into_tuple::<Option<i64>>()
                .all(db)
                .await?
                .into_iter()
                .flatten()
                .collect();
            if order_ids.is_empty() {
                // 该合同无关联订单，直接返回空结果
                query = query.filter(shipment::Column::OrderId.eq(-1));
            } else {
                query = query.filter(shipment::Column::OrderId.is_in(order_ids));
            }
        }
        if let Some(sd) = start_date {
            query = query.filter(shipment::Column::ShipmentDate.gte(sd));
        }
        if let Some(ed) = end_date {
            query = query.filter(shipment::Column::ShipmentDate.lte(ed));
        }

        let paginator = query.order_by_desc(shipment::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }

    pub async fn select_in_page_by_owner_user_ids<C: ConnectionTrait>(
        db: &C,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        status: Option<i32>,
        order_id: Option<i64>,
        customer_id: Option<i64>,
        contract_id: Option<i64>,
        start_date: Option<String>,
        end_date: Option<String>,
        owner_user_ids: Option<Vec<i64>>,
    ) -> Result<(Vec<shipment::Model>, i64), DbErr> {
        let mut query = SaleShipment::find()
            .filter(shipment::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            if !k.trim().is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(shipment::Column::ShipmentNo.contains(k.trim()))
                        .add(shipment::Column::ReceiverName.contains(k.trim()))
                        .add(shipment::Column::TrackingNo.contains(k.trim())),
                );
            }
        }
        if let Some(s) = status {
            query = query.filter(shipment::Column::Status.eq(s));
        }
        if let Some(o) = order_id {
            query = query.filter(shipment::Column::OrderId.eq(o));
        }
        if let Some(c) = customer_id {
            query = query.filter(shipment::Column::CustomerId.eq(c));
        }
        if let Some(cid) = contract_id {
            let order_ids: Vec<i64> = order::Entity::find()
                .select_only()
                .column(order::Column::Id)
                .filter(order::Column::ContractId.eq(cid))
                .filter(order::Column::Deleted.eq(0))
                .into_tuple::<Option<i64>>()
                .all(db)
                .await?
                .into_iter()
                .flatten()
                .collect();
            if order_ids.is_empty() {
                query = query.filter(shipment::Column::OrderId.eq(-1));
            } else {
                query = query.filter(shipment::Column::OrderId.is_in(order_ids));
            }
        }
        if let Some(sd) = start_date {
            query = query.filter(shipment::Column::ShipmentDate.gte(sd));
        }
        if let Some(ed) = end_date {
            query = query.filter(shipment::Column::ShipmentDate.lte(ed));
        }

        if let Some(ids) = owner_user_ids {
            if ids.is_empty() {
                return Ok((Vec::new(), 0));
            }
            let order_ids: Vec<i64> = order::Entity::find()
                .select_only()
                .column(order::Column::Id)
                .filter(order::Column::OwnerUserId.is_in(ids))
                .filter(order::Column::Deleted.eq(0))
                .into_tuple::<Option<i64>>()
                .all(db)
                .await?
                .into_iter()
                .flatten()
                .collect();
            if order_ids.is_empty() {
                return Ok((Vec::new(), 0));
            }
            query = query.filter(shipment::Column::OrderId.is_in(order_ids));
        }

        let paginator = query.order_by_desc(shipment::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }
}

pub struct ShipmentItemModel;

impl ShipmentItemModel {
    pub async fn insert_batch<C: ConnectionTrait>(db: &C, shipment_id: i64, items: &Vec<ShipmentItemSaveDTO>) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let models: Vec<shipment_item::ActiveModel> = items.iter().map(|item| {
            shipment_item::ActiveModel {
                shipment_id: Set(Some(shipment_id)),
                order_item_id: Set(item.order_item_id),
                product_name: Set(item.product_name.clone()),
                quantity: Set(item.quantity.or(Some(0))),
                create_time: Set(Some(now)),
                ..Default::default()
            }
        }).collect();

        if models.is_empty() {
            return Ok(0);
        }

        let result = SaleShipmentItem::insert_many(models)
            .exec(db)
            .await?;
        Ok(result.last_insert_id.unwrap_or_default())
    }

    pub async fn delete_by_shipment_id<C: ConnectionTrait>(db: &C, shipment_id: i64) -> Result<i64, DbErr> {
        let result = SaleShipmentItem::delete_many()
            .filter(shipment_item::Column::ShipmentId.eq(shipment_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_shipment_id<C: ConnectionTrait>(db: &C, shipment_id: i64) -> Result<Vec<shipment_item::Model>, DbErr> {
        SaleShipmentItem::find()
            .filter(shipment_item::Column::ShipmentId.eq(shipment_id))
            .order_by_asc(shipment_item::Column::Id)
            .all(db)
            .await
    }

    pub async fn find_by_order_id<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<Vec<shipment_item::Model>, DbErr> {
        // 通过 shipment 关联查询订单的所有发货明细
        let shipment_ids: Vec<i64> = SaleShipment::find()
            .filter(shipment::Column::OrderId.eq(order_id))
            .filter(shipment::Column::Deleted.eq(0))
            .all(db)
            .await?
            .into_iter()
            .filter_map(|s| s.id.into())
            .collect();

        if shipment_ids.is_empty() {
            return Ok(vec![]);
        }

        SaleShipmentItem::find()
            .filter(shipment_item::Column::ShipmentId.is_in(shipment_ids))
            .all(db)
            .await
    }

    /// 累加更新订单明细的已发数量（delivered_quantity 为 numeric 类型，使用 COALESCE 处理 NULL）
    pub async fn add_delivered_quantity<C: ConnectionTrait>(
        db: &C,
        order_item_id: i64,
        add_qty: i32,
    ) -> Result<i64, DbErr> {
        let add_decimal = Decimal::from(add_qty);
        // COALESCE(delivered_quantity, 0) + add_qty 避免 NULL + qty = NULL 的问题
        let result = order_item::Entity::update_many()
            .col_expr(
                order_item::Column::DeliveredQuantity,
                Expr::cust("COALESCE(delivered_quantity, 0)").add(Expr::value(add_decimal)),
            )
            .filter(order_item::Column::Id.eq(order_item_id))
            .filter(order_item::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 回滚订单明细的已发数量（减去对应数量，不低于0）
    pub async fn sub_delivered_quantity<C: ConnectionTrait>(
        db: &C,
        order_item_id: i64,
        sub_qty: i32,
    ) -> Result<i64, DbErr> {
        let sub_decimal = Decimal::from(sub_qty);
        // 先查询当前已发数量
        let item = order_item::Entity::find_by_id(order_item_id)
            .one(db)
            .await?;
        if let Some(model) = item {
            let current = model.delivered_quantity.unwrap_or(Decimal::from(0));
            let new_val = if current > sub_decimal {
                current - sub_decimal
            } else {
                Decimal::from(0)
            };
            let result = order_item::Entity::update_many()
                .col_expr(
                    order_item::Column::DeliveredQuantity,
                    Expr::value(new_val),
                )
                .filter(order_item::Column::Id.eq(order_item_id))
                .filter(order_item::Column::Deleted.eq(0))
                .exec(db)
                .await?;
            return Ok(result.rows_affected as i64);
        }
        Ok(0)
    }
}
