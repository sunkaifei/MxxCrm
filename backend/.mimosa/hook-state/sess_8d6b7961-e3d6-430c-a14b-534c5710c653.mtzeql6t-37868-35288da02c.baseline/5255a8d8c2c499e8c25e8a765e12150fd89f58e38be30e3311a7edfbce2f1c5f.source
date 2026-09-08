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
#[sea_orm(table_name = "mxx_system_admin_template_merge")]
pub struct Model {
    /// ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 用户id
    pub admin_id: Option<i64>,
    /// 岗位id
    pub template_id: Option<i64>,
    /// 创建日期
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::super::super::system::entity::admin::Entity",
        from = "Column::AdminId",
        to = "super::super::super::system::entity::admin::Column::Id")]
    Admin,
    #[sea_orm(
        belongs_to = "super::template::Entity",
        from = "Column::TemplateId",
        to = "super::template::Column::Id"
    )]
    Template,
}

impl Related<super::super::super::system::entity::admin::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Admin.def()
    }
}

impl Related<super::template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Template.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    
}
