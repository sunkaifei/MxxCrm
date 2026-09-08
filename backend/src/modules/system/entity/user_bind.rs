//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 第三方登录账号绑定实体（表见 d60_sso_user_bind.sql）

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_user_bind")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 系统账号 ID（mxx_system_admin.id）
    pub user_id: i64,
    /// 第三方提供商（wecom / dingtalk）
    pub provider: Option<String>,
    /// 第三方唯一 ID（企业微信 userid / 钉钉 unionId）
    pub provider_uid: Option<String>,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
    /// 软删（0未删除 1已解绑）
    pub deleted: Option<i16>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
