//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 费用申请模型层
//!

use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::finance::entity::{
    expense, expense::Entity as FinanceExpense,
    expense_item, expense_item::Entity as FinanceExpenseItem,
    expense_type, expense_type::Entity as FinanceExpenseType,
};
use crate::utils::string_utils::serialize_option_u64_to_string;

// ==================== 请求 DTO ====================

/// 费用申请新建/编辑请求（id 为空=新建，id 非空=编辑）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseSaveRequest {
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub id: Option<i64>,
    pub title: Option<String>,
    pub expense_type: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub applicant_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub dept_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub opportunity_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_id: Option<i64>,
    pub currency: Option<String>,
    /// 申请日期（YYYY-MM-DD）
    pub apply_date: Option<String>,
    pub remark: Option<String>,
    /// 附件URL列表（JSON数组）
    pub attachment: Option<serde_json::Value>,
    pub items: Option<Vec<ExpenseItemSaveDTO>>,
}

/// 费用申请列表查询
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseListQuery {
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    /// 业务状态：1=草稿,2=待审批,3=审批中,4=已通过,5=已驳回,6=已打款
    pub status: Option<i32>,
    /// 审批状态：0=草稿,1=待审批,2=审批中,3=已通过,4=已驳回
    pub approval_status: Option<i32>,
    pub expense_type: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub customer_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub opportunity_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub order_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub applicant_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    /// all=全部, my=我的, subordinate=下属
    pub list_type: Option<String>,
}

/// 审批请求（通过/驳回）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseApprovalReq {
    pub expense_id: i64,
    pub reason: Option<String>,
}

/// 打款请求
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpensePaymentReq {
    pub expense_id: i64,
    pub payment_account: Option<String>,
    pub transaction_no: Option<String>,
    pub remark: Option<String>,
}

/// 费用类型新建/编辑请求
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseTypeSaveRequest {
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub id: Option<i64>,
    pub type_name: Option<String>,
    pub type_code: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_u64")]
    pub parent_id: Option<i64>,
    pub sort: Option<i32>,
    pub status: Option<i32>,
}

// ==================== 内部 DTO ====================

#[derive(Debug, Clone)]
pub struct ExpenseSaveDTO {
    pub expense_no: Option<String>,
    pub title: Option<String>,
    pub expense_type: Option<i32>,
    pub applicant_id: Option<i64>,
    pub dept_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub order_id: Option<i64>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub apply_date: Option<chrono::NaiveDate>,
    pub status: Option<i32>,
    pub approval_status: Option<i32>,
    pub remark: Option<String>,
    pub attachment: Option<serde_json::Value>,
    pub create_by: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseItemSaveDTO {
    /// 明细日期（YYYY-MM-DD）
    pub item_date: Option<String>,
    pub item_amount: Option<Decimal>,
    pub item_category: Option<String>,
    pub item_description: Option<String>,
    pub item_attachment: Option<serde_json::Value>,
}

// ==================== 响应 VO ====================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub expense_no: Option<String>,
    pub title: Option<String>,
    pub expense_type: Option<i32>,
    pub expense_type_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub applicant_id: Option<i64>,
    pub applicant_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub dept_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub opportunity_id: Option<i64>,
    pub opportunity_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    pub order_no: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub apply_date: Option<chrono::NaiveDate>,
    pub status: Option<i32>,
    pub approval_status: Option<i32>,
    pub create_time: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub expense_no: Option<String>,
    pub title: Option<String>,
    pub expense_type: Option<i32>,
    pub expense_type_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub applicant_id: Option<i64>,
    pub applicant_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub dept_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub opportunity_id: Option<i64>,
    pub opportunity_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    pub order_no: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub apply_date: Option<chrono::NaiveDate>,
    pub status: Option<i32>,
    pub approval_status: Option<i32>,
    pub instance_id: Option<i64>,
    pub remark: Option<String>,
    pub attachment: Option<serde_json::Value>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub items: Vec<ExpenseItemVO>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseItemVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub expense_id: Option<i64>,
    pub item_date: Option<chrono::NaiveDate>,
    pub item_amount: Option<Decimal>,
    pub item_category: Option<String>,
    pub item_description: Option<String>,
    pub item_attachment: Option<serde_json::Value>,
    pub create_time: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseTypeVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub type_name: Option<String>,
    pub type_code: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    pub sort: Option<i32>,
    pub status: Option<i32>,
    pub is_system: Option<i32>,
    pub create_time: Option<DateTime>,
}

// ==================== From 转换 ====================

impl From<&expense::Model> for ExpenseListVO {
    fn from(model: &expense::Model) -> Self {
        Self {
            id: model.id.into(),
            expense_no: model.expense_no.clone(),
            title: model.title.clone(),
            expense_type: model.expense_type,
            expense_type_name: None,
            applicant_id: model.applicant_id,
            applicant_name: None,
            dept_id: model.dept_id,
            customer_id: model.customer_id,
            customer_name: None,
            opportunity_id: model.opportunity_id,
            opportunity_name: None,
            order_id: model.order_id,
            order_no: None,
            amount: model.amount,
            currency: model.currency.clone(),
            apply_date: model.apply_date,
            status: model.status,
            approval_status: model.approval_status,
            create_time: model.create_time,
        }
    }
}

impl From<&expense_item::Model> for ExpenseItemVO {
    fn from(model: &expense_item::Model) -> Self {
        Self {
            id: model.id.into(),
            expense_id: model.expense_id,
            item_date: model.item_date,
            item_amount: model.item_amount,
            item_category: model.item_category.clone(),
            item_description: model.item_description.clone(),
            item_attachment: model.item_attachment.clone(),
            create_time: model.create_time,
        }
    }
}

impl From<&expense_type::Model> for ExpenseTypeVO {
    fn from(model: &expense_type::Model) -> Self {
        Self {
            id: model.id.into(),
            type_name: model.type_name.clone(),
            type_code: model.type_code.clone(),
            parent_id: model.parent_id,
            sort: model.sort,
            status: model.status,
            is_system: model.is_system,
            create_time: model.create_time,
        }
    }
}

// ==================== 数据库操作方法 ====================

fn deserialize_option_string_to_u64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    match Option::<Value>::deserialize(deserializer)? {
        Some(Value::String(s)) => {
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<i64>().map(Some).map_err(D::Error::custom)
            }
        }
        Some(Value::Number(n)) => Ok(n.as_i64()),
        Some(_) => Err(D::Error::custom("expected string or number")),
        None => Ok(None),
    }
}

pub struct ExpenseModel;

impl ExpenseModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &ExpenseSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = expense::ActiveModel {
            expense_no: Set(req.expense_no.clone()),
            title: Set(req.title.clone()),
            expense_type: Set(req.expense_type),
            applicant_id: Set(req.applicant_id),
            dept_id: Set(req.dept_id),
            customer_id: Set(req.customer_id),
            opportunity_id: Set(req.opportunity_id),
            order_id: Set(req.order_id),
            amount: Set(req.amount.or(Some(Decimal::from(0)))),
            currency: Set(req.currency.clone().or(Some("CNY".to_string()))),
            apply_date: Set(req.apply_date),
            status: Set(req.status.or(Some(1))),
            approval_status: Set(req.approval_status.or(Some(0))),
            instance_id: Set(None),
            remark: Set(req.remark.clone()),
            attachment: Set(req.attachment.clone()),
            create_by: Set(req.create_by),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        FinanceExpense::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: i64, req: &ExpenseSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = expense::ActiveModel {
            update_time: Set(Some(now)),
            ..Default::default()
        };

        if let Some(v) = req.title.clone() { payload.title = Set(Some(v)); }
        if let Some(v) = req.expense_type { payload.expense_type = Set(Some(v)); }
        if let Some(v) = req.applicant_id { payload.applicant_id = Set(Some(v)); }
        if let Some(v) = req.dept_id { payload.dept_id = Set(Some(v)); }
        if let Some(v) = req.customer_id { payload.customer_id = Set(Some(v)); }
        if let Some(v) = req.opportunity_id { payload.opportunity_id = Set(Some(v)); }
        if let Some(v) = req.order_id { payload.order_id = Set(Some(v)); }
        if let Some(v) = req.amount { payload.amount = Set(Some(v)); }
        if let Some(v) = req.currency.clone() { payload.currency = Set(Some(v)); }
        if let Some(v) = req.apply_date { payload.apply_date = Set(Some(v)); }
        if let Some(v) = req.remark.clone() { payload.remark = Set(Some(v)); }
        if let Some(v) = req.attachment.clone() { payload.attachment = Set(Some(v)); }

        let result = FinanceExpense::update_many()
            .set(payload)
            .filter(expense::Column::Id.eq(id))
            .filter(expense::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i32) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result = FinanceExpense::update_many()
            .set(expense::ActiveModel {
                status: Set(Some(status)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(expense::Column::Id.eq(id))
            .filter(expense::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_approval<C: ConnectionTrait>(db: &C, id: i64, approval_status: i32, instance_id: Option<i64>) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = expense::ActiveModel {
            approval_status: Set(Some(approval_status)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        if let Some(iid) = instance_id {
            payload.instance_id = Set(Some(iid));
        }
        let result = FinanceExpense::update_many()
            .set(payload)
            .filter(expense::Column::Id.eq(id))
            .filter(expense::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn batch_delete_by_ids<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        FinanceExpense::update_many()
            .set(expense::ActiveModel {
                deleted: Set(Some(1)),
                update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(expense::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<expense::Model>, DbErr> {
        FinanceExpense::find_by_id(id)
            .filter(expense::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn get_max_expense_no_today<C: ConnectionTrait>(db: &C, date_prefix: &str) -> Result<Option<i64>, DbErr> {
        use sea_orm::QuerySelect;
        use sea_orm::prelude::Expr;

        let pattern = format!("{}%", date_prefix);
        // EXP{YYYYMMDD} 长度为 11，序号从第 12 位开始
        let result = FinanceExpense::find()
            .filter(expense::Column::ExpenseNo.like(&pattern))
            .select_only()
            .column_as(Expr::expr(Expr::cust("MAX(CAST(SUBSTRING(expense_no, 12) AS INTEGER))")), "max_seq")
            .into_tuple::<Option<i64>>()
            .one(db)
            .await?;

        Ok(result.flatten())
    }

    pub async fn select_in_page<C: ConnectionTrait>(
        db: &C,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        status: Option<i32>,
        approval_status: Option<i32>,
        expense_type: Option<i32>,
        customer_id: Option<i64>,
        opportunity_id: Option<i64>,
        order_id: Option<i64>,
        applicant_id: Option<i64>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<(Vec<expense::Model>, i64), DbErr> {
        let mut query = FinanceExpense::find()
            .filter(expense::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            if !k.trim().is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(expense::Column::ExpenseNo.contains(k.trim()))
                        .add(expense::Column::Title.contains(k.trim())),
                );
            }
        }
        if let Some(s) = status {
            query = query.filter(expense::Column::Status.eq(s));
        }
        if let Some(s) = approval_status {
            query = query.filter(expense::Column::ApprovalStatus.eq(s));
        }
        if let Some(t) = expense_type {
            query = query.filter(expense::Column::ExpenseType.eq(t));
        }
        if let Some(c) = customer_id {
            query = query.filter(expense::Column::CustomerId.eq(c));
        }
        if let Some(o) = opportunity_id {
            query = query.filter(expense::Column::OpportunityId.eq(o));
        }
        if let Some(o) = order_id {
            query = query.filter(expense::Column::OrderId.eq(o));
        }
        if let Some(a) = applicant_id {
            query = query.filter(expense::Column::ApplicantId.eq(a));
        }
        if let Some(sd) = start_date {
            if let Ok(d) = sd.parse::<chrono::NaiveDate>() {
                query = query.filter(expense::Column::ApplyDate.gte(d));
            }
        }
        if let Some(ed) = end_date {
            if let Ok(d) = ed.parse::<chrono::NaiveDate>() {
                query = query.filter(expense::Column::ApplyDate.lte(d));
            }
        }

        let paginator = query.order_by_desc(expense::Column::Id).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }

    pub async fn select_in_page_by_applicant_ids<C: ConnectionTrait>(
        db: &C,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        status: Option<i32>,
        approval_status: Option<i32>,
        expense_type: Option<i32>,
        customer_id: Option<i64>,
        opportunity_id: Option<i64>,
        order_id: Option<i64>,
        start_date: Option<String>,
        end_date: Option<String>,
        applicant_ids: Option<Vec<i64>>,
    ) -> Result<(Vec<expense::Model>, i64), DbErr> {
        let mut query = FinanceExpense::find()
            .filter(expense::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            if !k.trim().is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(expense::Column::ExpenseNo.contains(k.trim()))
                        .add(expense::Column::Title.contains(k.trim())),
                );
            }
        }
        if let Some(s) = status {
            query = query.filter(expense::Column::Status.eq(s));
        }
        if let Some(s) = approval_status {
            query = query.filter(expense::Column::ApprovalStatus.eq(s));
        }
        if let Some(t) = expense_type {
            query = query.filter(expense::Column::ExpenseType.eq(t));
        }
        if let Some(c) = customer_id {
            query = query.filter(expense::Column::CustomerId.eq(c));
        }
        if let Some(o) = opportunity_id {
            query = query.filter(expense::Column::OpportunityId.eq(o));
        }
        if let Some(o) = order_id {
            query = query.filter(expense::Column::OrderId.eq(o));
        }
        if let Some(sd) = start_date {
            if let Ok(d) = sd.parse::<chrono::NaiveDate>() {
                query = query.filter(expense::Column::ApplyDate.gte(d));
            }
        }
        if let Some(ed) = end_date {
            if let Ok(d) = ed.parse::<chrono::NaiveDate>() {
                query = query.filter(expense::Column::ApplyDate.lte(d));
            }
        }
        if let Some(ids) = applicant_ids {
            if ids.is_empty() {
                return Ok((Vec::new(), 0));
            }
            query = query.filter(expense::Column::ApplicantId.is_in(ids));
        }

        let paginator = query.order_by_desc(expense::Column::Id).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }
}

pub struct ExpenseItemModel;

impl ExpenseItemModel {
    pub async fn insert_batch<C: ConnectionTrait>(db: &C, expense_id: i64, items: &Vec<ExpenseItemSaveDTO>) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let models: Vec<expense_item::ActiveModel> = items.iter().map(|item| {
            let amt = item.item_amount.unwrap_or(Decimal::from(0));
            let item_date = item.item_date.as_ref().and_then(|s| s.parse::<chrono::NaiveDate>().ok());

            expense_item::ActiveModel {
                expense_id: Set(Some(expense_id)),
                item_date: Set(item_date),
                item_amount: Set(Some(amt)),
                item_category: Set(item.item_category.clone()),
                item_description: Set(item.item_description.clone()),
                item_attachment: Set(item.item_attachment.clone()),
                create_time: Set(Some(now)),
                ..Default::default()
            }
        }).collect();

        if models.is_empty() {
            return Ok(0);
        }

        let result = FinanceExpenseItem::insert_many(models)
            .exec(db)
            .await?;
        Ok(result.last_insert_id.unwrap_or_default())
    }

    pub async fn delete_by_expense_id<C: ConnectionTrait>(db: &C, expense_id: i64) -> Result<i64, DbErr> {
        let result = FinanceExpenseItem::delete_many()
            .filter(expense_item::Column::ExpenseId.eq(expense_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_expense_id<C: ConnectionTrait>(db: &C, expense_id: i64) -> Result<Vec<expense_item::Model>, DbErr> {
        FinanceExpenseItem::find()
            .filter(expense_item::Column::ExpenseId.eq(expense_id))
            .order_by_asc(expense_item::Column::Id)
            .all(db)
            .await
    }
}

pub struct ExpenseTypeModel;

impl ExpenseTypeModel {
    pub async fn find_all<C: ConnectionTrait>(db: &C) -> Result<Vec<expense_type::Model>, DbErr> {
        FinanceExpenseType::find()
            .filter(expense_type::Column::Deleted.eq(0))
            .order_by_asc(expense_type::Column::Sort)
            .order_by_asc(expense_type::Column::Id)
            .all(db)
            .await
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<expense_type::Model>, DbErr> {
        FinanceExpenseType::find_by_id(id)
            .filter(expense_type::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn find_by_code<C: ConnectionTrait>(db: &C, code: &str, exclude_id: Option<i64>) -> Result<Option<expense_type::Model>, DbErr> {
        let mut query = FinanceExpenseType::find()
            .filter(expense_type::Column::TypeCode.eq(code))
            .filter(expense_type::Column::Deleted.eq(0));
        if let Some(id) = exclude_id {
            query = query.filter(expense_type::Column::Id.ne(id));
        }
        query.one(db).await
    }

    pub async fn insert<C: ConnectionTrait>(db: &C, req: &ExpenseTypeSaveRequest) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = expense_type::ActiveModel {
            type_name: Set(req.type_name.clone()),
            type_code: Set(req.type_code.clone()),
            parent_id: Set(req.parent_id),
            sort: Set(req.sort.or(Some(0))),
            status: Set(req.status.or(Some(1))),
            is_system: Set(Some(0)),
            create_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        FinanceExpenseType::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: i64, req: &ExpenseTypeSaveRequest) -> Result<i64, DbErr> {
        let payload = expense_type::ActiveModel {
            type_name: Set(req.type_name.clone()),
            type_code: Set(req.type_code.clone()),
            parent_id: Set(req.parent_id),
            sort: Set(req.sort.or(Some(0))),
            status: Set(req.status.or(Some(1))),
            ..Default::default()
        };
        let result = FinanceExpenseType::update_many()
            .set(payload)
            .filter(expense_type::Column::Id.eq(id))
            .filter(expense_type::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 批量软删除（系统内置类型不允许删除）
    pub async fn batch_delete_by_ids<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        let result = FinanceExpenseType::update_many()
            .set(expense_type::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(expense_type::Column::Id.is_in(ids.clone()))
            .filter(expense_type::Column::Deleted.eq(0))
            .filter(expense_type::Column::IsSystem.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}
