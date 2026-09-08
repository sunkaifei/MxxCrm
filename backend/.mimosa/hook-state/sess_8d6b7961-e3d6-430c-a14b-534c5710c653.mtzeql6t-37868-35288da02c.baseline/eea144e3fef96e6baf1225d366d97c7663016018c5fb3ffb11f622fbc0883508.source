//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 离职交接项模板表（mxx_hr_resign_item_template）
#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_hr_resign_item_template")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 项标识：work/customer/account/asset/自定义
    pub item_key: Option<String>,
    /// 项名称
    pub item_name: Option<String>,
    /// 默认确认人规则：1=交接人 2=系统管理员 3=指定角色
    pub assignee_rule: Option<i32>,
    /// 规则为3时指定角色ID
    pub assignee_role_id: Option<i64>,
    /// 1启用 0停用（停用不生成对应交接项）
    pub enabled: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
