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
use crate::modules::crm::entity::contract_payment_plan::{self, Entity as PaymentPlan};
use crate::utils::string_utils::serialize_option_u64_to_string;

/// 单条回款计划项（用于批量保存请求中的元素）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PaymentPlanItem {
    /// 阶段名称
    pub stage_name: Option<String>,
    /// 付款类型（1-预付款 2-进度款 3-尾款 4-质保金 等）
    pub payment_type: Option<i32>,
    /// 计划回款金额
    pub plan_amount: Option<Decimal>,
    /// 已回款金额
    pub received_amount: Option<Decimal>,
    /// 计划回款日期
    pub plan_date: Option<NaiveDate>,
    /// 实际回款日期
    pub actual_date: Option<NaiveDate>,
    /// 状态（0-未开始 1-部分回款 2-已完成 3-已逾期）
    pub status: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
}

/// 回款计划批量保存请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PaymentPlanSaveRequest {
    /// 合同ID
    pub contract_id: i64,
    /// 回款计划列表
    pub plans: Vec<PaymentPlanItem>,
}

/// 回款计划列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PaymentPlanVO {
    /// 计划ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 合同ID
    pub contract_id: Option<i64>,
    /// 阶段名称
    pub stage_name: Option<String>,
    /// 付款类型
    pub payment_type: Option<i32>,
    /// 计划回款金额
    pub plan_amount: Option<Decimal>,
    /// 已回款金额
    pub received_amount: Option<Decimal>,
    /// 计划回款日期
    pub plan_date: Option<NaiveDate>,
    /// 实际回款日期
    pub actual_date: Option<NaiveDate>,
    /// 状态
    pub status: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    /// 负责人ID
    pub owner_user_id: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

impl From<contract_payment_plan::Model> for PaymentPlanVO {
    fn from(item: contract_payment_plan::Model) -> Self {
        PaymentPlanVO {
            id: Some(item.id),
            contract_id: item.contract_id,
            stage_name: item.stage_name,
            payment_type: item.payment_type,
            plan_amount: item.plan_amount,
            received_amount: item.received_amount,
            plan_date: item.plan_date,
            actual_date: item.actual_date,
            status: item.status,
            sort: item.sort,
            remark: item.remark,
            owner_user_id: item.owner_user_id,
            create_time: item.create_time,
            update_time: item.update_time,
        }
    }
}

/// 回款计划列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPlanListQuery {
    /// 页码
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 关键词（搜索阶段名称、合同编号等）
    pub keywords: Option<String>,
    /// 状态
    pub status: Option<i32>,
    /// 合同ID
    pub contract_id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 列表类型：all=全部 my=我的 subordinate=下属
    pub list_type: Option<String>,
}

/// 回款计划列表VO（带合同编号、客户名称）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PaymentPlanListVO {
    /// 计划ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 合同ID
    pub contract_id: Option<i64>,
    /// 合同编号
    pub contract_no: Option<String>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 客户名称
    pub customer_name: Option<String>,
    /// 阶段名称
    pub stage_name: Option<String>,
    /// 付款类型
    pub payment_type: Option<i32>,
    /// 计划回款金额
    pub plan_amount: Option<Decimal>,
    /// 已回款金额
    pub received_amount: Option<Decimal>,
    /// 计划回款日期
    pub plan_date: Option<NaiveDate>,
    /// 实际回款日期
    pub actual_date: Option<NaiveDate>,
    /// 状态
    pub status: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    /// 负责人ID
    pub owner_user_id: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

/// 回款计划数据模型操作类
pub struct PaymentPlanModel;

impl PaymentPlanModel {
    /// 查询合同下的所有回款计划
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `contract_id` - 合同ID
    ///
    /// # 返回
    /// * `Result<Vec<PaymentPlanVO>, DbErr>` - 回款计划列表
    pub async fn find_by_contract<C: ConnectionTrait>(db: &C, contract_id: i64) -> Result<Vec<PaymentPlanVO>, DbErr> {
        let list = PaymentPlan::find()
            .filter(contract_payment_plan::Column::ContractId.eq(contract_id))
            .filter(contract_payment_plan::Column::Deleted.eq(0))
            .order_by_asc(contract_payment_plan::Column::Sort)
            .all(db)
            .await?;

        Ok(list.into_iter().map(|m| m.into()).collect())
    }

    /// 批量保存回款计划（先删后插）
    ///
    /// # 参数
    /// * `db` - 数据库连接（事务）
    /// * `contract_id` - 合同ID
    /// * `plans` - 回款计划列表
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 插入的记录数
    pub async fn save_batch<C: ConnectionTrait>(
        db: &C,
        contract_id: i64,
        owner_user_id: Option<i64>,
        plans: Vec<PaymentPlanItem>,
    ) -> Result<i64, DbErr> {
        // 1. 软删除该合同下所有已存在的回款计划
        let now = chrono::Local::now().naive_local().to_owned();
        PaymentPlan::update_many()
            .set(contract_payment_plan::ActiveModel {
                deleted: Set(Some(1)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(contract_payment_plan::Column::ContractId.eq(contract_id))
            .filter(contract_payment_plan::Column::Deleted.eq(0))
            .exec(db)
            .await?;

        // 2. 批量插入新的回款计划
        if plans.is_empty() {
            return Ok(0);
        }

        let now = chrono::Local::now().naive_local().to_owned();
        let mut active_models: Vec<contract_payment_plan::ActiveModel> = Vec::with_capacity(plans.len());
        for item in plans {
            let am = contract_payment_plan::ActiveModel {
                contract_id: Set(Some(contract_id)),
                // 归属人 = 合同负责人（assigned_to），保证"我的回款计划"按人过滤可见
                owner_user_id: Set(owner_user_id),
                stage_name: Set(item.stage_name),
                payment_type: Set(item.payment_type),
                plan_amount: Set(item.plan_amount),
                received_amount: Set(item.received_amount),
                plan_date: Set(item.plan_date),
                actual_date: Set(item.actual_date),
                status: Set(item.status),
                sort: Set(item.sort),
                remark: Set(item.remark),
                create_time: Set(Some(now)),
                update_time: Set(Some(now)),
                deleted: Set(Some(0)),
                ..Default::default()
            };
            active_models.push(am);
        }

        let insert_count = active_models.len() as i64;
        let _ = PaymentPlan::insert_many(active_models).exec(db).await?;
        Ok(insert_count)
    }

    /// 删除合同下所有回款计划（软删除）
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `contract_id` - 合同ID
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 删除的记录数
    pub async fn delete_by_contract<C: ConnectionTrait>(db: &C, contract_id: i64) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result = PaymentPlan::update_many()
            .set(contract_payment_plan::ActiveModel {
                deleted: Set(Some(1)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(contract_payment_plan::Column::ContractId.eq(contract_id))
            .filter(contract_payment_plan::Column::Deleted.eq(0))
            .exec(db)
            .await?;

        Ok(result.rows_affected as i64)
    }

    /// 分页查询回款计划列表（按负责人ID过滤）
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `page` - 页码
    /// * `per_page` - 每页大小
    /// * `keywords` - 关键词（搜索阶段名称）
    /// * `status` - 状态
    /// * `contract_ids` - 合同ID列表（None表示不过滤）
    /// * `owner_user_ids` - 负责人ID列表（None表示不过滤）
    ///
    /// # 返回
    /// * `Result<(Vec<contract_payment_plan::Model>, i64), DbErr>` - (列表, 总数)
    pub async fn select_in_page_by_owner_user_ids(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        status: Option<i32>,
        contract_ids: Option<Vec<i64>>,
        owner_user_ids: Option<Vec<i64>>,
    ) -> Result<(Vec<contract_payment_plan::Model>, i64), DbErr> {
        let mut query = PaymentPlan::find()
            .filter(contract_payment_plan::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(contract_payment_plan::Column::StageName.contains(k));
        }
        if let Some(s) = status {
            query = query.filter(contract_payment_plan::Column::Status.eq(s));
        }
        if let Some(cids) = contract_ids {
            if cids.is_empty() {
                return Ok((Vec::new(), 0));
            }
            query = query.filter(contract_payment_plan::Column::ContractId.is_in(cids));
        }
        if let Some(ids) = owner_user_ids {
            if ids.is_empty() {
                return Ok((Vec::new(), 0));
            }
            query = query.filter(contract_payment_plan::Column::OwnerUserId.is_in(ids));
        }

        let paginator = query.order_by_desc(contract_payment_plan::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }
}
