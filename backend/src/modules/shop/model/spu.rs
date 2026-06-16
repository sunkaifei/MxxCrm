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
use crate::modules::shop::entity::shop_spu::{self, Entity as SpuEntity};
use crate::modules::shop::entity::shop_sku::{self, Entity as SkuEntity};
use crate::modules::shop::entity::shop_spec::{self, Entity as SpecEntity};
use crate::modules::shop::entity::shop_spec_value::{self, Entity as SpecValueEntity};
use crate::utils::string_utils::serialize_option_u64_to_string;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::{DateTime, Decimal, Json};
use sea_orm::*;

/// SPU Request
/// SPU创建/更新请求结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpuRequest {
    /// 店铺ID
    pub shop_id: Option<i64>,
    /// 分类ID
    pub category_id: Option<i64>,
    /// 商品标题
    pub title: Option<String>,
    /// 副标题/卖点
    pub subtitle: Option<String>,
    /// 主图
    pub primary_image: Option<String>,
    /// 商品轮播图列表(JSON)
    pub images: Option<Json>,
    /// 商品视频
    pub video: Option<String>,
    /// 商品详情(JSON)
    pub desc_content: Option<Json>,
    /// 是否参与佣金: 0=否, 1=是
    pub is_commission: Option<i16>,
    /// 状态: 0=待审核, 1=已上架, 2=下架, 3=审核退回
    pub status: Option<i16>,
    /// 总库存
    pub stock_total: Option<i32>,
    /// 已售数量
    pub sold_num: Option<i32>,
    /// 最低售价
    pub min_sale_price: Option<f64>,
    /// 最高售价
    pub max_sale_price: Option<f64>,
    /// 最低划线价
    pub min_line_price: Option<f64>,
    /// 最高划线价
    pub max_line_price: Option<f64>,
    /// 运费模板ID
    pub freight_template_id: Option<i64>,
}

/// SPU DTO
/// SPU数据传输对象
pub struct SpuDTO {
    /// 店铺ID
    pub shop_id: i64,
    /// 分类ID
    pub category_id: i64,
    /// 商品标题
    pub title: String,
    /// 副标题/卖点
    pub subtitle: Option<String>,
    /// 主图
    pub primary_image: String,
    /// 商品轮播图列表(JSON)
    pub images: Option<Json>,
    /// 商品视频
    pub video: Option<String>,
    /// 商品详情(JSON)
    pub desc_content: Option<Json>,
    /// 是否参与佣金: 0=否, 1=是
    pub is_commission: i16,
    /// 状态: 0=待审核, 1=已上架, 2=下架, 3=审核退回
    pub status: i16,
    /// 总库存
    pub stock_total: i32,
    /// 已售数量
    pub sold_num: i32,
    /// 最低售价
    pub min_sale_price: Decimal,
    /// 最高售价
    pub max_sale_price: Decimal,
    /// 最低划线价
    pub min_line_price: Decimal,
    /// 最高划线价
    pub max_line_price: Decimal,
    /// 运费模板ID
    pub freight_template_id: Option<i64>,
}

impl From<SpuRequest> for SpuDTO {
    fn from(req: SpuRequest) -> Self {
        SpuDTO {
            shop_id: req.shop_id.unwrap_or(0),
            category_id: req.category_id.unwrap_or(0),
            title: req.title.unwrap_or_default(),
            subtitle: req.subtitle,
            primary_image: req.primary_image.unwrap_or_default(),
            images: req.images,
            video: req.video,
            desc_content: req.desc_content,
            is_commission: req.is_commission.unwrap_or(0),
            status: req.status.unwrap_or(0),
            stock_total: req.stock_total.unwrap_or(0),
            sold_num: req.sold_num.unwrap_or(0),
            min_sale_price: rust_decimal::Decimal::from_f64_retain(req.min_sale_price.unwrap_or(0.0)).unwrap_or_default(),
            max_sale_price: rust_decimal::Decimal::from_f64_retain(req.max_sale_price.unwrap_or(0.0)).unwrap_or_default(),
            min_line_price: rust_decimal::Decimal::from_f64_retain(req.min_line_price.unwrap_or(0.0)).unwrap_or_default(),
            max_line_price: rust_decimal::Decimal::from_f64_retain(req.max_line_price.unwrap_or(0.0)).unwrap_or_default(),
            freight_template_id: req.freight_template_id,
        }
    }
}

/// SPU VO
/// SPU视图对象
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpuVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 店铺ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub shop_id: Option<i64>,
    /// 分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub category_id: Option<i64>,
    /// 商品标题
    pub title: Option<String>,
    /// 副标题/卖点
    pub subtitle: Option<String>,
    /// 主图
    pub primary_image: Option<String>,
    /// 商品轮播图列表(JSON)
    pub images: Option<Json>,
    /// 商品视频
    pub video: Option<String>,
    /// 商品详情(JSON)
    pub desc_content: Option<Json>,
    /// 是否参与佣金: 0=否, 1=是
    pub is_commission: Option<i16>,
    /// 状态: 0=待审核, 1=已上架, 2=下架, 3=审核退回
    pub status: Option<i16>,
    /// 总库存
    pub stock_total: Option<i32>,
    /// 已售数量
    pub sold_num: Option<i32>,
    /// 最低售价
    pub min_sale_price: Option<f64>,
    /// 最高售价
    pub max_sale_price: Option<f64>,
    /// 最低划线价
    pub min_line_price: Option<f64>,
    /// 最高划线价
    pub max_line_price: Option<f64>,
    /// 审核备注
    pub audit_remark: Option<String>,
    /// 运费模板ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub freight_template_id: Option<i64>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<shop_spu::Model> for SpuVO {
    fn from(model: shop_spu::Model) -> Self {
        Self {
            id: Some(model.id),
            shop_id: Some(model.shop_id),
            category_id: Some(model.category_id),
            title: Some(model.title),
            subtitle: model.subtitle,
            primary_image: Some(model.primary_image),
            images: model.images,
            video: model.video,
            desc_content: model.desc_content,
            is_commission: Some(model.is_commission),
            status: Some(model.status),
            stock_total: Some(model.stock_total),
            sold_num: Some(model.sold_num),
            min_sale_price: model.min_sale_price.to_f64(),
            max_sale_price: model.max_sale_price.to_f64(),
            min_line_price: model.min_line_price.to_f64(),
            max_line_price: model.max_line_price.to_f64(),
            audit_remark: model.audit_remark,
            freight_template_id: model.freight_template_id,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// SKU VO
/// SKU视图对象
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkuVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 所属SPU ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub spu_id: Option<i64>,
    /// 规格描述
    pub spec_desc: Option<String>,
    /// SKU编码
    pub sku_code: Option<String>,
    /// SKU图片
    pub image: Option<String>,
    /// 供货底价
    pub base_price: Option<f64>,
    /// 零售价
    pub retail_price: Option<f64>,
    /// 实际售价
    pub sale_price: Option<f64>,
    /// 划线价
    pub line_price: Option<f64>,
    /// 库存
    pub stock: Option<i32>,
    /// 安全库存
    pub safe_stock: Option<i32>,
    /// 已售数量
    pub sold_num: Option<i32>,
    /// 重量(kg)
    pub weight: Option<f64>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<shop_sku::Model> for SkuVO {
    fn from(model: shop_sku::Model) -> Self {
        Self {
            id: Some(model.id),
            spu_id: Some(model.spu_id),
            spec_desc: Some(model.spec_desc),
            sku_code: model.sku_code,
            image: model.image,
            base_price: model.base_price.to_f64(),
            retail_price: model.retail_price.to_f64(),
            sale_price: model.sale_price.to_f64(),
            line_price: model.line_price.to_f64(),
            stock: Some(model.stock),
            safe_stock: Some(model.safe_stock),
            sold_num: Some(model.sold_num),
            weight: model.weight.and_then(|w| w.to_f64()),
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// Spec VO
/// 规格视图对象
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpecVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 所属SPU ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub spu_id: Option<i64>,
    /// 规格名称
    pub spec_name: Option<String>,
    /// 排序值
    pub sort_order: Option<i32>,
    /// 规格值列表
    pub values: Option<Vec<SpecValueVO>>,
}

/// Spec Value VO
/// 规格值视图对象
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpecValueVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 所属规格ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub spec_id: Option<i64>,
    /// 规格值
    pub spec_value: Option<String>,
    /// 排序值
    pub sort_order: Option<i32>,
}

/// SpuModel
/// SPU数据操作模型
pub struct SpuModel;

impl SpuModel {
    /// 插入SPU记录
    pub async fn insert<C: ConnectionTrait>(db: &C, form: &SpuDTO) -> Result<i64, DbErr> {
        let active = shop_spu::ActiveModel {
            shop_id: Set(form.shop_id),
            category_id: Set(form.category_id),
            title: Set(form.title.clone()),
            subtitle: Set(form.subtitle.clone()),
            primary_image: Set(form.primary_image.clone()),
            images: Set(form.images.clone()),
            video: Set(form.video.clone()),
            desc_content: Set(form.desc_content.clone()),
            is_commission: Set(form.is_commission),
            status: Set(form.status),
            stock_total: Set(form.stock_total),
            sold_num: Set(form.sold_num),
            min_sale_price: Set(form.min_sale_price),
            max_sale_price: Set(form.max_sale_price),
            min_line_price: Set(form.min_line_price),
            max_line_price: Set(form.max_line_price),
            freight_template_id: Set(form.freight_template_id),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        SpuEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据ID查询SPU
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<shop_spu::Model>, DbErr> {
        SpuEntity::find_by_id(id).one(db).await
    }

    /// 根据店铺ID查询SPU列表
    pub async fn find_by_shop_id<C: ConnectionTrait>(db: &C, shop_id: i64) -> Result<Vec<shop_spu::Model>, DbErr> {
        SpuEntity::find()
            .filter(shop_spu::Column::ShopId.eq(shop_id))
            .order_by_desc(shop_spu::Column::Id)
            .all(db)
            .await
    }

    /// 分页查询SPU列表
    pub async fn find_page<C: ConnectionTrait>(
        db: &C,
        page: i64,
        per_page: i64,
        shop_id: Option<i64>,
        category_id: Option<i64>,
        status: Option<i16>,
    ) -> Result<(Vec<shop_spu::Model>, i64), DbErr> {
        let paginator = SpuEntity::find()
            .apply_if(shop_id, |query, v| query.filter(shop_spu::Column::ShopId.eq(v)))
            .apply_if(category_id, |query, v| query.filter(shop_spu::Column::CategoryId.eq(v)))
            .apply_if(status, |query, v| query.filter(shop_spu::Column::Status.eq(v)))
            .order_by_desc(shop_spu::Column::Id)
            .paginate(db, per_page as u64);

        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page(page.saturating_sub(1) as u64).await?;
        Ok((items, total))
    }

    /// 更新SPU状态
    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i16) -> Result<i64, DbErr> {
        let active = shop_spu::ActiveModel {
            id: Set(id),
            status: Set(status),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        SpuEntity::update_many()
            .set(active)
            .filter(shop_spu::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}

/// SpecModel
/// 规格数据操作模型
pub struct SpecModel;

impl SpecModel {
    /// 插入规格记录
    pub async fn insert<C: ConnectionTrait>(db: &C, spu_id: i64, spec_name: &str, sort_order: i32) -> Result<i64, DbErr> {
        let active = shop_spec::ActiveModel {
            spu_id: Set(spu_id),
            spec_name: Set(spec_name.to_string()),
            sort_order: Set(sort_order),
            ..Default::default()
        };
        SpecEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据SPU ID查询规格列表
    pub async fn find_by_spu_id<C: ConnectionTrait>(db: &C, spu_id: i64) -> Result<Vec<shop_spec::Model>, DbErr> {
        SpecEntity::find()
            .filter(shop_spec::Column::SpuId.eq(spu_id))
            .order_by_asc(shop_spec::Column::SortOrder)
            .all(db)
            .await
    }

    /// 根据SPU ID删除规格记录
    pub async fn delete_by_spu_id<C: ConnectionTrait>(db: &C, spu_id: i64) -> Result<i64, DbErr> {
        SpecEntity::delete_many()
            .filter(shop_spec::Column::SpuId.eq(spu_id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}

/// SpecValueModel
/// 规格值数据操作模型
pub struct SpecValueModel;

impl SpecValueModel {
    /// 插入规格值记录
    pub async fn insert<C: ConnectionTrait>(db: &C, spec_id: i64, spec_value: &str, sort_order: i32) -> Result<i64, DbErr> {
        let active = shop_spec_value::ActiveModel {
            spec_id: Set(spec_id),
            spec_value: Set(spec_value.to_string()),
            sort_order: Set(sort_order),
            ..Default::default()
        };
        SpecValueEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据规格ID查询规格值列表
    pub async fn find_by_spec_id<C: ConnectionTrait>(db: &C, spec_id: i64) -> Result<Vec<shop_spec_value::Model>, DbErr> {
        SpecValueEntity::find()
            .filter(shop_spec_value::Column::SpecId.eq(spec_id))
            .order_by_asc(shop_spec_value::Column::SortOrder)
            .all(db)
            .await
    }

    /// 根据规格ID删除规格值记录
    pub async fn delete_by_spec_id<C: ConnectionTrait>(db: &C, spec_id: i64) -> Result<i64, DbErr> {
        SpecValueEntity::delete_many()
            .filter(shop_spec_value::Column::SpecId.eq(spec_id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}

/// SkuModel
/// SKU数据操作模型
pub struct SkuModel;

impl SkuModel {
    /// 插入SKU记录
    pub async fn insert<C: ConnectionTrait>(db: &C, active: shop_sku::ActiveModel) -> Result<i64, DbErr> {
        SkuEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据SPU ID查询SKU列表
    pub async fn find_by_spu_id<C: ConnectionTrait>(db: &C, spu_id: i64) -> Result<Vec<shop_sku::Model>, DbErr> {
        SkuEntity::find()
            .filter(shop_sku::Column::SpuId.eq(spu_id))
            .order_by_asc(shop_sku::Column::Id)
            .all(db)
            .await
    }

    /// 根据ID查询SKU
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<shop_sku::Model>, DbErr> {
        SkuEntity::find_by_id(id).one(db).await
    }

    /// 根据SPU ID删除SKU记录
    pub async fn delete_by_spu_id<C: ConnectionTrait>(db: &C, spu_id: i64) -> Result<i64, DbErr> {
        SkuEntity::delete_many()
            .filter(shop_sku::Column::SpuId.eq(spu_id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新SKU库存
    pub async fn update_stock<C: ConnectionTrait>(db: &C, id: i64, stock: i32) -> Result<i64, DbErr> {
        let active = shop_sku::ActiveModel {
            id: Set(id),
            stock: Set(stock),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        SkuEntity::update_many()
            .set(active)
            .filter(shop_sku::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}
