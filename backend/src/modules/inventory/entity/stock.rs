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
#[sea_orm(table_name = "mxx_inventory_stock")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 库存ID
    pub id: i64,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 仓库ID
    pub warehouse_id: Option<i64>,
    /// SKU规格ID
    pub sku_id: Option<i64>,
    /// 库存数量
    pub quantity: Option<Decimal>,
    /// 预留数量
    pub reserved_quantity: Option<Decimal>,
    /// 可用数量
    pub available_quantity: Option<Decimal>,
    /// 在途数量（调拨出库后未入库）
    pub in_transit_quantity: Option<Decimal>,
    /// 冻结数量
    pub frozen_quantity: Option<Decimal>,
    /// 最低库存警戒线
    pub alert_min_quantity: Option<Decimal>,
    /// 最高库存警戒线
    pub alert_max_quantity: Option<Decimal>,
    /// 加权平均成本
    pub avg_cost: Option<Decimal>,
    /// 最后入库成本
    pub last_in_cost: Option<Decimal>,
    /// 库存总成本
    pub total_cost: Option<Decimal>,
    /// 最后入库时间
    pub last_inbound_time: Option<DateTime>,
    /// 最后出库时间
    pub last_outbound_time: Option<DateTime>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除标识（0未删除 1已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
