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
#[sea_orm(table_name = "mxx_inventory_outbound_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 主键
    pub id: i64,
    /// 出库单ID
    pub outbound_id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品SKU
    pub product_sku: Option<String>,
    /// 出库库位ID
    pub warehouse_area_id: Option<i64>,
    /// 出库数量
    pub quantity: Option<Decimal>,
    /// 批次号
    pub batch_no: Option<String>,
    /// 备注
    pub remark: Option<String>,
    /// 删除标识（0未删除 1已删除）
    pub deleted: Option<i32>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::outbound::Entity",
        from = "Column::OutboundId",
        to = "super::outbound::Column::Id"
    )]
    Outbound,
}

impl Related<super::outbound::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Outbound.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}