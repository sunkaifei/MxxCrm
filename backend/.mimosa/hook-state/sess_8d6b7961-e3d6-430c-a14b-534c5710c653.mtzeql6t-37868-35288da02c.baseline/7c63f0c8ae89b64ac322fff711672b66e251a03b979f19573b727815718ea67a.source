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
#[sea_orm(table_name = "mxx_chat_session_member")]
pub struct Model {
    /// 记录ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 会话ID
    pub session_id: i64,

    /// 用户ID
    pub user_id: i64,

    /// 用户类型
    pub user_type: Option<i32>,

    /// 昵称
    pub nickname: Option<String>,

    /// 头像
    pub avatar: Option<String>,

    /// 是否群主
    pub is_owner: Option<i32>,

    /// 是否免打扰
    pub is_muted: Option<i32>,

    /// 是否置顶
    pub is_pinned: Option<i32>,

    /// 未读消息数
    pub unread_count: Option<i32>,

    /// 最后已读消息ID
    pub last_read_message_id: Option<i64>,

    /// 加入时间
    pub join_time: Option<DateTime>,

    /// 离开时间
    pub leave_time: Option<DateTime>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 删除标识(0未删除,1已删除)
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
