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
use sea_orm::QuerySelect;
use sea_orm::sea_query::Expr;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::sale::entity::{quotation, quotation::Entity as Quotation, quotation_item, quotation_item::Entity as QuotationItem, quotation_approval, quotation_approval::Entity as QuotationApproval};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 报价单明细保存DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuotationItemSaveDTO {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub quotation_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_code: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub weight: Option<Decimal>,
    pub quantity: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub discount_rate: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub subtotal: Option<Decimal>,
    pub sort: Option<i32>,
    pub remark: Option<String>,
}

impl From<quotation_item::Model> for QuotationItemSaveDTO {
    fn from(item: quotation_item::Model) -> Self {
        QuotationItemSaveDTO {
            id: Option::from(item.id),
            quotation_id: item.quotation_id,
            product_id: item.product_id,
            product_name: item.product_name,
            product_code: item.product_code,
            spec: item.spec,
            unit: item.unit,
            weight: item.weight,
            quantity: item.quantity,
            unit_price: item.unit_price,
            discount_rate: item.discount_rate,
            discount_amount: item.discount_amount,
            tax_rate: item.tax_rate,
            tax_amount: item.tax_amount,
            subtotal: item.subtotal,
            sort: item.sort,
            remark: item.remark,
        }
    }
}

/// 报价单审批记录VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct QuotationApprovalVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub quotation_id: Option<i64>,
    pub version: Option<i32>,
    pub approval_type: Option<i32>,
    pub approval_status: Option<i32>,
    pub approver_id: Option<i64>,
    pub approver_name: Option<String>,
    pub original_amount: Option<Decimal>,
    pub adjusted_amount: Option<Decimal>,
    pub approval_remark: Option<String>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
}

impl From<quotation_approval::Model> for QuotationApprovalVO {
    fn from(item: quotation_approval::Model) -> Self {
        QuotationApprovalVO {
            id: Option::from(item.id),
            quotation_id: item.quotation_id,
            version: item.version,
            approval_type: item.approval_type,
            approval_status: item.approval_status,
            approver_id: item.approver_id,
            approver_name: item.approver_name,
            original_amount: item.original_amount,
            adjusted_amount: item.adjusted_amount,
            approval_remark: item.approval_remark,
            create_by: item.create_by,
            create_time: item.create_time,
        }
    }
}

/// 报价单新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct QuotationSaveRequest {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub contact_id: Option<i64>,
    pub contact_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub opportunity_id: Option<i64>,
    pub opportunity_title: Option<String>,
    pub title: Option<String>,
    pub items: Option<Vec<QuotationItemSaveDTO>>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub tax_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub grand_total: Option<Decimal>,
    pub valid_until: Option<Date>,
    pub quotation_date: Option<Date>,
    pub status: Option<i32>,
    pub approval_status: Option<i32>,
    pub current_version: Option<i32>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub delivery_date: Option<Date>,
    pub port_of_loading: Option<String>,
    pub port_of_destination: Option<String>,
    pub bank_info: Option<String>,
    pub remark: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub owner_user_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub dept_id: Option<i64>,
}

impl From<QuotationSaveRequest> for QuotationSaveDTO {
    fn from(item: QuotationSaveRequest) -> Self {
        QuotationSaveDTO {
            id: None,
            quotation_no: None,
            customer_id: item.customer_id,
            customer_name: item.customer_name,
            contact_id: item.contact_id,
            contact_name: item.contact_name,
            opportunity_id: item.opportunity_id,
            opportunity_title: item.opportunity_title,
            title: item.title,
            items: item.items,
            total_amount: item.total_amount,
            currency: item.currency,
            tax_amount: item.tax_amount,
            discount_amount: item.discount_amount,
            grand_total: item.grand_total,
            valid_until: item.valid_until,
            quotation_date: item.quotation_date,
            status: item.status,
            approval_status: item.approval_status,
            instance_id: None,
            current_version: item.current_version,
            payment_terms: item.payment_terms,
            delivery_terms: item.delivery_terms,
            delivery_date: item.delivery_date,
            port_of_loading: item.port_of_loading,
            port_of_destination: item.port_of_destination,
            bank_info: item.bank_info,
            remark: item.remark,
            owner_user_id: item.owner_user_id,
            dept_id: item.dept_id,
            deleted: None,
            create_by: None,
            create_time: None,
            update_by: None,
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
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub contact_id: Option<i64>,
    pub contact_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub opportunity_id: Option<i64>,
    pub opportunity_title: Option<String>,
    pub title: Option<String>,
    pub items: Option<Vec<QuotationItemSaveDTO>>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub tax_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub grand_total: Option<Decimal>,
    pub valid_until: Option<Date>,
    pub quotation_date: Option<Date>,
    pub status: Option<i32>,
    pub approval_status: Option<i32>,
    pub current_version: Option<i32>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub delivery_date: Option<Date>,
    pub port_of_loading: Option<String>,
    pub port_of_destination: Option<String>,
    pub bank_info: Option<String>,
    pub remark: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub owner_user_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub dept_id: Option<i64>,
}

impl From<QuotationUpdateRequest> for QuotationSaveDTO {
    fn from(item: QuotationUpdateRequest) -> Self {
        QuotationSaveDTO {
            id: item.id,
            quotation_no: None,
            customer_id: item.customer_id,
            customer_name: item.customer_name,
            contact_id: item.contact_id,
            contact_name: item.contact_name,
            opportunity_id: item.opportunity_id,
            opportunity_title: item.opportunity_title,
            title: item.title,
            items: item.items,
            total_amount: item.total_amount,
            currency: item.currency,
            tax_amount: item.tax_amount,
            discount_amount: item.discount_amount,
            grand_total: item.grand_total,
            valid_until: item.valid_until,
            quotation_date: item.quotation_date,
            status: item.status,
            approval_status: item.approval_status,
            instance_id: None,
            current_version: item.current_version,
            payment_terms: item.payment_terms,
            delivery_terms: item.delivery_terms,
            delivery_date: item.delivery_date,
            port_of_loading: item.port_of_loading,
            port_of_destination: item.port_of_destination,
            bank_info: item.bank_info,
            remark: item.remark,
            owner_user_id: item.owner_user_id,
            dept_id: item.dept_id,
            deleted: None,
            create_by: None,
            create_time: None,
            update_by: None,
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
    pub customer_name: Option<String>,
    pub contact_id: Option<i64>,
    pub contact_name: Option<String>,
    pub opportunity_id: Option<i64>,
    pub opportunity_title: Option<String>,
    pub title: Option<String>,
    pub items: Option<Vec<QuotationItemSaveDTO>>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub tax_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub grand_total: Option<Decimal>,
    pub valid_until: Option<Date>,
    pub quotation_date: Option<Date>,
    pub status: Option<i32>,
    pub approval_status: Option<i32>,
    pub instance_id: Option<i64>,
    pub current_version: Option<i32>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub delivery_date: Option<Date>,
    pub port_of_loading: Option<String>,
    pub port_of_destination: Option<String>,
    pub bank_info: Option<String>,
    pub remark: Option<String>,
    pub owner_user_id: Option<i64>,
    pub dept_id: Option<i64>,
    pub deleted: Option<i32>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
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
    pub customer_name: Option<String>,
    pub contact_id: Option<i64>,
    pub contact_name: Option<String>,
    pub opportunity_id: Option<i64>,
    pub opportunity_title: Option<String>,
    pub title: Option<String>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub tax_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub grand_total: Option<Decimal>,
    pub valid_until: Option<Date>,
    pub quotation_date: Option<Date>,
    pub status: Option<i32>,
    pub approval_status: Option<i32>,
    pub instance_id: Option<i64>,
    pub current_version: Option<i32>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub delivery_date: Option<Date>,
    pub port_of_loading: Option<String>,
    pub port_of_destination: Option<String>,
    pub bank_info: Option<String>,
    pub remark: Option<String>,
    pub owner_user_id: Option<i64>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
    pub items: Option<Vec<QuotationItemSaveDTO>>,
    pub approvals: Option<Vec<QuotationApprovalVO>>,
}

impl From<quotation::Model> for QuotationDetailVO {
    fn from(item: quotation::Model) -> Self {
        QuotationDetailVO {
            id: Option::from(item.id),
            quotation_no: item.quotation_no,
            customer_id: item.customer_id,
            customer_name: item.customer_name,
            contact_id: item.contact_id,
            contact_name: item.contact_name,
            opportunity_id: item.opportunity_id,
            opportunity_title: item.opportunity_title,
            title: item.title,
            total_amount: item.total_amount,
            currency: item.currency,
            tax_amount: item.tax_amount,
            discount_amount: item.discount_amount,
            grand_total: item.grand_total,
            valid_until: item.valid_until,
            quotation_date: item.quotation_date,
            status: item.status,
            approval_status: item.approval_status,
            instance_id: item.instance_id,
            current_version: item.current_version,
            payment_terms: item.payment_terms,
            delivery_terms: item.delivery_terms,
            delivery_date: item.delivery_date,
            port_of_loading: item.port_of_loading,
            port_of_destination: item.port_of_destination,
            bank_info: item.bank_info,
            remark: item.remark,
            owner_user_id: item.owner_user_id,
            create_by: item.create_by,
            create_time: item.create_time,
            update_by: item.update_by,
            update_time: item.update_time,
            items: None,
            approvals: None,
        }
    }
}

impl From<(quotation::Model, Vec<quotation_item::Model>, Vec<quotation_approval::Model>)> for QuotationDetailVO {
    fn from(data: (quotation::Model, Vec<quotation_item::Model>, Vec<quotation_approval::Model>)) -> Self {
        let (model, items, approvals) = data;
        let mut vo: QuotationDetailVO = model.into();
        vo.items = Some(items.into_iter().map(|i| i.into()).collect());
        vo.approvals = Some(approvals.into_iter().map(|a| a.into()).collect());
        vo
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
    pub customer_name: Option<String>,
    pub title: Option<String>,
    pub grand_total: Option<Decimal>,
    pub currency: Option<i32>,
    pub status: Option<i32>,
    pub approval_status: Option<i32>,
    pub quotation_date: Option<Date>,
    pub valid_until: Option<Date>,
    pub owner_user_id: Option<i64>,
    pub create_time: Option<DateTime>,
}

impl From<quotation::Model> for QuotationListVO {
    fn from(item: quotation::Model) -> Self {
        QuotationListVO {
            id: Option::from(item.id),
            quotation_no: item.quotation_no,
            customer_id: item.customer_id,
            customer_name: item.customer_name,
            title: item.title,
            grand_total: item.grand_total,
            currency: item.currency,
            status: item.status,
            approval_status: item.approval_status,
            quotation_date: item.quotation_date,
            valid_until: item.valid_until,
            owner_user_id: item.owner_user_id,
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
    pub status: Option<i32>,
    pub approval_status: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// 报价单数据模型操作类
pub struct QuotationModel;

impl QuotationModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &QuotationSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = quotation::ActiveModel {
            quotation_no: Set(req.quotation_no.clone()),
            customer_id: Set(req.customer_id),
            customer_name: Set(req.customer_name.clone()),
            contact_id: Set(req.contact_id),
            contact_name: Set(req.contact_name.clone()),
            opportunity_id: Set(req.opportunity_id),
            opportunity_title: Set(req.opportunity_title.clone()),
            title: Set(req.title.clone()),
            total_amount: Set(req.total_amount),
            currency: Set(req.currency),
            tax_amount: Set(req.tax_amount),
            discount_amount: Set(req.discount_amount),
            grand_total: Set(req.grand_total),
            valid_until: Set(req.valid_until),
            quotation_date: Set(req.quotation_date),
            status: Set(req.status),
            approval_status: Set(req.approval_status),
            instance_id: Set(req.instance_id),
            current_version: Set(req.current_version),
            payment_terms: Set(req.payment_terms.clone()),
            delivery_terms: Set(req.delivery_terms.clone()),
            delivery_date: Set(req.delivery_date),
            port_of_loading: Set(req.port_of_loading.clone()),
            port_of_destination: Set(req.port_of_destination.clone()),
            bank_info: Set(req.bank_info.clone()),
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
        Quotation::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Quotation::update_many()
            .set(quotation::ActiveModel { deleted: Set(Some(1)), ..Default::default() })
            .filter(quotation::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: &Option<i64>, req: &QuotationSaveDTO) -> Result<i64, DbErr> {
        let payload = quotation::ActiveModel {
            customer_id: Set(req.customer_id),
            customer_name: Set(req.customer_name.clone()),
            contact_id: Set(req.contact_id),
            contact_name: Set(req.contact_name.clone()),
            opportunity_id: Set(req.opportunity_id),
            opportunity_title: Set(req.opportunity_title.clone()),
            title: Set(req.title.clone()),
            total_amount: Set(req.total_amount),
            currency: Set(req.currency),
            tax_amount: Set(req.tax_amount),
            discount_amount: Set(req.discount_amount),
            grand_total: Set(req.grand_total),
            valid_until: Set(req.valid_until),
            quotation_date: Set(req.quotation_date),
            status: Set(req.status),
            approval_status: Set(req.approval_status),
            instance_id: Set(req.instance_id),
            current_version: Set(req.current_version),
            payment_terms: Set(req.payment_terms.clone()),
            delivery_terms: Set(req.delivery_terms.clone()),
            delivery_date: Set(req.delivery_date),
            port_of_loading: Set(req.port_of_loading.clone()),
            port_of_destination: Set(req.port_of_destination.clone()),
            bank_info: Set(req.bank_info.clone()),
            remark: Set(req.remark.clone()),
            owner_user_id: Set(req.owner_user_id),
            dept_id: Set(req.dept_id),
            update_by: Set(req.update_by.clone()),
            update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let result = Quotation::update_many()
            .set(payload)
            .filter(quotation::Column::Id.eq(id.unwrap_or_default()))
            .filter(quotation::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i32) -> Result<i64, DbErr> {
        let result = Quotation::update_many()
            .set(quotation::ActiveModel {
                status: Set(Some(status)),
                update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(quotation::Column::Id.eq(id))
            .filter(quotation::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 更新报价单状态及审批状态
    pub async fn update_status_and_approval<C: ConnectionTrait>(
        db: &C,
        id: i64,
        status: Option<i32>,
        approval_status: Option<i32>,
    ) -> Result<i64, DbErr> {
        let result = Quotation::update_many()
            .set(quotation::ActiveModel {
                status: Set(status),
                approval_status: Set(approval_status),
                update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(quotation::Column::Id.eq(id))
            .filter(quotation::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<quotation::Model>, DbErr> {
        Quotation::find_by_id(id)
            .filter(quotation::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn select_in_page<C: ConnectionTrait>(
        db: &C,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        customer_id: Option<i64>,
        status: Option<i32>,
        approval_status: Option<i32>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<(Vec<quotation::Model>, i64), DbErr> {
        let mut query = Quotation::find().filter(quotation::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            if !k.trim().is_empty() {
                let kw = k.trim().to_string();
                query = query.filter(
                    Condition::any()
                        .add(quotation::Column::QuotationNo.contains(kw.clone()))
                        .add(quotation::Column::Title.contains(kw.clone()))
                        .add(quotation::Column::CustomerName.contains(kw))
                );
            }
        }
        if let Some(cid) = customer_id {
            query = query.filter(quotation::Column::CustomerId.eq(cid));
        }
        if let Some(s) = status {
            query = query.filter(quotation::Column::Status.eq(s));
        }
        if let Some(aps) = approval_status {
            query = query.filter(quotation::Column::ApprovalStatus.eq(aps));
        }
        if let Some(start) = start_date {
            query = query.filter(quotation::Column::QuotationDate.gte(start));
        }
        if let Some(end) = end_date {
            query = query.filter(quotation::Column::QuotationDate.lte(end));
        }

        let paginator = query.order_by_desc(quotation::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }

    /// 根据当前最大版本号获取下一个版本号
    pub async fn get_max_version<C: ConnectionTrait>(db: &C, quotation_id: i64) -> Result<Option<i32>, DbErr> {
        let result = QuotationApproval::find()
            .filter(quotation_approval::Column::QuotationId.eq(quotation_id))
            .select_only()
            .column_as(Expr::col(quotation_approval::Column::Version).max(), "max_version")
            .into_tuple::<Option<i32>>()
            .one(db)
            .await?;
        Ok(result.flatten())
    }

    /// 获取最大编号
    pub async fn get_max_quotation_no_today<C: ConnectionTrait>(db: &C, date_prefix: &str) -> Result<Option<i64>, DbErr> {
        let pattern = format!("{}%", date_prefix);
        let result = Quotation::find()
            .filter(quotation::Column::QuotationNo.like(&pattern))
            .select_only()
            .column_as(Expr::expr(Expr::cust("MAX(CAST(SUBSTRING(quotation_no, 11) AS INTEGER))")), "max_seq")
            .into_tuple::<Option<i64>>()
            .one(db)
            .await?;

        Ok(result.flatten())
    }
}

/// 报价单明细数据模型操作类
pub struct QuotationItemModel;

impl QuotationItemModel {
    /// 批量插入报价单明细
    pub async fn insert_batch<C: ConnectionTrait>(db: &C, quotation_id: i64, items: &Vec<QuotationItemSaveDTO>) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let models: Vec<quotation_item::ActiveModel> = items.iter().enumerate().map(|(idx, item)| {
            let qty = item.quantity.unwrap_or_else(|| Decimal::from(1));
            let price = item.unit_price.unwrap_or_else(|| Decimal::from(0));
            let disc_rate = item.discount_rate.unwrap_or_else(|| Decimal::from(0));
            let tax_rate = item.tax_rate.unwrap_or_else(|| Decimal::from(0));

            let gross = qty * price;
            let disc_amt = gross * disc_rate / Decimal::from(100);
            let tax_amt = (gross - disc_amt) * tax_rate / Decimal::from(100);
            let line_subtotal = gross - disc_amt + tax_amt;

            quotation_item::ActiveModel {
                quotation_id: Set(Some(quotation_id)),
                product_id: Set(item.product_id),
                product_name: Set(item.product_name.clone()),
                product_code: Set(item.product_code.clone()),
                spec: Set(item.spec.clone()),
                unit: Set(item.unit.clone()),
                weight: Set(item.weight),
                quantity: Set(Some(qty)),
                unit_price: Set(Some(price)),
                discount_rate: Set(Some(disc_rate)),
                discount_amount: Set(item.discount_amount.or(Some(disc_amt))),
                tax_rate: Set(Some(tax_rate)),
                tax_amount: Set(item.tax_amount.or(Some(tax_amt))),
                subtotal: Set(item.subtotal.or(Some(line_subtotal))),
                sort: Set(item.sort.or(Some(idx as i32))),
                remark: Set(item.remark.clone()),
                create_time: Set(Some(now)),
                update_time: Set(Some(now)),
                deleted: Set(Some(0)),
                ..Default::default()
            }
        }).collect();

        if models.is_empty() {
            return Ok(0);
        }

        let result = QuotationItem::insert_many(models)
            .exec(db)
            .await?;
        Ok(result.last_insert_id.unwrap_or_default())
    }

    /// 软删除指定报价单的全部明细
    pub async fn delete_by_quotation_id<C: ConnectionTrait>(db: &C, quotation_id: i64) -> Result<i64, DbErr> {
        let result = QuotationItem::update_many()
            .set(quotation_item::ActiveModel {
                deleted: Set(Some(1)),
                update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(quotation_item::Column::QuotationId.eq(quotation_id))
            .filter(quotation_item::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 查询指定报价单的全部明细
    pub async fn find_by_quotation_id<C: ConnectionTrait>(db: &C, quotation_id: i64) -> Result<Vec<quotation_item::Model>, DbErr> {
        QuotationItem::find()
            .filter(quotation_item::Column::QuotationId.eq(quotation_id))
            .filter(quotation_item::Column::Deleted.eq(0))
            .order_by_asc(quotation_item::Column::Sort)
            .all(db)
            .await
    }
}

/// 报价单审批记录操作类
pub struct QuotationApprovalModel;

impl QuotationApprovalModel {
    /// 插入审批记录
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        quotation_id: i64,
        version: i32,
        approval_type: i32,
        approval_status: i32,
        approver_id: Option<i64>,
        approver_name: Option<String>,
        original_amount: Option<Decimal>,
        adjusted_amount: Option<Decimal>,
        approval_remark: Option<String>,
        create_by: Option<String>,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = quotation_approval::ActiveModel {
            quotation_id: Set(Some(quotation_id)),
            version: Set(Some(version)),
            approval_type: Set(Some(approval_type)),
            approval_status: Set(Some(approval_status)),
            approver_id: Set(approver_id),
            approver_name: Set(approver_name),
            original_amount: Set(original_amount),
            adjusted_amount: Set(adjusted_amount),
            approval_remark: Set(approval_remark),
            create_by: Set(create_by),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        QuotationApproval::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 查询指定报价单的审批记录列表（按创建时间倒序）
    pub async fn find_by_quotation_id<C: ConnectionTrait>(db: &C, quotation_id: i64) -> Result<Vec<quotation_approval::Model>, DbErr> {
        QuotationApproval::find()
            .filter(quotation_approval::Column::QuotationId.eq(quotation_id))
            .order_by_desc(quotation_approval::Column::CreateTime)
            .all(db)
            .await
    }
}

/// 根据明细重新计算主表金额
/// - total_amount = sum(quantity * unit_price)
/// - discount_amount = sum(discount_amount)
/// - tax_amount = sum(tax_amount)
/// - grand_total = total_amount - discount_amount + tax_amount
pub async fn recalculate_amounts<C: ConnectionTrait>(db: &C, quotation_id: i64) -> Result<i64, DbErr> {
    let items = QuotationItemModel::find_by_quotation_id(db, quotation_id).await?;

    let mut total_amount = Decimal::from(0);
    let mut discount_amount = Decimal::from(0);
    let mut tax_amount = Decimal::from(0);

    for item in items.iter() {
        let qty = item.quantity.unwrap_or_else(|| Decimal::from(0));
        let price = item.unit_price.unwrap_or_else(|| Decimal::from(0));
        total_amount = total_amount + qty * price;
        discount_amount = discount_amount + item.discount_amount.unwrap_or_else(|| Decimal::from(0));
        tax_amount = tax_amount + item.tax_amount.unwrap_or_else(|| Decimal::from(0));
    }

    let grand_total = total_amount - discount_amount + tax_amount;

    let result = Quotation::update_many()
        .set(quotation::ActiveModel {
            total_amount: Set(Some(total_amount)),
            discount_amount: Set(Some(discount_amount)),
            tax_amount: Set(Some(tax_amount)),
            grand_total: Set(Some(grand_total)),
            update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        })
        .filter(quotation::Column::Id.eq(quotation_id))
        .filter(quotation::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}
