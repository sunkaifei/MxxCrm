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
use sea_orm::sea_query::Expr;
use serde::{Deserialize, Serialize};
use crate::modules::inventory::entity::warehouse_area;

// 库位保存请求
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseAreaSaveRequest {
    pub warehouse_id: i64,
    pub area_code: Option<String>,
    pub area_name: Option<String>,
    pub area_type: Option<String>,
    pub status: Option<i32>,
    pub sort_order: Option<i32>,
    pub remark: Option<String>,
}

// 库位更新请求
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseAreaUpdateRequest {
    pub id: i64,
    pub warehouse_id: Option<i64>,
    pub area_code: Option<String>,
    pub area_name: Option<String>,
    pub area_type: Option<String>,
    pub status: Option<i32>,
    pub sort_order: Option<i32>,
    pub remark: Option<String>,
}

// 库位列表查询参数
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseAreaListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub warehouse_id: Option<i64>,
    pub area_name: Option<String>,
    pub area_type: Option<String>,
    pub status: Option<i32>,
}

// 库位列表VO
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseAreaListVO {
    pub total: i64,
    pub items: Vec<WarehouseAreaVO>,
}

// 库位列表项
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseAreaVO {
    pub id: i64,
    pub warehouse_id: Option<i64>,
    pub area_code: Option<String>,
    pub area_name: Option<String>,
    pub area_type: Option<String>,
    pub area_type_name: Option<String>,
    pub status: Option<i32>,
    pub sort_order: Option<i32>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
}

// 库位详情VO
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseAreaDetailVO {
    pub id: i64,
    pub warehouse_id: Option<i64>,
    pub area_code: Option<String>,
    pub area_name: Option<String>,
    pub area_type: Option<String>,
    pub status: Option<i32>,
    pub sort_order: Option<i32>,
    pub remark: Option<String>,
}

fn area_type_name(t: Option<&str>) -> Option<String> {
    match t? {
        "storage" => Some("存储区".to_string()),
        "picking" => Some("拣货区".to_string()),
        "return" => Some("退货区".to_string()),
        "quality" => Some("质检区".to_string()),
        other => Some(other.to_string()),
    }
}

impl From<warehouse_area::Model> for WarehouseAreaVO {
    fn from(m: warehouse_area::Model) -> Self {
        Self {
            id: m.id,
            warehouse_id: m.warehouse_id,
            area_code: m.area_code.clone(),
            area_name: m.area_name.clone(),
            area_type: m.area_type.clone(),
            area_type_name: area_type_name(m.area_type.as_deref()),
            status: m.status,
            sort_order: m.sort_order,
            remark: m.remark,
            create_time: m.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

impl From<warehouse_area::Model> for WarehouseAreaDetailVO {
    fn from(m: warehouse_area::Model) -> Self {
        Self {
            id: m.id,
            warehouse_id: m.warehouse_id,
            area_code: m.area_code,
            area_name: m.area_name,
            area_type: m.area_type,
            status: m.status,
            sort_order: m.sort_order,
            remark: m.remark,
        }
    }
}

// DB helper functions
pub async fn select_page<C: ConnectionTrait>(
    db: &C,
    query: &WarehouseAreaListQuery,
) -> Result<(Vec<warehouse_area::Model>, u64), DbErr> {
    let page_num = std::cmp::Ord::max(query.page_num.unwrap_or(1), 1);
    let page_size = std::cmp::Ord::max(query.page_size.unwrap_or(10), 1);

    let mut q = warehouse_area::Entity::find()
        .filter(warehouse_area::Column::Deleted.eq(0));

    if let Some(wid) = query.warehouse_id {
        q = q.filter(warehouse_area::Column::WarehouseId.eq(wid));
    }
    if let Some(ref name) = query.area_name {
        if !name.is_empty() {
            q = q.filter(warehouse_area::Column::AreaName.contains(name));
        }
    }
    if let Some(ref t) = query.area_type {
        if !t.is_empty() {
            q = q.filter(warehouse_area::Column::AreaType.eq(t));
        }
    }
    if let Some(s) = query.status {
        q = q.filter(warehouse_area::Column::Status.eq(s));
    }

    let total = q.clone().count(db).await?;
    let rows = q
        .order_by_asc(warehouse_area::Column::SortOrder)
        .order_by_desc(warehouse_area::Column::Id)
        .offset(((page_num - 1) as u64) * page_size as u64)
        .limit(page_size as u64)
        .all(db)
        .await?;

    Ok((rows, total))
}

pub async fn find_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
) -> Result<Option<warehouse_area::Model>, DbErr> {
    warehouse_area::Entity::find_by_id(id)
        .filter(warehouse_area::Column::Deleted.eq(0))
        .one(db)
        .await
}

pub async fn find_by_warehouse<C: ConnectionTrait>(
    db: &C,
    warehouse_id: i64,
) -> Result<Vec<warehouse_area::Model>, DbErr> {
    warehouse_area::Entity::find()
        .filter(warehouse_area::Column::WarehouseId.eq(warehouse_id))
        .filter(warehouse_area::Column::Deleted.eq(0))
        .order_by_asc(warehouse_area::Column::SortOrder)
        .all(db)
        .await
}

pub async fn insert<C: ConnectionTrait>(
    db: &C,
    req: &WarehouseAreaSaveRequest,
    created_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let active = warehouse_area::ActiveModel {
        warehouse_id: Set(Some(req.warehouse_id)),
        area_code: Set(req.area_code.clone()),
        area_name: Set(req.area_name.clone()),
        area_type: Set(req.area_type.clone()),
        status: Set(req.status.or(Some(0))),
        sort_order: Set(req.sort_order.or(Some(0))),
        remark: Set(req.remark.clone()),
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
    req: &WarehouseAreaUpdateRequest,
    updated_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = warehouse_area::Entity::update_many()
        .col_expr(warehouse_area::Column::WarehouseId, Expr::value(req.warehouse_id))
        .col_expr(warehouse_area::Column::AreaCode, Expr::value(req.area_code.clone()))
        .col_expr(warehouse_area::Column::AreaName, Expr::value(req.area_name.clone()))
        .col_expr(warehouse_area::Column::AreaType, Expr::value(req.area_type.clone()))
        .col_expr(warehouse_area::Column::Status, Expr::value(req.status))
        .col_expr(warehouse_area::Column::SortOrder, Expr::value(req.sort_order))
        .col_expr(warehouse_area::Column::Remark, Expr::value(req.remark.clone()))
        .col_expr(warehouse_area::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(warehouse_area::Column::UpdateTime, Expr::value(now))
        .filter(warehouse_area::Column::Id.eq(id))
        .filter(warehouse_area::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

pub async fn batch_delete<C: ConnectionTrait>(
    db: &C,
    ids: &[i64],
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = warehouse_area::Entity::update_many()
        .col_expr(warehouse_area::Column::Deleted, Expr::value(1))
        .col_expr(warehouse_area::Column::UpdateTime, Expr::value(now))
        .filter(warehouse_area::Column::Id.is_in(ids.iter().map(|&id| id).collect::<Vec<_>>()))
        .filter(warehouse_area::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}
