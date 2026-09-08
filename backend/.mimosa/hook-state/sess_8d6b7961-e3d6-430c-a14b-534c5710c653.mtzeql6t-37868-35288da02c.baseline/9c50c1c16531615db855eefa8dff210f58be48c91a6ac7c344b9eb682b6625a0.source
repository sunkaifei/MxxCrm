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
use crate::modules::system::entity::{perm_set_menu_merge, perm_set_menu_merge::Entity as PermSetMenuMerge};


#[derive(PartialEq, Default, Clone, Debug, Serialize, Deserialize)]
pub struct PermSetMenuMergeSaveDTO {
    /// ID
    pub id: Option<i64>,
    /// 菜单id
    pub menu_id: Option<i64>,
    /// 权限集id
    pub perm_set_id: Option<i64>,
    /// 创建日期
    pub create_time: Option<DateTime>,
    /// 更新日期
    pub update_time: Option<DateTime>,
}

pub struct PermSetMenuMergeModel;

impl PermSetMenuMergeModel {
    pub async fn insert_batch<C: ConnectionTrait>(db: &C, list: &Vec<PermSetMenuMergeSaveDTO>) -> Result<i64, DbErr> {
        let result: Vec<perm_set_menu_merge::ActiveModel> = list.iter().map(|item| perm_set_menu_merge::ActiveModel {
            menu_id: Set(item.menu_id),
            perm_set_id: Set(item.perm_set_id),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        }).collect();

        let insert_result = PermSetMenuMerge::insert_many(result)
            .exec(db)
            .await?;
        Ok(insert_result.last_insert_id.unwrap_or_default())
    }

    /// 按权限集id删除关联id
    pub async fn delete_by_perm_set_id<C: ConnectionTrait>(db: &C, perm_set_id: &Option<i64>) -> Result<i64, DbErr> {
        let result = PermSetMenuMerge::delete_many()
            .filter(perm_set_menu_merge::Column::PermSetId.eq(perm_set_id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 按权限集id查询关联id
    pub async fn find_by_perm_set_id(db: &DbConn, perm_set_id: &Option<i64>) -> Result<Vec<perm_set_menu_merge::Model>, DbErr> {
        PermSetMenuMerge::find()
            .filter(perm_set_menu_merge::Column::PermSetId.eq(perm_set_id.clone().unwrap_or_default()))
            .all(db)
            .await
    }

    pub async fn find_by_perm_set_ids(db: &DbConn, perm_set_id: &Vec<i64>) -> Result<Vec<perm_set_menu_merge::Model>, DbErr> {
        PermSetMenuMerge::find()
            .filter(perm_set_menu_merge::Column::PermSetId.is_in(perm_set_id.clone()))
            .all(db)
            .await
    }
}
