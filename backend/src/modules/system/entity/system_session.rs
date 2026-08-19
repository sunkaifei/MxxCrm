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

/// 在线会话实体（登录认证整改 v1.0）
///
/// 对应表 `mxx_system_session`：
/// - `token`：accessToken（JWT）原文，用于会话校验精确匹配
/// - `refresh_token`：refreshToken 的 SHA-256 hex（128 字符），明文不落库
/// - `refresh_expire_time`：refreshToken 过期时间（滑动续期基准）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_session")]
pub struct Model {
    /// 会话主键
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 用户ID
    pub user_id: i64,
    /// accessToken（JWT）原文
    pub token: String,
    /// refreshToken 的 SHA-256 hex（明文不落库）
    pub refresh_token: Option<String>,
    /// 登录IP
    pub login_ip: Option<String>,
    /// 登录时间
    pub login_time: Option<DateTime>,
    /// accessToken 过期时间（会话校验降级用）
    pub expire_time: Option<DateTime>,
    /// refreshToken 过期时间（滑动续期基准）
    pub refresh_expire_time: Option<DateTime>,
    /// 状态（1有效 0失效）
    pub status: Option<i16>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
