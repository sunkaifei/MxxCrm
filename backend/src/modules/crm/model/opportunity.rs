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
use crate::core::r#enum::lead_source_enum::LeadSource;
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
    pub stage: Option<i32>,
    /// 赢单概率
    pub probability: Option<i32>,
    /// 商机金额
    pub amount: Option<Decimal>,
    /// 币种（1=人民币, 2=美元, 3=欧元, 4=英镑, 5=日元, 6=港币, 7=澳元）
    pub currency: Option<i32>,
    /// 预计成交日期
    pub expected_close_date: Option<Date>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 商机来源（1=官网, 2=展会, 3=社交媒体, 4=客户转介, 5=陌生拜访, 6=海关数据, 7=邮件营销, 8=阿里国际站, 9=Amazon, 10=TikTok, 11=微信, 12=其他）
    pub source: Option<i32>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
    /// 需求摘要
    pub requirement_summary: Option<String>,
    /// 解决方案摘要
    pub solution_summary: Option<String>,
    /// 报价状态
    pub quote_status: Option<i32>,
    /// 订单状态
    pub order_status: Option<i32>,
    /// 合同状态
    pub contract_status: Option<i32>,
    /// 发货状态
    pub shipment_status: Option<i32>,
    /// 付款状态
    pub payment_status: Option<i32>,
    /// 发票状态
    pub invoice_status: Option<i32>,
}

/// i32 转 CurrencyCode
pub fn i32_to_currency_code(v: i32) -> Option<CurrencyCode> {
    match v {
        1 => Some(CurrencyCode::CNY),
        2 => Some(CurrencyCode::USD),
        3 => Some(CurrencyCode::EUR),
        4 => Some(CurrencyCode::GBP),
        5 => Some(CurrencyCode::JPY),
        6 => Some(CurrencyCode::HKD),
        7 => Some(CurrencyCode::AUD),
        _ => None,
    }
}

/// CurrencyCode 转 i32
pub fn currency_code_to_i32(v: CurrencyCode) -> i32 {
    match v {
        CurrencyCode::CNY => 1,
        CurrencyCode::USD => 2,
        CurrencyCode::EUR => 3,
        CurrencyCode::GBP => 4,
        CurrencyCode::JPY => 5,
        CurrencyCode::HKD => 6,
        CurrencyCode::AUD => 7,
    }
}

/// i32 转 LeadSource
pub fn i32_to_lead_source(v: i32) -> Option<LeadSource> {
    match v {
        1 => Some(LeadSource::Website),
        2 => Some(LeadSource::Exhibition),
        3 => Some(LeadSource::Social),
        4 => Some(LeadSource::Referral),
        5 => Some(LeadSource::ColdCall),
        6 => Some(LeadSource::Customs),
        7 => Some(LeadSource::Email),
        8 => Some(LeadSource::Alibaba),
        9 => Some(LeadSource::Amazon),
        10 => Some(LeadSource::Tiktok),
        11 => Some(LeadSource::Wechat),
        12 => Some(LeadSource::Other),
        _ => None,
    }
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
            currency: item.currency.and_then(i32_to_currency_code),
            expected_close_date: item.expected_close_date,
            assigned_to: item.assigned_to,
            source: item.source.and_then(i32_to_lead_source),
            tags: item.tags,
            custom_fields: item.custom_fields,
            requirement_summary: item.requirement_summary,
            solution_summary: item.solution_summary,
            quote_status: item.quote_status,
            order_status: item.order_status,
            contract_status: item.contract_status,
            shipment_status: item.shipment_status,
            payment_status: item.payment_status,
            invoice_status: item.invoice_status,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
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
    pub stage: Option<i32>,
    /// 赢单概率
    pub probability: Option<i32>,
    /// 商机金额
    pub amount: Option<Decimal>,
    /// 币种（1=人民币, 2=美元, 3=欧元, 4=英镑, 5=日元, 6=港币, 7=澳元）
    pub currency: Option<i32>,
    /// 预计成交日期
    pub expected_close_date: Option<Date>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 商机来源（1=官网, 2=展会, 3=社交媒体, 4=客户转介, 5=陌生拜访, 6=海关数据, 7=邮件营销, 8=阿里国际站, 9=Amazon, 10=TikTok, 11=微信, 12=其他）
    pub source: Option<i32>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
    /// 需求摘要
    pub requirement_summary: Option<String>,
    /// 解决方案摘要
    pub solution_summary: Option<String>,
    /// 报价状态
    pub quote_status: Option<i32>,
    /// 订单状态
    pub order_status: Option<i32>,
    /// 合同状态
    pub contract_status: Option<i32>,
    /// 发货状态
    pub shipment_status: Option<i32>,
    /// 付款状态
    pub payment_status: Option<i32>,
    /// 发票状态
    pub invoice_status: Option<i32>,
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
            currency: item.currency.and_then(i32_to_currency_code),
            expected_close_date: item.expected_close_date,
            assigned_to: item.assigned_to,
            source: item.source.and_then(i32_to_lead_source),
            tags: item.tags,
            custom_fields: item.custom_fields,
            requirement_summary: item.requirement_summary,
            solution_summary: item.solution_summary,
            quote_status: item.quote_status,
            order_status: item.order_status,
            contract_status: item.contract_status,
            shipment_status: item.shipment_status,
            payment_status: item.payment_status,
            invoice_status: item.invoice_status,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
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
    pub stage: Option<i32>,
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
    /// 需求摘要
    pub requirement_summary: Option<String>,
    /// 解决方案摘要
    pub solution_summary: Option<String>,
    /// 报价状态
    pub quote_status: Option<i32>,
    /// 订单状态
    pub order_status: Option<i32>,
    /// 合同状态
    pub contract_status: Option<i32>,
    /// 发货状态
    pub shipment_status: Option<i32>,
    /// 付款状态
    pub payment_status: Option<i32>,
    /// 发票状态
    pub invoice_status: Option<i32>,
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
    pub stage: Option<i32>,
    /// 赢单概率
    pub probability: Option<i32>,
    /// 商机金额
    pub amount: Option<Decimal>,
    /// 币种（1=人民币, 2=美元, 3=欧元, 4=英镑, 5=日元, 6=港币, 7=澳元）
    pub currency: Option<i32>,
    /// 预计成交日期
    pub expected_close_date: Option<Date>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 商机来源（1=官网, 2=展会, 3=社交媒体, 4=客户转介, 5=陌生拜访, 6=海关数据, 7=邮件营销, 8=阿里国际站, 9=Amazon, 10=TikTok, 11=微信, 12=其他）
    pub source: Option<i32>,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 自定义字段（JSON格式）
    pub custom_fields: Option<serde_json::Value>,
    /// 需求摘要
    pub requirement_summary: Option<String>,
    /// 解决方案摘要
    pub solution_summary: Option<String>,
    /// 报价状态
    pub quote_status: Option<i32>,
    /// 订单状态
    pub order_status: Option<i32>,
    /// 合同状态
    pub contract_status: Option<i32>,
    /// 发货状态
    pub shipment_status: Option<i32>,
    /// 付款状态
    pub payment_status: Option<i32>,
    /// 发票状态
    pub invoice_status: Option<i32>,
    /// 客户名称
    pub customer_name: Option<String>,
    /// 客户行业
    pub customer_industry: Option<i32>,
    /// 客户等级
    pub customer_level: Option<i32>,
    /// 客户国家
    pub customer_country: Option<String>,
    /// 客户地址
    pub customer_address: Option<String>,
    /// 客户官网
    pub customer_website: Option<String>,
    /// 客户编号
    pub customer_no: Option<String>,
    /// 客户公司简称
    pub customer_short_name: Option<String>,
    /// 客户信用额度
    pub customer_credit_limit: Option<Decimal>,
    /// 客户信用天数
    pub customer_credit_days: Option<i32>,
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 联系人职位
    pub contact_title: Option<String>,
    /// 联系人手机
    pub contact_mobile: Option<String>,
    /// 联系人邮箱
    pub contact_email: Option<String>,
    /// 联系人电话
    pub contact_phone: Option<String>,
    /// 联系人微信
    pub contact_wechat: Option<String>,
    /// 创建人名称
    pub created_by_name: Option<String>,
    /// 负责人名称
    pub assignee: Option<String>,
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
            currency: item.currency.map(currency_code_to_i32),
            expected_close_date: item.expected_close_date,
            assigned_to: item.assigned_to,
            source: item.source.map(|s| s.to_i32()),
            tags: item.tags,
            custom_fields: item.custom_fields,
            requirement_summary: item.requirement_summary,
            solution_summary: item.solution_summary,
            quote_status: item.quote_status,
            order_status: item.order_status,
            contract_status: item.contract_status,
            shipment_status: item.shipment_status,
            payment_status: item.payment_status,
            invoice_status: item.invoice_status,
            customer_name: None,
            customer_industry: None,
            customer_level: None,
            customer_country: None,
            customer_address: None,
            customer_website: None,
            customer_no: None,
            customer_short_name: None,
            customer_credit_limit: None,
            customer_credit_days: None,
            contact_name: None,
            contact_title: None,
            contact_mobile: None,
            contact_email: None,
            contact_phone: None,
            contact_wechat: None,
            created_by_name: None,
            assignee: None,
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
    /// 客户名称
    pub customer_name: Option<String>,
    /// 商机标题
    pub title: Option<String>,
    /// 销售阶段
    pub stage: Option<i32>,
    /// 赢单概率
    pub probability: Option<i32>,
    /// 商机金额
    pub amount: Option<Decimal>,
    /// 币种（1=人民币, 2=美元, 3=欧元, 4=英镑, 5=日元, 6=港币, 7=澳元）
    pub currency: Option<i32>,
    /// 预计成交日期
    pub expected_close_date: Option<Date>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 商机来源（1=官网, 2=展会, 3=社交媒体, 4=客户转介, 5=陌生拜访, 6=海关数据, 7=邮件营销, 8=阿里国际站, 9=Amazon, 10=TikTok, 11=微信, 12=其他）
    pub source: Option<i32>,
    /// 创建人ID
    pub created_by: Option<i64>,
    /// 创建人名称（录入人）
    pub created_by_name: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 报价数量
    pub quote_count: Option<i64>,
    /// 联系人ID
    pub contact_id: Option<i64>,
    /// 联系人姓名
    pub contact_name: Option<String>,
}

impl From<opportunity::Model> for OpportunityListVO {
    fn from(item: opportunity::Model) -> Self {
        OpportunityListVO {
            id: Option::from(item.id),
            opportunity_no: item.opportunity_no,
            customer_id: item.customer_id,
            customer_name: None,
            title: item.title,
            stage: item.stage,
            probability: item.probability,
            amount: item.amount,
            currency: item.currency.map(currency_code_to_i32),
            expected_close_date: item.expected_close_date,
            assigned_to: item.assigned_to,
            source: item.source.map(|s| s.to_i32()),
            created_by: item.created_by,
            created_by_name: None,
            create_time: item.create_time,
            quote_count: None,
            contact_id: item.contact_id,
            contact_name: None,
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
    pub stage: Option<i32>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 列表类型：all=全部 my=我的商机 subordinate=下属商机
    pub list_type: Option<String>,
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
            contact_id: Set(req.contact_id.clone()),
            lead_id: Set(req.lead_id.clone()),
            title: Set(req.title.clone()),
            description: Set(req.description.clone()),
            stage: Set(req.stage),
            probability: Set(req.probability.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            expected_close_date: Set(req.expected_close_date.clone()),
            source: Set(req.source.clone()),
            assigned_to: Set(req.assigned_to.clone()),
            created_by: Set(req.created_by.clone()),
            create_time: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(now)),
            requirement_summary: Set(req.requirement_summary.clone()),
            solution_summary: Set(req.solution_summary.clone()),
            quote_status: Set(req.quote_status),
            order_status: Set(req.order_status),
            contract_status: Set(req.contract_status),
            shipment_status: Set(req.shipment_status),
            payment_status: Set(req.payment_status),
            invoice_status: Set(req.invoice_status),
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
            contact_id: Set(req.contact_id.clone()),
            lead_id: Set(req.lead_id.clone()),
            title: Set(req.title.clone()),
            description: Set(req.description.clone()),
            stage: Set(req.stage),
            probability: Set(req.probability.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            expected_close_date: Set(req.expected_close_date.clone()),
            source: Set(req.source.clone()),
            assigned_to: Set(req.assigned_to.clone()),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            requirement_summary: Set(req.requirement_summary.clone()),
            solution_summary: Set(req.solution_summary.clone()),
            quote_status: Set(req.quote_status),
            order_status: Set(req.order_status),
            contract_status: Set(req.contract_status),
            shipment_status: Set(req.shipment_status),
            payment_status: Set(req.payment_status),
            invoice_status: Set(req.invoice_status),
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

    /// 根据客户ID和商机名称查询商机
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `customer_id` - 客户ID
    /// * `name` - 商机名称
    /// * `exclude_id` - 排除的商机ID（编辑时排除自身）
    ///
    /// # 返回
    /// * `Result<Option<opportunity::Model>, DbErr>` - 商机模型（未删除）
    pub async fn find_by_customer_and_name(
        db: &DbConn,
        customer_id: i64,
        name: &str,
        exclude_id: Option<i64>,
    ) -> Result<Option<opportunity::Model>, DbErr> {
        let mut query = Opportunity::find()
            .filter(opportunity::Column::CustomerId.eq(customer_id))
            .filter(opportunity::Column::Title.eq(name))
            .filter(opportunity::Column::Deleted.eq(0));

        if let Some(exclude_id) = exclude_id {
            query = query.filter(opportunity::Column::Id.ne(exclude_id));
        }

        query.one(db).await
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
        stage: Option<i32>,
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

        let paginator = query.order_by_desc(opportunity::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }

    /// 按负责人ID集合分页查询商机
    pub async fn select_in_page_by_assigned_ids(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        stage: Option<i32>,
        assigned_ids: Option<Vec<i64>>,
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
        if let Some(ids) = assigned_ids {
            if ids.is_empty() {
                // 空集合：返回空结果
                return Ok((Vec::new(), 0));
            }
            query = query.filter(opportunity::Column::AssignedTo.is_in(ids));
        }
        if let Some(c) = customer_id {
            query = query.filter(opportunity::Column::CustomerId.eq(c));
        }

        let paginator = query.order_by_desc(opportunity::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }
}
