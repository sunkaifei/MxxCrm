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
use sea_orm::prelude::{DateTime, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use crate::modules::sale::entity::{order, order::Entity as SaleOrder};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 销售订单新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderSaveRequest {
    /// 订单类型
    pub order_type: Option<String>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 合同ID
    pub contract_id: Option<i64>,
    /// 商机ID
    pub opportunity_id: Option<i64>,
    /// 订单金额(不含税)
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 订单总金额(含税)
    pub total_amount: Option<Decimal>,
    /// 送货地址
    pub shipping_address: Option<String>,
    /// 配送方式
    pub shipping_method: Option<String>,
    /// 备注
    pub remark: Option<String>,
}

impl From<OrderSaveRequest> for OrderSaveDTO {
    fn from(item: OrderSaveRequest) -> Self {
        OrderSaveDTO {
            id: None,
            order_type: item.order_type,
            customer_id: item.customer_id,
            contract_id: item.contract_id,
            opportunity_id: item.opportunity_id,
            amount: item.amount,
            currency: item.currency,
            tax_amount: item.tax_amount,
            total_amount: item.total_amount,
            shipping_address: item.shipping_address,
            shipping_method: item.shipping_method,
            remark: item.remark,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 销售订单更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderUpdateRequest {
    /// 订单ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 订单类型
    pub order_type: Option<String>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 合同ID
    pub contract_id: Option<i64>,
    /// 商机ID
    pub opportunity_id: Option<i64>,
    /// 订单金额(不含税)
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 订单总金额(含税)
    pub total_amount: Option<Decimal>,
    /// 送货地址
    pub shipping_address: Option<String>,
    /// 配送方式
    pub shipping_method: Option<String>,
    /// 备注
    pub remark: Option<String>,
}

impl From<OrderUpdateRequest> for OrderSaveDTO {
    fn from(item: OrderUpdateRequest) -> Self {
        OrderSaveDTO {
            id: item.id,
            order_type: item.order_type,
            customer_id: item.customer_id,
            contract_id: item.contract_id,
            opportunity_id: item.opportunity_id,
            amount: item.amount,
            currency: item.currency,
            tax_amount: item.tax_amount,
            total_amount: item.total_amount,
            shipping_address: item.shipping_address,
            shipping_method: item.shipping_method,
            remark: item.remark,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 销售订单保存DTO(包含新增和更新的所有字段)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OrderSaveDTO {
    /// 订单ID
    pub id: Option<i64>,
    /// 订单类型
    pub order_type: Option<String>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 合同ID
    pub contract_id: Option<i64>,
    /// 商机ID
    pub opportunity_id: Option<i64>,
    /// 订单金额(不含税)
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 订单总金额(含税)
    pub total_amount: Option<Decimal>,
    /// 送货地址
    pub shipping_address: Option<String>,
    /// 配送方式
    pub shipping_method: Option<String>,
    /// 备注
    pub remark: Option<String>,
    /// 软删除标识
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

/// 销售订单详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OrderDetailVO {
    /// 订单ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 订单编号
    pub order_no: Option<String>,
    /// 订单类型
    pub order_type: Option<String>,
    /// 订单状态
    pub status: Option<String>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 合同ID
    pub contract_id: Option<i64>,
    /// 商机ID
    pub opportunity_id: Option<i64>,
    /// 订单金额(不含税)
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 订单总金额(含税)
    pub total_amount: Option<Decimal>,
    /// 支付状态
    pub payment_status: Option<String>,
    /// 支付方式
    pub payment_method: Option<String>,
    /// 已付款金额
    pub paid_amount: Option<Decimal>,
    /// 送货地址
    pub shipping_address: Option<String>,
    /// 配送方式
    pub shipping_method: Option<String>,
    /// 物流单号
    pub tracking_no: Option<String>,
    /// 备注
    pub remark: Option<String>,
}

impl From<order::Model> for OrderDetailVO {
    fn from(item: order::Model) -> Self {
        OrderDetailVO {
            id: Option::from(item.id),
            order_no: item.order_no,
            order_type: item.order_type,
            status: item.status,
            customer_id: item.customer_id,
            contract_id: item.contract_id,
            opportunity_id: item.opportunity_id,
            amount: item.amount,
            currency: item.currency,
            tax_amount: item.tax_amount,
            total_amount: item.total_amount,
            payment_status: item.payment_status,
            payment_method: item.payment_method,
            paid_amount: item.paid_amount,
            shipping_address: item.shipping_address,
            shipping_method: item.shipping_method,
            tracking_no: item.tracking_no,
            remark: item.remark,
        }
    }
}

/// 销售订单列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OrderListVO {
    /// 订单ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 订单编号
    pub order_no: Option<String>,
    /// 订单类型
    pub order_type: Option<String>,
    /// 订单状态
    pub status: Option<String>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 订单金额(不含税)
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 支付状态
    pub payment_status: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

impl From<order::Model> for OrderListVO {
    fn from(item: order::Model) -> Self {
        OrderListVO {
            id: Option::from(item.id),
            order_no: item.order_no,
            order_type: item.order_type,
            status: item.status,
            customer_id: item.customer_id,
            amount: item.amount,
            currency: item.currency,
            payment_status: item.payment_status,
            create_time: item.create_time,
        }
    }
}

/// 销售订单列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderListQuery {
    /// 页码
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 关键词(搜索订单编号等)
    pub keywords: Option<String>,
    /// 订单状态
    pub status: Option<String>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 订单类型
    pub order_type: Option<String>,
}

/// 销售订单数据模型操作类
pub struct OrderModel;

impl OrderModel {
    /// 新增销售订单
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `req` - 订单保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 新增记录的ID
    pub async fn insert(db: &DbConn, req: &OrderSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = order::ActiveModel {
            order_type: Set(req.order_type.clone()),
            customer_id: Set(req.customer_id.clone()),
            contract_id: Set(req.contract_id.clone()),
            opportunity_id: Set(req.opportunity_id.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            tax_amount: Set(req.tax_amount.clone()),
            total_amount: Set(req.total_amount.clone()),
            shipping_address: Set(req.shipping_address.clone()),
            shipping_method: Set(req.shipping_method.clone()),
            remark: Set(req.remark.clone()),
            created_by: Set(req.created_by.clone()),
            create_time: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };
        
        SaleOrder::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除销售订单(软删除)
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `ids` - 要删除的订单ID列表
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 删除的记录数
    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        SaleOrder::update_many()
            .set(order::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(order::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新销售订单信息
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 订单ID
    /// * `req` - 订单保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 更新的记录数
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &OrderSaveDTO) -> Result<i64, DbErr> {
        let payload = order::ActiveModel {
            order_type: Set(req.order_type.clone()),
            customer_id: Set(req.customer_id.clone()),
            contract_id: Set(req.contract_id.clone()),
            opportunity_id: Set(req.opportunity_id.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            tax_amount: Set(req.tax_amount.clone()),
            total_amount: Set(req.total_amount.clone()),
            shipping_address: Set(req.shipping_address.clone()),
            shipping_method: Set(req.shipping_method.clone()),
            remark: Set(req.remark.clone()),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        
        let update_result: UpdateResult = SaleOrder::update_many()
            .set(payload)
            .filter(order::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据ID查询订单详情
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 订单ID
    ///
    /// # 返回
    /// * `Result<Option<order::Model>, DbErr>` - 订单模型(未删除)
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<order::Model>, DbErr> {
        SaleOrder::find_by_id(id)
            .filter(order::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询订单列表
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `page` - 页码
    /// * `per_page` - 每页大小
    /// * `keywords` - 关键词
    /// * `status` - 订单状态
    /// * `customer_id` - 客户ID
    /// * `order_type` - 订单类型
    ///
    /// # 返回
    /// * `Result<(Vec<order::Model>, i64), DbErr>` - (订单列表, 总页数)
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        status: Option<String>,
        customer_id: Option<i64>,
        order_type: Option<String>,
    ) -> Result<(Vec<order::Model>, i64), DbErr> {
        let mut query = SaleOrder::find()
            .filter(order::Column::Deleted.eq(0));
        
        if let Some(k) = keywords {
            query = query.filter(order::Column::OrderNo.contains(k));
        }
        if let Some(s) = status {
            query = query.filter(order::Column::Status.eq(s));
        }
        if let Some(c) = customer_id {
            query = query.filter(order::Column::CustomerId.eq(c));
        }
        if let Some(t) = order_type {
            query = query.filter(order::Column::OrderType.eq(t));
        }
        
        let paginator = query.order_by_desc(order::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    /// 查询订单总数
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `keywords` - 关键词
    /// * `status` - 订单状态
    /// * `customer_id` - 客户ID
    /// * `order_type` - 订单类型
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 订单总数
    pub async fn select_count(
        db: &DbConn,
        keywords: Option<String>,
        status: Option<String>,
        customer_id: Option<i64>,
        order_type: Option<String>,
    ) -> Result<i64, DbErr> {
        let mut query = SaleOrder::find()
            .filter(order::Column::Deleted.eq(0));
        
        if let Some(k) = keywords {
            query = query.filter(order::Column::OrderNo.contains(k));
        }
        if let Some(s) = status {
            query = query.filter(order::Column::Status.eq(s));
        }
        if let Some(c) = customer_id {
            query = query.filter(order::Column::CustomerId.eq(c));
        }
        if let Some(t) = order_type {
            query = query.filter(order::Column::OrderType.eq(t));
        }
        
        query.count(db).await.map(|c| c as i64)
    }
}
