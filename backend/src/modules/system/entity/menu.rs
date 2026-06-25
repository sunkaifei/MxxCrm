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
#[sea_orm(table_name = "mxx_system_menu")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub parent_id: i64,
    pub tree_path: Option<String>,
    pub name: Option<String>,
    #[sea_orm(column_name = "type")]
    pub r#type: Option<String>,
    pub route_name: Option<String>,
    pub path: Option<String>,
    pub component: Option<String>,
    pub perm: Option<String>,
    pub status: i32,
    pub affix_tab: i32,
    pub hide_children_in_menu: i32,
    pub hide_in_breadcrumb: i32,
    pub hide_in_menu: i32,
    pub hide_in_tab: i32,
    pub keep_alive: i32,
    pub sort: Option<i32>,
    pub icon: Option<String>,
    pub redirect: Option<String>,
    pub params: Option<serde_json::Value>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    ///删除标志（0未删除 1已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::role_menu_merge::Entity",
        from = "Column::Id",
        to = "super::role_menu_merge::Column::MenuId"
    )]
    RoleMenuMerge,
}

impl Related<super::role_menu_merge::Entity> for Entity {
    fn to() -> RelationDef {
        super::role_menu_merge::Relation::Menu.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::role_menu_merge::Relation::Menu.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}