//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::system::entity::{admin_notice_merge, admin_notice_merge::Entity as AdminNoticeMerge};
use sea_orm::prelude::DateTime;
use sea_orm::*;

pub struct AdminNoticeMergeSaveDTO {
    /// id
    pub id: Option<i64>,
    /// 公共通知id
    pub notice_id: Option<i64>,
    /// 用户id
    pub user_id: Option<i64>,
    /// 读取状态（0: 未读, 1: 已读）
    pub is_read: Option<i32>,
    /// 阅读时间
    pub read_time: Option<DateTime>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除标志（0未删除 1已删除）
    pub deleted: Option<i32>,
}

pub struct AdminNoticeMergeModel;

impl AdminNoticeMergeModel {

    /// ### 批量插入
    /// * `db`: 数据库连接
    /// * `list`: 批量插入数据
    pub async fn insert_batch(db: &DbConn, list: &Vec<AdminNoticeMergeSaveDTO>) -> Result<i64, DbErr> {
        let result: Vec<admin_notice_merge::ActiveModel> = list.iter().map(|item| admin_notice_merge::ActiveModel {
            notice_id:      Set(item.notice_id),
            user_id:        Set(item.user_id),
            is_read:        Set(item.is_read),
            read_time:      Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            create_time:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        }).collect();

        let insert_result = AdminNoticeMerge::insert_many(result)
            .exec(db)
            .await?;
        Ok(insert_result.last_insert_id.unwrap_or_default())
    }

    /// # 按用户id删除关联id
    /// * `db`: 数据库连接
    /// * `ids`: 用户关联通知id
    pub async fn delete_by_ids(db: &DbConn,ids: &Vec<i64>) -> Result<i64, DbErr> {
        let result = AdminNoticeMerge::delete_many()
            .filter(admin_notice_merge::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// ### 更新阅读状态为已读
    /// * `db`: 数据库连接
    /// * `notice_id`: 通知id
    /// * `user_id`: 用户id
    pub async fn update_by_read(db: &DbConn, notice_id: &Option<i64>, user_id: &Option<i64>) -> Result<i64, DbErr> {
        let payload = admin_notice_merge::ActiveModel {
            is_read:           Set(Some(1)),
            read_time:         Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = AdminNoticeMerge::update_many()
            .set(payload)
            .filter(admin_notice_merge::Column::NoticeId.eq(notice_id.clone()))
            .filter(admin_notice_merge::Column::UserId.eq(user_id.clone()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// ### 更新所有阅读状态为已读
    /// * `db`: 数据库连接
    /// * `user_id`: 用户id
    pub async fn update_by_read_all(db: &DbConn, user_id: &Option<i64>) -> Result<i64, DbErr> {
        let payload = admin_notice_merge::ActiveModel {
            is_read:           Set(Some(1)),
            read_time:         Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = AdminNoticeMerge::update_many()
            .set(payload)
            .filter(admin_notice_merge::Column::UserId.eq(user_id.clone()))
            .filter(admin_notice_merge::Column::IsRead.eq(Some(0)))
            .filter(admin_notice_merge::Column::Deleted.eq(0))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }
    
    

    /// ### 更新假删除通知信息
    /// * `db`: 数据库连接
    /// * `id`: 用户关联通知id
    /// * `is_read`: 阅读状态，0: 未读, 1: 已读
    pub async fn update_by_id_delete(db: &DbConn, id: &Option<i64>) -> Result<i64, DbErr> {
        let payload = admin_notice_merge::ActiveModel {
            deleted:           Set(Some(1)),
            update_time:       Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = AdminNoticeMerge::update_many()
            .set(payload)
            .filter(admin_notice_merge::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }
    
    /// ### 查询用户关联通知信息，根据通知id和用户id，如果假删除，则返回None
    /// * `db`: 数据库连接
    /// * `notice_id`: 通知id
    /// * `user_id`: 用户id
    /// 
    /// 返回值: 用户关联通知信息
    pub async fn find_merge_by_notice_and_user(db: &DbConn, notice_id: &Option<i64>, user_id: &Option<i64>) -> Result<Option<admin_notice_merge::Model>, DbErr> {
        let result = AdminNoticeMerge::find()
            .filter(admin_notice_merge::Column::NoticeId.eq(notice_id.clone()))
            .filter(admin_notice_merge::Column::UserId.eq(user_id.clone()))
            .filter(admin_notice_merge::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(result)
    }
}