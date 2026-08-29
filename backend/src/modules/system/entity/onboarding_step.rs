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

/// 入职引导步骤：step_type=1 内置（代码种子）/ 2 自定义（管理员配置）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_onboarding_step")]
pub struct Model {
    /// ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 步骤编码（全局唯一，创建后不可修改）
    pub step_code: String,
    /// 步骤名称
    pub step_name: String,
    /// 步骤说明
    pub step_desc: Option<String>,
    /// 步骤类型（1=内置 2=自定义）
    pub step_type: i16,
    /// 跳转链接（站内路径，如 /profile）
    pub link_url: Option<String>,
    /// 显示顺序
    pub sort_order: i32,
    /// 状态（1=启用 0=停用）
    pub status: i16,
    /// 软删除（0=正常 1=已删除）
    pub deleted: i16,
    /// 创建者
    pub create_by: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
