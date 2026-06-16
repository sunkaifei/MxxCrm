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
#[sea_orm(table_name = "mxx_system_dept")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 父部门ID
    pub parent_id: Option<i64>,
    pub ancestors: Option<String>,
    /// 部门名称
    pub dept_name: Option<String>,
    /// 部门编码
    pub code: Option<String>,
    /// 显示顺序
    pub sort: Option<i32>,
    ///负责人
    pub leader: Option<String>,
    /// 联系电话
    pub phone: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 显示状态（0正常 1停用）
    pub status: Option<i32>,
    /// 删除标志（0未删除 1已删除）
    pub deleted: Option<i32>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::admin_dept_merge::Entity",
        from = "Column::Id",
        to = "super::admin_dept_merge::Column::DeptId"
    )]
    AdminDeptMerge,
    #[sea_orm(
        belongs_to = "super::dept_menu_merge::Entity",
        from = "Column::Id",
        to = "super::dept_menu_merge::Column::DeptId"
    )]
    DeptMenuMerge,
}

impl Related<super::admin::Entity> for Entity {
    fn to() -> RelationDef {
        super::admin_dept_merge::Relation::Admin.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::admin_dept_merge::Relation::Dept.def().rev())
    }
}
impl Related<super::menu::Entity> for Entity {
    fn to() -> RelationDef {
        super::dept_menu_merge::Relation::Menu.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::dept_menu_merge::Relation::Dept.def().rev())
    }
}


impl ActiveModelBehavior for ActiveModel {}