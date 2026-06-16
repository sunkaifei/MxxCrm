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
use crate::core::errors::error::{Error, Result};
use crate::modules::user::model::user_menu::{MenuDetailVO, MenuSaveDTO, UserMenuModel};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

pub async fn insert(db: &DbConn, form_data: &MenuSaveDTO) -> Result<i64> {
    let result = UserMenuModel::insert(&db, form_data).await?;
    Ok(result)
}


pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64>{
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let config = UserMenuModel::batch_delete_by_ids(db, ids).await?;
    Ok(config)
}

pub async fn update_by_id(db: &DbConn, form_data: &MenuSaveDTO) -> Result<i64> {
    let result = UserMenuModel::update_by_id(&db, &form_data.id, form_data).await?;
    Ok(result)
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<MenuDetailVO> {
    let result_data = UserMenuModel::find_by_id(db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "菜单信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    let result_data = MenuDetailVO::from(result_data);
    Ok(result_data)
}