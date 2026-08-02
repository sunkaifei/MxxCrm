//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 资金池流水实体
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_commission_pool_log")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 资金池ID
    pub pool_id: i64,

    /// 类型: 1=存入(提成归集) 2=支出(团建活动)
    pub log_type: i16,

    /// 金额
    pub amount: Decimal,

    /// 来源规则ID(存入时)
    pub source_rule_id: Option<i64>,

    /// 来源员工ID(存入时)
    pub source_employee_id: Option<i64>,

    /// 来源年(存入时)
    pub source_year: Option<i32>,

    /// 来源月(存入时)
    pub source_month: Option<i32>,

    /// 支出事由(支出时)
    pub usage_description: Option<String>,

    /// 支出日期(支出时)
    pub usage_date: Option<chrono::NaiveDate>,

    /// 操作人ID
    pub operator_id: Option<i64>,

    /// 创建时间
    pub create_time: DateTime,

    /// 删除标识: 0=未删除 1=已删除
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
