//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use sea_orm::prelude::{DateTime, DateTimeWithTimeZone, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::core::r#enum::currency_code_enum::CurrencyCode;
use crate::modules::product::entity::{product, sku};
use crate::modules::product::entity::product::Entity as Product;
use crate::modules::product::entity::sku::Entity as ProductSku;
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use serde_json::Value as JsonValue;

/// SKU变体请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SkuRequest {
    /// SKU-ID（编辑时传入，新增时为0）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,

    /// SKU编码
    pub sku_code: Option<String>,

    /// 颜色/规格1
    pub color: Option<String>,

    /// 尺寸/规格2
    pub size: Option<String>,

    /// 动态规格JSON（前端 label 会映射为 {specs: label}）
    pub specs: Option<JsonValue>,

    /// 销售价
    pub price: Option<Decimal>,

    /// 成本价
    pub cost_price: Option<Decimal>,

    /// 市场价/原价
    pub original_price: Option<Decimal>,

    /// 库存数量
    pub stock: Option<i32>,

    /// 重量（kg）
    pub weight: Option<Decimal>,

    /// 体积（m³）
    pub volume: Option<Decimal>,

    /// 变体图片URL
    pub image_url: Option<String>,

    /// 是否默认选中
    pub is_default: Option<bool>,

    /// 是否启用
    pub is_active: Option<bool>,
}

/// 产品新增请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ProductSaveRequest {
    /// 产品编号
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
    /// 单位
    pub unit: Option<String>,
    /// 成本价
    pub cost_price: Option<Decimal>,
    /// 销售价
    pub sale_price: Option<Decimal>,
    /// 市场价/原价
    pub market_price: Option<Decimal>,
    /// 币种
    pub currency: Option<String>,
    /// 重量（kg）
    pub weight: Option<Decimal>,
    /// 尺寸
    pub dimensions: Option<String>,
    /// 产品描述
    pub description: Option<String>,
    /// 产品详情（富文本/HTML）
    pub detail: Option<String>,
    /// 主图URL
    pub image_url: Option<String>,
    /// 轮播图URL数组
    pub carousel_images: Option<Vec<String>>,
    /// 是否启用
    pub is_active: Option<bool>,
    /// 规格类型（single/multiple）
    pub spec_type: Option<String>,
    /// 关键字
    pub keywords: Option<String>,
    /// 库存数量（单规格模式）
    pub stock: Option<i32>,
    /// SKU变体列表
    pub skus: Option<Vec<SkuRequest>>,
}

impl From<ProductSaveRequest> for ProductSaveDTO {
    fn from(item: ProductSaveRequest) -> Self {
        ProductSaveDTO {
            id: None,
            product_no: item.product_no,
            name: item.name,
            category_id: item.category_id,
            template_id: item.template_id,
            sku: item.sku,
            barcode: item.barcode,
            unit: item.unit,
            cost_price: item.cost_price,
            sale_price: item.sale_price,
            market_price: item.market_price,
            currency: item.currency.and_then(|c| c.parse::<CurrencyCode>().ok()),
            weight: item.weight,
            dimensions: item.dimensions,
            description: item.description,
            detail: item.detail,
            image_url: item.image_url,
            carousel_images: item.carousel_images.map(JsonValue::from),
            is_active: item.is_active,
            spec_type: item.spec_type,
            keywords: item.keywords,
            stock: item.stock,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 产品更新请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ProductUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub product_no: Option<String>,
    pub name: Option<String>,
    pub category_id: Option<i64>,
    pub template_id: Option<i64>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub unit: Option<String>,
    pub cost_price: Option<Decimal>,
    pub sale_price: Option<Decimal>,
    pub market_price: Option<Decimal>,
    pub currency: Option<String>,
    pub weight: Option<Decimal>,
    pub dimensions: Option<String>,
    pub description: Option<String>,
    pub detail: Option<String>,
    pub image_url: Option<String>,
    pub carousel_images: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub spec_type: Option<String>,
    pub keywords: Option<String>,
    pub stock: Option<i32>,
    pub skus: Option<Vec<SkuRequest>>,
}

impl From<ProductUpdateRequest> for ProductSaveDTO {
    fn from(item: ProductUpdateRequest) -> Self {
        ProductSaveDTO {
            id: item.id,
            product_no: item.product_no,
            name: item.name,
            category_id: item.category_id,
            template_id: item.template_id,
            sku: item.sku,
            barcode: item.barcode,
            unit: item.unit,
            cost_price: item.cost_price,
            sale_price: item.sale_price,
            market_price: item.market_price,
            currency: item.currency.and_then(|c| c.parse::<CurrencyCode>().ok()),
            weight: item.weight,
            dimensions: item.dimensions,
            description: item.description,
            detail: item.detail,
            image_url: item.image_url,
            carousel_images: item.carousel_images.map(JsonValue::from),
            is_active: item.is_active,
            spec_type: item.spec_type,
            keywords: item.keywords,
            stock: item.stock,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 产品保存DTO（内部使用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ProductSaveDTO {
    pub id: Option<i64>,
    pub product_no: Option<String>,
    pub name: Option<String>,
    pub category_id: Option<i64>,
    pub template_id: Option<i64>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub unit: Option<String>,
    pub cost_price: Option<Decimal>,
    pub sale_price: Option<Decimal>,
    pub market_price: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub weight: Option<Decimal>,
    pub dimensions: Option<String>,
    pub description: Option<String>,
    pub detail: Option<String>,
    pub image_url: Option<String>,
    pub carousel_images: Option<JsonValue>,
    pub is_active: Option<bool>,
    pub spec_type: Option<String>,
    pub keywords: Option<String>,
    pub stock: Option<i32>,
    pub deleted: Option<i32>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTimeWithTimeZone>,
    pub updated_by: Option<i64>,
    pub update_time: Option<DateTimeWithTimeZone>,
}

/// SKU变体VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SkuVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub product_id: Option<i64>,
    pub sku_code: Option<String>,
    pub color: Option<String>,
    pub size: Option<String>,
    /// 动态规格JSON
    pub specs: Option<JsonValue>,
    /// 规格组合标签
    pub label: Option<String>,
    pub price: Option<Decimal>,
    pub cost_price: Option<Decimal>,
    pub original_price: Option<Decimal>,
    pub stock: Option<i32>,
    pub weight: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub image_url: Option<String>,
    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
}

impl From<sku::Model> for SkuVO {
    fn from(item: sku::Model) -> Self {
        SkuVO {
            id: Some(item.id),
            product_id: Some(item.product_id),
            sku_code: item.sku_code,
            color: item.color,
            size: item.size,
            specs: item.specs.clone(),
            label: item.specs.as_ref().and_then(|v| v.as_str().map(|s| s.to_string())),
            price: item.price,
            cost_price: item.cost_price,
            original_price: item.original_price,
            stock: item.stock,
            weight: item.weight,
            volume: item.volume,
            image_url: item.image_url,
            is_default: item.is_default,
            is_active: item.is_active,
        }
    }
}

/// 产品详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ProductDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub product_no: Option<String>,
    pub name: Option<String>,
    pub category_id: Option<i64>,
    pub template_id: Option<i64>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub unit: Option<String>,
    pub cost_price: Option<Decimal>,
    pub sale_price: Option<Decimal>,
    pub market_price: Option<Decimal>,
    pub currency: Option<CurrencyCode>,
    pub weight: Option<Decimal>,
    pub dimensions: Option<String>,
    pub description: Option<String>,
    pub detail: Option<String>,
    pub image_url: Option<String>,
    pub carousel_images: Option<JsonValue>,
    pub is_active: Option<bool>,
    pub spec_type: Option<String>,
    pub keywords: Option<String>,
    pub stock: Option<i32>,
    pub create_time: Option<DateTimeWithTimeZone>,
    /// SKU变体列表
    pub skus: Option<Vec<SkuVO>>,
}

impl From<product::Model> for ProductDetailVO {
    fn from(item: product::Model) -> Self {
        ProductDetailVO {
            id: Some(item.id),
            product_no: item.product_no,
            name: item.name,
            category_id: item.category_id,
            template_id: item.template_id,
            sku: item.sku,
            barcode: item.barcode,
            unit: item.unit,
            cost_price: item.cost_price,
            sale_price: item.sale_price,
            market_price: item.market_price,
            currency: item.currency,
            weight: item.weight,
            dimensions: item.dimensions,
            description: item.description,
            detail: item.detail,
            image_url: item.image_url,
            carousel_images: item.carousel_images.clone(),
            is_active: item.is_active,
            spec_type: item.spec_type,
            keywords: item.keywords,
            stock: item.stock,
            create_time: item.create_time,
            skus: None,
        }
    }
}

/// 产品列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ProductListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub product_no: Option<String>,
    pub name: Option<String>,
    pub category_id: Option<i64>,
    pub template_id: Option<i64>,
    pub sku: Option<String>,
    pub unit: Option<String>,
    pub sale_price: Option<Decimal>,
    pub image_url: Option<String>,
    pub is_active: Option<bool>,
    pub create_time: Option<DateTimeWithTimeZone>,
}

impl From<product::Model> for ProductListVO {
    fn from(item: product::Model) -> Self {
        ProductListVO {
            id: Some(item.id),
            product_no: item.product_no,
            name: item.name,
            category_id: item.category_id,
            template_id: item.template_id,
            sku: item.sku,
            unit: item.unit,
            sale_price: item.sale_price,
            image_url: item.image_url,
            is_active: item.is_active,
            create_time: item.create_time,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub category_id: Option<i64>,
    pub is_active: Option<bool>,
}

pub struct ProductModel;

impl ProductModel {
    pub async fn insert<C>(db: &C, req: &ProductSaveDTO) -> Result<i64, DbErr> where C: ConnectionTrait {
        let now = chrono::Utc::now().fixed_offset();
        
        let product_no = match &req.product_no {
            Some(no) if !no.trim().is_empty() => no.clone(),
            _ => {
                let timestamp = now.timestamp_millis();
                format!("P{}", timestamp)
            }
        };
        
        let payload = product::ActiveModel {
            product_no: Set(Some(product_no)),
            name: Set(req.name.clone()),
            category_id: Set(req.category_id),
            template_id: Set(req.template_id),
            sku: Set(req.sku.clone()),
            barcode: Set(req.barcode.clone()),
            unit: Set(req.unit.clone()),
            cost_price: Set(req.cost_price.clone()),
            sale_price: Set(req.sale_price.clone()),
            market_price: Set(req.market_price.clone()),
            currency: Set(req.currency.clone()),
            weight: Set(req.weight.clone()),
            dimensions: Set(req.dimensions.clone()),
            description: Set(req.description.clone()),
            detail: Set(req.detail.clone()),
            image_url: Set(req.image_url.clone()),
            carousel_images: Set(req.carousel_images.clone()),
            is_active: Set(req.is_active),
            spec_type: Set(req.spec_type.clone()),
            keywords: Set(req.keywords.clone()),
            stock: Set(req.stock),
            created_by: Set(req.created_by),
            create_time: Set(Some(now)),
            updated_by: Set(req.updated_by),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        Product::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Product::update_many()
            .set(product::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(product::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id<C>(db: &C, id: &Option<i64>, req: &ProductSaveDTO) -> Result<i64, DbErr> where C: ConnectionTrait {
        let payload = product::ActiveModel {
            product_no: Set(req.product_no.clone()),
            name: Set(req.name.clone()),
            category_id: Set(req.category_id),
            template_id: Set(req.template_id),
            sku: Set(req.sku.clone()),
            barcode: Set(req.barcode.clone()),
            unit: Set(req.unit.clone()),
            cost_price: Set(req.cost_price.clone()),
            sale_price: Set(req.sale_price.clone()),
            market_price: Set(req.market_price.clone()),
            currency: Set(req.currency.clone()),
            weight: Set(req.weight.clone()),
            dimensions: Set(req.dimensions.clone()),
            description: Set(req.description.clone()),
            detail: Set(req.detail.clone()),
            image_url: Set(req.image_url.clone()),
            carousel_images: Set(req.carousel_images.clone()),
            is_active: Set(req.is_active),
            spec_type: Set(req.spec_type.clone()),
            keywords: Set(req.keywords.clone()),
            stock: Set(req.stock),
            updated_by: Set(req.updated_by),
            update_time: Set(Some(chrono::Utc::now().fixed_offset())),
            ..Default::default()
        };

        let update_result: UpdateResult = Product::update_many()
            .set(payload)
            .filter(product::Column::Id.eq(id.unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<product::Model>, DbErr> {
        Product::find_by_id(id)
            .filter(product::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn exists_by_name<C>(db: &C, name: &str, exclude_id: Option<i64>) -> Result<bool, DbErr> where C: ConnectionTrait {
        let mut query = Product::find()
            .filter(product::Column::Name.eq(name))
            .filter(product::Column::Deleted.eq(0));
        if let Some(id) = exclude_id {
            query = query.filter(product::Column::Id.ne(id));
        }
        query.count(db).await.map(|c| c > 0)
    }

    pub async fn find_skus_by_product_id(db: &DbConn, product_id: i64) -> Result<Vec<sku::Model>, DbErr> {
        ProductSku::find()
            .filter(sku::Column::ProductId.eq(product_id))
            .filter(sku::Column::IsActive.eq(true))
            .all(db)
            .await
    }

    pub async fn batch_save_skus<C>(db: &C, product_id: i64, skus: &[SkuRequest]) -> Result<(), DbErr> where C: ConnectionTrait {
        // 1. 删除旧的SKU（物理删除，然后重新插入）
        ProductSku::delete_many()
            .filter(sku::Column::ProductId.eq(product_id))
            .exec(db)
            .await?;

        // 2. 插入新的SKU
        let now = chrono::Utc::now().fixed_offset();
        let timestamp = now.timestamp_millis();
        for (idx, s) in skus.iter().enumerate() {
            // 前端 label 映射为 specs JSON 字符串
            let specs = s.specs.clone().or_else(|| {
                None
            });
            
            let sku_code = match &s.sku_code {
                Some(code) if !code.trim().is_empty() => code.clone(),
                _ => format!("SKU-{}-{}", product_id, timestamp + idx as i64),
            };
            
            let payload = sku::ActiveModel {
                product_id: Set(product_id),
                sku_code: Set(Some(sku_code)),
                color: Set(s.color.clone()),
                size: Set(s.size.clone()),
                specs: Set(specs),
                price: Set(s.price.clone()),
                cost_price: Set(s.cost_price.clone()),
                original_price: Set(s.original_price.clone()),
                stock: Set(s.stock),
                weight: Set(s.weight.clone()),
                volume: Set(s.volume.clone()),
                image_url: Set(s.image_url.clone()),
                is_default: Set(s.is_default),
                is_active: Set(s.is_active),
                create_time: Set(Some(now)),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            ProductSku::insert(payload).exec(db).await?;
        }

        Ok(())
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        category_id: Option<i64>,
        is_active: Option<bool>,
    ) -> Result<(Vec<product::Model>, i64), DbErr> {
        let mut query = Product::find()
            .filter(product::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(product::Column::Name.contains(k.clone()))
                    .add(product::Column::ProductNo.contains(k.clone()))
                    .add(product::Column::Sku.contains(k))
            );
        }
        if let Some(c) = category_id {
            query = query.filter(product::Column::CategoryId.eq(c));
        }
        if let Some(a) = is_active {
            query = query.filter(product::Column::IsActive.eq(a));
        }

        let paginator = query.order_by_desc(product::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    pub async fn select_count(
        db: &DbConn,
        keywords: Option<String>,
        category_id: Option<i64>,
        is_active: Option<bool>,
    ) -> Result<i64, DbErr> {
        let mut query = Product::find()
            .filter(product::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(product::Column::Name.contains(k.clone()))
                    .add(product::Column::ProductNo.contains(k.clone()))
                    .add(product::Column::Sku.contains(k))
            );
        }
        if let Some(c) = category_id {
            query = query.filter(product::Column::CategoryId.eq(c));
        }
        if let Some(a) = is_active {
            query = query.filter(product::Column::IsActive.eq(a));
        }

        query.count(db).await.map(|c| c as i64)
    }
}
