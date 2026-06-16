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
use crate::modules::shop::entity::shop_user_merge::{self, Entity as ShopUserMergeEntity};

/// 鎻掑叆搴楅摵涓庣敤鎴峰叧鑱旇褰?
/// 
/// # 鍙傛暟
/// * `db` - 鏁版嵁搴撹繛鎺?
/// * `shop_id` - 搴楅摵ID
/// * `user_id` - 鐢ㄦ埛ID
/// * `is_admin` - 鏄惁绠＄悊鍛?(0: 鏅€氱敤鎴? 1: 绠＄悊鍛?
/// 
/// # 杩斿洖鍊?
/// 杩斿洖鎻掑叆璁板綍鐨処D
/// 
/// # 浣跨敤绀轰緥
/// ```rust
/// let record_id = shop_user_merge::insert(db, 1, 100, Some(1)).await?;
/// ```
pub async fn insert<C: ConnectionTrait>(db: &C, shop_id: i64, user_id: i64, is_admin: Option<i32>) -> Result<i64, DbErr> {
    let active_model = shop_user_merge::ActiveModel {
        shop_id: Set(shop_id),
        user_id: Set(user_id),
        is_admin: Set(is_admin),
        ..Default::default()
    };
    ShopUserMergeEntity::insert(active_model).exec(db).await.map(|r| r.last_insert_id)
}

/// 鍒犻櫎搴楅摵涓庣敤鎴峰叧鑱旇褰?
/// 
/// # 鍙傛暟
/// * `db` - 鏁版嵁搴撹繛鎺?
/// * `shop_id` - 搴楅摵ID
/// * `user_id` - 鐢ㄦ埛ID
/// 
/// # 杩斿洖鍊?
/// 杩斿洖鍒犻櫎鐨勮褰曟暟
/// 
/// # 浣跨敤绀轰緥
/// ```rust
/// let affected = shop_user_merge::delete(db, 1, 100).await?;
/// ```
pub async fn delete(db: &DbConn, shop_id: i64, user_id: i64) -> Result<i64, DbErr> {
    ShopUserMergeEntity::delete_many()
        .filter(shop_user_merge::Column::ShopId.eq(shop_id))
        .filter(shop_user_merge::Column::UserId.eq(user_id))
        .exec(db).await.map(|r| r.rows_affected as i64)
}

/// 鏍规嵁搴楅摵ID鍒犻櫎鎵€鏈夊叧鑱旇褰?
/// 
/// # 鍙傛暟
/// * `db` - 鏁版嵁搴撹繛鎺?
/// * `shop_id` - 搴楅摵ID
/// 
/// # 杩斿洖鍊?
/// 杩斿洖鍒犻櫎鐨勮褰曟暟
/// 
/// # 浣跨敤绀轰緥
/// ```rust
/// let affected = shop_user_merge::delete_by_shop_id(db, 1).await?;
/// ```
pub async fn delete_by_shop_id(db: &DbConn, shop_id: i64) -> Result<i64, DbErr> {
    ShopUserMergeEntity::delete_many()
        .filter(shop_user_merge::Column::ShopId.eq(shop_id))
        .exec(db).await.map(|r| r.rows_affected as i64)
}

/// 鏍规嵁鐢ㄦ埛ID鍒犻櫎鎵€鏈夊叧鑱旇褰?
/// 
/// # 鍙傛暟
/// * `db` - 鏁版嵁搴撹繛鎺?
/// * `user_id` - 鐢ㄦ埛ID
/// 
/// # 杩斿洖鍊?
/// 杩斿洖鍒犻櫎鐨勮褰曟暟
/// 
/// # 浣跨敤绀轰緥
/// ```rust
/// let affected = shop_user_merge::delete_by_user_id(db, 100).await?;
/// ```
pub async fn delete_by_user_id(db: &DbConn, user_id: i64) -> Result<i64, DbErr> {
    ShopUserMergeEntity::delete_many()
        .filter(shop_user_merge::Column::UserId.eq(user_id))
        .exec(db).await.map(|r| r.rows_affected as i64)
}

/// 鏌ヨ搴楅摵鏄惁涓庣敤鎴峰叧鑱?
/// 
/// # 鍙傛暟
/// * `db` - 鏁版嵁搴撹繛鎺?
/// * `shop_id` - 搴楅摵ID
/// * `user_id` - 鐢ㄦ埛ID
/// 
/// # 杩斿洖鍊?
/// 濡傛灉宸插叧鑱旇繑鍥?true锛屽惁鍒欒繑鍥?false
/// 
/// # 浣跨敤绀轰緥
/// ```rust
/// let exists = shop_user_merge::exists(db, 1, 100).await?;
/// if exists {
///     // 宸插叧鑱?
/// }
/// ```
pub async fn exists(db: &DbConn, shop_id: i64, user_id: i64) -> Result<bool, DbErr> {
    ShopUserMergeEntity::find()
        .filter(shop_user_merge::Column::ShopId.eq(shop_id))
        .filter(shop_user_merge::Column::UserId.eq(user_id))
        .one(db).await.map(|r| r.is_some())
}

/// 鏍规嵁搴楅摵ID鏌ヨ鎵€鏈夊叧鑱旂殑鐢ㄦ埛ID
/// 
/// # 鍙傛暟
/// * `db` - 鏁版嵁搴撹繛鎺?
/// * `shop_id` - 搴楅摵ID
/// 
/// # 杩斿洖鍊?
/// 杩斿洖鐢ㄦ埛ID鍒楄〃
/// 
/// # 浣跨敤绀轰緥
/// ```rust
/// let user_ids = shop_user_merge::find_user_ids_by_shop_id(db, 1).await?;
/// for user_id in user_ids {
///     // 澶勭悊姣忎釜鐢ㄦ埛
/// }
/// ```
pub async fn find_user_ids_by_shop_id(db: &DbConn, shop_id: i64) -> Result<Vec<i64>, DbErr> {
    ShopUserMergeEntity::find()
        .filter(shop_user_merge::Column::ShopId.eq(shop_id))
        .select_only()
        .column(shop_user_merge::Column::UserId)
        .into_tuple::<i64>()
        .all(db).await
}

/// 鏍规嵁鐢ㄦ埛ID鏌ヨ鎵€鏈夊叧鑱旂殑搴楅摵ID
/// 
/// # 鍙傛暟
/// * `db` - 鏁版嵁搴撹繛鎺?
/// * `user_id` - 鐢ㄦ埛ID
/// 
/// # 杩斿洖鍊?
/// 杩斿洖搴楅摵ID鍒楄〃
/// 
/// # 浣跨敤绀轰緥
/// ```rust
/// let shop_ids = shop_user_merge::find_shop_ids_by_user_id(db, 100).await?;
/// for shop_id in shop_ids {
///     // 澶勭悊姣忎釜搴楅摵
/// }
/// ```
pub async fn find_shop_ids_by_user_id<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<Vec<i64>, DbErr> {
    ShopUserMergeEntity::find()
        .filter(shop_user_merge::Column::UserId.eq(user_id))
        .select_only()
        .column(shop_user_merge::Column::ShopId)
        .into_tuple::<i64>()
        .all(db).await
}

/// 鏍规嵁搴楅摵ID鍜岀敤鎴稩D鏌ヨ鍏宠仈璁板綍
/// 
/// # 鍙傛暟
/// * `db` - 鏁版嵁搴撹繛鎺?
/// * `shop_id` - 搴楅摵ID
/// * `user_id` - 鐢ㄦ埛ID
/// 
/// # 杩斿洖鍊?
/// 杩斿洖鍏宠仈璁板綍锛堝寘鍚?is_admin 瀛楁锛?
/// 
/// # 浣跨敤绀轰緥
/// ```rust
/// if let Some(record) = shop_user_merge::find_by_shop_and_user(db, 1, 100).await? {
///     if record.is_admin == Some(1) {
///         // 鏄鐞嗗憳
///     }
/// }
/// ```
pub async fn find_by_shop_and_user(db: &DbConn, shop_id: i64, user_id: i64) -> Result<Option<shop_user_merge::Model>, DbErr> {
    ShopUserMergeEntity::find()
        .filter(shop_user_merge::Column::ShopId.eq(shop_id))
        .filter(shop_user_merge::Column::UserId.eq(user_id))
        .one(db).await
}

/// 鍒ゆ柇鐢ㄦ埛鏄惁鏄簵閾虹鐞嗗憳
/// 
/// # 鍙傛暟
/// * `db` - 鏁版嵁搴撹繛鎺?
/// * `shop_id` - 搴楅摵ID
/// * `user_id` - 鐢ㄦ埛ID
/// 
/// # 杩斿洖鍊?
/// 濡傛灉鏄鐞嗗憳杩斿洖 true锛屽惁鍒欒繑鍥?false
/// 
/// # 浣跨敤绀轰緥
/// ```rust
/// let is_admin = shop_user_merge::is_admin(db, 1, 100).await?;
/// if is_admin {
///     // 鏈夌鐞嗗憳鏉冮檺
/// }
/// ```
pub async fn is_admin(db: &DbConn, shop_id: i64, user_id: i64) -> Result<bool, DbErr> {
    Ok(match ShopUserMergeEntity::find()
        .filter(shop_user_merge::Column::ShopId.eq(shop_id))
        .filter(shop_user_merge::Column::UserId.eq(user_id))
        .select_only()
        .column(shop_user_merge::Column::IsAdmin)
        .into_tuple::<Option<i32>>()
        .one(db).await? {
        Some(Some(1)) => true,
        _ => false,
    })
}

/// 鏇存柊鐢ㄦ埛鐨勭鐞嗗憳鐘舵€?
/// 
/// # 鍙傛暟
/// * `db` - 鏁版嵁搴撹繛鎺?
/// * `shop_id` - 搴楅摵ID
/// * `user_id` - 鐢ㄦ埛ID
/// * `is_admin` - 鏄惁绠＄悊鍛?(0: 鏅€氱敤鎴? 1: 绠＄悊鍛?
/// 
/// # 杩斿洖鍊?
/// 杩斿洖鏇存柊鐨勮褰曟暟
/// 
/// # 浣跨敤绀轰緥
/// ```rust
/// let affected = shop_user_merge::update_admin_status(db, 1, 100, Some(1)).await?;
/// ```
pub async fn update_admin_status(db: &DbConn, shop_id: i64, user_id: i64, is_admin: Option<i32>) -> Result<i64, DbErr> {
    let active_model = shop_user_merge::ActiveModel {
        shop_id: Set(shop_id),
        user_id: Set(user_id),
        is_admin: Set(is_admin),
        ..Default::default()
    };
    ShopUserMergeEntity::update_many()
        .set(active_model)
        .filter(shop_user_merge::Column::ShopId.eq(shop_id))
        .filter(shop_user_merge::Column::UserId.eq(user_id))
        .exec(db).await.map(|r| r.rows_affected as i64)
}

/// 鏍规嵁搴楅摵ID鏌ヨ鎵€鏈夌鐞嗗憳鐢ㄦ埛ID
/// 
/// # 鍙傛暟
/// * `db` - 鏁版嵁搴撹繛鎺?
/// * `shop_id` - 搴楅摵ID
/// 
/// # 杩斿洖鍊?
/// 杩斿洖绠＄悊鍛樼敤鎴稩D鍒楄〃
/// 
/// # 浣跨敤绀轰緥
/// ```rust
/// let admin_ids = shop_user_merge::find_admin_user_ids_by_shop_id(db, 1).await?;
/// for admin_id in admin_ids {
///     // 澶勭悊姣忎釜绠＄悊鍛?
/// }
/// ```
pub async fn find_admin_user_ids_by_shop_id(db: &DbConn, shop_id: i64) -> Result<Vec<i64>, DbErr> {
    ShopUserMergeEntity::find()
        .filter(shop_user_merge::Column::ShopId.eq(shop_id))
        .filter(shop_user_merge::Column::IsAdmin.eq(Some(1)))
        .select_only()
        .column(shop_user_merge::Column::UserId)
        .into_tuple::<i64>()
        .all(db).await
}
