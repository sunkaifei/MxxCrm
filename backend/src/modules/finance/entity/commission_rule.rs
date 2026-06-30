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
#[sea_orm(table_name = "mxx_finance_commission_rule")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 规则名称
    pub rule_name: Option<String>,

    /// 部门ID
    pub department_id: Option<i64>,

    /// 岗位ID
    pub post_id: Option<i64>,

    /// 触发条件: 1=回款触发 2=合同签订触发
    pub trigger_condition: Option<i32>,

    /// 生效日期
    pub effective_date: Option<chrono::NaiveDate>,

    /// 失效日期
    pub expiry_date: Option<chrono::NaiveDate>,

    /// 是否启用: 0=禁用 1=启用
    pub enabled: Option<i32>,

    /// 描述
    pub description: Option<String>,

    /// 创建人ID
    pub created_by: Option<i64>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新人ID
    pub updated_by: Option<i64>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 删除标识: 0=未删除 1=已删除
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
