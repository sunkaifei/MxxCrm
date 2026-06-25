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

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mxx_chat_message")]
pub struct Model {
    /// 消息ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 会话ID
    pub session_id: i64,

    /// 发送者用户ID
    pub sender_id: i64,

    /// 发送者昵称
    pub sender_nickname: String,

    /// 发送者头像
    pub sender_avatar: Option<String>,

    /// 消息内容
    pub content: String,

    /// 消息类型: 1=系统消息, 2=用户消息
    pub message_type: Option<i32>,

    /// 是否撤回: 0=否, 1=是
    pub is_recalled: Option<i32>,

    /// 发送时间
    pub send_time: Option<DateTime>,

    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
