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
use crate::core::r#enum::currency_code_enum::CurrencyCode;

/// 产品主表（mxx_product_main）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_product_main")]
pub struct Model {
    /// 产品ID（主键）
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 产品编号（唯一）
    pub product_no: Option<String>,

    /// 产品名称
    pub name: Option<String>,

    /// 分类ID
    pub category_id: Option<i64>,

    /// SKU模板ID
    pub template_id: Option<i64>,

    /// 默认SKU编码
    pub sku: Option<String>,

    /// 条码
    pub barcode: Option<String>,

    /// 单位（如件/箱/个）
    pub unit: Option<String>,

    /// 成本价
    pub cost_price: Option<Decimal>,

    /// 销售价
    pub sale_price: Option<Decimal>,

    /// 币种
    pub currency: Option<CurrencyCode>,

    /// 重量（kg）
    pub weight: Option<Decimal>,

    /// 尺寸（如 30x20x10cm）
    pub dimensions: Option<String>,

    /// 产品描述/介绍
    pub description: Option<String>,

    /// 产品详情（富文本/HTML）
    pub detail: Option<String>,

    /// 主图URL
    pub image_url: Option<String>,

    /// 轮播图URL数组（JSON）
    pub carousel_images: Option<serde_json::Value>,

    /// 是否启用
    pub is_active: Option<bool>,

    /// 规格类型（single-单规格，multiple-多规格）
    pub spec_type: Option<String>,

    /// 关键字
    pub keywords: Option<String>,

    /// 市场价/原价
    pub market_price: Option<Decimal>,

    /// 库存数量（单规格模式）
    pub stock: Option<i32>,

    /// 创建人ID
    pub created_by: Option<i64>,

    /// 创建时间
    pub create_time: Option<DateTimeWithTimeZone>,

    /// 更新人ID
    pub updated_by: Option<i64>,

    /// 更新时间
    pub update_time: Option<DateTimeWithTimeZone>,

    /// 软删除标识（0-未删除，1-已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
