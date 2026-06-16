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
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mxx_chat_session")]
pub struct Model {
    /// 会话ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 会话类型: 1=私聊, 2=系统消息
    pub session_type: i32,

    /// 会话名称(见面时显示对方昵称)
    pub session_name: Option<String>,

    /// 会话头像
    pub avatar_url: Option<String>,

    /// 最后一条消息ID
    pub last_message_id: Option<i64>,

    /// 最后消息内容预览
    pub last_message_content: Option<String>,

    /// 最后消息时间
    pub last_message_time: Option<DateTime<Utc>>,

    /// 成员数量
    pub member_count: Option<i32>,

    /// 创建时间
    pub create_time: Option<DateTime<Utc>>,

    /// 更新时间
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
