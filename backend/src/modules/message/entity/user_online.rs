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
#[sea_orm(table_name = "mxx_user_online")]
pub struct Model {
    /// 记录ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 用户ID
    pub user_id: i64,

    /// 会话ID
    #[sea_orm(string_len = 64)]
    pub session_id: String,

    /// 设备类型
    pub device_type: Option<i32>,

    /// IP地址
    pub ip_address: Option<String>,

    /// User-Agent
    pub user_agent: Option<String>,

    /// 最后心跳时间
    pub last_heartbeat: DateTime,

    /// 状态
    pub status: Option<i32>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
