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
#[sea_orm(table_name = "mxx_finance_commission_detail")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 工资记录ID
    pub salary_record_id: i64,

    /// 合同ID
    pub contract_id: Option<i64>,

    /// 合同名称
    pub contract_name: Option<String>,

    /// 合同金额
    pub contract_amount: Option<Decimal>,

    /// 回款金额
    pub payment_amount: Option<Decimal>,

    /// 提成基数
    pub commission_base: Option<Decimal>,

    /// 提成比例
    pub commission_rate: Option<Decimal>,

    /// 提成金额
    pub commission_amount: Option<Decimal>,

    /// 规则名称
    pub rule_name: Option<String>,

    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
