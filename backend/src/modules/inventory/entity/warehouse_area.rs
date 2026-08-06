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
#[sea_orm(table_name = "mxx_inventory_warehouse_area")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 主键
    pub id: i64,
    /// 仓库ID
    pub warehouse_id: Option<i64>,
    /// 库位编码
    pub area_code: Option<String>,
    /// 库位名称
    pub area_name: Option<String>,
    /// 库位类型：storage/picking/return/quality
    pub area_type: Option<String>,
    /// 状态：0=正常 1=停用
    pub status: Option<i32>,
    /// 排序
    pub sort_order: Option<i32>,
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
