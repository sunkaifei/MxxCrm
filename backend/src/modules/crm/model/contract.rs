use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal, Date};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::core::r#enum::contract_status_enum::ContractStatus;
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use crate::modules::crm::entity::{contract, contract::Entity as Contract};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 合同新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContractSaveRequest {
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 商机ID
    pub opportunity_id: Option<i64>,
    /// 合同标题
    pub title: Option<String>,
    /// 合同类型
    pub contract_type: Option<String>,
    /// 合同金额（不含税）
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 合同总金额（含税）
    pub total_amount: Option<Decimal>,
    /// 合同状态
    pub status: Option<ContractStatus>,
    /// 合同开始日期
    pub start_date: Option<Date>,
    /// 合同结束日期
    pub end_date: Option<Date>,
    /// 签署日期
    pub sign_date: Option<Date>,
    /// 付款条款
    pub payment_terms: Option<String>,
    /// 合同文件路径
    pub contract_file: Option<String>,
    /// 备注信息
    pub remark: Option<String>,
}

impl From<ContractSaveRequest> for ContractSaveDTO {
    fn from(item: ContractSaveRequest) -> Self {
        ContractSaveDTO {
            id: None,
            customer_id: item.customer_id,
            opportunity_id: item.opportunity_id,
            title: item.title,
            contract_type: item.contract_type,
            amount: item.amount,
            currency: item.currency,
            tax_amount: item.tax_amount,
            total_amount: item.total_amount,
            status: item.status,
            start_date: item.start_date,
            end_date: item.end_date,
            sign_date: item.sign_date,
            payment_terms: item.payment_terms,
            contract_file: item.contract_file,
            remark: item.remark,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 合同更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContractUpdateRequest {
    /// 合同ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 商机ID
    pub opportunity_id: Option<i64>,
    /// 合同标题
    pub title: Option<String>,
    /// 合同类型
    pub contract_type: Option<String>,
    /// 合同金额（不含税）
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 合同总金额（含税）
    pub total_amount: Option<Decimal>,
    /// 合同状态
    pub status: Option<ContractStatus>,
    /// 合同开始日期
    pub start_date: Option<Date>,
    /// 合同结束日期
    pub end_date: Option<Date>,
    /// 签署日期
    pub sign_date: Option<Date>,
    /// 付款条款
    pub payment_terms: Option<String>,
    /// 合同文件路径
    pub contract_file: Option<String>,
    /// 备注信息
    pub remark: Option<String>,
}

impl From<ContractUpdateRequest> for ContractSaveDTO {
    fn from(item: ContractUpdateRequest) -> Self {
        ContractSaveDTO {
            id: item.id,
            customer_id: item.customer_id,
            opportunity_id: item.opportunity_id,
            title: item.title,
            contract_type: item.contract_type,
            amount: item.amount,
            currency: item.currency,
            tax_amount: item.tax_amount,
            total_amount: item.total_amount,
            status: item.status,
            start_date: item.start_date,
            end_date: item.end_date,
            sign_date: item.sign_date,
            payment_terms: item.payment_terms,
            contract_file: item.contract_file,
            remark: item.remark,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 合同保存DTO（包含新增和更新的所有字段）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContractSaveDTO {
    /// 合同ID
    pub id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 商机ID
    pub opportunity_id: Option<i64>,
    /// 合同标题
    pub title: Option<String>,
    /// 合同类型
    pub contract_type: Option<String>,
    /// 合同金额（不含税）
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 合同总金额（含税）
    pub total_amount: Option<Decimal>,
    /// 合同状态
    pub status: Option<ContractStatus>,
    /// 合同开始日期
    pub start_date: Option<Date>,
    /// 合同结束日期
    pub end_date: Option<Date>,
    /// 签署日期
    pub sign_date: Option<Date>,
    /// 付款条款
    pub payment_terms: Option<String>,
    /// 合同文件路径
    pub contract_file: Option<String>,
    /// 备注信息
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

/// 合同详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ContractDetailVO {
    /// 合同ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 合同编号
    pub contract_no: Option<String>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 商机ID
    pub opportunity_id: Option<i64>,
    /// 合同标题
    pub title: Option<String>,
    /// 合同类型
    pub contract_type: Option<String>,
    /// 合同金额（不含税）
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 合同总金额（含税）
    pub total_amount: Option<Decimal>,
    /// 合同状态
    pub status: Option<ContractStatus>,
    /// 合同开始日期
    pub start_date: Option<Date>,
    /// 合同结束日期
    pub end_date: Option<Date>,
    /// 签署日期
    pub sign_date: Option<Date>,
    /// 付款条款
    pub payment_terms: Option<String>,
    /// 合同文件路径
    pub contract_file: Option<String>,
    /// 备注信息
    pub remark: Option<String>,
}

impl From<contract::Model> for ContractDetailVO {
    fn from(item: contract::Model) -> Self {
        ContractDetailVO {
            id: Option::from(item.id),
            contract_no: item.contract_no,
            customer_id: item.customer_id,
            opportunity_id: item.opportunity_id,
            title: item.title,
            contract_type: item.contract_type,
            amount: item.amount,
            currency: item.currency,
            tax_amount: item.tax_amount,
            total_amount: item.total_amount,
            status: item.status,
            start_date: item.start_date,
            end_date: item.end_date,
            sign_date: item.sign_date,
            payment_terms: item.payment_terms,
            contract_file: item.contract_file,
            remark: item.remark,
        }
    }
}

/// 合同列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ContractListVO {
    /// 合同ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 合同编号
    pub contract_no: Option<String>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 合同标题
    pub title: Option<String>,
    /// 合同类型
    pub contract_type: Option<String>,
    /// 合同金额（不含税）
    pub amount: Option<Decimal>,
    /// 合同总金额（含税）
    pub total_amount: Option<Decimal>,
    /// 合同状态
    pub status: Option<ContractStatus>,
    /// 合同开始日期
    pub start_date: Option<Date>,
    /// 合同结束日期
    pub end_date: Option<Date>,
}

impl From<contract::Model> for ContractListVO {
    fn from(item: contract::Model) -> Self {
        ContractListVO {
            id: Option::from(item.id),
            contract_no: item.contract_no,
            customer_id: item.customer_id,
            title: item.title,
            contract_type: item.contract_type,
            amount: item.amount,
            total_amount: item.total_amount,
            status: item.status,
            start_date: item.start_date,
            end_date: item.end_date,
        }
    }
}

/// 合同列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContractListQuery {
    /// 页码
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 关键词（搜索合同标题、编号等）
    pub keywords: Option<String>,
    /// 合同状态
    pub status: Option<ContractStatus>,
    /// 客户ID
    pub customer_id: Option<i64>,
}

/// 合同数据模型操作类
pub struct ContractModel;

impl ContractModel {
    /// 新增合同
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `req` - 合同保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 新增记录的ID
    pub async fn insert(db: &DbConn, req: &ContractSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = contract::ActiveModel {
            customer_id: Set(req.customer_id.clone()),
            opportunity_id: Set(req.opportunity_id.clone()),
            title: Set(req.title.clone()),
            contract_type: Set(req.contract_type.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            tax_amount: Set(req.tax_amount.clone()),
            total_amount: Set(req.total_amount.clone()),
            status: Set(req.status.clone()),
            start_date: Set(req.start_date.clone()),
            end_date: Set(req.end_date.clone()),
            sign_date: Set(req.sign_date.clone()),
            payment_terms: Set(req.payment_terms.clone()),
            contract_file: Set(req.contract_file.clone()),
            remark: Set(req.remark.clone()),
            created_by: Set(req.created_by.clone()),
            create_time: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };

        Contract::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除合同（软删除）
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `ids` - 要删除的合同ID列表
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 删除的记录数
    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Contract::update_many()
            .set(contract::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(contract::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新合同信息
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 合同ID
    /// * `req` - 合同保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 更新的记录数
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &ContractSaveDTO) -> Result<i64, DbErr> {
        let payload = contract::ActiveModel {
            customer_id: Set(req.customer_id.clone()),
            opportunity_id: Set(req.opportunity_id.clone()),
            title: Set(req.title.clone()),
            contract_type: Set(req.contract_type.clone()),
            amount: Set(req.amount.clone()),
            currency: Set(req.currency.clone()),
            tax_amount: Set(req.tax_amount.clone()),
            total_amount: Set(req.total_amount.clone()),
            status: Set(req.status.clone()),
            start_date: Set(req.start_date.clone()),
            end_date: Set(req.end_date.clone()),
            sign_date: Set(req.sign_date.clone()),
            payment_terms: Set(req.payment_terms.clone()),
            contract_file: Set(req.contract_file.clone()),
            remark: Set(req.remark.clone()),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = Contract::update_many()
            .set(payload)
            .filter(contract::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据ID查询合同详情
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 合同ID
    ///
    /// # 返回
    /// * `Result<Option<contract::Model>, DbErr>` - 合同模型（未删除）
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<contract::Model>, DbErr> {
        Contract::find_by_id(id)
            .filter(contract::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询合同列表
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `page` - 页码
    /// * `per_page` - 每页大小
    /// * `keywords` - 关键词
    /// * `status` - 合同状态
    /// * `customer_id` - 客户ID
    ///
    /// # 返回
    /// * `Result<(Vec<contract::Model>, i64), DbErr>` - (合同列表, 总页数)
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        status: Option<ContractStatus>,
        customer_id: Option<i64>,
    ) -> Result<(Vec<contract::Model>, i64), DbErr> {
        let mut query = Contract::find()
            .filter(contract::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(
                Condition::any()
                    .add(contract::Column::Title.contains(k.clone()))
                    .add(contract::Column::ContractNo.contains(k)),
            );
        }
        if let Some(s) = status {
            query = query.filter(contract::Column::Status.eq(s));
        }
        if let Some(c) = customer_id {
            query = query.filter(contract::Column::CustomerId.eq(c));
        }

        let paginator = query.order_by_desc(contract::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}