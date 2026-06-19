use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use crate::core::r#enum::industry_type_enum::IndustryType;
use crate::modules::purchase::entity::{supplier, supplier::Entity as Supplier};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 供应商新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SupplierSaveRequest {
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 联系电话
    pub contact_phone: Option<String>,
    /// 联系邮箱
    pub contact_email: Option<String>,
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
    /// 供应商等级
    pub level: Option<i32>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 信用额度
    pub credit_limit: Option<Decimal>,
    /// 信用天数
    pub credit_days: Option<i32>,
    /// 开户银行
    pub bank_name: Option<String>,
    /// 银行账号
    pub bank_account: Option<String>,
    /// 税务登记号
    pub tax_id: Option<String>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 供应商状态
    pub status: Option<i32>,
    /// 描述/备注
    pub description: Option<String>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
}

impl From<SupplierSaveRequest> for SupplierSaveDTO {
    fn from(item: SupplierSaveRequest) -> Self {
        SupplierSaveDTO {
            id: None,
            supplier_code: None,
            company_name: item.company_name,
            short_name: item.short_name,
            contact_name: item.contact_name,
            contact_phone: item.contact_phone,
            contact_email: item.contact_email,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry,
            level: item.level,
            currency: item.currency,
            credit_limit: item.credit_limit,
            credit_days: item.credit_days,
            bank_name: item.bank_name,
            bank_account: item.bank_account,
            tax_id: item.tax_id,
            tags: item.tags,
            status: item.status,
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

/// 供应商更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SupplierUpdateRequest {
    /// 供应商ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 联系电话
    pub contact_phone: Option<String>,
    /// 联系邮箱
    pub contact_email: Option<String>,
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
    /// 供应商等级
    pub level: Option<i32>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 信用额度
    pub credit_limit: Option<Decimal>,
    /// 信用天数
    pub credit_days: Option<i32>,
    /// 开户银行
    pub bank_name: Option<String>,
    /// 银行账号
    pub bank_account: Option<String>,
    /// 税务登记号
    pub tax_id: Option<String>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 供应商状态
    pub status: Option<i32>,
    /// 描述/备注
    pub description: Option<String>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
}

impl From<SupplierUpdateRequest> for SupplierSaveDTO {
    fn from(item: SupplierUpdateRequest) -> Self {
        SupplierSaveDTO {
            id: item.id,
            supplier_code: None,
            company_name: item.company_name,
            short_name: item.short_name,
            contact_name: item.contact_name,
            contact_phone: item.contact_phone,
            contact_email: item.contact_email,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry,
            level: item.level,
            currency: item.currency,
            credit_limit: item.credit_limit,
            credit_days: item.credit_days,
            bank_name: item.bank_name,
            bank_account: item.bank_account,
            tax_id: item.tax_id,
            tags: item.tags,
            status: item.status,
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

/// 供应商保存DTO（包含新增和更新的所有字段）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SupplierSaveDTO {
    /// 供应商ID
    pub id: Option<i64>,
    /// 供应商编号
    pub supplier_code: Option<String>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 联系电话
    pub contact_phone: Option<String>,
    /// 联系邮箱
    pub contact_email: Option<String>,
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
    /// 供应商等级
    pub level: Option<i32>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 信用额度
    pub credit_limit: Option<Decimal>,
    /// 信用天数
    pub credit_days: Option<i32>,
    /// 开户银行
    pub bank_name: Option<String>,
    /// 银行账号
    pub bank_account: Option<String>,
    /// 税务登记号
    pub tax_id: Option<String>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 供应商状态
    pub status: Option<i32>,
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

/// 供应商详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SupplierDetailVO {
    /// 供应商ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 供应商编号
    pub supplier_code: Option<String>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 联系电话
    pub contact_phone: Option<String>,
    /// 联系邮箱
    pub contact_email: Option<String>,
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
    /// 供应商等级
    pub level: Option<i32>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 信用额度
    pub credit_limit: Option<Decimal>,
    /// 信用天数
    pub credit_days: Option<i32>,
    /// 开户银行
    pub bank_name: Option<String>,
    /// 银行账号
    pub bank_account: Option<String>,
    /// 税务登记号
    pub tax_id: Option<String>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 供应商状态
    pub status: Option<i32>,
    /// 描述/备注
    pub description: Option<String>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
}

impl From<supplier::Model> for SupplierDetailVO {
    fn from(item: supplier::Model) -> Self {
        SupplierDetailVO {
            id: Option::from(item.id),
            supplier_code: item.supplier_no,
            company_name: item.company_name,
            short_name: item.short_name,
            contact_name: item.contact_name,
            contact_phone: item.contact_phone,
            contact_email: item.contact_email,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry,
            level: item.level,
            currency: item.currency,
            credit_limit: item.credit_limit,
            credit_days: item.credit_days,
            bank_name: item.bank_name,
            bank_account: item.bank_account,
            tax_id: item.tax_id,
            tags: item.tags,
            status: item.status,
            description: item.description,
            custom_fields: item.custom_fields,
        }
    }
}

/// 供应商列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SupplierListVO {
    /// 供应商ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 供应商编号
    pub supplier_code: Option<String>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 公司简称
    pub short_name: Option<String>,
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 联系电话
    pub contact_phone: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 供应商等级
    pub level: Option<i32>,
    /// 供应商状态
    pub status: Option<i32>,
}

impl From<supplier::Model> for SupplierListVO {
    fn from(item: supplier::Model) -> Self {
        SupplierListVO {
            id: Option::from(item.id),
            supplier_code: item.supplier_no,
            company_name: item.company_name,
            short_name: item.short_name,
            contact_name: item.contact_name,
            contact_phone: item.contact_phone,
            country: item.country,
            region: item.region,
            level: item.level,
            status: item.status,
        }
    }
}

/// 供应商列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SupplierListQuery {
    /// 页码
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 关键词（搜索公司名称、简称、联系人等）
    pub keywords: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 供应商等级
    pub level: Option<i32>,
    /// 供应商状态
    pub status: Option<i32>,
}

/// 供应商数据模型操作类
pub struct SupplierModel;

impl SupplierModel {
    /// 新增供应商
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `req` - 供应商保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 新增记录的ID
    pub async fn insert(db: &DbConn, req: &SupplierSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = supplier::ActiveModel {
            company_name: Set(req.company_name.clone()),
            short_name: Set(req.short_name.clone()),
            contact_name: Set(req.contact_name.clone()),
            contact_phone: Set(req.contact_phone.clone()),
            contact_email: Set(req.contact_email.clone()),
            country: Set(req.country.clone()),
            address: Set(req.address.clone()),
            website: Set(req.website.clone()),
            industry: Set(req.industry.clone()),
            level: Set(req.level.clone()),
            status: Set(req.status.clone()),
            created_by: Set(req.created_by.clone()),
            created_at: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(now)),
            ..Default::default()
        };

        Supplier::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除供应商（软删除）
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `ids` - 要删除的供应商ID列表
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 删除的记录数
    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Supplier::update_many()
            .set(supplier::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(supplier::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新供应商信息
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 供应商ID
    /// * `req` - 供应商保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 更新的记录数
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &SupplierSaveDTO) -> Result<i64, DbErr> {
        let payload = supplier::ActiveModel {
            company_name: Set(req.company_name.clone()),
            short_name: Set(req.short_name.clone()),
            contact_name: Set(req.contact_name.clone()),
            contact_phone: Set(req.contact_phone.clone()),
            contact_email: Set(req.contact_email.clone()),
            country: Set(req.country.clone()),
            address: Set(req.address.clone()),
            website: Set(req.website.clone()),
            industry: Set(req.industry.clone()),
            level: Set(req.level.clone()),
            status: Set(req.status.clone()),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = Supplier::update_many()
            .set(payload)
            .filter(supplier::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据ID查询供应商详情
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 供应商ID
    ///
    /// # 返回
    /// * `Result<Option<supplier::Model>, DbErr>` - 供应商模型（未删除）
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<supplier::Model>, DbErr> {
        Supplier::find_by_id(id)
            .filter(supplier::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询供应商列表
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `page` - 页码
    /// * `per_page` - 每页大小
    /// * `keywords` - 关键词
    /// * `level` - 供应商等级
    /// * `country` - 国家
    /// * `status` - 供应商状态
    ///
    /// # 返回
    /// * `Result<(Vec<supplier::Model>, i64), DbErr>` - (供应商列表, 总页数)
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        level: Option<i32>,
        country: Option<String>,
        status: Option<i32>,
    ) -> Result<(Vec<supplier::Model>, i64), DbErr> {
        let mut query = Supplier::find()
            .filter(supplier::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(
                Condition::any()
                    .add(supplier::Column::CompanyName.contains(k.clone()))
                    .add(supplier::Column::ContactName.contains(k)),
            );
        }
        if let Some(l) = level {
            query = query.filter(supplier::Column::Level.eq(l));
        }
        if let Some(c) = country {
            query = query.filter(supplier::Column::Country.eq(c));
        }
        if let Some(s) = status {
            query = query.filter(supplier::Column::Status.eq(s));
        }

        let paginator = query.order_by_desc(supplier::Column::CreatedAt).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}