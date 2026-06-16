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

/// 网站链接数据模型
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_website_links")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 网站id
    pub website_id: Option<i64>,
    /// 链接类型：0文字链接，1logo链接
    #[serde(default)]
    pub link_type: Option<i32>,
    /// 网站名称
    #[serde(default)]
    pub link_name: Option<String>,
    /// 网站地址
    #[serde(default)]
    pub link_url: Option<String>,
    /// 网站logo地址
    #[serde(default)]
    pub link_logo: Option<String>,
    /// 显示状态：0不显示，1显示
    #[serde(default)]
    pub status: Option<i32>,
    /// 排序序号
    #[serde(default)]
    pub sort: Option<i32>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 删除标记：0删除，1正常
    #[serde(default)]
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}