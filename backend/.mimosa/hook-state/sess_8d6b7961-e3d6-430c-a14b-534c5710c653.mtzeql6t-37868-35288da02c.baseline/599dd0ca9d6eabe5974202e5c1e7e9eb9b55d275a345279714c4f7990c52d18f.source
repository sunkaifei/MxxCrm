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
#[sea_orm(table_name = "mxx_system_post")]
pub struct Model {
    /// ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 岗位编码，权限控制的时候使用
    pub post_code: Option<String>,
    /// 岗位名称
    pub post_name: Option<String>,
    /// 岗位状态
    pub status: Option<i32>,
    /// 岗位排序
    pub sort: Option<i64>,
    /// 备注
    pub remark: Option<String>,
    /// 创建人
    pub create_by: Option<String>,
    /// 创建日期
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 是否删除
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::admin_post_merge::Entity",
        from = "Column::Id",
        to = "super::admin_post_merge::Column::PostId"
    )]
    AdminPostMerge
}

impl Related<super::admin_post_merge::Entity> for Entity {
    fn to() -> RelationDef {
        super::admin_post_merge::Relation::Admin.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::admin_post_merge::Relation::Post.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}