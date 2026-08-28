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
use crate::modules::crm::entity::{customer, customer::Entity as Customer};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 客户新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerSaveRequest {
    /// 客户类型: 1=企业, 2=个人
    pub customer_type: Option<i32>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 客户编号
    pub customer_no: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 个人姓名（个人客户必填）
    pub person_name: Option<String>,
    /// 性别: 1=男, 2=女, 0=未知
    pub gender: Option<i32>,
    /// 出生日期
    pub birthday: Option<Date>,
    /// 微信
    pub wechat: Option<String>,
    /// QQ
    pub qq: Option<String>,
    /// 个人手机号
    pub personal_mobile: Option<String>,
    /// 个人邮箱
    pub personal_email: Option<String>,
    /// 昵称/别名
    pub nickname: Option<String>,
    /// 职业/职务
    pub occupation: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 公司官网
    pub website: Option<String>,
    /// 所属行业
    pub industry: Option<i32>,
    /// 客户等级
    pub level: Option<i32>,
    /// 客户来源
    pub source: Option<i32>,
    /// 币种（1=人民币, 2=美元, 3=欧元, 4=英镑, 5=日元, 6=港币, 7=澳元）
    pub currency: Option<i32>,
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
            from_pool: None,
            customer_no: item.customer_no,
            customer_type: item.customer_type,
            company_name: item.company_name,
            short_name: item.short_name,
            person_name: item.person_name,
            gender: item.gender,
            birthday: item.birthday,
            wechat: item.wechat.clone(),
            qq: item.qq.clone(),
            personal_mobile: item.personal_mobile,
            personal_email: item.personal_email,
            nickname: item.nickname,
            occupation: item.occupation,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry,
            level: item.level,
            source: item.source,
            currency: item.currency.and_then(CurrencyCode::from_i32),
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
    /// 客户类型: 1=企业, 2=个人（编辑时一般不允许变更）
    pub customer_type: Option<i32>,
    /// 客户编号
    pub customer_no: Option<String>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 个人姓名
    pub person_name: Option<String>,
    /// 性别: 1=男, 2=女, 0=未知
    pub gender: Option<i32>,
    /// 出生日期
    pub birthday: Option<Date>,
    /// 微信
    pub wechat: Option<String>,
    /// QQ
    pub qq: Option<String>,
    /// 个人手机号
    pub personal_mobile: Option<String>,
    /// 个人邮箱
    pub personal_email: Option<String>,
    /// 昵称/别名
    pub nickname: Option<String>,
    /// 职业/职务
    pub occupation: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 公司官网
    pub website: Option<String>,
    /// 所属行业
    pub industry: Option<i32>,
    /// 客户等级
    pub level: Option<i32>,
    /// 客户来源
    pub source: Option<i32>,
    /// 币种（1=人民币, 2=美元, 3=欧元, 4=英镑, 5=日元, 6=港币, 7=澳元）
    pub currency: Option<i32>,
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
            from_pool: None,
            customer_no: item.customer_no,
            customer_type: item.customer_type,
            company_name: item.company_name,
            short_name: item.short_name,
            person_name: item.person_name,
            gender: item.gender,
            birthday: item.birthday,
            wechat: item.wechat.clone(),
            qq: item.qq.clone(),
            personal_mobile: item.personal_mobile,
            personal_email: item.personal_email,
            nickname: item.nickname,
            occupation: item.occupation,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry,
            level: item.level,
            source: item.source,
            currency: item.currency.and_then(CurrencyCode::from_i32),
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
    /// 客户来源：0=自建，1=公海/线索来源（线索转客户置1，删除按钮显隐依赖）
    pub from_pool: Option<i16>,
    /// 客户编号
    pub customer_no: Option<String>,
    /// 客户类型: 1=企业, 2=个人
    pub customer_type: Option<i32>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 个人姓名
    pub person_name: Option<String>,
    /// 性别
    pub gender: Option<i32>,
    /// 出生日期
    pub birthday: Option<Date>,
    /// 微信
    pub wechat: Option<String>,
    /// QQ
    pub qq: Option<String>,
    /// 个人手机号
    pub personal_mobile: Option<String>,
    /// 个人邮箱
    pub personal_email: Option<String>,
    /// 昵称/别名
    pub nickname: Option<String>,
    /// 职业/职务
    pub occupation: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 公司官网
    pub website: Option<String>,
    /// 所属行业
    pub industry: Option<i32>,
    /// 客户等级
    pub level: Option<i32>,
    /// 客户来源
    pub source: Option<i32>,
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
    /// 客户类型: 1=企业, 2=个人
    pub customer_type: Option<i32>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 个人姓名
    pub person_name: Option<String>,
    /// 性别
    pub gender: Option<i32>,
    /// 出生日期
    pub birthday: Option<Date>,
    /// 微信
    pub wechat: Option<String>,
    /// QQ
    pub qq: Option<String>,
    /// 个人手机号
    pub personal_mobile: Option<String>,
    /// 个人邮箱
    pub personal_email: Option<String>,
    /// 昵称/别名
    pub nickname: Option<String>,
    /// 职业/职务
    pub occupation: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 公司官网
    pub website: Option<String>,
    /// 所属行业
    pub industry: Option<i32>,
    /// 客户等级
    pub level: Option<i32>,
    /// 客户来源
    pub source: Option<i32>,
    /// 币种（1=人民币, 2=美元, 3=欧元, 4=英镑, 5=日元, 6=港币, 7=澳元）
    pub currency: Option<i32>,
    /// 信用额度
    pub credit_limit: Option<Decimal>,
    /// 信用天数
    pub credit_days: Option<i32>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 负责人名称
    pub assigned_to_name: Option<String>,
    /// 客户来源：0=自建，1=公海来源（删除按钮显隐依赖）
    pub from_pool: Option<i16>,
    /// 创建人ID
    pub created_by: Option<i64>,
    /// 创建人名称
    pub created_by_name: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
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
    /// 跟进记录列表
    pub followups: Option<Vec<crate::modules::crm::model::followup::FollowupListVO>>,
}

impl From<customer::Model> for CustomerDetailVO {
    fn from(item: customer::Model) -> Self {
        CustomerDetailVO {
            id: Option::from(item.id),
            customer_no: item.customer_no,
            customer_type: item.customer_type,
            company_name: item.company_name,
            short_name: item.short_name,
            person_name: item.person_name,
            gender: item.gender,
            birthday: item.birthday,
            wechat: item.wechat.clone(),
            qq: item.qq.clone(),
            personal_mobile: item.personal_mobile,
            personal_email: item.personal_email,
            nickname: item.nickname,
            occupation: item.occupation,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry,
            level: item.level,
            source: item.source,
            currency: item.currency.map(|c| c.to_i32()),
            credit_limit: item.credit_limit,
            credit_days: item.credit_days,
            assigned_to: item.assigned_to,
            assigned_to_name: None,
            from_pool: item.from_pool,
            created_by: item.created_by,
            created_by_name: None,
            create_time: item.create_time,
            cooperated_at: item.cooperated_at,
            birthday_month: item.birthday_month,
            description: item.description,
            custom_fields: item.custom_fields,
            total_deal_amount: item.total_deal_amount,
            total_deal_count: item.total_deal_count,
            last_deal_at: item.last_deal_at,
            next_follow_at: item.next_follow_at,
            followups: None,
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
    /// 客户类型: 1=企业, 2=个人
    pub customer_type: Option<i32>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 个人姓名
    pub person_name: Option<String>,
    /// 性别
    pub gender: Option<i32>,
    /// 个人手机号
    pub personal_mobile: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 所属行业
    pub industry: Option<i32>,
    /// 客户等级
    pub level: Option<i32>,
    /// 客户来源
    pub source: Option<i32>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 负责人名称
    pub assignee_name: Option<String>,
    /// 客户来源：0=自建，1=公海来源（列表按钮显隐依赖）
    pub from_pool: Option<i16>,
    /// 累计成交金额
    pub total_deal_amount: Option<Decimal>,
    /// 最后成交时间
    pub last_deal_at: Option<DateTime>,
    /// 创建人ID
    pub created_by: Option<i64>,
    /// 创建人名称
    pub created_by_name: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 关联标签列表
    pub tags: Option<Vec<CustomerTagVO>>,
    /// 商机数量
    pub opportunity_count: Option<i64>,
    /// 联系人数量
    pub contact_count: Option<i64>,
}

/// 客户标签简要信息（列表展示用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerTagVO {
    pub id: Option<i64>,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
}

impl From<customer::Model> for CustomerListVO {
    fn from(item: customer::Model) -> Self {
        CustomerListVO {
            id: Option::from(item.id),
            customer_no: item.customer_no,
            customer_type: item.customer_type,
            company_name: item.company_name,
            short_name: item.short_name,
            person_name: item.person_name,
            gender: item.gender,
            personal_mobile: item.personal_mobile,
            country: item.country,
            region: item.region,
            industry: item.industry,
            level: item.level,
            source: item.source,
            assigned_to: item.assigned_to,
            assignee_name: None,
            from_pool: item.from_pool,
            total_deal_amount: item.total_deal_amount,
            last_deal_at: item.last_deal_at,
            created_by: item.created_by,
            created_by_name: None,
            create_time: item.create_time,
            tags: None,
            opportunity_count: None,
            contact_count: None,
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
    /// 关键词(搜索公司名称、简称、个人姓名等)
    #[serde(alias = "companyName")]
    pub keywords: Option<String>,
    /// 客户类型: 1=企业, 2=个人
    pub customer_type: Option<i32>,
    /// 客户等级
    pub level: Option<i32>,
    /// 国家
    pub country: Option<String>,
    /// 客户来源
    pub source: Option<i32>,
    /// 行业
    pub industry: Option<i32>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 列表类型：all=全部客户, my=我的客户, subordinate=下属客户, todayFollow=今日跟进客户
    pub list_type: Option<String>,
}

/// 退回公海请求（退回原因两级结构：原因类型必选，选"其他"时补充说明必填）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomerPoolReleaseRequest {
    /// 客户ID
    pub id: Option<i64>,
    /// 退回原因类型：1=跟进无回应 2=客户无意向 3=客户信息无效 4=换业务方向 9=其他
    pub reason_type: Option<i16>,
    /// 退回补充说明（原因类型为"其他"时必填）
    pub reason: Option<String>,
}

/// 客户数据模型操作类
pub struct CustomerModel;

impl CustomerModel {
    /// 新增客户
    pub async fn insert(db: &impl ConnectionTrait, req: &CustomerSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = customer::ActiveModel {
            customer_no: Set(req.customer_no.clone()),
            customer_type: Set(req.customer_type.or(Some(1))),
            company_name: Set(req.company_name.clone()),
            short_name: Set(req.short_name.clone()),
            person_name: Set(req.person_name.clone()),
            gender: Set(req.gender.clone()),
            birthday: Set(req.birthday.clone()),
            wechat: Set(req.wechat.clone()),
            qq: Set(req.qq.clone()),
            personal_mobile: Set(req.personal_mobile.clone()),
            personal_email: Set(req.personal_email.clone()),
            nickname: Set(req.nickname.clone()),
            occupation: Set(req.occupation.clone()),
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
            from_pool: Set(req.from_pool.or(Some(0))),
            ..Default::default()
        };

        Customer::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除客户(软删除，写入删除人/删除时间供回收站使用)
    pub async fn batch_delete_by_ids(db: &impl ConnectionTrait, ids: &Vec<i64>, deleted_by: i64) -> Result<i64, DbErr> {
        Customer::update_many()
            .set(customer::ActiveModel {
                deleted: Set(Some(1)),
                delete_by: Set(Some(deleted_by)),
                delete_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(customer::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新客户信息
    pub async fn update_by_id(db: &impl ConnectionTrait, id: &Option<i64>, req: &CustomerSaveDTO) -> Result<i64, DbErr> {
        // assigned_to 为 None 时不更新，避免误将客户移入公海
        let assigned_to_active = match req.assigned_to {
            Some(v) => Set(Some(v)),
            None => ActiveValue::NotSet,
        };
        // customer_type 不在更新中覆盖，避免误改类型
        let payload = customer::ActiveModel {
            company_name: Set(req.company_name.clone()),
            short_name: Set(req.short_name.clone()),
            person_name: Set(req.person_name.clone()),
            gender: Set(req.gender.clone()),
            birthday: Set(req.birthday.clone()),
            wechat: Set(req.wechat.clone()),
            qq: Set(req.qq.clone()),
            personal_mobile: Set(req.personal_mobile.clone()),
            personal_email: Set(req.personal_email.clone()),
            nickname: Set(req.nickname.clone()),
            occupation: Set(req.occupation.clone()),
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
            assigned_to: assigned_to_active,
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
    pub async fn find_by_id(db: &impl ConnectionTrait, id: i64) -> Result<Option<customer::Model>, DbErr> {
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
        customer_type: Option<i32>,
        level: Option<i32>,
        country: Option<String>,
        source: Option<i32>,
        assigned_to: Option<i64>,
    ) -> Result<(Vec<customer::Model>, i64), DbErr> {
        let mut query = Customer::find()
            .filter(customer::Column::Deleted.eq(0));

        if let Some(k) = keywords.filter(|v| !v.trim().is_empty()) {
            // 关键词同时搜索公司名称、简称、个人姓名、个人手机号
            query = query.filter(
                customer::Column::CompanyName.contains(k.clone())
                    .or(customer::Column::ShortName.contains(k.clone()))
                    .or(customer::Column::PersonName.contains(k.clone()))
                    .or(customer::Column::PersonalMobile.contains(k))
            );
        }
        if let Some(t) = customer_type {
            query = query.filter(customer::Column::CustomerType.eq(t));
        }
        if let Some(l) = level {
            query = query.filter(customer::Column::Level.eq(l));
        }
        if let Some(c) = country.filter(|v| !v.trim().is_empty()) {
            query = query.filter(customer::Column::Country.eq(c));
        }
        if let Some(s) = source {
            query = query.filter(customer::Column::Source.eq(s));
        }
        if let Some(a) = assigned_to {
            query = query.filter(customer::Column::AssignedTo.eq(a));
        } else {
            // 未指定负责人过滤时，默认排除公海客户（assigned_to IS NULL）
            // 公海客户有专门的 select_pool_in_page 查询
            query = query.filter(customer::Column::AssignedTo.is_not_null());
        }

        let paginator = query.order_by_desc(customer::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    /// 分页查询客户列表（支持多负责人过滤）
    pub async fn select_in_page_by_assigned_ids(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        customer_type: Option<i32>,
        level: Option<i32>,
        country: Option<String>,
        source: Option<i32>,
        assigned_ids: Option<Vec<i64>>,
    ) -> Result<(Vec<customer::Model>, i64), DbErr> {
        let mut query = Customer::find()
            .filter(customer::Column::Deleted.eq(0));

        if let Some(k) = keywords.filter(|v| !v.trim().is_empty()) {
            query = query.filter(
                customer::Column::CompanyName.contains(k.clone())
                    .or(customer::Column::ShortName.contains(k.clone()))
                    .or(customer::Column::PersonName.contains(k.clone()))
                    .or(customer::Column::PersonalMobile.contains(k))
            );
        }
        if let Some(t) = customer_type {
            query = query.filter(customer::Column::CustomerType.eq(t));
        }
        if let Some(l) = level {
            query = query.filter(customer::Column::Level.eq(l));
        }
        if let Some(c) = country.filter(|v| !v.trim().is_empty()) {
            query = query.filter(customer::Column::Country.eq(c));
        }
        if let Some(s) = source {
            query = query.filter(customer::Column::Source.eq(s));
        }
        if let Some(ids) = assigned_ids {
            if ids.is_empty() {
                // 没有可查看的用户，返回空结果
                return Ok((vec![], 0));
            }
            query = query.filter(customer::Column::AssignedTo.is_in(ids));
        }

        let paginator = query.order_by_desc(customer::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        let rows = paginator.fetch_page((page - 1) as u64).await?;
        Ok((rows, total))
    }

    /// 查询今日跟进客户（关联 followup 表，按创建人和创建时间过滤）
    /// user_ids: None 表示不过滤（全部数据权限），Some(vec) 表示按用户ID过滤
    pub async fn select_today_follow_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        customer_type: Option<i32>,
        level: Option<i32>,
        country: Option<String>,
        source: Option<i32>,
        user_ids: Option<Vec<i64>>,
    ) -> Result<(Vec<customer::Model>, i64), DbErr> {
        use crate::modules::crm::entity::followup;

        let today = chrono::Local::now().naive_local().date();
        let today_start = chrono::NaiveDateTime::new(today, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let today_end = chrono::NaiveDateTime::new(today, chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap());

        // 子查询：今日有跟进记录的客户ID列表
        let mut fq = followup::Entity::find()
            .filter(followup::Column::Deleted.eq(0))
            .filter(followup::Column::CustomerId.is_not_null())
            .filter(followup::Column::CreateTime.gte(today_start))
            .filter(followup::Column::CreateTime.lte(today_end));

        if let Some(ref ids) = user_ids {
            if ids.is_empty() {
                return Ok((vec![], 0));
            }
            fq = fq.filter(followup::Column::CreatedBy.is_in(ids.clone()));
        }

        let followup_customer_ids = fq
            .all(db)
            .await?
            .into_iter()
            .filter_map(|f| f.customer_id)
            .collect::<std::collections::HashSet<i64>>()
            .into_iter()
            .collect::<Vec<i64>>();

        if followup_customer_ids.is_empty() {
            return Ok((vec![], 0));
        }

        let mut query = Customer::find()
            .filter(customer::Column::Deleted.eq(0))
            .filter(customer::Column::Id.is_in(followup_customer_ids));

        if let Some(k) = keywords.filter(|v| !v.trim().is_empty()) {
            query = query.filter(
                customer::Column::CompanyName.contains(k.clone())
                    .or(customer::Column::ShortName.contains(k.clone()))
                    .or(customer::Column::PersonName.contains(k.clone()))
                    .or(customer::Column::PersonalMobile.contains(k))
            );
        }
        if let Some(t) = customer_type {
            query = query.filter(customer::Column::CustomerType.eq(t));
        }
        if let Some(l) = level {
            query = query.filter(customer::Column::Level.eq(l));
        }
        if let Some(c) = country.filter(|v| !v.trim().is_empty()) {
            query = query.filter(customer::Column::Country.eq(c));
        }
        if let Some(s) = source {
            query = query.filter(customer::Column::Source.eq(s));
        }

        let paginator = query.order_by_desc(customer::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        let rows = paginator.fetch_page((page - 1) as u64).await?;
        Ok((rows, total))
    }

    /// 查询客户总数
    pub async fn select_count(
        db: &DbConn,
        keywords: Option<String>,
        customer_type: Option<i32>,
        level: Option<String>,
        country: Option<String>,
        source: Option<i32>,
        assigned_to: Option<i64>,
    ) -> Result<i64, DbErr> {
        let mut query = Customer::find()
            .filter(customer::Column::Deleted.eq(0));

        if let Some(k) = keywords.filter(|v| !v.trim().is_empty()) {
            query = query.filter(
                customer::Column::CompanyName.contains(k.clone())
                    .or(customer::Column::ShortName.contains(k.clone()))
                    .or(customer::Column::PersonName.contains(k.clone()))
                    .or(customer::Column::PersonalMobile.contains(k))
            );
        }
        if let Some(t) = customer_type {
            query = query.filter(customer::Column::CustomerType.eq(t));
        }
        if let Some(l) = level.filter(|v| !v.trim().is_empty()) {
            query = query.filter(customer::Column::Level.eq(l));
        }
        if let Some(c) = country.filter(|v| !v.trim().is_empty()) {
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

    /// 分页查询公海客户（assigned_to IS NULL）
    pub async fn select_pool_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        customer_type: Option<i32>,
        level: Option<i32>,
        country: Option<String>,
        source: Option<i32>,
        industry: Option<i32>,
    ) -> Result<(Vec<customer::Model>, i64), DbErr> {
        let mut query = Customer::find()
            .filter(customer::Column::Deleted.eq(0))
            .filter(customer::Column::AssignedTo.is_null());

        if let Some(k) = keywords.filter(|v| !v.trim().is_empty()) {
            query = query.filter(
                customer::Column::CompanyName.contains(k.clone())
                    .or(customer::Column::ShortName.contains(k.clone()))
                    .or(customer::Column::PersonName.contains(k.clone()))
                    .or(customer::Column::PersonalMobile.contains(k))
            );
        }
        if let Some(t) = customer_type {
            query = query.filter(customer::Column::CustomerType.eq(t));
        }
        if let Some(l) = level {
            query = query.filter(customer::Column::Level.eq(l));
        }
        if let Some(c) = country.filter(|v| !v.trim().is_empty()) {
            query = query.filter(customer::Column::Country.eq(c));
        }
        if let Some(s) = source {
            query = query.filter(customer::Column::Source.eq(s));
        }
        if let Some(i) = industry {
            query = query.filter(customer::Column::Industry.eq(i));
        }

        let paginator = query.order_by_desc(customer::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        let rows = paginator.fetch_page((page - 1) as u64).await?;
        Ok((rows, total))
    }

    /// 领取公海客户（设置负责人，并标记为公海来源）
    pub async fn claim(db: &impl ConnectionTrait, id: i64, user_id: i64) -> Result<i64, DbErr> {
        let payload = customer::ActiveModel {
            assigned_to: Set(Some(user_id)),
            from_pool: Set(Some(1)),
            updated_by: Set(Some(user_id)),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let result = Customer::update_many()
            .set(payload)
            .filter(customer::Column::Id.eq(id))
            .filter(customer::Column::Deleted.eq(0))
            .filter(customer::Column::AssignedTo.is_null())
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 退回公海（清除负责人）
    pub async fn add_to_pool(db: &impl ConnectionTrait, id: i64, user_id: i64) -> Result<i64, DbErr> {
        let payload = customer::ActiveModel {
            assigned_to: Set(None),
            updated_by: Set(Some(user_id)),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let result = Customer::update_many()
            .set(payload)
            .filter(customer::Column::Id.eq(id))
            .filter(customer::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}
