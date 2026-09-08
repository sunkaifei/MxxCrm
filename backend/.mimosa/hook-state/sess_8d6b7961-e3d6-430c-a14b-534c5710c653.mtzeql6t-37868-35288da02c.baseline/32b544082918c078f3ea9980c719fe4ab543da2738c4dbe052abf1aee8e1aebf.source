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
#[sea_orm(table_name = "mxx_system_dept_menu_merge")]
pub struct Model {
    /// ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 岗位id
    pub dept_id: Option<i64>,
    /// 菜单id
    pub menu_id: Option<i64>,
    /// 创建日期
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::dept::Entity",
        from = "Column::DeptId",
        to = "super::dept::Column::Id"
    )]
    Dept,
    #[sea_orm(
        belongs_to = "super::menu::Entity",
        from = "Column::MenuId",
        to = "super::menu::Column::Id"
    )]
    Menu,
}

impl Related<super::dept::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Dept.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}