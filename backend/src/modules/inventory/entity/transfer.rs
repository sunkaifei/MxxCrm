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
#[sea_orm(table_name = "mxx_inventory_transfer")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 主键
    pub id: i64,
    /// 调拨单号（DB+yyyyMMdd+流水号）
    pub transfer_no: Option<String>,
    /// 源仓库ID
    pub from_warehouse_id: Option<i64>,
    /// 目标仓库ID
    pub to_warehouse_id: Option<i64>,
    /// 状态：0=草稿 1=已出库 2=已入库 3=已完成 4=已取消
    pub status: Option<i32>,
    /// 总数量
    pub total_quantity: Option<Decimal>,
    /// 备注
    pub remark: Option<String>,
    /// 删除标识（0未删除 1已删除）
    pub deleted: Option<i32>,
    /// 创建人
    pub created_by: Option<i64>,
    /// 更新人
    pub updated_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::transfer_item::Entity")]
    TransferItem,
}

impl Related<super::transfer_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TransferItem.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
