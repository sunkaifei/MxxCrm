//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{DbConn, ConnectionTrait};
use crate::core::errors::error::Result;
use crate::modules::user::model::user_platform::{UserPlatformModel, UserPlatformSaveDTO};

pub async fn insert<C: ConnectionTrait>(db: &C, form_data: &UserPlatformSaveDTO) -> Result<i64> {
    let result = UserPlatformModel::insert(db, form_data).await?;
    Ok(result)
}

pub async fn batch_delete(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    let result = UserPlatformModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn update_by_id(db: &DbConn, id: i64, form_data: &UserPlatformSaveDTO) -> Result<i64> {
    let result = UserPlatformModel::update_by_id(db, id, form_data).await?;
    Ok(result)
}