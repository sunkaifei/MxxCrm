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
#[sea_orm(table_name = "mxx_inventory_batch")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 主键
    pub id: i64,
    /// 批次号
    pub batch_no: Option<String>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品SKU
    pub product_sku: Option<String>,
    /// 仓库ID
    pub warehouse_id: Option<i64>,
    /// 生产日期
    pub production_date: Option<DateTime>,
    /// 有效期
    pub expiry_date: Option<DateTime>,
    /// 初始数量
    pub initial_quantity: Option<Decimal>,
    /// 当前数量
    pub current_quantity: Option<Decimal>,
    /// 状态：0=正常 1=已用完 2=已过期
    pub status: Option<i32>,
    /// 供应商ID
    pub supplier_id: Option<i64>,
    /// 入库单ID
    pub inbound_id: Option<i64>,
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
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
