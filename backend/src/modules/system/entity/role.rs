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
#[sea_orm(table_name = "mxx_system_role")]
pub struct Model {
    /// 角色ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 角色名称
    pub role_name: Option<String>,
    /// 角色权限字符串
    pub role_key: Option<String>,
    /// 显示顺序
    pub sort: Option<i32>,
    /// 数据范围（1：全部数据权限 2：自定数据权限 3：本部门数据权限 4：本部门及以下数据权限）
    pub data_scope: Option<i32>,
    /// 角色状态（0停用 1正常）
    pub status: Option<i32>,
    /// 删除标志（0代表存在 2代表删除）
    pub deleted: Option<i32>,
    /// 创建者
    pub create_by: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新者
    pub update_by: Option<String>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::admin_role_merge::Entity",
        from = "Column::Id",
        to = "super::admin_role_merge::Column::RoleId"
    )]
    AdminRoleMerge,
    #[sea_orm(
        belongs_to = "super::role_menu_merge::Entity",
        from = "Column::Id",
        to = "super::role_menu_merge::Column::RoleId"
    )]
    RoleMenuMerge,
}

impl Related<super::admin::Entity> for Entity {
    fn to() -> RelationDef {
        super::admin_role_merge::Relation::Admin.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::admin_role_merge::Relation::Role.def().rev())
    }
}

impl Related<super::menu::Entity> for Entity {
    fn to() -> RelationDef {
        super::role_menu_merge::Relation::Menu.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::role_menu_merge::Relation::Role.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
