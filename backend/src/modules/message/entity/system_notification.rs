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
#[sea_orm(table_name = "mxx_system_notification")]
pub struct Model {
    /// 通知ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 通知标题
    #[sea_orm(string_len = 200)]
    pub title: String,

    /// 通知内容
    pub content: Option<String>,

    /// 通知类型
    pub r#type: i32,

    /// 业务类型
    pub biz_type: Option<String>,

    /// 业务ID
    pub biz_id: Option<i64>,

    /// 发送者ID
    pub sender_id: Option<i64>,

    /// 接收者ID
    pub receiver_id: i64,

    /// 是否已读
    pub is_read: Option<i32>,

    /// 已读时间
    pub read_time: Option<DateTime>,

    /// 跳转链接
    pub link_url: Option<String>,

    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
