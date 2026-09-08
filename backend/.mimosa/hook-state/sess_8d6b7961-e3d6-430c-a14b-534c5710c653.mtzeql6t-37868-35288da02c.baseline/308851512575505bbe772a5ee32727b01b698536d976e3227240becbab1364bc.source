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
#[sea_orm(table_name = "mxx_notice")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 通知标题
    pub title: Option<String>,
    /// 通知内容
    pub content: Option<String>,
    /// 通知类型（关联字典编码：notice_type）
    pub r#type: Option<i32>,
    /// 通知等级（字典code：notice_level）
    pub level: Option<String>,
    /// 目标类型（1: 全体, 2: 指定）
    pub target_type: Option<i32>,
    /// 目标人ID集合（多个使用英文逗号,分割）
    pub target_user_ids: Option<String>,
    /// 发布人ID
    pub publisher_id: Option<i64>,
    /// 发布状态（0: 未发布, 1: 已发布, -1: 已撤回）
    pub publish_status: Option<i32>,
    /// 发布时间
    pub publish_time: Option<DateTime>,
    /// 撤回时间
    pub revoke_time: Option<DateTime>,
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
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::admin_notice_merge::Entity",
        from = "Column::Id",
        to = "super::admin_notice_merge::Column::NoticeId"
    )]
    AdminNoticeMerge,
}

impl Related<super::admin_notice_merge::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AdminNoticeMerge.def()
    }
    fn via() -> Option<RelationDef> {
        Some(Relation::AdminNoticeMerge.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {

}