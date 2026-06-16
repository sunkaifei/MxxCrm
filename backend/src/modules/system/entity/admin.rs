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
#[sea_orm(table_name = "mxx_system_admin")]
pub struct Model {
    ///用户ID
    #[sea_orm(primary_key)]
    pub id: i64,
    ///用户账号
    pub user_name: Option<String>,
    ///用户昵称
    pub nick_name: Option<String>,
    ///用户类型：0普通用户，1超级管理员
    pub user_type: Option<i32>,
    ///用户邮箱
    pub email: Option<String>,
    ///手机号码
    pub mobile: Option<String>,
    ///用户性别（0男 1女 2未知）
    pub gender: Option<i32>,
    ///头像地址
    pub avatar: Option<String>,
    ///密码
    pub password: Option<String>,
    ///帐号状态（0正常 1停用）
    pub status: Option<i32>,
    ///删除标志（0未删除 1已删除）
    pub deleted: Option<i32>,
    ///最后登陆IP
    pub login_ip: Option<String>,
    ///最后登陆时间
    pub login_date: Option<DateTime>,
    ///创建者
    pub create_by: Option<String>,
    ///创建时间
    pub create_time: Option<DateTime>,
    ///更新者
    pub update_by: Option<String>,
    ///更新时间
    pub update_time: Option<DateTime>,
    ///备注
    pub remark: Option<String>,
    ///用户排序
    pub sort: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::admin_dept_merge::Entity",
        from = "Column::Id",
        to = "super::admin_dept_merge::Column::AdminId"
    )]
    AdminDeptMerge,
    #[sea_orm(
        belongs_to = "super::admin_role_merge::Entity",
        from = "Column::Id",
        to = "super::admin_role_merge::Column::AdminId"
    )]
    AdminRoleMerge,
    #[sea_orm(
        belongs_to = "super::admin_post_merge::Entity",
        from = "Column::Id",
        to = "super::admin_post_merge::Column::AdminId"
    )]
    AdminPostMerge,
    #[sea_orm(
        belongs_to = "super::super::super::website::entity::admin_template_merge::Entity",
        from = "Column::Id",
        to = "super::super::super::website::entity::admin_template_merge::Column::AdminId"
    )]
    AdminTemplateMerge,
    #[sea_orm(
        belongs_to = "super::admin_notice_merge::Entity",
        from = "Column::Id",
        to = "super::admin_notice_merge::Column::UserId"
    )]
    AdminNoticeMerge,
}

impl Related<super::dept::Entity> for Entity {
    fn to() -> RelationDef {
        super::admin_dept_merge::Relation::Dept.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::admin_dept_merge::Relation::Admin.def().rev())
    }
}

impl Related<super::role::Entity> for Entity {
    fn to() -> RelationDef {
        super::admin_role_merge::Relation::Role.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::admin_role_merge::Relation::Admin.def().rev())
    }
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        super::admin_post_merge::Relation::Post.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::admin_post_merge::Relation::Admin.def().rev())
    }
}

impl Related<super::super::super::website::entity::template::Entity> for Entity {
    fn to() -> RelationDef {
        super::super::super::website::entity::admin_template_merge::Relation::Template.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::super::super::website::entity::admin_template_merge::Relation::Admin.def().rev())
    }
}

impl Related<super::notice::Entity> for Entity {
    fn to() -> RelationDef {
        super::admin_notice_merge::Relation::Notice.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::admin_notice_merge::Relation::Admin.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}