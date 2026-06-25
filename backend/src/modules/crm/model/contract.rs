use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal, Date};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::core::r#enum::contract_status_enum::ContractStatus;
use crate::core::r#enum::contract_type_enum::ContractType;
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use crate::modules::crm::entity::{contract, contract_approval_log, contract::Entity as Contract, contract_approval_log::Entity as ContractApprovalLog};
use crate::modules::crm::entity::customer::{Entity as Customer, Column as CustomerColumn};
use crate::modules::approval::model::approval::ApprovalInstanceVO;
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
    pub contract_type: Option<ContractType>,
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
    /// 交付条款
    pub delivery_terms: Option<String>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 合同文件路径
    pub contract_file: Option<String>,
    /// 合同扫描件图片（JSON数组）
    pub contract_images: Option<String>,
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
            delivery_terms: item.delivery_terms,
            assigned_to: item.assigned_to,
            contract_file: item.contract_file,
            contract_images: item.contract_images,
            approval_status: None,
            current_approval_stage: None,
            next_approver_id: None,
            approval_amount_limit: None,
            instance_id: None,
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
    pub contract_type: Option<ContractType>,
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
    /// 交付条款
    pub delivery_terms: Option<String>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 合同文件路径
    pub contract_file: Option<String>,
    /// 合同扫描件图片（JSON数组）
    pub contract_images: Option<String>,
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
            delivery_terms: item.delivery_terms,
            assigned_to: item.assigned_to,
            contract_file: item.contract_file,
            contract_images: item.contract_images,
            approval_status: None,
            current_approval_stage: None,
            next_approver_id: None,
            approval_amount_limit: None,
            instance_id: None,
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
    pub contract_type: Option<ContractType>,
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
    /// 交付条款
    pub delivery_terms: Option<String>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 合同文件路径
    pub contract_file: Option<String>,
    /// 合同扫描件图片（JSON数组）
    pub contract_images: Option<String>,
    /// 审批状态（0-草稿, 1-待审批, 2-审批中, 3-已通过, 4-已驳回）
    pub approval_status: Option<i32>,
    /// 当前审批阶段（1-第一级审批, 2-第二级审批）
    pub current_approval_stage: Option<i32>,
    /// 下一审批人ID
    pub next_approver_id: Option<i64>,
    /// 审批金额阈值（用于分级审批）
    pub approval_amount_limit: Option<Decimal>,
    /// 审批实例ID
    pub instance_id: Option<i64>,
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
    pub contract_type: Option<ContractType>,
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
    /// 交付条款
    pub delivery_terms: Option<String>,
    /// 负责人ID
    pub assigned_to: Option<i64>,
    /// 合同文件路径
    pub contract_file: Option<String>,
    /// 合同扫描件图片（JSON数组）
    pub contract_images: Option<String>,
    /// 审批状态（0-草稿, 1-待审批, 2-审批中, 3-已通过, 4-已驳回）
    pub approval_status: Option<i32>,
    /// 当前审批阶段（1-第一级审批, 2-第二级审批）
    pub current_approval_stage: Option<i32>,
    /// 下一审批人ID
    pub next_approver_id: Option<i64>,
    /// 审批金额阈值
    pub approval_amount_limit: Option<Decimal>,
    /// 审批实例ID
    pub instance_id: Option<i64>,
    /// 备注信息
    pub remark: Option<String>,
    /// 审批日志列表
    pub approval_logs: Option<Vec<ContractApprovalLogVO>>,
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
            delivery_terms: item.delivery_terms,
            assigned_to: item.assigned_to,
            contract_file: item.contract_file,
            contract_images: item.contract_images,
            approval_status: item.approval_status,
            current_approval_stage: item.current_approval_stage,
            next_approver_id: item.next_approver_id,
            approval_amount_limit: item.approval_amount_limit,
            instance_id: item.instance_id,
            remark: item.remark,
            approval_logs: None,
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
    /// 审批状态（0-草稿, 1-待审批, 2-审批中, 3-已通过, 4-已驳回）
    pub approval_status: Option<i32>,
    /// 审批实例ID
    pub instance_id: Option<i64>,
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
            contract_type: item.contract_type.map(|c| c.to_string()),
            amount: item.amount,
            total_amount: item.total_amount,
            status: item.status,
            approval_status: item.approval_status,
            instance_id: item.instance_id,
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
            delivery_terms: Set(req.delivery_terms.clone()),
            assigned_to: Set(req.assigned_to.clone()),
            contract_file: Set(req.contract_file.clone()),
            contract_images: Set(req.contract_images.clone()),
            approval_status: Set(req.approval_status.clone()),
            current_approval_stage: Set(req.current_approval_stage.clone()),
            next_approver_id: Set(req.next_approver_id.clone()),
            approval_amount_limit: Set(req.approval_amount_limit.clone()),
            instance_id: Set(req.instance_id.clone()),
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
            delivery_terms: Set(req.delivery_terms.clone()),
            assigned_to: Set(req.assigned_to.clone()),
            contract_file: Set(req.contract_file.clone()),
            contract_images: Set(req.contract_images.clone()),
            approval_status: Set(req.approval_status.clone()),
            current_approval_stage: Set(req.current_approval_stage.clone()),
            next_approver_id: Set(req.next_approver_id.clone()),
            approval_amount_limit: Set(req.approval_amount_limit.clone()),
            instance_id: Set(req.instance_id.clone()),
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

    pub async fn get_approval_logs(db: &DbConn, contract_id: i64) -> Result<Vec<contract_approval_log::Model>, DbErr> {
        ContractApprovalLog::find()
            .filter(contract_approval_log::Column::ContractId.eq(contract_id))
            .filter(contract_approval_log::Column::Deleted.eq(0))
            .order_by_desc(contract_approval_log::Column::CreateTime)
            .all(db)
            .await
    }

    pub async fn insert_approval_log(
        db: &DbConn,
        contract_id: i64,
        action: i32,
        operator_id: i64,
        operator_name: Option<String>,
        reason: Option<String>,
        previous_status: Option<i32>,
        new_status: Option<i32>,
        current_stage: Option<i32>,
        next_stage: Option<i32>,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = contract_approval_log::ActiveModel {
            contract_id: Set(contract_id),
            action: Set(action),
            operator_id: Set(operator_id),
            operator_name: Set(operator_name),
            reason: Set(reason),
            previous_status: Set(previous_status),
            new_status: Set(new_status),
            current_stage: Set(current_stage),
            next_stage: Set(next_stage),
            create_time: Set(Option::from(now)),
            ..Default::default()
        };

        ContractApprovalLog::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }
}

/// 合同审批日志VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ContractApprovalLogVO {
    pub id: Option<i64>,
    pub contract_id: Option<i64>,
    pub action: Option<i32>,
    pub operator_id: Option<i64>,
    pub operator_name: Option<String>,
    pub reason: Option<String>,
    pub previous_status: Option<i32>,
    pub new_status: Option<i32>,
    pub current_stage: Option<i32>,
    pub next_stage: Option<i32>,
    pub create_time: Option<String>,
}

impl From<contract_approval_log::Model> for ContractApprovalLogVO {
    fn from(item: contract_approval_log::Model) -> Self {
        ContractApprovalLogVO {
            id: Some(item.id),
            contract_id: Some(item.contract_id),
            action: Some(item.action),
            operator_id: Some(item.operator_id),
            operator_name: item.operator_name,
            reason: item.reason,
            previous_status: item.previous_status,
            new_status: item.new_status,
            current_stage: item.current_stage,
            next_stage: item.next_stage,
            create_time: item.create_time.map(|t| t.to_string()),
        }
    }
}

/// 合同审批请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContractApprovalRequest {
    pub contract_id: Option<i64>,
    pub reason: Option<String>,
}

/// 合同审批详情VO（聚合合同+审批实例数据）
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ContractApprovalDetailVO {
    pub contract_id: Option<i64>,
    pub contract_no: Option<String>,
    pub title: Option<String>,
    pub customer_name: Option<String>,
    pub contract_type: Option<String>,
    pub amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub approval_status: Option<i32>,
    pub instance: Option<ApprovalInstanceVO>,
}

/// 合同审批流程配置
pub struct ContractApprovalConfig {
    pub first_level_limit: Decimal,
    pub second_level_limit: Decimal,
    pub first_level_approver_role: String,
    pub second_level_approver_role: String,
}

impl Default for ContractApprovalConfig {
    fn default() -> Self {
        ContractApprovalConfig {
            first_level_limit: Decimal::from(5000),
            second_level_limit: Decimal::from(100000),
            first_level_approver_role: "manager".to_string(),
            second_level_approver_role: "boss".to_string(),
        }
    }
}