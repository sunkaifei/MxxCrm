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
#[sea_orm(table_name = "mxx_inventory_stocktake_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 主键
    pub id: i64,
    /// 盘点单ID
    pub stocktake_id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品SKU
    pub product_sku: Option<String>,
    /// 系统数量
    pub system_quantity: Option<Decimal>,
    /// 实盘数量
    pub actual_quantity: Option<Decimal>,
    /// 差异（实盘-系统）
    pub difference: Option<Decimal>,
    /// 差异类型：0=一致 1=盘盈 2=盘亏
    pub difference_type: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    /// 删除标识（0未删除 1已删除）
    pub deleted: Option<i32>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::stocktake::Entity",
        from = "Column::StocktakeId",
        to = "super::stocktake::Column::Id"
    )]
    Stocktake,
}

impl Related<super::stocktake::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Stocktake.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
