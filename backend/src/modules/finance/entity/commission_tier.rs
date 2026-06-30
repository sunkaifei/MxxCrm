//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_commission_tier")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 规则ID
    pub rule_id: i64,

    /// 最低金额
    pub min_amount: Decimal,

    /// 最高金额
    pub max_amount: Option<Decimal>,

    /// 提成比例
    pub commission_rate: Decimal,

    /// 排序
    pub sort: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
