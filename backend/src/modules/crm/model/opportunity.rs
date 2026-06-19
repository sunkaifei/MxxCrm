use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal, Date};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use crate::core::r#enum::lead_source_enum::LeadSource;
use crate::core::r#enum::opportunity_stage_enum::OpportunityStage;
use crate::modules::crm::entity::{opportunity, opportunity::Entity as Opportunity};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 商机新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OpportunitySaveRequest {
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 联系人ID
    pub contact_id: Option<i64>,
    /// 线索ID
    pub lead_id: Option<i64>,
    /// 商机标题
    pub title: Option<String>,
    /// 商机描述
    pub description: Option<String>,
    /// 销售阶段
    pub stage: Option<OpportunityStage>,
    /// 赢单概率
    pub probability: Option<i32>,
    /// 商机金额
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 预计成交日期
    pub expected_close_date: Option<Date>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 商机来源
    pub source: Option<LeadSource>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
}

impl From<OpportunitySaveRequest> for OpportunitySaveDTO {
    fn from(item: OpportunitySaveRequest) -> Self {
        OpportunitySaveDTO {
            id: None,
            customer_id: item.customer_id,
            contact_id: item.contact_id,
            lead_id: item.lead_id,
            title: item.title,
            description: item.description,
            stage: item.stage,
            probability: item.probability,
            amount: item.amount,
            currency: item.currency,
            expected_close_date: item.expected_close_date,
            assigned_to: item.assigned_to,
            source: item.source,
            tags: item.tags,
            custom_fields: item.custom_fields,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
        }
    }
}

/// 商机更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OpportunityUpdateRequest {
    /// 商机ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 联系人ID
    pub contact_id: Option<i64>,
    /// 线索ID
    pub lead_id: Option<i64>,
    /// 商机标题
    pub title: Option<String>,
    /// 商机描述
    pub description: Option<String>,
    /// 销售阶段
    pub stage: Option<OpportunityStage>,
    /// 赢单概率
    pub probability: Option<i32>,
    /// 商机金额
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 预计成交日期
    pub expected_close_date: Option<Date>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 商机来源
    pub source: Option<LeadSource>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
}

impl From<OpportunityUpdateRequest> for OpportunitySaveDTO {
    fn from(item: OpportunityUpdateRequest) -> Self {
        OpportunitySaveDTO {
            id: item.id,
            customer_id: item.customer_id,
            contact_id: item.contact_id,
            lead_id: item.lead_id,
            title: item.title,
            description: item.description,
            stage: item.stage,
            probability: item.probability,
            amount: item.amount,
            currency: item.currency,
            expected_close_date: item.expected_close_date,
            assigned_to: item.assigned_to,
            source: item.source,
            tags: item.tags,
            custom_fields: item.custom_fields,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
        }
    }
}

/// 商机保存DTO（包含新增和更新的所有字段）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OpportunitySaveDTO {
    /// 商机ID
    pub id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 联系人ID
    pub contact_id: Option<i64>,
    /// 线索ID
    pub lead_id: Option<i64>,
    /// 商机标题
    pub title: Option<String>,
    /// 商机描述
    pub description: Option<String>,
    /// 销售阶段
    pub stage: Option<OpportunityStage>,
    /// 赢单概率
    pub probability: Option<i32>,
    /// 商机金额
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 预计成交日期
    pub expected_close_date: Option<Date>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 商机来源
    pub source: Option<LeadSource>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
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

/// 商机详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OpportunityDetailVO {
    /// 商机ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 商机编号
    pub opportunity_no: Option<String>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 联系人ID
    pub contact_id: Option<i64>,
    /// 线索ID
    pub lead_id: Option<i64>,
    /// 商机标题
    pub title: Option<String>,
    /// 商机描述
    pub description: Option<String>,
    /// 销售阶段
    pub stage: Option<OpportunityStage>,
    /// 赢单概率
    pub probability: Option<i32>,
    /// 商机金额
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 预计成交日期
    pub expected_close_date: Option<Date>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 商机来源
    pub source: Option<LeadSource>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
}

impl From<opportunity::Model> for OpportunityDetailVO {
    fn from(item: opportunity::Model) -> Self {
        OpportunityDetailVO {
            id: Option::from(item.id),
            opportunity_no: item.opportunity_no,
            customer_id: item.customer_id,
            contact_id: item.contact_id,
            lead_id: item.lead_id,
            title: item.title,
            description: item.description,
            stage: item.stage,
            probability: item.probability,
            amount: item.amount,
            currency: item.currency,
            expected_close_date: item.expected_close_date,
            assigned_to: item.assigned_to,
            source: item.source,
            tags: item.tags,
            custom_fields: item.custom_fields,
        }
    }
}

/// 商机列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OpportunityListVO {
    /// 商机ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 商机编号
    pub opportunity_no: Option<String>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 商机标题
    pub title: Option<String>,
    /// 销售阶段
    pub stage: Option<OpportunityStage>,
    /// 赢单概率
    pub probability: Option<i32>,
    /// 商机金额
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 预计成交日期
    pub expected_close_date: Option<Date>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
}

impl From<opportunity::Model> for OpportunityListVO {
    fn from(item: opportunity::Model) -> Self {
        OpportunityListVO {
            id: Option::from(item.id),
            opportunity_no: item.opportunity_no,
            customer_id: item.customer_id,
            title: item.title,
            stage: item.stage,
            probability: item.probability,
            amount: item.amount,
            currency: item.currency,
            expected_close_date: item.expected_close_date,
            assigned_to: item.assigned_to,
        }
    }
}

/// 商机列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpportunityListQuery {
    /// 页码
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 关键词（搜索商机标题等）
    pub keywords: Option<String>,
    /// 销售阶段
    pub stage: Option<OpportunityStage>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
}

/// 商机数据模型操作类
pub struct OpportunityModel;

impl OpportunityModel {
    /// 新增商机
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `req` - 商机保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 新增记录的ID
    pub async fn insert(db: &DbConn, req: &OpportunitySaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = opportunity::ActiveModel {
            customer_id: Set(req.customer_id.clone()),
            title: Set(req.title.clone()),
            description: Set(req.description.clone()),
            stage: Set(req.stage.clone()),
            probability: Set(req.probability.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            expected_close_date: Set(req.expected_close_date.clone()),
            assigned_to: Set(req.assigned_to.clone()),
            created_by: Set(req.created_by.clone()),
            created_at: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(now)),
            ..Default::default()
        };

        Opportunity::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除商机（软删除）
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `ids` - 要删除的商机ID列表
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 删除的记录数
    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Opportunity::update_many()
            .set(opportunity::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(opportunity::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新商机信息
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 商机ID
    /// * `req` - 商机保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 更新的记录数
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &OpportunitySaveDTO) -> Result<i64, DbErr> {
        let payload = opportunity::ActiveModel {
            customer_id: Set(req.customer_id.clone()),
            title: Set(req.title.clone()),
            description: Set(req.description.clone()),
            stage: Set(req.stage.clone()),
            probability: Set(req.probability.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            expected_close_date: Set(req.expected_close_date.clone()),
            assigned_to: Set(req.assigned_to.clone()),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = Opportunity::update_many()
            .set(payload)
            .filter(opportunity::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据ID查询商机详情
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 商机ID
    ///
    /// # 返回
    /// * `Result<Option<opportunity::Model>, DbErr>` - 商机模型（未删除）
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<opportunity::Model>, DbErr> {
        Opportunity::find_by_id(id)
            .filter(opportunity::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询商机列表
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `page` - 页码
    /// * `per_page` - 每页大小
    /// * `keywords` - 关键词
    /// * `stage` - 销售阶段
    /// * `assigned_to` - 负责人ID
    /// * `customer_id` - 客户ID
    ///
    /// # 返回
    /// * `Result<(Vec<opportunity::Model>, i64), DbErr>` - (商机列表, 总页数)
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        stage: Option<OpportunityStage>,
        assigned_to: Option<i64>,
        customer_id: Option<i64>,
    ) -> Result<(Vec<opportunity::Model>, i64), DbErr> {
        let mut query = Opportunity::find()
            .filter(opportunity::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(opportunity::Column::Title.contains(k));
        }
        if let Some(s) = stage {
            query = query.filter(opportunity::Column::Stage.eq(s));
        }
        if let Some(a) = assigned_to {
            query = query.filter(opportunity::Column::AssignedTo.eq(a));
        }
        if let Some(c) = customer_id {
            query = query.filter(opportunity::Column::CustomerId.eq(c));
        }

        let paginator = query.order_by_desc(opportunity::Column::CreatedAt).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}