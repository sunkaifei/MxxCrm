//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::inventory::entity::stock::Model;
use serde::{Deserialize, Serialize};
use sea_orm::prelude::{DateTime, Decimal};

/// 库存列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InventoryListQuery {
    /// 当前页码
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    /// 每页条数
    pub page_size: Option<i64>,
    /// 产品名称（模糊搜索）
    pub product_name: Option<String>,
    /// 仓库ID
    pub warehouse_id: Option<i64>,
    /// 是否低库存查询
    pub low_stock: Option<bool>,
}

/// 库存列表响应项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InventoryListVO {
    /// 库存ID
    pub id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品编码
    pub product_code: Option<String>,
    /// 仓库ID
    pub warehouse_id: Option<i64>,
    /// 仓库名称
    pub warehouse_name: Option<String>,
    /// 库存数量
    pub quantity: Option<Decimal>,
    /// 预留数量
    pub reserved_quantity: Option<Decimal>,
    /// 可用数量
    pub available_quantity: Option<Decimal>,
    /// 最后更新时间
    pub update_time: Option<String>,
}

/// 库存列表分页响应
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InventoryListData {
    /// 数据总数
    pub total: i64,
    /// 库存列表
    pub items: Vec<InventoryListVO>,
}

/// 库存详情响应项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InventoryDetailVO {
    /// 库存ID
    pub id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品编码
    pub product_code: Option<String>,
    /// 产品规格
    pub spec: Option<String>,
    /// 产品单位
    pub unit: Option<String>,
    /// 仓库ID
    pub warehouse_id: Option<i64>,
    /// 仓库名称
    pub warehouse_name: Option<String>,
    /// 仓库编码
    pub warehouse_code: Option<String>,
    /// 库存数量
    pub quantity: Option<Decimal>,
    /// 预留数量
    pub reserved_quantity: Option<Decimal>,
    /// 可用数量
    pub available_quantity: Option<Decimal>,
    /// 最后更新时间
    pub update_time: Option<String>,
}
