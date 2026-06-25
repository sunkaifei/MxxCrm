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
use serde_json;

/// 产品SKU变体表（mxx_product_sku）
/// 存储产品的多规格变体，如不同颜色+尺寸组合的库存和价格
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_product_sku")]
pub struct Model {
    /// SKU-ID（主键）
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 所属产品ID
    pub product_id: i64,

    /// SKU编码（唯一，如 PROD-001-BLK-M）
    pub sku_code: Option<String>,

    /// 动态规格键值对，如 {"颜色": "红色", "尺寸": "S"}
    pub specs: Option<serde_json::Value>,

    /// 销售价（覆盖产品默认价格）
    pub price: Option<Decimal>,

    /// 成本价（覆盖产品默认成本价）
    pub cost_price: Option<Decimal>,

    /// 市场价/原价
    pub original_price: Option<Decimal>,

    /// 库存数量
    pub stock: Option<i32>,

    /// 重量（kg）
    pub weight: Option<Decimal>,

    /// 体积（m³）
    pub volume: Option<Decimal>,

    /// 变体图片URL
    pub image_url: Option<String>,

    /// 是否默认选中
    pub is_default: Option<bool>,

    /// 是否启用
    pub is_active: Option<bool>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
