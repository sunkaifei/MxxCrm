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

/// 编号流水号计数实体
/// 表 mxx_system_code_sequence：按 模块+年份+部门 维度独立计数
/// 命名沿用设计文档（mxx_system_ 前缀），与规则表前缀不一致是设计原貌
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_code_sequence")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub module_code: Option<String>,
    pub year: Option<i32>,
    /// 部门编码，无部门时为空字符串
    pub dept_code: Option<String>,
    /// 当前已分配的最大流水号
    pub current_seq: Option<i32>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
