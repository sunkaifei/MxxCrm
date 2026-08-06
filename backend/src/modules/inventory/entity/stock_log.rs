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
#[sea_orm(table_name = "mxx_inventory_stock_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 主键
    pub id: i64,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 仓库ID
    pub warehouse_id: Option<i64>,
    /// 库位ID
    pub warehouse_area_id: Option<i64>,
    /// 变动类型：inbound/outbound/transfer_in/transfer_out/check/freeze/unfreeze/setup/adjust
    pub change_type: Option<String>,
    /// 业务类型：purchase_in/return_in/sale_out/initial/check_surplus/check_shortage/freeze/unfreeze/adjust
    pub biz_type: Option<String>,
    /// 关联业务单据ID
    pub biz_id: Option<i64>,
    /// 关联业务单据号
    pub biz_no: Option<String>,
    /// 变动前数量
    pub quantity_before: Option<Decimal>,
    /// 变动数量（正=增加，负=减少）
    pub change_quantity: Option<Decimal>,
    /// 变动后数量
    pub quantity_after: Option<Decimal>,
    /// 操作人
    pub operator_id: Option<i64>,
    /// 备注
    pub remark: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}