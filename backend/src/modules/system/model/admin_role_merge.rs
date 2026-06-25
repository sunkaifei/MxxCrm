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
use crate::modules::system::entity::{admin_role_merge, admin_role_merge::Entity as AdminRoleMerge};
use sea_orm::prelude::DateTime;
use sea_orm::*;


#[derive(PartialEq, Default, Clone, Debug, Serialize, Deserialize)]
pub struct AdminRolesMergeSaveDTO {
    ///角色ID
    pub id: Option<i64>,
    ///角色名称
    pub admin_id: Option<i64>,
    ///角色权限字符串
    pub role_id: Option<i64>,
    ///创建时间
    pub create_time: Option<DateTime>,
}

pub struct AdminRoleMergeModel;

impl AdminRoleMergeModel {
    pub async fn insert_batch<C: ConnectionTrait>(db: &C,list: &Vec<AdminRolesMergeSaveDTO>) -> Result<i64, DbErr> {
        let result: Vec<admin_role_merge::ActiveModel> = list.iter().map(|item| admin_role_merge::ActiveModel {
            admin_id:       Set(item.admin_id),
            role_id:        Set(item.role_id),
            create_time:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        }).collect();

        let insert_result = AdminRoleMerge::insert_many(result)
            .exec(db)
            .await?;
        Ok(insert_result.last_insert_id.unwrap_or_default())
    }

    /// 按用户id删除关联id
    pub async fn delete_by_admin_id<C: ConnectionTrait>(db: &C, admin_id: &Option<i64>) -> Result<i64, DbErr> {
        let result = AdminRoleMerge::delete_many()
            .filter(admin_role_merge::Column::AdminId.eq(admin_id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 按角色id删除关联id
    pub async fn delete_by_role_id<C: ConnectionTrait>(db: &C, role_id: &Option<i64>) -> Result<i64, DbErr> {
        let result = AdminRoleMerge::delete_many()
            .filter(admin_role_merge::Column::RoleId.eq(role_id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
    

    pub async fn find_by_admin_id(db: &DbConn, admin_id: &Option<i64>) -> Result<Vec<admin_role_merge::Model>, DbErr> {
        let result =  AdminRoleMerge::find()
            .distinct()
            .filter(admin_role_merge::Column::AdminId.eq(admin_id.clone()))
            .all(db)
            .await?;
        Ok(result)
    }


    pub async fn find_by_admin_ids(db: &DbConn, admin_id: Vec<i64>) -> Result<Vec<admin_role_merge::Model>, DbErr> {
        let result =  AdminRoleMerge::find()
            .filter(admin_role_merge::Column::AdminId.is_in(admin_id))
            .all(db)
            .await?;
        Ok(result)
    }
}

