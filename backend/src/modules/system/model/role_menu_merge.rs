//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use sea_orm::prelude::DateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{role_menu_merge, role_menu_merge::Entity as RoleMenuMerge};


#[derive(PartialEq, Default, Clone, Debug, Serialize, Deserialize)]
pub struct RoleMenuMergeSaveDTO {
    /// ID
    pub id: Option<i64>,
    /// 菜单id
    pub menu_id: Option<i64>,
    /// 角色id
    pub role_id: Option<i64>,
    /// 创建日期
    pub create_time: Option<DateTime>,
    /// 更新日期
    pub update_time: Option<DateTime>,
}

pub struct RoleMenuMergeModel;

impl RoleMenuMergeModel {
    pub async fn insert_batch(db: &DbConn, list: &Vec<RoleMenuMergeSaveDTO>) -> Result<i64, DbErr> {
        let result: Vec<role_menu_merge::ActiveModel> = list.iter().map(|item| role_menu_merge::ActiveModel {
            menu_id: Set(item.menu_id),
            role_id: Set(item.role_id),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        }).collect();

        let insert_result = RoleMenuMerge::insert_many(result)
            .exec(db)
            .await?;
        Ok(insert_result.last_insert_id.unwrap_or_default())
    }

    /// 按角色id删除关联id
    pub async fn delete_by_role_id(db: &DbConn, role_id: &Option<i64>) -> Result<i64, DbErr> {
        let result = RoleMenuMerge::delete_many()
            .filter(role_menu_merge::Column::RoleId.eq(role_id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
    
    /// 按角色id查询关联id
    pub async fn find_by_role_id(db: &DbConn, role_id: &Option<i64>) -> Result<Vec<role_menu_merge::Model>, DbErr> {
        RoleMenuMerge::find()
            .filter(role_menu_merge::Column::RoleId.eq(role_id.clone().unwrap_or_default()))
            .all(db)
            .await
    }

    pub async fn find_by_role_ids(db: &DbConn, role_id: &Vec<i64>) -> Result<Vec<role_menu_merge::Model>, DbErr> {
        RoleMenuMerge::find()
            .filter(role_menu_merge::Column::RoleId.is_in(role_id.clone()))
            .all(db)
            .await
    }
}
