//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_admin_notice_merge")]
pub struct Model {
    /// id
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 公共通知id
    pub notice_id: Option<i64>,
    /// 用户id
    pub user_id: Option<i64>,
    /// 读取状态（0: 未读, 1: 已读）
    pub is_read: Option<i32>,
    /// 阅读时间
    pub read_time: Option<DateTime>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除标志（0未删除 1已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::admin::Entity",
        from = "Column::UserId",
        to = "super::admin::Column::Id")]
    Admin,
    #[sea_orm(
        belongs_to = "super::notice::Entity",
        from = "Column::NoticeId",
        to = "super::notice::Column::Id"
    )]
    Notice,
}

impl Related<super::admin::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Admin.def()
    }
}
impl Related<super::notice::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Notice.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}