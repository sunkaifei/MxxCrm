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

/// 产品规格定义表（mxx_product_spec）
/// 存储产品的规格名称，如颜色、尺寸等
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_product_spec")]
pub struct Model {
    /// 规格ID（主键）
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 所属产品ID
    pub product_id: i64,

    /// 规格名称（如颜色、尺寸）
    pub name: String,

    /// 排序号
    pub sort_order: Option<i32>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
