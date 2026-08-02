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
#[sea_orm(table_name = "mxx_finance_social_insurance_policy")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub city_code: String,
    pub city_name: String,
    pub year: i32,
    /// 缴费基数下限
    pub base_lower: Decimal,
    /// 缴费基数上限
    pub base_upper: Decimal,
    /// 养老保险单位比例
    pub pension_company_rate: Decimal,
    /// 养老保险个人比例
    pub pension_personal_rate: Decimal,
    /// 医疗保险单位比例
    pub medical_company_rate: Decimal,
    /// 医疗保险个人比例
    pub medical_personal_rate: Decimal,
    /// 失业保险单位比例
    pub unemployment_company_rate: Decimal,
    /// 失业保险个人比例
    pub unemployment_personal_rate: Decimal,
    /// 工伤保险单位比例
    pub workinjury_company_rate: Decimal,
    /// 生育保险单位比例
    pub maternity_company_rate: Decimal,
    /// 住房公积金单位比例
    pub housing_fund_company_rate: Decimal,
    /// 住房公积金个人比例
    pub housing_fund_personal_rate: Decimal,
    /// 生效月份
    pub effective_month: Option<i32>,
    /// 是否启用
    pub enabled: Option<i32>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
