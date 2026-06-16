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
#[sea_orm(table_name = "mxx_navigation")]
pub struct Model {
    /// 主键
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 网站id
    pub website_id: Option<i64>,
    /// 父id
    pub parent_id: Option<i64>,
    /// 导航名称
    pub name: Option<String>,
    /// 自定义url地址
    pub web_url: Option<String>,
    /// 数据 id
    pub value: Option<u32>,
    /// 数据类型（custom:自定义导航, article_class:文章分类, customview:自定义页面）
    pub data_type: Option<String>,
    /// 导航类型（header:顶部导航, footer:底部导航）
    pub nav_type: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 是否显示（0否，1是）
    pub is_show: Option<i32>,
    /// 是否新窗口打开（0否，1是）
    pub is_new_window_open: Option<i32>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
