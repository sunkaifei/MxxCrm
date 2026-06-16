//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, QueryOrder, Set, DbConn};
use crate::modules::user::entity::user_level;
use crate::modules::user::entity::user_level::Entity as UserLevel;

pub struct UserLevelModel;

impl UserLevelModel {
    /// 鏍规嵁ID鑾峰彇浼氬憳绛夌骇閰嶇疆
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<user_level::Model>, sea_orm::DbErr> {
        UserLevel::find_by_id(id).one(db).await
    }

    /// 鏍规嵁绛夌骇鏍囪瘑鑾峰彇閰嶇疆
    pub async fn find_by_code(db: &DbConn, code: &str) -> Result<Option<user_level::Model>, sea_orm::DbErr> {
        UserLevel::find()
            .filter(user_level::Column::LevelCode.eq(code))
            .filter(user_level::Column::Status.eq(1))
            .one(db)
            .await
    }

    /// 鑾峰彇鎵€鏈夊惎鐢ㄧ殑浼氬憳绛夌骇
    pub async fn find_all_enabled(db: &DbConn) -> Result<Vec<user_level::Model>, sea_orm::DbErr> {
        UserLevel::find()
            .filter(user_level::Column::Status.eq(1))
            .order_by_asc(user_level::Column::SortOrder)
            .all(db)
            .await
    }

    /// 鑾峰彇鎵€鏈変細鍛樼瓑绾э紙鍖呭惈绂佺敤鐨勶級
    pub async fn find_all(db: &DbConn) -> Result<Vec<user_level::Model>, sea_orm::DbErr> {
        UserLevel::find()
            .order_by_asc(user_level::Column::SortOrder)
            .all(db)
            .await
    }

    /// 鍒涘缓浼氬憳绛夌骇
    pub async fn create(db: &DbConn, model: &user_level::Model) -> Result<i64, sea_orm::DbErr> {
        let active_model = user_level::ActiveModel {
            level_code: Set(model.level_code.clone()),
            level_name: Set(model.level_name.clone()),
            level_desc: Set(model.level_desc.clone()),
            turtle_quota: Set(model.turtle_quota),
            qrcode_quota: Set(model.qrcode_quota),
            tag_quota: Set(model.tag_quota),
            permissions: Set(model.permissions.clone()),
            sort_order: Set(model.sort_order),
            status: Set(model.status),
            ..Default::default()
        };
        UserLevel::insert(active_model).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 鏇存柊浼氬憳绛夌骇
    pub async fn update(db: &DbConn, id: i64, model: &user_level::Model) -> Result<user_level::Model, sea_orm::DbErr> {
        let active_model = user_level::ActiveModel {
            id: Set(id),
            level_code: Set(model.level_code.clone()),
            level_name: Set(model.level_name.clone()),
            level_desc: Set(model.level_desc.clone()),
            turtle_quota: Set(model.turtle_quota),
            qrcode_quota: Set(model.qrcode_quota),
            tag_quota: Set(model.tag_quota),
            permissions: Set(model.permissions.clone()),
            sort_order: Set(model.sort_order),
            status: Set(model.status),
            ..Default::default()
        };
        UserLevel::update(active_model).exec(db).await
    }

    /// 鍒犻櫎浼氬憳绛夌骇锛堣蒋鍒犻櫎锛?
    pub async fn delete(db: &DbConn, id: i64) -> Result<user_level::Model, sea_orm::DbErr> {
        let model = UserLevel::find_by_id(id).one(db).await?;
        if let Some(m) = model {
            let mut active_model: user_level::ActiveModel = m.into();
            active_model.status = Set(0);
            UserLevel::update(active_model).exec(db).await
        } else {
            Err(sea_orm::DbErr::RecordNotFound(String::from("UserLevel not found")))
        }
    }
}