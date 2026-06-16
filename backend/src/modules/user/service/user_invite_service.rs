//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::DbConn;
use crate::core::errors::error::Result;
use crate::modules::user::model::user_invite::{UserInviteModel, UserInviteSaveDTO};

pub async fn insert(db: &DbConn, form_data: &UserInviteSaveDTO) -> Result<i64> {
    let result = UserInviteModel::insert(db, form_data).await?;
    Ok(result)
}

pub async fn batch_delete(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    let result = UserInviteModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn update_by_id(db: &DbConn, id: i64, form_data: &UserInviteSaveDTO) -> Result<i64> {
    let result = UserInviteModel::update_by_id(db, id, form_data).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: &i64) -> Result<Option<crate::modules::user::entity::user_invite::Model>> {
    let result = UserInviteModel::find_by_id(db, id).await?;
    Ok(result)
}

pub async fn find_by_to_user_id(db: &DbConn, to_user_id: &i64) -> Result<Vec<crate::modules::user::entity::user_invite::Model>> {
    let result = UserInviteModel::find_by_to_user_id(db, to_user_id).await?;
    Ok(result)
}

pub async fn find_by_form_user_id(db: &DbConn, form_user_id: &i64) -> Result<Vec<crate::modules::user::entity::user_invite::Model>> {
    let result = UserInviteModel::find_by_form_user_id(db, form_user_id).await?;
    Ok(result)
}

pub async fn select_count(
    db: &DbConn,
    wheres: crate::modules::user::model::user_invite::PageWhere,
) -> Result<i64> {
    let result = UserInviteModel::select_count(db, wheres).await?;
    Ok(result)
}

pub async fn select_in_page(
    db: &DbConn,
    page: i64,
    per_page: i64,
    wheres: crate::modules::user::model::user_invite::PageWhere,
) -> Result<(Vec<crate::modules::user::entity::user_invite::Model>, i64)> {
    let result = UserInviteModel::select_in_page(db, page, per_page, wheres).await?;
    Ok(result)
}
