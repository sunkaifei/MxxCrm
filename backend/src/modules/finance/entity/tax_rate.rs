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
#[sea_orm(table_name = "mxx_finance_tax_rate")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    /// 税率级别
    pub level: i32,
    /// 最小金额
    pub min_amount: Decimal,
    /// 最大金额
    pub max_amount: Option<Decimal>,
    /// 税率
    pub rate: Decimal,
    /// 速算扣除数
    pub quick_deduction: Decimal,
    /// 税类型
    pub tax_type: i32,
    /// 生效日期
    pub effective_date: Option<chrono::NaiveDate>,
    /// 失效日期
    pub expiry_date: Option<chrono::NaiveDate>,
    /// 是否启用
    pub enabled: Option<i32>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
