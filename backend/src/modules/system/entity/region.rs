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
#[sea_orm(table_name = "mxx_system_region")]
pub struct Model {
    /// 主键
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 短网址
    pub short_url: Option<String>,
    /// 行政区域父ID，例如区县的pid指向市，市的pid指向省，省的pid则是0
    pub parent_id: Option<i64>,
    /// 地区别名
    pub title: Option<String>,
    /// 行政区域名称
    pub region_name: Option<String>,
    /// 行政区域类型，如如1则是省， 如果是2则是市，如果是3则是区县
    pub region_type: Option<i32>,
    /// 行政区域编码
    pub region_code: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 显示状态
    pub status: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}