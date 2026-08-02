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
#[sea_orm(table_name = "mxx_finance_employee_tax_config")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub employee_id: i64,
    pub year: i32,
    /// 起征点
    pub tax_threshold: Decimal,
    /// 子女教育
    pub children_education: Decimal,
    /// 继续教育
    pub continuing_education: Decimal,
    /// 住房贷款利息
    pub housing_loan: Decimal,
    /// 住房租金
    pub housing_rent: Decimal,
    /// 赡养老人
    pub supporting_elderly: Decimal,
    /// 婴幼儿照护
    pub infant_care: Decimal,
    /// 大病医疗
    pub serious_illness: Decimal,
    /// 其他扣除
    pub other_deduction: Decimal,
    /// 外籍人员补贴
    pub foreigner_allowance: Decimal,
    /// 累计收入
    pub cumulative_income: Decimal,
    /// 累计减除费用
    pub cumulative_threshold_deduction: Decimal,
    /// 累计专项扣除
    pub cumulative_special_deduction: Decimal,
    /// 累计其他扣除
    pub cumulative_other_deduction: Decimal,
    /// 累计已预缴税额
    pub cumulative_tax_paid: Decimal,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
