//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::website::entity::{website_cart, website_cart::Entity as WebsiteCart};

// ==================== DTO ====================

/// 添加购物车请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CartAddRequest {
    pub product_id: i64,
    pub sku_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_image: Option<String>,
    pub sku_code: Option<String>,
    pub sku_specs: Option<String>,
    #[serde(default)]
    pub price: Decimal,
    #[serde(default = "default_quantity")]
    pub quantity: i32,
    pub website_id: Option<i64>,
}

fn default_quantity() -> i32 {
    1
}

/// 更新购物车请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CartUpdateRequest {
    pub quantity: Option<i32>,
    pub selected: Option<i32>,
}

/// 批量删除购物车请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CartBatchDeleteRequest {
    pub ids: Vec<i64>,
}

// ==================== VO ====================

/// 购物车项 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CartVO {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub product_id: Option<i64>,
    pub sku_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_image: Option<String>,
    pub sku_code: Option<String>,
    pub sku_specs: Option<String>,
    pub price: Option<Decimal>,
    pub quantity: Option<i32>,
    pub selected: Option<i32>,
    pub website_id: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

impl From<website_cart::Model> for CartVO {
    fn from(item: website_cart::Model) -> Self {
        CartVO {
            id: Option::from(item.id),
            user_id: Some(item.user_id),
            product_id: Some(item.product_id),
            sku_id: item.sku_id,
            product_name: item.product_name,
            product_image: item.product_image,
            sku_code: item.sku_code,
            sku_specs: item.sku_specs,
            price: Some(item.price),
            quantity: Some(item.quantity),
            selected: item.selected,
            website_id: item.website_id,
            create_time: item.create_time,
            update_time: item.update_time,
        }
    }
}

/// 购物车汇总 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CartSummaryVO {
    pub items: Vec<CartVO>,
    pub total_count: i64,
    pub selected_count: i64,
    pub selected_amount: Decimal,
}

// ==================== Model ====================

pub struct WebsiteCartModel;

impl WebsiteCartModel {
    /// 新增购物车项（若同用户同SKU已存在，则累加数量）
    pub async fn upsert<C: ConnectionTrait>(db: &C, user_id: i64, req: &CartAddRequest) -> Result<i64, DbErr> {
        // 查找已存在的相同项
        let mut query = WebsiteCart::find()
            .filter(website_cart::Column::UserId.eq(user_id))
            .filter(website_cart::Column::ProductId.eq(req.product_id));
        if let Some(sku_id) = req.sku_id {
            query = query.filter(website_cart::Column::SkuId.eq(sku_id));
        } else {
            query = query.filter(website_cart::Column::SkuId.is_null());
        }
        let existing = query.one(db).await?;

        if let Some(item) = existing {
            // 累加数量
            let new_qty = item.quantity + req.quantity;
            let now = chrono::Local::now().naive_local().to_owned();
            let result: UpdateResult = WebsiteCart::update_many()
                .col_expr(website_cart::Column::Quantity, sea_orm::sea_query::Expr::value(new_qty))
                .col_expr(website_cart::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
                .filter(website_cart::Column::Id.eq(item.id))
                .exec(db)
                .await?;
            Ok(result.rows_affected as i64)
        } else {
            let now = chrono::Local::now().naive_local().to_owned();
            let payload = website_cart::ActiveModel {
                user_id: Set(user_id),
                product_id: Set(req.product_id),
                sku_id: Set(req.sku_id),
                product_name: Set(req.product_name.clone()),
                product_image: Set(req.product_image.clone()),
                sku_code: Set(req.sku_code.clone()),
                sku_specs: Set(req.sku_specs.clone()),
                price: Set(req.price),
                quantity: Set(req.quantity),
                selected: Set(Some(1)),
                website_id: Set(req.website_id),
                create_time: Set(Some(now.clone())),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            WebsiteCart::insert(payload).exec(db).await.map(|r| r.last_insert_id)
        }
    }

    /// 查询用户购物车列表
    pub async fn find_by_user<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<Vec<website_cart::Model>, DbErr> {
        WebsiteCart::find()
            .filter(website_cart::Column::UserId.eq(user_id))
            .order_by_desc(website_cart::Column::CreateTime)
            .all(db)
            .await
    }

    /// 根据ID查询
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<website_cart::Model>, DbErr> {
        WebsiteCart::find_by_id(id).one(db).await
    }

    /// 更新数量/选中状态
    pub async fn update<C: ConnectionTrait>(db: &C, id: i64, user_id: i64, req: &CartUpdateRequest) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = website_cart::ActiveModel {
            update_time: Set(Some(now)),
            ..Default::default()
        };
        if let Some(q) = req.quantity { payload.quantity = Set(q); }
        if let Some(s) = req.selected { payload.selected = Set(Some(s)); }

        let result: UpdateResult = WebsiteCart::update_many()
            .set(payload)
            .filter(website_cart::Column::Id.eq(id))
            .filter(website_cart::Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 删除单条
    pub async fn delete<C: ConnectionTrait>(db: &C, id: i64, user_id: i64) -> Result<i64, DbErr> {
        let result: DeleteResult = WebsiteCart::delete_many()
            .filter(website_cart::Column::Id.eq(id))
            .filter(website_cart::Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 批量删除
    pub async fn batch_delete<C: ConnectionTrait>(db: &C, ids: Vec<i64>, user_id: i64) -> Result<i64, DbErr> {
        if ids.is_empty() { return Ok(0); }
        let result: DeleteResult = WebsiteCart::delete_many()
            .filter(website_cart::Column::Id.is_in(ids))
            .filter(website_cart::Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 清空用户购物车
    pub async fn clear_by_user<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<i64, DbErr> {
        let result: DeleteResult = WebsiteCart::delete_many()
            .filter(website_cart::Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 删除已下单的购物车项（按ID列表）
    pub async fn delete_by_ids<C: ConnectionTrait>(db: &C, ids: Vec<i64>) -> Result<i64, DbErr> {
        if ids.is_empty() { return Ok(0); }
        let result: DeleteResult = WebsiteCart::delete_many()
            .filter(website_cart::Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}
