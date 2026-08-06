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
#[sea_orm(table_name = "mxx_inventory_stock_freeze")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 主键
    pub id: i64,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 仓库ID
    pub warehouse_id: Option<i64>,
    /// 冻结数量
    pub freeze_quantity: Option<Decimal>,
    /// 冻结原因
    pub reason: Option<String>,
    /// 状态：0=冻结中 1=已解冻
    pub status: Option<i32>,
    /// 冻结人
    pub freeze_by: Option<i64>,
    /// 冻结时间
    pub freeze_time: Option<DateTime>,
    /// 解冻人
    pub unfreeze_by: Option<i64>,
    /// 解冻时间
    pub unfreeze_time: Option<DateTime>,
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
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}