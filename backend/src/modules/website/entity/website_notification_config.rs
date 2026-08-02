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

/// 网站通知配置（mxx_website_notification_config）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_website_notification_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 网站ID
    pub website_id: Option<i64>,

    /// 场景编码
    pub scene_code: String,

    /// 场景名称
    pub scene_name: Option<String>,

    /// 通知渠道（逗号分隔，如 email,sms）
    #[serde(default = "default_channels")]
    pub channels: Option<String>,

    /// 收件人邮箱（逗号分隔）
    pub recipient_emails: Option<String>,

    /// 邮件主题
    pub email_subject: Option<String>,

    /// 邮件正文
    pub email_body: Option<String>,

    /// 是否启用：0停用 1启用
    #[serde(default)]
    pub enabled: Option<i32>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 软删除：0未删除 1已删除
    #[serde(default)]
    pub deleted: Option<i32>,
}

fn default_channels() -> Option<String> {
    Some("email".to_string())
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
