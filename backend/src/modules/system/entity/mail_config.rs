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
#[sea_orm(table_name = "mxx_system_mail_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 配置名称
    pub name: Option<String>,
    /// SMTP 主机
    pub host: Option<String>,
    /// SMTP 端口
    pub port: Option<i32>,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 发件人邮箱
    pub from_email: Option<String>,
    /// 发件人名称
    pub from_name: Option<String>,
    /// 是否 SSL（1 是 0 否）
    pub is_ssl: Option<i32>,
    /// 是否默认（1 是 0 否）
    pub is_default: Option<i32>,
    /// 状态（1 启用 0 停用）
    pub status: Option<i32>,
    /// 创建人ID
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新人ID
    pub update_by: Option<i64>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除标志（0未删除 1已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
