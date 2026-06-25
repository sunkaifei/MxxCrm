use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::sale::entity::{order_item, order_item::Entity as OrderItem};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 订单明细新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderItemSaveRequest {
    /// 订单ID
    pub order_id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品编码
    pub product_code: Option<String>,
    /// 规格型号
    pub spec: Option<String>,
    /// 单位
    pub unit: Option<String>,
    /// 数量
    pub quantity: Option<i32>,
    /// 单价
    pub unit_price: Option<Decimal>,
    /// 金额（不含税）
    pub amount: Option<Decimal>,
    /// 税率
    pub tax_rate: Option<Decimal>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 备注
    pub remark: Option<String>,
}

impl From<OrderItemSaveRequest> for OrderItemSaveDTO {
    fn from(item: OrderItemSaveRequest) -> Self {
        OrderItemSaveDTO {
            id: None,
            order_id: item.order_id,
            product_id: item.product_id,
            product_name: item.product_name,
            product_code: item.product_code,
            spec: item.spec,
            unit: item.unit,
            quantity: item.quantity,
            unit_price: item.unit_price,
            amount: item.amount,
            tax_rate: item.tax_rate,
            tax_amount: item.tax_amount,
            remark: item.remark,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 订单明细更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderItemUpdateRequest {
    /// 明细ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 订单ID
    pub order_id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品编码
    pub product_code: Option<String>,
    /// 规格型号
    pub spec: Option<String>,
    /// 单位
    pub unit: Option<String>,
    /// 数量
    pub quantity: Option<i32>,
    /// 单价
    pub unit_price: Option<Decimal>,
    /// 金额（不含税）
    pub amount: Option<Decimal>,
    /// 税率
    pub tax_rate: Option<Decimal>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 备注
    pub remark: Option<String>,
}

impl From<OrderItemUpdateRequest> for OrderItemSaveDTO {
    fn from(item: OrderItemUpdateRequest) -> Self {
        OrderItemSaveDTO {
            id: item.id,
            order_id: item.order_id,
            product_id: item.product_id,
            product_name: item.product_name,
            product_code: item.product_code,
            spec: item.spec,
            unit: item.unit,
            quantity: item.quantity,
            unit_price: item.unit_price,
            amount: item.amount,
            tax_rate: item.tax_rate,
            tax_amount: item.tax_amount,
            remark: item.remark,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 订单明细保存DTO（包含新增和更新的所有字段）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderItemSaveDTO {
    /// 明细ID
    pub id: Option<i64>,
    /// 订单ID
    pub order_id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品编码
    pub product_code: Option<String>,
    /// 规格型号
    pub spec: Option<String>,
    /// 单位
    pub unit: Option<String>,
    /// 数量
    pub quantity: Option<i32>,
    /// 单价
    pub unit_price: Option<Decimal>,
    /// 金额（不含税）
    pub amount: Option<Decimal>,
    /// 税率
    pub tax_rate: Option<Decimal>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 备注
    pub remark: Option<String>,
    /// 软删除标记
    pub deleted: Option<i32>,
    /// 创建人ID
    pub created_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新人ID
    pub updated_by: Option<i64>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

/// 订单明细详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OrderItemDetailVO {
    /// 明细ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 订单ID
    pub order_id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品编码
    pub product_code: Option<String>,
    /// 规格型号
    pub spec: Option<String>,
    /// 单位
    pub unit: Option<String>,
    /// 数量
    pub quantity: Option<i32>,
    /// 单价
    pub unit_price: Option<Decimal>,
    /// 金额（不含税）
    pub amount: Option<Decimal>,
    /// 税率
    pub tax_rate: Option<Decimal>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 备注
    pub remark: Option<String>,
}

impl From<order_item::Model> for OrderItemDetailVO {
    fn from(item: order_item::Model) -> Self {
        OrderItemDetailVO {
            id: Option::from(item.id),
            order_id: item.order_id,
            product_id: item.product_id,
            product_name: item.product_name,
            product_code: item.product_code,
            spec: item.spec,
            unit: item.unit,
            quantity: item.quantity,
            unit_price: item.unit_price,
            amount: item.amount,
            tax_rate: item.tax_rate,
            tax_amount: item.tax_amount,
            remark: item.remark,
        }
    }
}

/// 订单明细列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OrderItemListVO {
    /// 明细ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 订单ID
    pub order_id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品编码
    pub product_code: Option<String>,
    /// 规格型号
    pub spec: Option<String>,
    /// 单位
    pub unit: Option<String>,
    /// 数量
    pub quantity: Option<i32>,
    /// 单价
    pub unit_price: Option<Decimal>,
    /// 金额（不含税）
    pub amount: Option<Decimal>,
    /// 备注
    pub remark: Option<String>,
}

impl From<order_item::Model> for OrderItemListVO {
    fn from(item: order_item::Model) -> Self {
        OrderItemListVO {
            id: Option::from(item.id),
            order_id: item.order_id,
            product_id: item.product_id,
            product_name: item.product_name,
            product_code: item.product_code,
            spec: item.spec,
            unit: item.unit,
            quantity: item.quantity,
            unit_price: item.unit_price,
            amount: item.amount,
            remark: item.remark,
        }
    }
}

/// 订单明细列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemListQuery {
    /// 页码
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 关键词
    pub keywords: Option<String>,
    /// 订单ID
    pub order_id: Option<i64>,
}

/// 订单明细数据模型操作类
pub struct OrderItemModel;

impl OrderItemModel {
    /// 新增订单明细
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `req` - 订单明细保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 新增记录的ID
    pub async fn insert(db: &DbConn, req: &OrderItemSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = order_item::ActiveModel {
            order_id: Set(req.order_id.clone()),
            product_id: Set(req.product_id.clone()),
            product_name: Set(req.product_name.clone()),
            product_code: Set(req.product_code.clone()),
            spec: Set(req.spec.clone()),
            unit: Set(req.unit.clone()),
            quantity: Set(req.quantity.clone()),
            unit_price: Set(req.unit_price.clone()),
            amount: Set(req.amount.clone()),
            tax_rate: Set(req.tax_rate.clone()),
            tax_amount: Set(req.tax_amount.clone()),
            remark: Set(req.remark.clone()),
            create_time: Set(Option::from(now)),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };

        OrderItem::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除订单明细（软删除）
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `ids` - 要删除的明细ID列表
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 删除的记录数
    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        OrderItem::update_many()
            .set(order_item::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(order_item::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新订单明细信息
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 明细ID
    /// * `req` - 订单明细保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 更新的记录数
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &OrderItemSaveDTO) -> Result<i64, DbErr> {
        let payload = order_item::ActiveModel {
            order_id: Set(req.order_id.clone()),
            product_id: Set(req.product_id.clone()),
            product_name: Set(req.product_name.clone()),
            product_code: Set(req.product_code.clone()),
            spec: Set(req.spec.clone()),
            unit: Set(req.unit.clone()),
            quantity: Set(req.quantity.clone()),
            unit_price: Set(req.unit_price.clone()),
            amount: Set(req.amount.clone()),
            tax_rate: Set(req.tax_rate.clone()),
            tax_amount: Set(req.tax_amount.clone()),
            remark: Set(req.remark.clone()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = OrderItem::update_many()
            .set(payload)
            .filter(order_item::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据ID查询订单明细详情
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 明细ID
    ///
    /// # 返回
    /// * `Result<Option<order_item::Model>, DbErr>` - 订单明细模型（未删除）
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<order_item::Model>, DbErr> {
        OrderItem::find_by_id(id)
            .filter(order_item::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询订单明细列表
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `page` - 页码
    /// * `per_page` - 每页大小
    /// * `keywords` - 关键词
    /// * `order_id` - 订单ID
    ///
    /// # 返回
    /// * `Result<(Vec<order_item::Model>, i64), DbErr>` - (明细列表, 总页数)
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