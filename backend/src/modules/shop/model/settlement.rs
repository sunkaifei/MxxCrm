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
use crate::modules::shop::entity::shop_settlement::{self, Entity as SettlementEntity};
use crate::utils::string_utils::serialize_option_u64_to_string;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::{DateTime, Decimal};
use sea_orm::*;

/// Settlement Request
/// 结算请求结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettlementRequest {
    /// 店铺ID
    pub shop_id: Option<i64>,
    /// 结算周期开始时间
    pub period_start: Option<String>,
    /// 结算周期结束时间
    pub period_end: Option<String>,
    /// 订单数量
    pub order_count: Option<i32>,
    /// 总金额(元)
    pub total_amount: Option<f64>,
    /// 佣金金额(元)
    pub commission_amount: Option<f64>,
    /// 结算金额(元)
    pub settlement_amount: Option<f64>,
    /// 备注
    pub remark: Option<String>,
}

/// Settlement DTO
/// 结算数据传输对象
pub struct SettlementDTO {
    /// 店铺ID
    pub shop_id: i64,
    /// 结算单号
    pub settlement_no: String,
    /// 结算周期开始时间
    pub period_start: DateTime,
    /// 结算周期结束时间
    pub period_end: DateTime,
    /// 订单数量
    pub order_count: i32,
    /// 总金额(元)
    pub total_amount: Decimal,
    /// 佣金金额(元)
    pub commission_amount: Decimal,
    /// 结算金额(元)
    pub settlement_amount: Decimal,
    /// 结算状态: 0=待结算, 1=已结算
    pub settlement_status: i16,
    /// 备注
    pub remark: Option<String>,
}

/// Settlement VO
/// 结算视图对象
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettlementVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 店铺ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub shop_id: Option<i64>,
    /// 结算单号
    pub settlement_no: Option<String>,
    /// 结算周期开始时间
    pub period_start: Option<String>,
    /// 结算周期结束时间
    pub period_end: Option<String>,
    /// 订单数量
    pub order_count: Option<i32>,
    /// 总金额(元)
    pub total_amount: Option<f64>,
    /// 佣金金额(元)
    pub commission_amount: Option<f64>,
    /// 结算金额(元)
    pub settlement_amount: Option<f64>,
    /// 结算状态: 0=待结算, 1=已结算
    pub settlement_status: Option<i16>,
    /// 结算时间
    pub settle_time: Option<String>,
    /// 备注
    pub remark: Option<String>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<shop_settlement::Model> for SettlementVO {
    fn from(model: shop_settlement::Model) -> Self {
        Self {
            id: Some(model.id),
            shop_id: Some(model.shop_id),
            settlement_no: Some(model.settlement_no),
            period_start: Some(model.period_start.format("%Y-%m-%d %H:%M:%S").to_string()),
            period_end: Some(model.period_end.format("%Y-%m-%d %H:%M:%S").to_string()),
            order_count: Some(model.order_count),
            total_amount: model.total_amount.to_f64(),
            commission_amount: model.commission_amount.to_f64(),
            settlement_amount: model.settlement_amount.to_f64(),
            settlement_status: Some(model.settlement_status),
            settle_time: model.settle_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            remark: model.remark,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// SettlementModel
/// 结算数据操作模型
pub struct SettlementModel;

impl SettlementModel {
    /// 插入结算记录
    pub async fn insert<C: ConnectionTrait>(db: &C, form: &SettlementDTO) -> Result<i64, DbErr> {
        let active = shop_settlement::ActiveModel {
            shop_id: Set(form.shop_id),
            settlement_no: Set(form.settlement_no.clone()),
            period_start: Set(form.period_start),
            period_end: Set(form.period_end),
            order_count: Set(form.order_count),
            total_amount: Set(form.total_amount),
            commission_amount: Set(form.commission_amount),
            settlement_amount: Set(form.settlement_amount),
            settlement_status: Set(form.settlement_status),
            remark: Set(form.remark.clone()),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        SettlementEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据店铺ID查询结算列表
    pub async fn find_by_shop_id<C: ConnectionTrait>(db: &C, shop_id: i64) -> Result<Vec<shop_settlement::Model>, DbErr> {
        SettlementEntity::find()
            .filter(shop_settlement::Column::ShopId.eq(shop_id))
            .order_by_desc(shop_settlement::Column::Id)
            .all(db)
            .await
    }

    /// 根据ID查询结算
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<shop_settlement::Model>, DbErr> {
        SettlementEntity::find_by_id(id).one(db).await
    }

    /// 执行结算
    pub async fn settle<C: ConnectionTrait>(db: &C, id: i64) -> Result<i64, DbErr> {
        let active = shop_settlement::ActiveModel {
            id: Set(id),
            settlement_status: Set(1),
            settle_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        SettlementEntity::update_many()
            .set(active)
            .filter(shop_settlement::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 分页查询结算列表
    pub async fn find_page<C: ConnectionTrait>(
        db: &C,
        page: i64,
        per_page: i64,
        shop_id: Option<i64>,
    ) -> Result<(Vec<shop_settlement::Model>, i64), DbErr> {
        let paginator = SettlementEntity::find()
            .apply_if(shop_id, |query, v| query.filter(shop_settlement::Column::ShopId.eq(v)))
            .order_by_desc(shop_settlement::Column::Id)
            .paginate(db, per_page as u64);

        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page(page.saturating_sub(1) as u64).await?;
        Ok((items, total))
    }
}
