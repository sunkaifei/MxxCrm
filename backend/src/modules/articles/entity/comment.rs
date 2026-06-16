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


#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_article_comment")]
pub struct Model {
    // 帖子分类ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    ///上级评论ID
    pub refer_id: Option<i64>,
    ///文章ID
    pub article_id: Option<i64>,
    ///用户ID
    pub user_id: Option<i64>,
    ///回复内容
    pub content: Option<String>,
    ///被评论数量
    pub count_comment: Option<i32>,
    ///支持数
    pub count_digg: Option<i32>,
    ///反对次数
    pub count_burys: Option<i32>,
    ///0公开1不公开（仅自己和发帖者可看）
    pub ispublic: Option<i32>,
    ///楼层数量
    pub storey: Option<i32>,
    ///创建时间
    pub create_time: Option<DateTime>,
    ///更新时间
    pub update_time: Option<DateTime>,
    ///0未审核，1审核，2未通过
    pub status: Option<i32>,
    ///'0不删除1删除
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
