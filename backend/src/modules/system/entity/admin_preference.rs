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
#[sea_orm(table_name = "mxx_system_admin_preference")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub admin_id: i64,
    /// 偏好键：quick_nav / dashboard_layout
    pub pref_key: Option<String>,
    /// 偏好值（JSONB）
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub pref_value: Option<serde_json::Value>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
