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
use chrono::NaiveDate;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use crate::core::r#enum::payment_method_enum::PaymentMethod;
use crate::modules::sale::entity::{payment, payment::Entity as Payment};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 收款记录新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PaymentSaveRequest {
    /// 订单ID
    pub order_id: Option<i64>,
    /// 合同ID
    pub contract_id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 收款金额
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 收款状态
    pub status: Option<String>,
    /// 收款方式
    pub payment_method: Option<PaymentMethod>,
    /// 交易流水号
    pub transaction_no: Option<String>,
    /// 收款日期
    pub payment_date: Option<NaiveDate>,
    /// 备注
    pub remark: Option<String>,
}

impl From<PaymentSaveRequest> for PaymentSaveDTO {
    fn from(item: PaymentSaveRequest) -> Self {
        PaymentSaveDTO {
            id: None,
            payment_no: None,
            order_id: item.order_id,
            contract_id: item.contract_id,
            customer_id: item.customer_id,
            amount: item.amount,
            currency: item.currency,
            status: item.status,
            payment_method: item.payment_method,
            transaction_no: item.transaction_no,
            payment_date: item.payment_date,
            remark: item.remark,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 收款记录更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PaymentUpdateRequest {
    /// 收款记录ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 订单ID
    pub order_id: Option<i64>,
    /// 合同ID
    pub contract_id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 收款金额
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 收款状态
    pub status: Option<String>,
    /// 收款方式
    pub payment_method: Option<PaymentMethod>,
    /// 交易流水号
    pub transaction_no: Option<String>,
    /// 收款日期
    pub payment_date: Option<NaiveDate>,
    /// 备注
    pub remark: Option<String>,
}

impl From<PaymentUpdateRequest> for PaymentSaveDTO {
    fn from(item: PaymentUpdateRequest) -> Self {
        PaymentSaveDTO {
            id: item.id,
            payment_no: None,
            order_id: item.order_id,
            contract_id: item.contract_id,
            customer_id: item.customer_id,
            amount: item.amount,
            currency: item.currency,
            status: item.status,
            payment_method: item.payment_method,
            transaction_no: item.transaction_no,
            payment_date: item.payment_date,
            remark: item.remark,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 收款记录保存DTO(包含新增和更新的所有字段)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PaymentSaveDTO {
    /// 收款记录ID
    pub id: Option<i64>,
    /// 收款单号
    pub payment_no: Option<String>,
    /// 订单ID
    pub order_id: Option<i64>,
    /// 合同ID
    pub contract_id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 收款金额
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 收款状态
    pub status: Option<String>,
    /// 收款方式
    pub payment_method: Option<PaymentMethod>,
    /// 交易流水号
    pub transaction_no: Option<String>,
    /// 收款日期
    pub payment_date: Option<NaiveDate>,
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

/// 收款记录详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PaymentDetailVO {
    /// 收款记录ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 收款单号
    pub payment_no: Option<String>,
    /// 订单ID
    pub order_id: Option<i64>,
    /// 合同ID
    pub contract_id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 收款金额
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 收款状态
    pub status: Option<String>,
    /// 收款方式
    pub payment_method: Option<PaymentMethod>,
    /// 交易流水号
    pub transaction_no: Option<String>,
    /// 收款日期
    pub payment_date: Option<NaiveDate>,
    /// 备注
    pub remark: Option<String>,
}

impl From<payment::Model> for PaymentDetailVO {
    fn from(item: payment::Model) -> Self {
        PaymentDetailVO {
            id: Option::from(item.id),
            payment_no: item.payment_no,
            order_id: item.order_id,
            contract_id: item.contract_id,
            customer_id: item.customer_id,
            amount: item.amount,
            currency: item.currency,
            status: item.status,
            payment_method: item.payment_method,
            transaction_no: item.transaction_no,
            payment_date: item.payment_date,
            remark: item.remark,
        }
    }
}

/// 收款记录列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PaymentListVO {
    /// 收款记录ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 收款单号
    pub payment_no: Option<String>,
    /// 订单ID
    pub order_id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 收款金额
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 收款状态
    pub status: Option<String>,
    /// 收款方式
    pub payment_method: Option<PaymentMethod>,
    /// 收款日期
    pub payment_date: Option<NaiveDate>,
}

impl From<payment::Model> for PaymentListVO {
    fn from(item: payment::Model) -> Self {
        PaymentListVO {
            id: Option::from(item.id),
            payment_no: item.payment_no,
            order_id: item.order_id,
            customer_id: item.customer_id,
            amount: item.amount,
            currency: item.currency,
            status: item.status,
            payment_method: item.payment_method,
            payment_date: item.payment_date,
        }
    }
}

/// 收款记录列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaymentListQuery {
    /// 页码
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 关键词
    pub keywords: Option<String>,
    /// 收款方式
    pub payment_type: Option<PaymentMethod>,
    /// 关联类型
    pub related_type: Option<String>,
    /// 关联ID
    pub related_id: Option<i64>,
    /// 订单ID
    pub order_id: Option<i64>,
    /// 收款状态
    pub status: Option<String>,
    /// 客户ID
    pub customer_id: Option<i64>,
}

/// 收款记录数据模型操作类
pub struct PaymentModel;

impl PaymentModel {
    /// 新增收款记录
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `req` - 收款记录保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 新增记录的ID
    pub async fn insert(db: &DbConn, req: &PaymentSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = payment::ActiveModel {
            order_id: Set(req.order_id.clone()),
            contract_id: Set(req.contract_id.clone()),
            customer_id: Set(req.customer_id.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            status: Set(req.status.clone()),
            payment_method: Set(req.payment_method.clone()),
            transaction_no: Set(req.transaction_no.clone()),
            payment_date: Set(req.payment_date.clone()),
            remark: Set(req.remark.clone()),
            created_by: Set(req.created_by.clone()),
            create_time: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };

        Payment::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除收款记录(软删除)
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `ids` - 要删除的收款记录ID列表
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 删除的记录数
    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Payment::update_many()
            .set(payment::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(payment::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新收款记录信息
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 收款记录ID
    /// * `req` - 收款记录保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 更新的记录数
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &PaymentSaveDTO) -> Result<i64, DbErr> {
        let payload = payment::ActiveModel {
            order_id: Set(req.order_id.clone()),
            contract_id: Set(req.contract_id.clone()),
            customer_id: Set(req.customer_id.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            status: Set(req.status.clone()),
            payment_method: Set(req.payment_method.clone()),
            transaction_no: Set(req.transaction_no.clone()),
            payment_date: Set(req.payment_date.clone()),
            remark: Set(req.remark.clone()),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = Payment::update_many()
            .set(payload)
            .filter(payment::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据ID查询收款记录详情
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 收款记录ID
    ///
    /// # 返回
    /// * `Result<Option<payment::Model>, DbErr>` - 收款记录模型(未删除)
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<payment::Model>, DbErr> {
        Payment::find_by_id(id)
            .filter(payment::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询收款记录列表
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `page` - 页码
    /// * `per_page` - 每页大小
    /// * `keywords` - 关键词
    /// * `payment_type` - 收款方式
    /// * `related_type` - 关联类型
    /// * `related_id` - 关联ID
    ///
    /// # 返回
    /// * `Result<(Vec<payment::Model>, i64), DbErr>` - (收款记录列表, 总页数)
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        payment_type: Option<PaymentMethod>,
        _related_type: Option<String>,
        _related_id: Option<i64>,
    ) -> Result<(Vec<payment::Model>, i64), DbErr> {
        let mut query = Payment::find()
            .filter(payment::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(payment::Column::PaymentNo.contains(k));
        }
        if let Some(p) = payment_type {
            query = query.filter(payment::Column::PaymentMethod.eq(p));
        }

        let paginator = query.order_by_desc(payment::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}
