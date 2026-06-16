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
use sea_orm::prelude::{DateTime, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::product::entity::{product, product::Entity as Product};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ProductSaveRequest {
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub category_id: Option<i64>,
    pub brand: Option<String>,
    pub origin: Option<String>,
    pub material: Option<String>,
    pub weight: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub purchase_price: Option<Decimal>,
    pub sale_price: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub min_stock: Option<i32>,
    pub max_stock: Option<i32>,
    pub description: Option<String>,
    pub images: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

impl From<ProductSaveRequest> for ProductSaveDTO {
    fn from(item: ProductSaveRequest) -> Self {
        ProductSaveDTO {
            id: None,
            product_code: item.product_code,
            product_name: item.product_name,
            spec: item.spec,
            unit: item.unit,
            category_id: item.category_id,
            brand: item.brand,
            origin: item.origin,
            material: item.material,
            weight: item.weight,
            volume: item.volume,
            purchase_price: item.purchase_price,
            sale_price: item.sale_price,
            tax_rate: item.tax_rate,
            min_stock: item.min_stock,
            max_stock: item.max_stock,
            description: item.description,
            images: item.images,
            tags: item.tags,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ProductUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub category_id: Option<i64>,
    pub brand: Option<String>,
    pub origin: Option<String>,
    pub material: Option<String>,
    pub weight: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub purchase_price: Option<Decimal>,
    pub sale_price: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub min_stock: Option<i32>,
    pub max_stock: Option<i32>,
    pub description: Option<String>,
    pub images: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

impl From<ProductUpdateRequest> for ProductSaveDTO {
    fn from(item: ProductUpdateRequest) -> Self {
        ProductSaveDTO {
            id: item.id,
            product_code: item.product_code,
            product_name: item.product_name,
            spec: item.spec,
            unit: item.unit,
            category_id: item.category_id,
            brand: item.brand,
            origin: item.origin,
            material: item.material,
            weight: item.weight,
            volume: item.volume,
            purchase_price: item.purchase_price,
            sale_price: item.sale_price,
            tax_rate: item.tax_rate,
            min_stock: item.min_stock,
            max_stock: item.max_stock,
            description: item.description,
            images: item.images,
            tags: item.tags,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ProductSaveDTO {
    pub id: Option<i64>,
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub category_id: Option<i64>,
    pub brand: Option<String>,
    pub origin: Option<String>,
    pub material: Option<String>,
    pub weight: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub purchase_price: Option<Decimal>,
    pub sale_price: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub min_stock: Option<i32>,
    pub max_stock: Option<i32>,
    pub description: Option<String>,
    pub images: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub deleted: Option<i32>,
    pub created_by: Option<i64>,
    pub created_at: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ProductDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub category_id: Option<i64>,
    pub brand: Option<String>,
    pub origin: Option<String>,
    pub material: Option<String>,
    pub weight: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub purchase_price: Option<Decimal>,
    pub sale_price: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub stock: Option<i32>,
    pub min_stock: Option<i32>,
    pub max_stock: Option<i32>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub images: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

impl From<product::Model> for ProductDetailVO {
    fn from(item: product::Model) -> Self {
        ProductDetailVO {
            id: Option::from(item.id),
            product_code: item.product_code,
            product_name: item.product_name,
            spec: item.spec,
            unit: item.unit,
            category_id: item.category_id,
            brand: item.brand,
            origin: item.origin,
            material: item.material,
            weight: item.weight,
            volume: item.volume,
            purchase_price: item.purchase_price,
            sale_price: item.sale_price,
            tax_rate: item.tax_rate,
            stock: item.stock,
            min_stock: item.min_stock,
            max_stock: item.max_stock,
            status: item.status,
            description: item.description,
            images: item.images,
            tags: item.tags,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ProductListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub category_id: Option<i64>,
    pub brand: Option<String>,
    pub sale_price: Option<Decimal>,
    pub stock: Option<i32>,
    pub status: Option<String>,
    pub created_at: Option<DateTime>,
}

impl From<product::Model> for ProductListVO {
    fn from(item: product::Model) -> Self {
        ProductListVO {
            id: Option::from(item.id),
            product_code: item.product_code,
            product_name: item.product_name,
            spec: item.spec,
            unit: item.unit,
            category_id: item.category_id,
            brand: item.brand,
            sale_price: item.sale_price,
            stock: item.stock,
            status: item.status,
            created_at: item.created_at,
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
    pub status: Option<String>,
}

pub struct ProductModel;

impl ProductModel {
    pub async fn insert(db: &DbConn, req: &ProductSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = product::ActiveModel {
            product_code: Set(req.product_code.clone()),
            product_name: Set(req.product_name.clone()),
            spec: Set(req.spec.clone()),
            unit: Set(req.unit.clone()),
            category_id: Set(req.category_id.clone()),
            brand: Set(req.brand.clone()),
            origin: Set(req.origin.clone()),
            material: Set(req.material.clone()),
            weight: Set(req.weight.clone()),
            volume: Set(req.volume.clone()),
            purchase_price: Set(req.purchase_price.clone()),
            sale_price: Set(req.sale_price.clone()),
            tax_rate: Set(req.tax_rate.clone()),
            min_stock: Set(req.min_stock.clone()),
            max_stock: Set(req.max_stock.clone()),
            description: Set(req.description.clone()),
            images: Set(req.images.clone()),
            tags: Set(req.tags.clone()),
            created_by: Set(req.created_by.clone()),
            created_at: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(now)),
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

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &ProductSaveDTO) -> Result<i64, DbErr> {
        let payload = product::ActiveModel {
            product_code: Set(req.product_code.clone()),
            product_name: Set(req.product_name.clone()),
            spec: Set(req.spec.clone()),
            unit: Set(req.unit.clone()),
            category_id: Set(req.category_id.clone()),
            brand: Set(req.brand.clone()),
            origin: Set(req.origin.clone()),
            material: Set(req.material.clone()),
            weight: Set(req.weight.clone()),
            volume: Set(req.volume.clone()),
            purchase_price: Set(req.purchase_price.clone()),
            sale_price: Set(req.sale_price.clone()),
            tax_rate: Set(req.tax_rate.clone()),
            min_stock: Set(req.min_stock.clone()),
            max_stock: Set(req.max_stock.clone()),
            description: Set(req.description.clone()),
            images: Set(req.images.clone()),
            tags: Set(req.tags.clone()),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        
        let update_result: UpdateResult = Product::update_many()
            .set(payload)
            .filter(product::Column::Id.eq(id.clone().unwrap_or_default()))
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

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        category_id: Option<i64>,
        status: Option<String>,
    ) -> Result<(Vec<product::Model>, i64), DbErr> {
        let mut query = Product::find()
            .filter(product::Column::Deleted.eq(0));
        
        if let Some(k) = keywords {
            query = query.filter(product::Column::ProductName.contains(k));
        }
        if let Some(c) = category_id {
            query = query.filter(product::Column::CategoryId.eq(c));
        }
        if let Some(s) = status {
            query = query.filter(product::Column::Status.eq(s));
        }
        
        let paginator = query.order_by_desc(product::Column::CreatedAt).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    pub async fn select_count(
        db: &DbConn,
        keywords: Option<String>,
        category_id: Option<i64>,
        status: Option<String>,
    ) -> Result<i64, DbErr> {
        let mut query = Product::find()
            .filter(product::Column::Deleted.eq(0));
        
        if let Some(k) = keywords {
            query = query.filter(product::Column::ProductName.contains(k));
        }
        if let Some(c) = category_id {
            query = query.filter(product::Column::CategoryId.eq(c));
        }
        if let Some(s) = status {
            query = query.filter(product::Column::Status.eq(s));
        }
        
        query.count(db).await.map(|c| c as i64)
    }
}
