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

/// 入职引导配置（单行表）：区块开关 + 快捷入口预设 + 乐观锁版本号
#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_onboarding_config")]
pub struct Model {
    /// ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 公告区块开关（1=开 0=关）
    pub announce_enabled: i16,
    /// 待办区块开关（1=开 0=关）
    pub todo_enabled: i16,
    /// 快捷入口区块开关（1=开 0=关）
    pub quick_enabled: i16,
    /// 快捷入口预设（JSONB 数组 [{code,label,path}]）
    pub quick_preset: Option<serde_json::Value>,
    /// 乐观锁版本号（保存时 +1，不匹配返回 409）
    pub version: i32,
    /// 更新者
    pub update_by: Option<String>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
