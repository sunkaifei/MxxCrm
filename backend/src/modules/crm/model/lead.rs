use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use crate::core::r#enum::industry_enum::IndustryType;
use crate::core::r#enum::lead_source_enum::LeadSource;
use crate::core::r#enum::lead_status_enum::LeadStatus;
use crate::modules::crm::entity::{lead, lead::Entity as Lead};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 线索新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadSaveRequest {
    /// 公司名称
    pub company_name: Option<String>,
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 职位
    pub title: Option<String>,
    /// 邮箱地址
    pub email: Option<String>,
    /// 固定电话
    pub phone: Option<String>,
    /// 手机号码
    pub mobile: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 公司官网
    pub website: Option<String>,
    /// 所属行业
    pub industry: Option<String>,
    /// 线索来源
    pub source: Option<String>,
    /// 来源详情
    pub source_detail: Option<String>,
    /// 线索状态
    pub status: Option<LeadStatus>,
    /// 线索等级
    pub level: Option<String>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 预算金额
    pub budget: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 下次跟进时间
    pub next_follow_at: Option<DateTime>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 描述/备注
    pub description: Option<String>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
}

impl From<LeadSaveRequest> for LeadSaveDTO {
    fn from(item: LeadSaveRequest) -> Self {
        LeadSaveDTO {
            id: None,
            company_name: item.company_name,
            contact_name: item.contact_name,
            title: item.title,
            email: item.email,
            phone: item.phone,
            mobile: item.mobile,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry.map(|i| i.to_string()),
            source: item.source.map(|s| s.to_string()),
            source_detail: item.source_detail,
            status: item.status,
            level: item.level,
            tags: item.tags,
            budget: item.budget,
            currency: item.currency,
            next_follow_at: item.next_follow_at,
            assigned_to: item.assigned_to,
            converted_to_customer_id: None,
            converted_at: None,
            description: item.description,
            custom_fields: item.custom_fields,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
        }
    }
}

/// 线索更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadUpdateRequest {
    /// 线索ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 职位
    pub title: Option<String>,
    /// 邮箱地址
    pub email: Option<String>,
    /// 固定电话
    pub phone: Option<String>,
    /// 手机号码
    pub mobile: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 公司官网
    pub website: Option<String>,
    /// 所属行业
    pub industry: Option<String>,
    /// 线索来源
    pub source: Option<String>,
    /// 来源详情
    pub source_detail: Option<String>,
    /// 线索状态
    pub status: Option<LeadStatus>,
    /// 线索等级
    pub level: Option<String>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 预算金额
    pub budget: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 下次跟进时间
    pub next_follow_at: Option<DateTime>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 描述/备注
    pub description: Option<String>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
}

impl From<LeadUpdateRequest> for LeadSaveDTO {
    fn from(item: LeadUpdateRequest) -> Self {
        LeadSaveDTO {
            id: item.id,
            company_name: item.company_name,
            contact_name: item.contact_name,
            title: item.title,
            email: item.email,
            phone: item.phone,
            mobile: item.mobile,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry,
            source: item.source,
            source_detail: item.source_detail,
            status: item.status,
            level: item.level,
            tags: item.tags,
            budget: item.budget,
            currency: item.currency,
            next_follow_at: item.next_follow_at,
            assigned_to: item.assigned_to,
            converted_to_customer_id: None,
            converted_at: None,
            description: item.description,
            custom_fields: item.custom_fields,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
        }
    }
}

/// 线索保存DTO（包含新增和更新的所有字段）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadSaveDTO {
    /// 线索ID
    pub id: Option<i64>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 职位
    pub title: Option<String>,
    /// 邮箱地址
    pub email: Option<String>,
    /// 固定电话
    pub phone: Option<String>,
    /// 手机号码
    pub mobile: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 公司官网
    pub website: Option<String>,
    /// 所属行业
    pub industry: Option<String>,
    /// 线索来源
    pub source: Option<String>,
    /// 来源详情
    pub source_detail: Option<String>,
    /// 线索状态
    pub status: Option<LeadStatus>,
    /// 线索等级
    pub level: Option<String>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 预算金额
    pub budget: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 下次跟进时间
    pub next_follow_at: Option<DateTime>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 转换为客户的ID
    pub converted_to_customer_id: Option<i64>,
    /// 转换时间
    pub converted_at: Option<DateTime>,
    /// 描述/备注
    pub description: Option<String>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
    /// 软删除标记
    pub deleted: Option<i32>,
    /// 创建人ID
    pub created_by: Option<i64>,
    /// 创建时间
    pub created_at: Option<DateTime>,
    /// 更新人ID
    pub updated_by: Option<i64>,
    /// 更新时间
    pub updated_at: Option<DateTime>,
}

/// 线索详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LeadDetailVO {
    /// 线索ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 职位
    pub title: Option<String>,
    /// 邮箱地址
    pub email: Option<String>,
    /// 固定电话
    pub phone: Option<String>,
    /// 手机号码
    pub mobile: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 公司官网
    pub website: Option<String>,
    /// 所属行业
    pub industry: Option<String>,
    /// 线索来源
    pub source: Option<String>,
    /// 来源详情
    pub source_detail: Option<String>,
    /// 线索状态
    pub status: Option<LeadStatus>,
    /// 线索等级
    pub level: Option<String>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 预算金额
    pub budget: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 下次跟进时间
    pub next_follow_at: Option<DateTime>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 转换为客户的ID
    pub converted_to_customer_id: Option<i64>,
    /// 转换时间
    pub converted_at: Option<DateTime>,
    /// 描述/备注
    pub description: Option<String>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
}

impl From<lead::Model> for LeadDetailVO {
    fn from(item: lead::Model) -> Self {
        LeadDetailVO {
            id: Option::from(item.id),
            company_name: item.company_name,
            contact_name: item.contact_name,
            title: item.title,
            email: item.email,
            phone: item.phone,
            mobile: item.mobile,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry.map(|i| i.to_string()),
            source: item.source.map(|s| s.to_string()),
            source_detail: item.source_detail,
            status: item.status,
            level: item.level,
            tags: item.tags,
            budget: item.budget,
            currency: item.currency,
            next_follow_at: item.next_follow_at,
            assigned_to: item.assigned_to,
            converted_to_customer_id: item.converted_to_customer_id,
            converted_at: item.converted_at,
            description: item.description,
            custom_fields: item.custom_fields,
        }
    }
}

/// 线索列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LeadListVO {
    /// 线索ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 职位
    pub title: Option<String>,
    /// 邮箱地址
    pub email: Option<String>,
    /// 固定电话
    pub phone: Option<String>,
    /// 手机号码
    pub mobile: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 线索来源
    pub source: Option<String>,
    /// 线索状态
    pub status: Option<LeadStatus>,
    /// 线索等级
    pub level: Option<String>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 创建人ID
    pub created_by: Option<i64>,
    /// 创建时间
    pub created_at: Option<DateTime>,
    /// 下次跟进时间
    pub next_follow_at: Option<DateTime>,
}

impl From<lead::Model> for LeadListVO {
    fn from(item: lead::Model) -> Self {
        LeadListVO {
            id: Option::from(item.id),
            company_name: item.company_name,
            contact_name: item.contact_name,
            title: item.title,
            email: item.email,
            phone: item.phone,
            mobile: item.mobile,
            country: item.country,
            region: item.region,
            source: item.source.map(|s| s.to_string()),
            status: item.status,
            level: item.level,
            assigned_to: item.assigned_to,
            created_by: item.created_by,
            created_at: item.created_at,
            next_follow_at: item.next_follow_at,
        }
    }
}

/// 线索列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LeadListQuery {
    /// 页码
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 关键词（搜索公司名称、联系人等）
    pub keywords: Option<String>,
    /// 线索状态
    pub status: Option<String>,
    /// 线索等级
    pub level: Option<String>,
    /// 线索来源
    pub source: Option<String>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
}

/// 线索状态更新参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LeadStatusUpdateQuery {
    /// 线索ID
    pub id: Option<i64>,
    /// 状态值
    pub status: Option<String>,
}

/// 线索数据模型操作类
pub struct LeadModel;

impl LeadModel {
    /// 新增线索
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `req` - 线索保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 新增记录的ID
    pub async fn insert(db: &DbConn, req: &LeadSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = lead::ActiveModel {
            company_name: Set(req.company_name.clone()),
            contact_name: Set(req.contact_name.clone()),
            title: Set(req.title.clone()),
            email: Set(req.email.clone()),
            phone: Set(req.phone.clone()),
            mobile: Set(req.mobile.clone()),
            country: Set(req.country.clone()),
            region: Set(req.region.clone()),
            address: Set(req.address.clone()),
            website: Set(req.website.clone()),
            industry: Set(req.industry.clone().and_then(|s| IndustryType::from_str(&s))),
            source: Set(req.source.clone().and_then(|s| LeadSource::from_str(&s))),
            source_detail: Set(req.source_detail.clone()),
            status: Set(req.status.clone()),
            level: Set(req.level.clone()),
            tags: Set(req.tags.clone()),
            budget: Set(req.budget.clone()),
            currency: Set(req.currency.clone()),
            next_follow_at: Set(req.next_follow_at.clone()),
            assigned_to: Set(req.assigned_to.clone()),
            description: Set(req.description.clone()),
            custom_fields: Set(req.custom_fields.clone()),
            created_by: Set(req.created_by.clone()),
            created_at: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(now)),
            ..Default::default()
        };

        Lead::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除线索（软删除）
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `ids` - 要删除的线索ID列表
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 删除的记录数
    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Lead::update_many()
            .set(lead::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(lead::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新线索信息
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 线索ID
    /// * `req` - 线索保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 更新的记录数
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &LeadSaveDTO) -> Result<i64, DbErr> {
        let payload = lead::ActiveModel {
            company_name: Set(req.company_name.clone()),
            contact_name: Set(req.contact_name.clone()),
            title: Set(req.title.clone()),
            email: Set(req.email.clone()),
            phone: Set(req.phone.clone()),
            mobile: Set(req.mobile.clone()),
            country: Set(req.country.clone()),
            region: Set(req.region.clone()),
            address: Set(req.address.clone()),
            website: Set(req.website.clone()),
            industry: Set(req.industry.clone().and_then(|s| IndustryType::from_str(&s))),
            source: Set(req.source.clone().and_then(|s| LeadSource::from_str(&s))),
            source_detail: Set(req.source_detail.clone()),
            status: Set(req.status.clone()),
            level: Set(req.level.clone()),
            tags: Set(req.tags.clone()),
            budget: Set(req.budget.clone()),
            currency: Set(req.currency.clone()),
            next_follow_at: Set(req.next_follow_at.clone()),
            assigned_to: Set(req.assigned_to.clone()),
            description: Set(req.description.clone()),
            custom_fields: Set(req.custom_fields.clone()),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = Lead::update_many()
            .set(payload)
            .filter(lead::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据ID查询线索详情
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 线索ID
    ///
    /// # 返回
    /// * `Result<Option<lead::Model>, DbErr>` - 线索模型（未删除）
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<lead::Model>, DbErr> {
        Lead::find_by_id(id)
            .filter(lead::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询线索列表
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `page` - 页码
    /// * `per_page` - 每页大小
    /// * `keywords` - 关键词
    /// * `status` - 线索状态
    /// * `level` - 线索等级
    /// * `source` - 线索来源
    /// * `assigned_to` - 负责人ID
    ///
    /// # 返回
    /// * `Result<(Vec<lead::Model>, i64), DbErr>` - (线索列表, 总页数)
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        status: Option<String>,
        level: Option<String>,
        source: Option<String>,
        assigned_to: Option<i64>,
    ) -> Result<(Vec<lead::Model>, i64), DbErr> {
        let mut query = Lead::find()
            .filter(lead::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(lead::Column::CompanyName.contains(k));
        }
        if let Some(s) = status {
            let status_value: Option<LeadStatus> = match s.as_str() {
                "new" => Some(LeadStatus::New),
                "following" => Some(LeadStatus::Following),
                "converted" => Some(LeadStatus::Converted),
                "invalid" => Some(LeadStatus::Invalid),
                "recycled" => Some(LeadStatus::Recycled),
                "unchecked" => Some(LeadStatus::Unchecked),
                "checking" => Some(LeadStatus::Checking),
                "valid" => Some(LeadStatus::Valid),
                _ => None,
            };
            if let Some(st) = status_value {
                query = query.filter(lead::Column::Status.eq(st));
            }
        }
        if let Some(l) = level {
            query = query.filter(lead::Column::Level.eq(l));
        }
        if let Some(s) = source {
            query = query.filter(lead::Column::Source.eq(s));
        }
        if let Some(a) = assigned_to {
            query = query.filter(lead::Column::AssignedTo.eq(a));
        }

        let paginator = query.order_by_desc(lead::Column::CreatedAt).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    pub async fn update_status(db: &DbConn, id: i64, status: LeadStatus, updated_by: Option<i64>) -> Result<i64, DbErr> {
        let payload = lead::ActiveModel {
            status: Set(Some(status)),
            updated_by: Set(updated_by),
            updated_at: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = Lead::update_many()
            .set(payload)
            .filter(lead::Column::Id.eq(id))
            .filter(lead::Column::Deleted.eq(0))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    pub async fn add_to_pool(db: &DbConn, id: i64, updated_by: Option<i64>) -> Result<i64, DbErr> {
        Self::update_status(db, id, LeadStatus::Valid, updated_by).await
    }
}