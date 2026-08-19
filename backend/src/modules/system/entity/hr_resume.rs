//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_hr_resume")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub admin_id: i64,
    /// 条目类型：1教育经历 2工作经历 3证书
    pub kind: Option<i32>,
    pub title: Option<String>,
    pub org: Option<String>,
    pub start_date: Option<NaiveDate>,
    /// 结束日期，NULL=至今
    pub end_date: Option<NaiveDate>,
    pub remark: Option<String>,
    /// 证书条目是否公开展示：0否 1是
    pub is_public: Option<i32>,
    pub deleted: Option<i32>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
