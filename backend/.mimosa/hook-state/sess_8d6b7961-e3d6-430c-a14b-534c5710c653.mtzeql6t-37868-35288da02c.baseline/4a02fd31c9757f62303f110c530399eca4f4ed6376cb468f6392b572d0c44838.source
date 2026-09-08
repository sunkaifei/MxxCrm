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
#[sea_orm(table_name = "mxx_finance_payslip")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub salary_record_id: i64,
    pub employee_id: i64,
    pub year: i32,
    pub month: i32,
    /// 应发工资
    pub total_salary: Option<Decimal>,
    /// 个人社保
    pub social_insurance_personal: Option<Decimal>,
    /// 个税金额
    pub tax_amount: Option<Decimal>,
    /// 实发工资
    pub net_salary: Option<Decimal>,
    /// 明细JSON
    pub detail_json: Option<sea_orm::prelude::Json>,
    /// 发送状态: 0=未发送 1=已发送 2=已读 3=已确认
    pub send_status: Option<i32>,
    /// 发送渠道
    pub send_channels: Option<String>,
    pub send_time: Option<DateTime>,
    pub read_time: Option<DateTime>,
    pub confirm_time: Option<DateTime>,
    /// 是否加密
    pub password_protected: Option<i32>,
    pub password_hash: Option<String>,
    /// V8-4: 撤回相关字段
    pub withdraw_time: Option<DateTime>,
    pub withdraw_reason: Option<String>,
    pub withdrawn_by: Option<i64>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
