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
#[sea_orm(table_name = "mxx_inventory_stock_snapshot")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 主键
    pub id: i64,
    /// 快照日期
    pub snapshot_date: Option<Date>,
    /// 仓库ID
    pub warehouse_id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品SKU
    pub product_sku: Option<String>,
    /// 库存数量
    pub quantity: Option<Decimal>,
    /// 可用数量
    pub available_quantity: Option<Decimal>,
    /// 冻结数量
    pub frozen_quantity: Option<Decimal>,
    /// 在途数量
    pub in_transit_quantity: Option<Decimal>,
    /// 加权平均成本
    pub avg_cost: Option<Decimal>,
    /// 库存总成本
    pub total_cost: Option<Decimal>,
    /// 删除标识（0未删除 1已删除）
    pub deleted: Option<i32>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
