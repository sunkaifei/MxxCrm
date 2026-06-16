//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mxx_sms_verification")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column_name = "phone")]
    pub phone: String,
    #[sea_orm(column_name = "code")]
    pub code: String,
    #[sea_orm(column_name = "scene")]
    pub scene: String,
    #[sea_orm(column_name = "status")]
    pub status: i32,
    #[sea_orm(column_name = "send_count")]
    pub send_count: i32,
    #[sea_orm(column_name = "error_count")]
    pub error_count: i32,
    #[sea_orm(column_name = "lock_until")]
    pub lock_until: Option<DateTime>,
    #[sea_orm(column_name = "expire_time")]
    pub expire_time: DateTime,
    #[sea_orm(column_name = "ip")]
    pub ip: Option<String>,
    #[sea_orm(column_name = "create_time")]
    pub create_time: DateTime,
    #[sea_orm(column_name = "update_time")]
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}