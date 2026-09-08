//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_article_revision")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub article_id: i64,
    pub revision_no: i32,
    pub title: Option<String>,
    pub short_title: Option<String>,
    pub title_image: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub snapshot: Option<String>,
    pub editor_id: Option<i64>,
    pub editor_name: Option<String>,
    pub edit_remark: Option<String>,
    pub create_time: Option<DateTime>,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
