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

/// 部门默认角色映射：用户审核通过落库部门后，按部门追加缺失的默认角色
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_dept_default_role")]
pub struct Model {
    /// ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 部门ID（唯一约束：一个部门一个默认角色）
    pub dept_id: i64,
    /// 默认角色ID
    pub role_id: i64,
    /// 创建者
    pub create_by: Option<String>,
    /// 创建时间
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
        belongs_to = "super::role::Entity",
        from = "Column::RoleId",
        to = "super::role::Column::Id"
    )]
    Role,
}

impl Related<super::dept::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Dept.def()
    }
}

impl Related<super::role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Role.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
