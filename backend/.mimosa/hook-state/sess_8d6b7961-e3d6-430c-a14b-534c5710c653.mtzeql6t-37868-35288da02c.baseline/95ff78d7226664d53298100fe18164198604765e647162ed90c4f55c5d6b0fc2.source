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
#[sea_orm(table_name = "mxx_finance_salary_tax_detail")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub salary_record_id: i64,
    pub employee_id: i64,
    pub year: i32,
    pub month: i32,
    /// 当月收入
    pub monthly_income: Option<Decimal>,
    /// 当月减除费用
    pub monthly_threshold: Option<Decimal>,
    /// 当月专项扣除
    pub monthly_special_deduction: Option<Decimal>,
    /// 当月其他扣除
    pub monthly_other_deduction: Option<Decimal>,
    /// 累计收入
    pub cumulative_income: Option<Decimal>,
    /// 累计应纳税所得额
    pub cumulative_taxable: Option<Decimal>,
    /// 适用税率
    pub applicable_rate: Option<Decimal>,
    /// 速算扣除数
    pub quick_deduction: Option<Decimal>,
    /// 累计应纳税额
    pub cumulative_tax_should: Option<Decimal>,
    /// 累计已预缴税额
    pub cumulative_tax_paid: Option<Decimal>,
    /// 当月应纳税额
    pub monthly_tax: Option<Decimal>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
