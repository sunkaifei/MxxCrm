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
#[sea_orm(table_name = "mxx_statistics_performance_plan")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub employee_id: i64,
    pub year: i32,
    pub status: Option<i32>,
    pub apply_reason: Option<String>,
    pub version: Option<i32>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
    /// 当前审批人 ID（逐级审批流转时更新）
    pub current_approver_id: Option<i64>,
    /// 当前审批人姓名
    pub current_approver_name: Option<String>,
    /// 当前审批层级（1=一级，2=二级...）
    pub approval_level: Option<i32>,
    /// 总审批层级数
    pub total_levels: Option<i32>,
    /// 提交审批时间（用于超时计算）
    pub submit_time: Option<DateTime>,
    /// 是否已冻结（年底后禁止修改）
    pub is_frozen: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}