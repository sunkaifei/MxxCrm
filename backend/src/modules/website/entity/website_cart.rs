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

/// 购物车（mxx_website_cart）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_website_cart")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 用户ID
    pub user_id: i64,

    /// 产品ID
    pub product_id: i64,

    /// SKU ID
    pub sku_id: Option<i64>,

    /// 产品名称（冗余快照）
    pub product_name: Option<String>,

    /// 产品图片（冗余快照）
    pub product_image: Option<String>,

    /// SKU 编码
    pub sku_code: Option<String>,

    /// SKU 规格（JSON）
    pub sku_specs: Option<String>,

    /// 单价
    #[serde(default)]
    pub price: Decimal,

    /// 数量
    #[serde(default)]
    pub quantity: i32,

    /// 是否选中：0未选 1已选
    #[serde(default)]
    pub selected: Option<i32>,

    /// 站点ID
    pub website_id: Option<i64>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
