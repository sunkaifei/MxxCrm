//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! API 个人访问令牌（PAT）实体（批2 P1，表见 d45）

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_api_token")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 所属用户
    pub user_id: i64,
    /// 令牌名称
    pub name: Option<String>,
    /// 展示用前缀（mxxpat_xxxxxxxx）
    pub token_prefix: Option<String>,
    /// SHA-256 哈希（明文不落库）
    pub token_hash: Option<String>,
    /// 每小时请求上限（固定窗口）
    pub rate_limit: Option<i32>,
    /// 过期时间（NULL=永不过期）
    pub expire_time: Option<DateTime>,
    /// 最近使用时间
    pub last_used_at: Option<DateTime>,
    /// 状态（1启用 0停用）
    pub status: Option<i32>,
    /// 删除标志
    pub deleted: Option<i32>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
