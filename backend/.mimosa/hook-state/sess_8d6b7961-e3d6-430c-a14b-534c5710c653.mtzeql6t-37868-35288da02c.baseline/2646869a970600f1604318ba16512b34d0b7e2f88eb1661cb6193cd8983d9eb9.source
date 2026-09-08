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
#[sea_orm(table_name = "mxx_template_category")]
pub struct Model {
    // 分类ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    // 父分类ID
    pub parent_id: Option<i64>,
    // 分类名称
    pub name: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 添加时间
    pub create_time: Option<DateTime>,
    // 更新时间
    pub update_time: Option<DateTime>,
    // 导航是否显示
    pub is_show: Option<i32>,
    // 审核状态，0未审核，1已审核
    pub status: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
