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
#[sea_orm(table_name = "mxx_finance_employee_insurance_config")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub employee_id: i64,
    pub city_code: String,
    /// 社保缴费基数
    pub base_amount: Decimal,
    /// 住房公积金基数
    pub housing_fund_base: Option<Decimal>,
    /// 住房公积金单位比例
    pub housing_fund_company_rate: Option<Decimal>,
    /// 住房公积金个人比例
    pub housing_fund_personal_rate: Option<Decimal>,
    /// 参加养老保险
    pub participate_pension: Option<i32>,
    /// 参加医疗保险
    pub participate_medical: Option<i32>,
    /// 参加失业保险
    pub participate_unemployment: Option<i32>,
    /// 参加工伤保险
    pub participate_workinjury: Option<i32>,
    /// 参加生育保险
    pub participate_maternity: Option<i32>,
    /// 参加住房公积金
    pub participate_housing_fund: Option<i32>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub enabled: Option<i32>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
