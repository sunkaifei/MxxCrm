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
use crate::modules::system::entity::{role_dept_merge, role_dept_merge::Entity as RoleDeptMerge};


#[derive(PartialEq, Default, Clone, Debug, Serialize, Deserialize)]
pub struct RoleDeptMergeSaveDTO {
    pub id: Option<i64>,
    pub dept_id: Option<i64>,
    pub role_id: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

pub struct RoleDeptMergeModel;

impl RoleDeptMergeModel {
    pub async fn insert_batch<C: ConnectionTrait>(db: &C, list: &Vec<RoleDeptMergeSaveDTO>) -> Result<i64, DbErr> {
        let result: Vec<role_dept_merge::ActiveModel> = list.iter().map(|item| role_dept_merge::ActiveModel {
            dept_id: Set(item.dept_id),
            role_id: Set(item.role_id),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        }).collect();

        let insert_result = RoleDeptMerge::insert_many(result)
            .exec(db)
            .await?;
        Ok(insert_result.last_insert_id.unwrap_or_default())
    }

    pub async fn delete_by_role_id<C: ConnectionTrait>(db: &C, role_id: &Option<i64>) -> Result<i64, DbErr> {
        let result = RoleDeptMerge::delete_many()
            .filter(role_dept_merge::Column::RoleId.eq(role_id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_role_id(db: &DbConn, role_id: &Option<i64>) -> Result<Vec<role_dept_merge::Model>, DbErr> {
        RoleDeptMerge::find()
            .filter(role_dept_merge::Column::RoleId.eq(role_id.clone().unwrap_or_default()))
            .all(db)
            .await
    }
}
