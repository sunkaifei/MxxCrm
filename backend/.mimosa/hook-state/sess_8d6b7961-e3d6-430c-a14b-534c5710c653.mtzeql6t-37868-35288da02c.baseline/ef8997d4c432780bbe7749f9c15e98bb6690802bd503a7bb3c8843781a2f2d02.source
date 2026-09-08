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

/// 网站Banner实体
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_website_banner")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 标题
    pub title: Option<String>,
    /// 图片地址
    pub image_url: Option<String>,
    /// 链接地址
    pub link_url: Option<String>,
    /// 替换文本
    pub alt_text: Option<String>,
    /// 显示位置
    pub position: Option<String>,
    /// 打开方式：_self, _blank
    pub target: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 开始时间
    pub start_time: Option<DateTime>,
    /// 结束时间
    pub end_time: Option<DateTime>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 删除标记：0未删除，1已删除
    pub deleted: Option<i32>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
