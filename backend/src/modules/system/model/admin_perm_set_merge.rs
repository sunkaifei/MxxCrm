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
use crate::modules::system::entity::{admin_perm_set_merge, admin_perm_set_merge::Entity as AdminPermSetMerge};
use sea_orm::prelude::DateTime;
use sea_orm::*;


#[derive(PartialEq, Default, Clone, Debug, Serialize, Deserialize)]
pub struct AdminPermSetMergeSaveDTO {
    ///ID
    pub id: Option<i64>,
    ///用户id
    pub admin_id: Option<i64>,
    ///权限集id
    pub perm_set_id: Option<i64>,
    ///创建时间
    pub create_time: Option<DateTime>,
}

pub struct AdminPermSetMergeModel;

impl AdminPermSetMergeModel {
    pub async fn insert_batch<C: ConnectionTrait>(db: &C,list: &Vec<AdminPermSetMergeSaveDTO>) -> Result<i64, DbErr> {
        let result: Vec<admin_perm_set_merge::ActiveModel> = list.iter().map(|item| admin_perm_set_merge::ActiveModel {
            admin_id:       Set(item.admin_id),
            perm_set_id:    Set(item.perm_set_id),
            create_time:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        }).collect();

        let insert_result = AdminPermSetMerge::insert_many(result)
            .exec(db)
            .await?;
        Ok(insert_result.last_insert_id.unwrap_or_default())
    }

    /// 按用户id删除关联id
    pub async fn delete_by_admin_id<C: ConnectionTrait>(db: &C, admin_id: &Option<i64>) -> Result<i64, DbErr> {
        let result = AdminPermSetMerge::delete_many()
            .filter(admin_perm_set_merge::Column::AdminId.eq(admin_id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 按权限集id删除关联id
    pub async fn delete_by_perm_set_id<C: ConnectionTrait>(db: &C, perm_set_id: &Option<i64>) -> Result<i64, DbErr> {
        let result = AdminPermSetMerge::delete_many()
            .filter(admin_perm_set_merge::Column::PermSetId.eq(perm_set_id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }


    pub async fn find_by_admin_id(db: &DbConn, admin_id: &Option<i64>) -> Result<Vec<admin_perm_set_merge::Model>, DbErr> {
        let result =  AdminPermSetMerge::find()
            .distinct()
            .filter(admin_perm_set_merge::Column::AdminId.eq(admin_id.clone()))
            .all(db)
            .await?;
        Ok(result)
    }


    pub async fn find_by_perm_set_id(db: &DbConn, perm_set_id: &Option<i64>) -> Result<Vec<admin_perm_set_merge::Model>, DbErr> {
        let result =  AdminPermSetMerge::find()
            .filter(admin_perm_set_merge::Column::PermSetId.eq(perm_set_id.clone()))
            .all(db)
            .await?;
        Ok(result)
    }

    /// 按权限集ID查询关联的所有用户ID（用于权限缓存失效）
    pub async fn find_admin_ids_by_perm_set_id(db: &DbConn, perm_set_id: &Option<i64>) -> Result<Vec<i64>, DbErr> {
        let result = AdminPermSetMerge::find()
            .filter(admin_perm_set_merge::Column::PermSetId.eq(perm_set_id.unwrap_or_default()))
            .all(db)
            .await?;
        Ok(result.into_iter().filter_map(|m| m.admin_id).collect())
    }
}
