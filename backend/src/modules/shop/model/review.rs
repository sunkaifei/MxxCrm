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
use crate::modules::shop::entity::shop_review::{self, Entity as ReviewEntity};
use crate::utils::string_utils::serialize_option_u64_to_string;
use sea_orm::prelude::{DateTime, Json};
use sea_orm::*;

/// Review Request
/// 评价请求结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReviewRequest {
    /// 订单ID
    pub order_id: Option<i64>,
    /// 商品ID
    pub spu_id: Option<i64>,
    /// SKU ID
    pub sku_id: Option<i64>,
    /// 评分(1~5)
    pub score: Option<i16>,
    /// 评价内容
    pub content: Option<String>,
    /// 评价图片列表(JSON)
    pub images: Option<Json>,
    /// 是否匿名: 0=否, 1=是
    pub is_anonymous: Option<i16>,
}

/// Review DTO
/// 评价数据传输对象
pub struct ReviewDTO {
    /// 订单ID
    pub order_id: i64,
    /// 商品ID
    pub spu_id: i64,
    /// SKU ID
    pub sku_id: i64,
    /// 买家用户ID
    pub user_id: i64,
    /// 店铺ID
    pub shop_id: i64,
    /// 评分(1~5)
    pub score: i16,
    /// 评价内容
    pub content: Option<String>,
    /// 评价图片列表(JSON)
    pub images: Option<Json>,
    /// 是否匿名: 0=否, 1=是
    pub is_anonymous: i16,
}

impl From<ReviewRequest> for ReviewDTO {
    fn from(req: ReviewRequest) -> Self {
        ReviewDTO {
            order_id: req.order_id.unwrap_or(0),
            spu_id: req.spu_id.unwrap_or(0),
            sku_id: req.sku_id.unwrap_or(0),
            user_id: 0,
            shop_id: 0,
            score: req.score.unwrap_or(5),
            content: req.content,
            images: req.images,
            is_anonymous: req.is_anonymous.unwrap_or(0),
        }
    }
}

/// Review VO
/// 评价视图对象
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReviewVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 订单ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub order_id: Option<i64>,
    /// 商品ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub spu_id: Option<i64>,
    /// SKU ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub sku_id: Option<i64>,
    /// 买家用户ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 店铺ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub shop_id: Option<i64>,
    /// 评分(1~5)
    pub score: Option<i16>,
    /// 评价内容
    pub content: Option<String>,
    /// 评价图片列表(JSON)
    pub images: Option<Json>,
    /// 是否匿名: 0=否, 1=是
    pub is_anonymous: Option<i16>,
    /// 供货商回复
    pub reply_content: Option<String>,
    /// 回复时间
    pub reply_time: Option<String>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<shop_review::Model> for ReviewVO {
    fn from(model: shop_review::Model) -> Self {
        Self {
            id: Some(model.id),
            order_id: Some(model.order_id),
            spu_id: Some(model.spu_id),
            sku_id: Some(model.sku_id),
            user_id: Some(model.user_id),
            shop_id: Some(model.shop_id),
            score: Some(model.score),
            content: model.content,
            images: model.images,
            is_anonymous: Some(model.is_anonymous),
            reply_content: model.reply_content,
            reply_time: model.reply_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// ReviewModel
/// 评价数据操作模型
pub struct ReviewModel;

impl ReviewModel {
    /// 插入评价记录
    pub async fn insert<C: ConnectionTrait>(db: &C, form: &ReviewDTO) -> Result<i64, DbErr> {
        let active = shop_review::ActiveModel {
            order_id: Set(form.order_id),
            spu_id: Set(form.spu_id),
            sku_id: Set(form.sku_id),
            user_id: Set(form.user_id),
            shop_id: Set(form.shop_id),
            score: Set(form.score),
            content: Set(form.content.clone()),
            images: Set(form.images.clone()),
            is_anonymous: Set(form.is_anonymous),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        ReviewEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据商品ID查询评价列表
    pub async fn find_by_spu_id<C: ConnectionTrait>(db: &C, spu_id: i64) -> Result<Vec<shop_review::Model>, DbErr> {
        ReviewEntity::find()
            .filter(shop_review::Column::SpuId.eq(spu_id))
            .order_by_desc(shop_review::Column::Id)
            .all(db)
            .await
    }

    /// 根据订单ID查询评价列表
    pub async fn find_by_order_id<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<Vec<shop_review::Model>, DbErr> {
        ReviewEntity::find()
            .filter(shop_review::Column::OrderId.eq(order_id))
            .all(db)
            .await
    }

    /// 根据ID查询评价
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<shop_review::Model>, DbErr> {
        ReviewEntity::find_by_id(id).one(db).await
    }

    /// 回复评价
    pub async fn reply<C: ConnectionTrait>(db: &C, id: i64, reply_content: &str) -> Result<i64, DbErr> {
        let active = shop_review::ActiveModel {
            id: Set(id),
            reply_content: Set(Some(reply_content.to_string())),
            reply_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        ReviewEntity::update_many()
            .set(active)
            .filter(shop_review::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}
