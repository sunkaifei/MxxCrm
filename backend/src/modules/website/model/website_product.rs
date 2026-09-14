//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 网站展示产品 Model 层：请求 DTO 与列表 VO
//!

use rust_decimal::Decimal;
use crate::core::kit::global::{Deserialize, Serialize};

// ==================== 请求 DTO ====================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductListQuery {
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub page_num: Option<i64>,
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub page_size: Option<i64>,
    /// 按产品名称/关键词过滤
    pub keywords: Option<String>,
    /// 按上架状态过滤：1=上架 0=下架
    pub status: Option<i32>,
    /// 按栏目分类过滤（栏目管理中内容类型=产品的栏目 ID）
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub category_id: Option<i64>,
    /// 按推荐状态过滤：1=推荐 0=非推荐
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub is_recommend: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductAddItem {
    /// 产品库产品 ID
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub product_id: i64,
    /// 上架展示数量（上限为当前仓储库存，后端钳制）
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub quantity: i64,
    /// 展示的 SKU ID 列表（空 = 全部 SKU）
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_or_num_vec_to_i64_vec")]
    pub sku_ids: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductAddRequest {
    /// 从产品库选择的产品与数量（自动去重，已在清单内的跳过）
    #[serde(default)]
    pub items: Vec<WebsiteProductAddItem>,
    /// 主产品的相关产品 ID 列表（相关产品会一并加入上架清单）
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_or_num_vec_to_i64_vec")]
    pub related_product_ids: Vec<i64>,
    /// 主产品的相关文章 ID 列表
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_or_num_vec_to_i64_vec")]
    pub related_article_ids: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductSkusRequest {
    /// 清单行 ID
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub id: i64,
    /// 展示的 SKU ID 列表（空 = 全部 SKU）
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_or_num_vec_to_i64_vec")]
    pub sku_ids: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductSkuPricesRequest {
    /// 清单行 ID
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub id: i64,
    /// SKU 前台零售价映射：{ "skuId": 价格 }；仅作用于前台在线销售
    #[serde(default)]
    pub prices: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductSkuQuantitiesRequest {
    /// 清单行 ID
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub id: i64,
    /// SKU 前台销售库存映射：{ "skuId": 数量 }；仅作用于前台在线销售，钳制为不超过该 SKU 仓储库存
    #[serde(default)]
    pub quantities: std::collections::HashMap<String, i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductShelfRequest {
    /// 清单行 ID 列表（兼容数字/字符串数组）
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_or_num_vec_to_i64_vec")]
    pub ids: Vec<i64>,
    /// 目标状态：1=上架 0=下架
    pub status: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductQuantityRequest {
    /// 清单行 ID
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub id: i64,
    /// 展示数量（后端钳制为不超过当前仓储库存）
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub quantity: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductCategoryRequest {
    /// 清单行 ID
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub id: i64,
    /// 栏目管理中内容类型=产品(2)的栏目 ID；0=清除归属
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub category_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductSortRequest {
    /// 清单行 ID
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub id: i64,
    /// 目标排序值（越小越靠前）
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub sort: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductSeoRequest {
    /// 清单行 ID
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub id: i64,
    pub seo_title: Option<String>,
    pub seo_keywords: Option<String>,
    pub seo_description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductLimitBuyRequest {
    /// 清单行 ID
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub id: i64,
    /// 每人限购数量（0=不限）
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub limit_buy: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductScheduleRequest {
    /// 清单行 ID
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub id: i64,
    /// 定时上架时间（YYYY-MM-DD HH:MM:SS；空=清除）
    pub list_at: Option<String>,
    /// 定时下架时间
    pub unlist_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductRecommendRequest {
    /// 清单行 ID 列表
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_or_num_vec_to_i64_vec")]
    pub ids: Vec<i64>,
    /// 1=推荐 0=取消推荐
    pub is_recommend: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductRelatedRequest {
    /// 清单行 ID
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub id: i64,
    /// 相关产品 ID 列表
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_or_num_vec_to_i64_vec")]
    pub related_product_ids: Vec<i64>,
    /// 相关文章 ID 列表
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_or_num_vec_to_i64_vec")]
    pub related_article_ids: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteProductDeleteRequest {
    /// 清单行 ID 列表（兼容数字/字符串数组）
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_or_num_vec_to_i64_vec")]
    pub ids: Vec<i64>,
}

// ==================== 列表 VO ====================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WebsiteProductListVO {
    pub id: i64,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_image: Option<String>,
    pub sale_price: Option<Decimal>,
    /// 上架展示数量（存储值）
    pub quantity: Option<i64>,
    /// 当前仓储总库存（实时汇总）
    pub total_stock: Option<i64>,
    /// 生效展示数量 = min(quantity, total_stock)，库存减少时自动跟随下降
    pub effective_quantity: Option<i64>,
    /// 所属栏目分类（栏目管理中内容类型=产品的栏目）
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    /// 展示的 SKU ID 列表（空 = 全部 SKU）
    pub sku_ids: Option<Vec<i64>>,
    /// SKU 展示数量说明：None=全部 SKU
    pub sku_count: Option<i64>,
    /// SKU 前台零售价映射（{"skuId": 价格}；未设置的 SKU 沿用产品库价格）
    pub sku_prices: Option<serde_json::Value>,
    /// SKU 前台销售库存映射（{"skuId": 数量}；未设置的 SKU 沿用该 SKU 仓储库存）
    pub sku_quantities: Option<serde_json::Value>,
    /// 推荐标记：1=推荐
    pub is_recommend: Option<i32>,
    /// 相关产品 ID 列表
    pub related_product_ids: Option<Vec<i64>>,
    /// 相关文章 ID 列表
    pub related_article_ids: Option<Vec<i64>>,
    /// SEO 自定义标题/关键词/描述
    pub seo_title: Option<String>,
    pub seo_keywords: Option<String>,
    pub seo_description: Option<String>,
    /// 前台零售价/促销价/促销期（双价格体系：前台在线销售口径）
    pub retail_price: Option<Decimal>,
    pub promo_price: Option<Decimal>,
    pub promo_start: Option<String>,
    pub promo_end: Option<String>,
    pub sku_promo_prices: Option<serde_json::Value>,
    /// 每人限购（0=不限）
    pub limit_buy: Option<i32>,
    /// 库存预警阈值
    pub stock_warn_threshold: Option<i32>,
    /// 定时上架/下架时间
    pub list_at: Option<String>,
    pub unlist_at: Option<String>,
    /// 产品库自身分类名称（参考信息）
    pub product_category_name: Option<String>,
    pub is_active: Option<bool>,
    pub status: Option<i32>,
    pub sort: Option<i32>,
    pub create_time: Option<String>,
}
