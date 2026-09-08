//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::prelude::Decimal;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};

use crate::modules::product::entity::unit_conversion;

/// 单位换算保存请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnitConversionSaveRequest {
    pub id: Option<i64>,
    pub product_id: Option<i64>,
    pub from_unit: Option<String>,
    pub to_unit: Option<String>,
    pub conversion_ratio: Option<Decimal>,
    pub is_default: Option<i32>,
    pub status: Option<i32>,
}

/// 单位换算列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnitConversionListQuery {
    #[serde(rename = "page")]
    pub page_num: u64,
    pub page_size: u64,
    pub product_id: Option<i64>,
    pub from_unit: Option<String>,
    pub to_unit: Option<String>,
    pub status: Option<i32>,
}

/// 单位换算列表 VO
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnitConversionListVO {
    pub list: Vec<unit_conversion::Model>,
    pub total: u64,
}

// ==================== DB 辅助方法 ====================

pub async fn find_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
) -> Result<Option<unit_conversion::Model>, DbErr> {
    unit_conversion::Entity::find_by_id(id)
        .filter(unit_conversion::Column::Deleted.eq(0))
        .one(db)
        .await
}

pub async fn select_page<C: ConnectionTrait>(
    db: &C,
    query: &UnitConversionListQuery,
) -> Result<(Vec<unit_conversion::Model>, u64), DbErr> {
    let mut q = unit_conversion::Entity::find().filter(unit_conversion::Column::Deleted.eq(0));

    if let Some(pid) = query.product_id {
        q = q.filter(unit_conversion::Column::ProductId.eq(pid));
    }
    if let Some(ref u) = query.from_unit {
        q = q.filter(unit_conversion::Column::FromUnit.eq(u));
    }
    if let Some(ref u) = query.to_unit {
        q = q.filter(unit_conversion::Column::ToUnit.eq(u));
    }
    if let Some(s) = query.status {
        q = q.filter(unit_conversion::Column::Status.eq(s));
    }

    let total = q.clone().count(db).await?;
    let rows = q
        .order_by_desc(unit_conversion::Column::CreateTime)
        .offset((query.page_num - 1) * query.page_size)
        .limit(query.page_size)
        .all(db)
        .await?;

    Ok((rows, total))
}

pub async fn find_by_product<C: ConnectionTrait>(
    db: &C,
    product_id: i64,
) -> Result<Vec<unit_conversion::Model>, DbErr> {
    unit_conversion::Entity::find()
        .filter(unit_conversion::Column::ProductId.eq(product_id))
        .filter(unit_conversion::Column::Status.eq(0))
        .filter(unit_conversion::Column::Deleted.eq(0))
        .order_by_desc(unit_conversion::Column::IsDefault)
        .order_by_desc(unit_conversion::Column::CreateTime)
        .all(db)
        .await
}

pub async fn insert<C: ConnectionTrait>(
    db: &C,
    req: &UnitConversionSaveRequest,
    created_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let active = unit_conversion::ActiveModel {
        product_id: Set(req.product_id),
        from_unit: Set(req.from_unit.clone()),
        to_unit: Set(req.to_unit.clone()),
        conversion_ratio: Set(req.conversion_ratio),
        is_default: Set(req.is_default.or(Some(0))),
        status: Set(req.status.or(Some(0))),
        deleted: Set(Some(0)),
        created_by: Set(Some(created_by)),
        updated_by: Set(Some(created_by)),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let result = active.insert(db).await?;
    Ok(result.id)
}

pub async fn update_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
    req: &UnitConversionSaveRequest,
    updated_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = unit_conversion::Entity::update_many()
        .col_expr(
            unit_conversion::Column::ProductId,
            Expr::value(req.product_id),
        )
        .col_expr(unit_conversion::Column::FromUnit, Expr::value(req.from_unit.clone()))
        .col_expr(unit_conversion::Column::ToUnit, Expr::value(req.to_unit.clone()))
        .col_expr(
            unit_conversion::Column::ConversionRatio,
            Expr::value(req.conversion_ratio),
        )
        .col_expr(unit_conversion::Column::IsDefault, Expr::value(req.is_default))
        .col_expr(unit_conversion::Column::Status, Expr::value(req.status))
        .col_expr(unit_conversion::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(unit_conversion::Column::UpdateTime, Expr::value(now))
        .filter(unit_conversion::Column::Id.eq(id))
        .filter(unit_conversion::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

pub async fn batch_delete<C: ConnectionTrait>(db: &C, ids: &[i64]) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = unit_conversion::Entity::update_many()
        .col_expr(unit_conversion::Column::Deleted, Expr::value(1))
        .col_expr(unit_conversion::Column::UpdateTime, Expr::value(now))
        .filter(unit_conversion::Column::Id.is_in(ids.iter().map(|&id| id).collect::<Vec<_>>()))
        .filter(unit_conversion::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}
