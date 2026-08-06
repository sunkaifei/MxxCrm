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
#[sea_orm(table_name = "mxx_inventory_inbound_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 主键
    pub id: i64,
    /// 入库单ID
    pub inbound_id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品SKU
    pub product_sku: Option<String>,
    /// 入库库位ID
    pub warehouse_area_id: Option<i64>,
    /// 入库数量
    pub quantity: Option<Decimal>,
    /// 入库单价
    pub unit_price: Option<Decimal>,
    /// 入库金额
    pub amount: Option<Decimal>,
    /// 批次号
    pub batch_no: Option<String>,
    /// 生产日期
    pub production_date: Option<DateTime>,
    /// 有效期
    pub expiry_date: Option<DateTime>,
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
        belongs_to = "super::inbound::Entity",
        from = "Column::InboundId",
        to = "super::inbound::Column::Id"
    )]
    Inbound,
}

impl Related<super::inbound::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Inbound.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}