//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 费用申请主表实体层
//!

use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::core::r#enum::currency_code_enum::CurrencyCode;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_expense")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub expense_no: Option<String>,
    pub title: Option<String>,
    /// 费用类型ID（关联 mxx_finance_expense_type）
    pub expense_type: Option<i32>,
    /// 申请人ID
    pub applicant_id: Option<i64>,
    pub dept_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub order_id: Option<i64>,
    pub amount: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub apply_date: Option<chrono::NaiveDate>,
    /// 状态：1=草稿,2=待审批,3=审批中,4=已通过,5=已驳回,6=已打款
    pub status: Option<i32>,
    /// 审批状态：0=草稿,1=待审批,2=审批中,3=已通过,4=已驳回
    pub approval_status: Option<i32>,
    pub instance_id: Option<i64>,
    pub remark: Option<String>,
    /// 附件列表（JSONB 数组，存储附件 URL）
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub attachment: Option<serde_json::Value>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
