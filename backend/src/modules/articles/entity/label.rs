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
#[sea_orm(table_name = "mxx_label")]
pub struct Model {
    //标签ID
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 短网址
    pub short_url: Option<String>,
    /// 标签名称
    pub title: Option<String>,
    /// 话题描述
    pub content: Option<String>,
    /// 话题图片
    pub label_pic: Option<String>,
    /// 访问统计
    pub count_view: Option<i32>,
    /// 话题是否锁定 1 锁定 0 未锁定
    pub label_lock: Option<i32>,
    /// 标签使用数量
    pub count_topic: Option<i32>,
    /// 关注数
    pub count_follow: Option<i32>,
    /// 1为推荐
    pub isrecommend: Option<i32>,
    ///创建时间
    pub create_time: Option<DateTime>,
    /// 0未审核，1：正常显示；2：隐藏不显示
    pub status: Option<i32>,
    /// 是否删除：0否，1是
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

