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
#[sea_orm(table_name = "mxx_inventory_stocktake")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 主键
    pub id: i64,
    /// 盘点单号（PD+yyyyMMdd+流水号）
    pub stocktake_no: Option<String>,
    /// 盘点仓库ID
    pub warehouse_id: Option<i64>,
    /// 盘点类型：full/partial
    pub stocktake_type: Option<String>,
    /// 状态：0=草稿 1=盘点中 2=已完成 3=已取消
    pub status: Option<i32>,
    /// 总明细数
    pub total_items: Option<i32>,
    /// 盘盈条数
    pub surplus_count: Option<i32>,
    /// 盘亏条数
    pub shortage_count: Option<i32>,
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
    #[sea_orm(has_many = "super::stocktake_item::Entity")]
    StocktakeItem,
}

impl Related<super::stocktake_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StocktakeItem.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
