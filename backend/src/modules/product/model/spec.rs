//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::utils::string_utils::serialize_option_u64_to_string;

/// 规格值保存项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SpecValueSaveItem {
    /// 规格值（如红色、S、M、L）
    pub value: String,
    /// 排序号
    pub sort_order: Option<i32>,
}

/// 规格保存项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SpecSaveItem {
    /// 规格名称（如颜色、尺寸）
    pub name: String,
    /// 排序号
    pub sort_order: Option<i32>,
    /// 规格值列表
    pub values: Vec<SpecValueSaveItem>,
}

/// 规格批量保存请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SpecBatchSaveRequest {
    /// 产品ID
    pub product_id: i64,
    /// 规格列表
    pub specs: Vec<SpecSaveItem>,
}

/// 规格值VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SpecValueVO {
    /// 规格值ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 规格值
    pub value: Option<String>,
    /// 排序号
    pub sort_order: Option<i32>,
}

/// 规格VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SpecVO {
    /// 规格ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 规格名称
    pub name: Option<String>,
    /// 排序号
    pub sort_order: Option<i32>,
    /// 规格值列表
    pub values: Vec<SpecValueVO>,
}

/// 生成的SKU项（临时，未持久化）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct GeneratedSkuVO {
    /// 规格组合描述，如 "红色/S"
    pub label: String,
    /// 动态规格键值对
    pub specs: serde_json::Value,
    /// SKU编码（空，待编辑）
    pub sku_code: Option<String>,
    /// 销售价
    pub price: f64,
    /// 库存
    pub stock: i32,
}

/// 规格及SKU组合VO（get_specs 返回值）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SpecGroupVO {
    /// 规格定义列表
    pub specs: Vec<SpecVO>,
    /// 已保存的SKU列表
    pub skus: Vec<super::product::SkuVO>,
}

/// SKU批量保存请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SkuBatchSaveRequest {
    /// 产品ID
    pub product_id: i64,
    /// SKU列表
    pub skus: Option<Vec<super::product::SkuRequest>>,
}
