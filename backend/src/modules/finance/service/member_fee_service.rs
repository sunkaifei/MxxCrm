//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::DatabaseConnection;
use crate::modules::finance::model::member_fee::{MemberFeeDTO, MemberFeeSaveRequest, MemberFeeQuery, MemberFeeModel};

pub async fn get_list(db: &DatabaseConnection, query: MemberFeeQuery) -> Result<(Vec<MemberFeeDTO>, i64), String> {
    MemberFeeModel::find_list(db, query).await.map_err(|e| e.to_string())
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<MemberFeeDTO>, String> {
    MemberFeeModel::find_by_id(db, id).await.map_err(|e| e.to_string())
}

pub async fn get_by_user_id(db: &DatabaseConnection, user_id: i64) -> Result<Option<MemberFeeDTO>, String> {
    MemberFeeModel::find_by_user_id(db, user_id).await.map_err(|e| e.to_string())
}

pub async fn insert(db: &DatabaseConnection, req: MemberFeeSaveRequest) -> Result<MemberFeeDTO, String> {
    MemberFeeModel::insert(db, req).await.map_err(|e| e.to_string())
}

pub async fn update(db: &DatabaseConnection, id: i64, req: MemberFeeSaveRequest) -> Result<MemberFeeDTO, String> {
    MemberFeeModel::update(db, id, req).await.map_err(|e| e.to_string())
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<bool, String> {
    MemberFeeModel::delete(db, id).await.map_err(|e| e.to_string())
}
