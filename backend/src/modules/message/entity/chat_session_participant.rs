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
#[sea_orm(table_name = "mxx_chat_session_participant")]
pub struct Model {
    /// 记录ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 会话ID
    pub session_id: i64,

    /// 用户ID
    pub user_id: i64,

    /// 未读消息数
    pub unread_count: Option<i32>,

    /// 删除标识(0未删除,1已删除)
    pub deleted: Option<i32>,

    /// 加入时间
    pub joined_at: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
