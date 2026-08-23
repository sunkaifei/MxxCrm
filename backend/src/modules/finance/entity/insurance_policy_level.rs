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

/// 社保政策档次明细表
/// 一个政策（城市+年度）下可配置多个缴费档次（最低档/最高档/自定义档），
/// 每档独立设置基数、各险种比例与重大保险固定金额
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_insurance_policy_level")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    /// 关联政策表头 ID
    pub policy_id: i64,
    /// 档次类型：0=最低档 1=最高档 2=自定义档
            pub level_type: Option<i16>,
    /// 档次名称，如 最低基数 / 最高基数
    pub level_name: Option<String>,
    /// 该档次的缴费基数
    pub base_amount: Decimal,
    /// 缴费基数下限
    pub base_lower: Option<Decimal>,
    /// 缴费基数上限
    pub base_upper: Option<Decimal>,
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
    /// 工伤保险单位比例（行业浮动比例）
    pub workinjury_company_rate: Decimal,
    /// 工伤保险个人比例（通常为 0）
    pub workinjury_personal_rate: Decimal,
    /// 生育保险单位比例
    pub maternity_company_rate: Decimal,
    /// 生育保险个人比例（通常为 0）
    pub maternity_personal_rate: Decimal,
    /// 住房公积金单位比例
    pub housing_fund_company_rate: Decimal,
    /// 住房公积金个人比例
    pub housing_fund_personal_rate: Decimal,
    /// 重大疾病保险单位固定金额
    pub critical_illness_company_amount: Decimal,
    /// 重大疾病保险个人固定金额
    pub critical_illness_personal_amount: Decimal,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
