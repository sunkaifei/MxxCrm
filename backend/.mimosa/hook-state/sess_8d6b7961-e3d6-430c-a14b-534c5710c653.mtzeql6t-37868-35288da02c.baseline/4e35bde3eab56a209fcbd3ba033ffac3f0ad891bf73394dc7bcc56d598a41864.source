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

/// 多工作台定义（方案 5.1 变更 4：default/sales/warehouse/finance/hr 预置五套）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_workspace")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 工作台编码（全局唯一，同时作为 page_key 路由）
    pub workspace_code: Option<String>,
    /// 工作台名称
    pub workspace_name: Option<String>,
    /// 图标（lucide: 前缀）
    pub icon: Option<String>,
    /// 是否默认工作台（1是 0否）
    pub is_default: Option<i16>,
    /// 状态（1启用 0停用）
    pub status: Option<i16>,
    /// 显示顺序
    pub sort: Option<i32>,
    /// 删除标志（0存在 1删除）
    pub deleted: Option<i16>,
    /// 创建者
    pub create_by: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
