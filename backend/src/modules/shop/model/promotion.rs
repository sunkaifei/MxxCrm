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
use crate::modules::shop::entity::shop_promotion::{self, Entity as PromotionEntity};
use crate::modules::shop::entity::shop_promotion_spu::{self, Entity as PromotionSpuEntity};
use crate::utils::string_utils::serialize_option_u64_to_string;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::{DateTime, Decimal};
use sea_orm::*;

/// Promotion Request
/// 促销活动请求结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PromotionRequest {
    /// 店铺ID
    pub shop_id: Option<i64>,
    /// 活动标题
    pub title: Option<String>,
    /// 活动类型: 1=满减, 2=折扣, 3=优惠券
    pub promotion_type: Option<i16>,
    /// 优惠值(元或折扣率)
    pub discount_value: Option<f64>,
    /// 条件值(满多少金额满足条件)
    pub condition_value: Option<f64>,
    /// 活动开始时间
    pub start_time: Option<String>,
    /// 活动结束时间
    pub end_time: Option<String>,
    /// 状态: 0=草稿, 1=进行中, 2=已结束
    pub status: Option<i16>,
    /// 关联商品ID列表
    pub spu_ids: Option<Vec<i64>>,
}

/// Promotion DTO
/// 促销活动数据传输对象
pub struct PromotionDTO {
    /// 店铺ID
    pub shop_id: i64,
    /// 活动标题
    pub title: String,
    /// 活动类型: 1=满减, 2=折扣, 3=优惠券
    pub promotion_type: i16,
    /// 优惠值(元或折扣率)
    pub discount_value: Decimal,
    /// 条件值(满多少金额满足条件)
    pub condition_value: Option<Decimal>,
    /// 活动开始时间
    pub start_time: DateTime,
    /// 活动结束时间
    pub end_time: DateTime,
    /// 状态: 0=草稿, 1=进行中, 2=已结束
    pub status: i16,
}

/// Promotion VO
/// 促销活动视图对象
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PromotionVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 店铺ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub shop_id: Option<i64>,
    /// 活动标题
    pub title: Option<String>,
    /// 活动类型: 1=满减, 2=折扣, 3=优惠券
    pub promotion_type: Option<i16>,
    /// 优惠值(元或折扣率)
    pub discount_value: Option<f64>,
    /// 条件值(满多少金额满足条件)
    pub condition_value: Option<f64>,
    /// 活动开始时间
    pub start_time: Option<String>,
    /// 活动结束时间
    pub end_time: Option<String>,
    /// 状态: 0=草稿, 1=进行中, 2=已结束
    pub status: Option<i16>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<shop_promotion::Model> for PromotionVO {
    fn from(model: shop_promotion::Model) -> Self {
        Self {
            id: Some(model.id),
            shop_id: Some(model.shop_id),
            title: Some(model.title),
            promotion_type: Some(model.promotion_type),
            discount_value: model.discount_value.to_f64(),
            condition_value: model.condition_value.and_then(|v| v.to_f64()),
            start_time: Some(model.start_time.format("%Y-%m-%d %H:%M:%S").to_string()),
            end_time: Some(model.end_time.format("%Y-%m-%d %H:%M:%S").to_string()),
            status: Some(model.status),
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// PromotionModel
/// 促销活动数据操作模型
pub struct PromotionModel;

impl PromotionModel {
    /// 插入促销活动记录
    pub async fn insert<C: ConnectionTrait>(db: &C, form: &PromotionDTO) -> Result<i64, DbErr> {
        let active = shop_promotion::ActiveModel {
            shop_id: Set(form.shop_id),
            title: Set(form.title.clone()),
            promotion_type: Set(form.promotion_type),
            discount_value: Set(form.discount_value),
            condition_value: Set(form.condition_value),
            start_time: Set(form.start_time),
            end_time: Set(form.end_time),
            status: Set(form.status),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        PromotionEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据ID查询促销活动
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<shop_promotion::Model>, DbErr> {
        PromotionEntity::find_by_id(id).one(db).await
    }

    /// 根据店铺ID查询促销活动列表
    pub async fn find_by_shop_id<C: ConnectionTrait>(db: &C, shop_id: i64) -> Result<Vec<shop_promotion::Model>, DbErr> {
        PromotionEntity::find()
            .filter(shop_promotion::Column::ShopId.eq(shop_id))
            .order_by_desc(shop_promotion::Column::Id)
            .all(db)
            .await
    }

    /// 分页查询促销活动列表
    pub async fn find_page<C: ConnectionTrait>(
        db: &C,
        page: i64,
        per_page: i64,
        shop_id: Option<i64>,
        status: Option<i16>,
    ) -> Result<(Vec<shop_promotion::Model>, i64), DbErr> {
        let paginator = PromotionEntity::find()
            .apply_if(shop_id, |query, v| query.filter(shop_promotion::Column::ShopId.eq(v)))
            .apply_if(status, |query, v| query.filter(shop_promotion::Column::Status.eq(v)))
            .order_by_desc(shop_promotion::Column::Id)
            .paginate(db, per_page as u64);

        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page(page.saturating_sub(1) as u64).await?;
        Ok((items, total))
    }

    /// 更新促销活动状态
    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i16) -> Result<i64, DbErr> {
        let active = shop_promotion::ActiveModel {
            id: Set(id),
            status: Set(status),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        PromotionEntity::update_many()
            .set(active)
            .filter(shop_promotion::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}

/// PromotionSpuModel
/// 促销活动商品关联数据操作模型
pub struct PromotionSpuModel;

impl PromotionSpuModel {
    /// 插入促销活动商品关联记录
    pub async fn insert<C: ConnectionTrait>(db: &C, promotion_id: i64, spu_id: i64, sku_id: Option<i64>) -> Result<i64, DbErr> {
        let active = shop_promotion_spu::ActiveModel {
            promotion_id: Set(promotion_id),
            spu_id: Set(spu_id),
            sku_id: Set(sku_id),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        PromotionSpuEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据促销活动ID查询关联商品列表
    pub async fn find_by_promotion_id<C: ConnectionTrait>(db: &C, promotion_id: i64) -> Result<Vec<shop_promotion_spu::Model>, DbErr> {
        PromotionSpuEntity::find()
            .filter(shop_promotion_spu::Column::PromotionId.eq(promotion_id))
            .all(db)
            .await
    }

    /// 根据促销活动ID删除关联商品记录
    pub async fn delete_by_promotion_id<C: ConnectionTrait>(db: &C, promotion_id: i64) -> Result<i64, DbErr> {
        PromotionSpuEntity::delete_many()
            .filter(shop_promotion_spu::Column::PromotionId.eq(promotion_id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}
