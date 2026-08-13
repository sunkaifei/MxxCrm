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
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_statistics_agg_batch")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub topic: String,
    pub start_date: Date,
    pub end_date: Date,
    pub row_count: i32,
    /// 1=定时 2=手动（数字类型，遵循项目状态字段规范）
    pub trigger_type: i16,
    pub trigger_by: i64,
    /// 1=成功 2=失败
    pub status: i16,
    pub message: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
