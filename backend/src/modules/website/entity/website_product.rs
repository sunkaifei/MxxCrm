//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 网站展示产品实体：前台"产品中心"上架清单（引用产品库，支持上架/下架与排序）
//!

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mxx_website_product")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 站点 ID（单站模式下为默认站点）
    pub website_id: i64,
    /// 产品库产品 ID（mxx_product.id）
    pub product_id: i64,
    /// 上架状态：1=上架（前台展示） 0=下架（仅保留清单）
    pub status: Option<i32>,
    /// 展示排序（越小越靠前）
    pub sort: Option<i32>,
    /// 上架展示数量（防超卖：展示口径钳制为不超过当前仓储库存）
    pub quantity: Option<i32>,
    /// 所属栏目分类（栏目管理中内容类型=产品(2)的栏目 ID）
    pub category_id: Option<i64>,
    /// 展示的 SKU ID 列表（逗号分隔；空/NULL = 显示全部 SKU）
    pub sku_ids: Option<String>,
    /// SKU 前台零售价（JSON：{"<skuId>": 价格}；仅作用于前台在线销售，不写回产品库）
    pub sku_prices: Option<String>,
    /// 推荐标记：1=推荐（前台推荐位可取）
    pub is_recommend: Option<i32>,
    /// 相关产品 ID 列表（逗号分隔的 mxx_product.id）
    pub related_product_ids: Option<String>,
    /// 相关文章 ID 列表（逗号分隔的文章 ID）
    pub related_article_ids: Option<String>,
    /// SEO 自定义标题（前台详情 head；空=回退产品名）
    pub seo_title: Option<String>,
    /// SEO 关键词
    pub seo_keywords: Option<String>,
    /// SEO 描述
    pub seo_description: Option<String>,
    /// 前台零售价（单规格/未单独定价 SKU 的默认；双价格体系隔离，不写回产品库）
    pub retail_price: Option<rust_decimal::Decimal>,
    /// 前台促销价
    pub promo_price: Option<rust_decimal::Decimal>,
    /// 促销开始时间
    pub promo_start: Option<chrono::NaiveDateTime>,
    /// 促销结束时间
    pub promo_end: Option<chrono::NaiveDateTime>,
    /// 多规格促销价 JSON：{"skuId": {"price": 促销价, "start": 起, "end": 止}}
    pub sku_promo_prices: Option<String>,
    /// 每人限购数量（0=不限）
    pub limit_buy: Option<i32>,
    /// 库存预警阈值（生效展示数量 ≤ 阈值时预警；NULL=不预警）
    pub stock_warn_threshold: Option<i32>,
    /// 定时上架时间
    pub list_at: Option<chrono::NaiveDateTime>,
    /// 定时下架时间
    pub unlist_at: Option<chrono::NaiveDateTime>,
    pub remark: Option<String>,
    pub deleted: Option<i32>,
    pub create_time: Option<chrono::NaiveDateTime>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
