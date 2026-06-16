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
use crate::modules::shop::entity::shop_cart::{self, Entity as CartEntity};
use crate::modules::shop::entity::shop_spu::Entity as SpuEntity;
use crate::modules::shop::entity::shop_sku::Entity as SkuEntity;
use crate::utils::string_utils::serialize_option_u64_to_string;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::DateTime;
use sea_orm::*;

/// Cart Request
/// 购物车请求结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CartRequest {
    /// 买家用户ID
    pub user_id: Option<i64>,
    /// 店铺ID
    pub shop_id: Option<i64>,
    /// 商品ID
    pub spu_id: Option<i64>,
    /// SKU ID
    pub sku_id: Option<i64>,
    /// 数量
    pub quantity: Option<i32>,
    /// 是否选中: 0=否, 1=是
    pub selected: Option<i16>,
}

/// Cart DTO
/// 购物车数据传输对象
pub struct CartDTO {
    /// 买家用户ID
    pub user_id: i64,
    /// 店铺ID
    pub shop_id: i64,
    /// 商品ID
    pub spu_id: i64,
    /// SKU ID
    pub sku_id: i64,
    /// 数量
    pub quantity: i32,
    /// 是否选中: 0=否, 1=是
    pub selected: i16,
}

impl From<CartRequest> for CartDTO {
    fn from(req: CartRequest) -> Self {
        CartDTO {
            user_id: req.user_id.unwrap_or(0),
            shop_id: req.shop_id.unwrap_or(0),
            spu_id: req.spu_id.unwrap_or(0),
            sku_id: req.sku_id.unwrap_or(0),
            quantity: req.quantity.unwrap_or(1),
            selected: req.selected.unwrap_or(1),
        }
    }
}

/// Cart VO
/// 购物车视图对象
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CartVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 买家用户ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 店铺ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub shop_id: Option<i64>,
    /// 商品ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub spu_id: Option<i64>,
    /// SKU ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub sku_id: Option<i64>,
    /// 数量
    pub quantity: Option<i32>,
    /// 是否选中: 0=否, 1=是
    pub selected: Option<i16>,
    /// 商品标题
    pub goods_title: Option<String>,
    /// 商品图片
    pub goods_image: Option<String>,
    /// 规格描述
    pub spec_desc: Option<String>,
    /// 售价
    pub sale_price: Option<f64>,
    /// 库存
    pub stock: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

/// CartModel
/// 购物车数据操作模型
pub struct CartModel;

impl CartModel {
    /// 插入购物车记录
    pub async fn insert<C: ConnectionTrait>(db: &C, form: &CartDTO) -> Result<i64, DbErr> {
        let active = shop_cart::ActiveModel {
            user_id: Set(form.user_id),
            shop_id: Set(form.shop_id),
            spu_id: Set(form.spu_id),
            sku_id: Set(form.sku_id),
            quantity: Set(form.quantity),
            selected: Set(form.selected),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        CartEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据用户ID查询购物车列表
    pub async fn find_by_user_id<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<Vec<shop_cart::Model>, DbErr> {
        CartEntity::find()
            .filter(shop_cart::Column::UserId.eq(user_id))
            .order_by_desc(shop_cart::Column::Id)
            .all(db)
            .await
    }

    /// 根据ID查询购物车
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<shop_cart::Model>, DbErr> {
        CartEntity::find_by_id(id).one(db).await
    }

    /// 根据用户ID和SKU ID查询购物车
    pub async fn find_by_user_and_sku<C: ConnectionTrait>(db: &C, user_id: i64, sku_id: i64) -> Result<Option<shop_cart::Model>, DbErr> {
        CartEntity::find()
            .filter(shop_cart::Column::UserId.eq(user_id))
            .filter(shop_cart::Column::SkuId.eq(sku_id))
            .one(db)
            .await
    }

    /// 更新购物车数量
    pub async fn update_quantity<C: ConnectionTrait>(db: &C, id: i64, quantity: i32) -> Result<i64, DbErr> {
        let active = shop_cart::ActiveModel {
            id: Set(id),
            quantity: Set(quantity),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        CartEntity::update_many()
            .set(active)
            .filter(shop_cart::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新购物车选中状态
    pub async fn update_selected<C: ConnectionTrait>(db: &C, user_id: i64, sku_ids: Vec<i64>, selected: i16) -> Result<i64, DbErr> {
        let active = shop_cart::ActiveModel {
            selected: Set(selected),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        CartEntity::update_many()
            .set(active)
            .filter(shop_cart::Column::UserId.eq(user_id))
            .filter(shop_cart::Column::SkuId.is_in(sku_ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 根据ID删除购物车
    pub async fn delete_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<i64, DbErr> {
        CartEntity::delete_by_id(id).exec(db).await.map(|r| r.rows_affected as i64)
    }

    /// 根据用户ID清空购物车
    pub async fn delete_by_user_id<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<i64, DbErr> {
        CartEntity::delete_many()
            .filter(shop_cart::Column::UserId.eq(user_id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 批量删除购物车
    pub async fn delete_by_ids<C: ConnectionTrait>(db: &C, ids: Vec<i64>) -> Result<i64, DbErr> {
        CartEntity::delete_many()
            .filter(shop_cart::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 统计用户购物车数量
    pub async fn count_by_user_id<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<i64, DbErr> {
        CartEntity::find()
            .filter(shop_cart::Column::UserId.eq(user_id))
            .count(db)
            .await
            .map(|c| c as i64)
    }
}
