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
use crate::modules::system::entity::{admin_post_merge, admin_post_merge::Entity as AdminPostMerge};


#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct AdminPostMergeSaveDTO {
    /// ID
    pub id: Option<i64>,
    /// 用户id
    pub admin_id: Option<i64>,
    /// 岗位id
    pub post_id: Option<i64>,
    /// 创建日期
    pub create_time: Option<DateTime>,
}


pub struct AdminPostMergeModel;

impl AdminPostMergeModel {
    pub async fn insert_batch<C: ConnectionTrait>(db: &C,list: &Vec<AdminPostMergeSaveDTO>) -> Result<i64, DbErr> {
        let result: Vec<admin_post_merge::ActiveModel> = list.iter().map(|item| admin_post_merge::ActiveModel {
            admin_id:       Set(item.admin_id),
            post_id:        Set(item.post_id),
            create_time:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        }).collect();

        let insert_result = AdminPostMerge::insert_many(result)
            .exec(db)
            .await?;
        Ok(insert_result.last_insert_id.unwrap_or_default())
    }
    pub async fn delete_by_admin_id<C: ConnectionTrait>(db: &C, admin_id: &Option<i64>) -> Result<DeleteResult, DbErr> {
        let result = AdminPostMerge::delete_many()
            .filter(admin_post_merge::Column::AdminId.eq(admin_id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result)
    }
    
    pub async fn delete_by_post_id(db: &DbConn,post_id: i64) -> Result<i64, DbErr> {
        let result = AdminPostMerge::delete_many()
            .filter(admin_post_merge::Column::PostId.eq(post_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
    


    /// 按用户id查询关联岗位
    pub async fn find_by_admin_id(db: &DbConn, admin_id: i64) -> Result<Vec<admin_post_merge::Model>, DbErr> {
        let result =  AdminPostMerge::find()
            .filter(admin_post_merge::Column::AdminId.eq(admin_id))
            .all(db)
            .await?;
        Ok(result)
    }
    pub async fn find_by_admin_ids(db: &DbConn,admin_id: Vec<i64>) -> Result<Vec<admin_post_merge::Model>, DbErr> {
        let result =  AdminPostMerge::find()
            .filter(admin_post_merge::Column::AdminId.is_in(admin_id))
            .all(db)
            .await?;
        Ok(result)
    }
}