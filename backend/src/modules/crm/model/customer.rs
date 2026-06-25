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
use crate::core::kit::global::{Deserialize, Serialize};
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use crate::core::r#enum::industry_enum::IndustryType;
use crate::core::r#enum::lead_source_enum::LeadSource;
use crate::modules::crm::entity::{customer, customer::Entity as Customer};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 客户新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerSaveRequest {
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 公司官网
    pub website: Option<String>,
    /// 所属行业
    pub industry: Option<IndustryType>,
    /// 客户等级
    pub level: Option<i32>,
    /// 客户来源
    pub source: Option<LeadSource>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 信用额度
    pub credit_limit: Option<Decimal>,
    /// 信用天数
    pub credit_days: Option<i32>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 合作日期
    pub cooperated_at: Option<Date>,
    /// 生日月份
    pub birthday_month: Option<i32>,
    /// 描述/备注
    pub description: Option<String>,
    /// 自定义字段(JSON格式)
    pub custom_fields: Option<serde_json::Value>,
}

impl From<CustomerSaveRequest> for CustomerSaveDTO {
    fn from(item: CustomerSaveRequest) -> Self {
        CustomerSaveDTO {
            id: None,
            company_name: item.company_name,
            short_name: item.short_name,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry,
            level: item.level,
            source: item.source,
            currency: item.currency,
            credit_limit: item.credit_limit,
            credit_days: item.credit_days,
            assigned_to: item.assigned_to,
            cooperated_at: item.cooperated_at,
            birthday_month: item.birthday_month,
            description: item.description,
            custom_fields: item.custom_fields,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 客户更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerUpdateRequest {
    /// 客户ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 公司官网
    pub website: Option<String>,
    /// 所属行业
    pub industry: Option<IndustryType>,
    /// 客户等级
    pub level: Option<i32>,
    /// 客户来源
    pub source: Option<LeadSource>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 信用额度
    pub credit_limit: Option<Decimal>,
    /// 信用天数
    pub credit_days: Option<i32>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 合作日期
    pub cooperated_at: Option<Date>,
    /// 生日月份
    pub birthday_month: Option<i32>,
    /// 描述/备注
    pub description: Option<String>,
    /// 自定义字段(JSON格式)
    pub custom_fields: Option<serde_json::Value>,
}

impl From<CustomerUpdateRequest> for CustomerSaveDTO {
    fn from(item: CustomerUpdateRequest) -> Self {
        CustomerSaveDTO {
            id: item.id,
            company_name: item.company_name,
            short_name: item.short_name,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry,
            level: item.level,
            source: item.source,
            currency: item.currency,
            credit_limit: item.credit_limit,
            credit_days: item.credit_days,
            assigned_to: item.assigned_to,
            cooperated_at: item.cooperated_at,
            birthday_month: item.birthday_month,
            description: item.description,
            custom_fields: item.custom_fields,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 客户保存DTO(包含新增和更新的所有字段)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerSaveDTO {
    /// 客户ID
    pub id: Option<i64>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 公司官网
    pub website: Option<String>,
    /// 所属行业
    pub industry: Option<IndustryType>,
    /// 客户等级
    pub level: Option<i32>,
    /// 客户来源
    pub source: Option<LeadSource>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 信用额度
    pub credit_limit: Option<Decimal>,
    /// 信用天数
    pub credit_days: Option<i32>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 合作日期
    pub cooperated_at: Option<Date>,
    /// 生日月份
    pub birthday_month: Option<i32>,
    /// 描述/备注
    pub description: Option<String>,
    /// 自定义字段(JSON格式)
    pub custom_fields: Option<serde_json::Value>,
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

/// 客户详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerDetailVO {
    /// 客户ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 客户编号
    pub customer_no: Option<String>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 公司官网
    pub website: Option<String>,
    /// 所属行业
    pub industry: Option<IndustryType>,
    /// 客户等级
    pub level: Option<i32>,
    /// 客户来源
    pub source: Option<LeadSource>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 信用额度
    pub credit_limit: Option<Decimal>,
    /// 信用天数
    pub credit_days: Option<i32>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 合作日期
    pub cooperated_at: Option<Date>,
    /// 生日月份
    pub birthday_month: Option<i32>,
    /// 描述/备注
    pub description: Option<String>,
    /// 自定义字段(JSON格式)
    pub custom_fields: Option<serde_json::Value>,
    /// 累计成交金额
    pub total_deal_amount: Option<Decimal>,
    /// 累计成交次数
    pub total_deal_count: Option<i32>,
    /// 最后成交时间
    pub last_deal_at: Option<DateTime>,
    /// 下次跟进时间
    pub next_follow_at: Option<DateTime>,
}

impl From<customer::Model> for CustomerDetailVO {
    fn from(item: customer::Model) -> Self {
        CustomerDetailVO {
            id: Option::from(item.id),
            customer_no: item.customer_no,
            company_name: item.company_name,
            short_name: item.short_name,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry,
            level: item.level,
            source: item.source,
            currency: item.currency,
            credit_limit: item.credit_limit,
            credit_days: item.credit_days,
            assigned_to: item.assigned_to,
            cooperated_at: item.cooperated_at,
            birthday_month: item.birthday_month,
            description: item.description,
            custom_fields: item.custom_fields,
            total_deal_amount: item.total_deal_amount,
            total_deal_count: item.total_deal_count,
            last_deal_at: item.last_deal_at,
            next_follow_at: item.next_follow_at,
        }
    }
}

/// 客户列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerListVO {
    /// 客户ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 客户编号
    pub customer_no: Option<String>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 客户等级
    pub level: Option<i32>,
    /// 客户来源
    pub source: Option<LeadSource>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 累计成交金额
    pub total_deal_amount: Option<Decimal>,
    /// 最后成交时间
    pub last_deal_at: Option<DateTime>,
}

impl From<customer::Model> for CustomerListVO {
    fn from(item: customer::Model) -> Self {
        CustomerListVO {
            id: Option::from(item.id),
            customer_no: item.customer_no,
            company_name: item.company_name,
            short_name: item.short_name,
            country: item.country,
            region: item.region,
            level: item.level,
            source: item.source,
            assigned_to: item.assigned_to,
            total_deal_amount: item.total_deal_amount,
            last_deal_at: item.last_deal_at,
        }
    }
}

/// 客户列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomerListQuery {
    /// 页码
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 关键词(搜索公司名称、简称等)
    #[serde(alias = "companyName")]
    pub keywords: Option<String>,
    /// 客户等级
    pub level: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 客户来源
    pub source: Option<String>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
}

/// 客户数据模型操作类
pub struct CustomerModel;

impl CustomerModel {
    /// 新增客户
    pub async fn insert(db: &impl ConnectionTrait, req: &CustomerSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = customer::ActiveModel {
            company_name: Set(req.company_name.clone()),
            short_name: Set(req.short_name.clone()),
            country: Set(req.country.clone()),
            region: Set(req.region.clone()),
            address: Set(req.address.clone()),
            website: Set(req.website.clone()),
            industry: Set(req.industry.clone()),
            level: Set(req.level.clone()),
            source: Set(req.source.clone()),
            currency: Set(req.currency.clone()),
            credit_limit: Set(req.credit_limit.clone()),
            credit_days: Set(req.credit_days.clone()),
            assigned_to: Set(req.assigned_to.clone()),
            cooperated_at: Set(req.cooperated_at.clone()),
            birthday_month: Set(req.birthday_month.clone()),
            description: Set(req.description.clone()),
            custom_fields: Set(req.custom_fields.clone()),
            created_by: Set(req.created_by.clone()),
            create_time: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };
        
        Customer::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除客户(软删除)
    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Customer::update_many()
            .set(customer::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(customer::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新客户信息
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &CustomerSaveDTO) -> Result<i64, DbErr> {
        let payload = customer::ActiveModel {
            company_name: Set(req.company_name.clone()),
            short_name: Set(req.short_name.clone()),
            country: Set(req.country.clone()),
            region: Set(req.region.clone()),
            address: Set(req.address.clone()),
            website: Set(req.website.clone()),
            industry: Set(req.industry.clone()),
            level: Set(req.level.clone()),
            source: Set(req.source.clone()),
            currency: Set(req.currency.clone()),
            credit_limit: Set(req.credit_limit.clone()),
            credit_days: Set(req.credit_days.clone()),
            assigned_to: Set(req.assigned_to.clone()),
            cooperated_at: Set(req.cooperated_at.clone()),
            birthday_month: Set(req.birthday_month.clone()),
            description: Set(req.description.clone()),
            custom_fields: Set(req.custom_fields.clone()),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        
        let update_result: UpdateResult = Customer::update_many()
            .set(payload)
            .filter(customer::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据ID查询客户详情
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<customer::Model>, DbErr> {
        Customer::find_by_id(id)
            .filter(customer::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询客户列表
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        level: Option<String>,
        country: Option<String>,
        source: Option<String>,
        assigned_to: Option<i64>,
    ) -> Result<(Vec<customer::Model>, i64), DbErr> {
        let mut query = Customer::find()
            .filter(customer::Column::Deleted.eq(0));
        
        if let Some(k) = keywords {
            query = query.filter(customer::Column::CompanyName.contains(k));
        }
        if let Some(l) = level {
            query = query.filter(customer::Column::Level.eq(l));
        }
        if let Some(c) = country {
            query = query.filter(customer::Column::Country.eq(c));
        }
        if let Some(s) = source {
            query = query.filter(customer::Column::Source.eq(s));
        }
        if let Some(a) = assigned_to {
            query = query.filter(customer::Column::AssignedTo.eq(a));
        }
        
        let paginator = query.order_by_desc(customer::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    /// 查询客户总数
    pub async fn select_count(
        db: &DbConn,
        keywords: Option<String>,
        level: Option<String>,
        country: Option<String>,
        source: Option<String>,
        assigned_to: Option<i64>,
    ) -> Result<i64, DbErr> {
        let mut query = Customer::find()
            .filter(customer::Column::Deleted.eq(0));
        
        if let Some(k) = keywords {
            query = query.filter(customer::Column::CompanyName.contains(k));
        }
        if let Some(l) = level {
            query = query.filter(customer::Column::Level.eq(l));
        }
        if let Some(c) = country {
            query = query.filter(customer::Column::Country.eq(c));
        }
        if let Some(s) = source {
            query = query.filter(customer::Column::Source.eq(s));
        }
        if let Some(a) = assigned_to {
            query = query.filter(customer::Column::AssignedTo.eq(a));
        }
        
        query.count(db).await.map(|c| c as i64)
    }
}
