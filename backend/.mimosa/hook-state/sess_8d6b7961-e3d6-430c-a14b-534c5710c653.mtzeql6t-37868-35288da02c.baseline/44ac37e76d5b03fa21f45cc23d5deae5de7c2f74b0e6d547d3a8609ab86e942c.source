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
use crate::modules::statistics::model::access_record::{AccessRecordModel, AccessRecordSaveDTO, RecordSaveRequest};
use crate::SNOWFLAKE;

pub async fn insert(db: &DbConn, record: &RecordSaveRequest) -> Result<i64> {
    let mut record_entity = AccessRecordSaveDTO::from(record.clone());
    record_entity.id = Some(SNOWFLAKE.generate() as i64);
    let result = AccessRecordModel::insert(&db, &record_entity).await;
    Ok(result.unwrap_or_default())
}