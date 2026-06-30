use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal, Date};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::sale::entity::{order_item, order_item::Entity as OrderItem};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 订单明细新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderItemSaveRequest {
    pub order_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub sku: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
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

impl From<OrderItemSaveRequest> for OrderItemSaveDTO {
    fn from(item: OrderItemSaveRequest) -> Self {
        OrderItemSaveDTO {
            id: None,
            order_id: item.order_id,
            product_id: item.product_id,
            product_name: item.product_name,
            product_code: item.product_code,
            sku: item.sku,
            spec: item.spec,
            unit: item.unit,
            unit_id: item.unit_id,
            quantity: item.quantity,
            unit_price: item.unit_price,
            discount_rate: item.discount_rate,
            discount: item.discount,
            discount_amount: item.discount_amount,
            tax_rate: item.tax_rate,
            tax_amount: item.tax_amount,
            amount: item.amount,
            total_amount: item.total_amount,
            delivery_date: item.delivery_date,
            delivered_quantity: item.delivered_quantity,
            remark: item.remark,
            sort: item.sort,
            deleted: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
        }
    }
}

/// 订单明细更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderItemUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub order_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub sku: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
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

impl From<OrderItemUpdateRequest> for OrderItemSaveDTO {
    fn from(item: OrderItemUpdateRequest) -> Self {
        OrderItemSaveDTO {
            id: item.id,
            order_id: item.order_id,
            product_id: item.product_id,
            product_name: item.product_name,
            product_code: item.product_code,
            sku: item.sku,
            spec: item.spec,
            unit: item.unit,
            unit_id: item.unit_id,
            quantity: item.quantity,
            unit_price: item.unit_price,
            discount_rate: item.discount_rate,
            discount: item.discount,
            discount_amount: item.discount_amount,
            tax_rate: item.tax_rate,
            tax_amount: item.tax_amount,
            amount: item.amount,
            total_amount: item.total_amount,
            delivery_date: item.delivery_date,
            delivered_quantity: item.delivered_quantity,
            remark: item.remark,
            sort: item.sort,
            deleted: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
        }
    }
}

/// 订单明细保存DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderItemSaveDTO {
    pub id: Option<i64>,
    pub order_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub sku: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
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
    pub deleted: Option<i32>,
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<i64>,
    pub update_time: Option<DateTime>,
}

/// 订单明细详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OrderItemDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub order_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub sku: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
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
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<i64>,
    pub update_time: Option<DateTime>,
}

impl From<order_item::Model> for OrderItemDetailVO {
    fn from(item: order_item::Model) -> Self {
        OrderItemDetailVO {
            id: Option::from(item.id),
            order_id: item.order_id,
            product_id: item.product_id,
            product_name: item.product_name,
            product_code: item.product_code,
            sku: item.sku,
            spec: item.spec,
            unit: item.unit,
            unit_id: item.unit_id,
            quantity: item.quantity,
            unit_price: item.unit_price,
            discount_rate: item.discount_rate,
            discount: item.discount,
            discount_amount: item.discount_amount,
            tax_rate: item.tax_rate,
            tax_amount: item.tax_amount,
            amount: item.amount,
            total_amount: item.total_amount,
            delivery_date: item.delivery_date,
            delivered_quantity: item.delivered_quantity,
            remark: item.remark,
            sort: item.sort,
            create_by: item.create_by,
            create_time: item.create_time,
            update_by: item.update_by,
            update_time: item.update_time,
        }
    }
}

/// 订单明细列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OrderItemListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub order_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub sku: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
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

impl From<order_item::Model> for OrderItemListVO {
    fn from(item: order_item::Model) -> Self {
        OrderItemListVO {
            id: Option::from(item.id),
            order_id: item.order_id,
            product_id: item.product_id,
            product_name: item.product_name,
            product_code: item.product_code,
            sku: item.sku,
            spec: item.spec,
            unit: item.unit,
            unit_id: item.unit_id,
            quantity: item.quantity,
            unit_price: item.unit_price,
            discount_rate: item.discount_rate,
            discount: item.discount,
            discount_amount: item.discount_amount,
            tax_rate: item.tax_rate,
            tax_amount: item.tax_amount,
            amount: item.amount,
            total_amount: item.total_amount,
            delivery_date: item.delivery_date,
            delivered_quantity: item.delivered_quantity,
            remark: item.remark,
            sort: item.sort,
        }
    }
}

/// 订单明细列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub order_id: Option<i64>,
}

/// 订单明细数据模型操作类
pub struct OrderItemModel;

impl OrderItemModel {
    pub async fn insert(db: &DbConn, req: &OrderItemSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = order_item::ActiveModel {
            order_id: Set(req.order_id),
            product_id: Set(req.product_id),
            product_name: Set(req.product_name.clone()),
            product_code: Set(req.product_code.clone()),
            sku: Set(req.sku.clone()),
            spec: Set(req.spec.clone()),
            unit: Set(req.unit.clone()),
            unit_id: Set(req.unit_id),
            quantity: Set(req.quantity),
            unit_price: Set(req.unit_price),
            discount_rate: Set(req.discount_rate),
            discount: Set(req.discount),
            discount_amount: Set(req.discount_amount),
            tax_rate: Set(req.tax_rate),
            tax_amount: Set(req.tax_amount),
            amount: Set(req.amount),
            total_amount: Set(req.total_amount),
            delivery_date: Set(req.delivery_date),
            delivered_quantity: Set(req.delivered_quantity),
            remark: Set(req.remark.clone()),
            sort: Set(req.sort),
            create_by: Set(req.create_by),
            create_time: Set(Some(now)),
            update_by: Set(req.update_by),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };

        OrderItem::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        OrderItem::update_many()
            .set(order_item::ActiveModel {
                deleted: Set(Some(1)),
                update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(order_item::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &OrderItemSaveDTO) -> Result<i64, DbErr> {
        let payload = order_item::ActiveModel {
            order_id: Set(req.order_id),
            product_id: Set(req.product_id),
            product_name: Set(req.product_name.clone()),
            product_code: Set(req.product_code.clone()),
            sku: Set(req.sku.clone()),
            spec: Set(req.spec.clone()),
            unit: Set(req.unit.clone()),
            unit_id: Set(req.unit_id),
            quantity: Set(req.quantity),
            unit_price: Set(req.unit_price),
            discount_rate: Set(req.discount_rate),
            discount: Set(req.discount),
            discount_amount: Set(req.discount_amount),
            tax_rate: Set(req.tax_rate),
            tax_amount: Set(req.tax_amount),
            amount: Set(req.amount),
            total_amount: Set(req.total_amount),
            delivery_date: Set(req.delivery_date),
            delivered_quantity: Set(req.delivered_quantity),
            remark: Set(req.remark.clone()),
            sort: Set(req.sort),
            update_by: Set(req.update_by),
            update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = OrderItem::update_many()
            .set(payload)
            .filter(order_item::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<order_item::Model>, DbErr> {
        OrderItem::find_by_id(id)
            .filter(order_item::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        order_id: Option<i64>,
    ) -> Result<(Vec<order_item::Model>, i64), DbErr> {
        let mut query = OrderItem::find()
            .filter(order_item::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(
                Condition::any()
                    .add(order_item::Column::ProductName.contains(k.clone()))
                    .add(order_item::Column::ProductCode.contains(k)),
            );
        }

        if let Some(o) = order_id {
            query = query.filter(order_item::Column::OrderId.eq(o));
        }

        let paginator = query.order_by_desc(order_item::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}
