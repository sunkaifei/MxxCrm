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

/// 工作台卡片个人布局覆盖（方案 5.1 变更 3；UNIQUE(admin_id, page_key, card_code)）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_dashboard_user_layout")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 用户ID（mxx_system_admin.id）
    pub admin_id: Option<i64>,
    /// 页面/工作台标识（与 workspace_code 对应）
    pub page_key: Option<String>,
    /// 卡片编码
    pub card_code: Option<String>,
    /// 栅格列偏移（0-11）
    pub x: Option<i32>,
    /// 栅格行偏移
    pub y: Option<i32>,
    /// 宽（列数，1-12）
    pub w: Option<i32>,
    /// 高（行数）
    pub h: Option<i32>,
    /// 是否隐藏（1隐藏 0显示）
    pub hidden: Option<i32>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
