//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_commission_rule_member")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 规则ID
    pub rule_id: i64,

    /// 成员类型: 1=业务员 2=直属经理 3=部门总监 4=其他
    pub member_type: i32,

    /// 角色名称
    pub role_name: Option<String>,

    /// 成员名称
    pub member_name: String,

    /// 分配类型: 1=固定比例
    pub distribution_type: i32,

    /// 固定比例(如0.6000=60%)
    pub fixed_rate: Decimal,

    /// 默认分成比例 0.6000=60%
    pub default_ratio: Decimal,

    /// 是否必选: 0=否 1=是
    pub required: i32,

    /// 排序
    pub sort: i32,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::commission_rule::Entity",
        from = "Column::RuleId",
        to = "super::commission_rule::Column::Id"
    )]
    CommissionRule,
}

impl ActiveModelBehavior for ActiveModel {}
