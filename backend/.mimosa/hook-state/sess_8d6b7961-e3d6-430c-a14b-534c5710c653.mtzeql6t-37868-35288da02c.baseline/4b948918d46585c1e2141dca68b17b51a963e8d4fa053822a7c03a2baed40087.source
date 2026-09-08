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
#[sea_orm(table_name = "mxx_inventory_quality_check")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 主键
    pub id: i64,
    /// 质检单号（QC + yyyyMMdd + 流水号）
    pub check_no: Option<String>,
    /// 关联入库单ID
    pub inbound_id: Option<i64>,
    /// 仓库ID
    pub warehouse_id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品SKU
    pub product_sku: Option<String>,
    /// 质检数量
    pub quantity: Option<Decimal>,
    /// 合格数量
    pub qualified_quantity: Option<Decimal>,
    /// 不合格数量
    pub unqualified_quantity: Option<Decimal>,
    /// 质检结果：0=待检 1=合格 2=不合格 3=部分合格
    pub check_result: Option<i32>,
    /// 质检人ID
    pub checker: Option<i64>,
    /// 质检时间
    pub check_time: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
    /// 状态：0=草稿 1=已质检
    pub status: Option<i32>,
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
