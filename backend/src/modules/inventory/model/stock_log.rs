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
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use crate::modules::inventory::entity::stock_log;

// 库存流水查询参数
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StockLogListQuery {
    pub page_num: u64,
    pub page_size: u64,
    pub product_id: Option<i64>,
    pub warehouse_id: Option<i64>,
    pub change_type: Option<String>,
    pub start_time: Option<chrono::NaiveDateTime>,
    pub end_time: Option<chrono::NaiveDateTime>,
}

// 库存流水列表VO
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StockLogListVO {
    pub list: Vec<StockLogListItem>,
    pub total: u64,
}

// 库存流水列表项
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StockLogListItem {
    pub id: i64,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    pub change_type: Option<String>,
    pub biz_type: Option<String>,
    pub biz_id: Option<i64>,
    pub biz_no: Option<String>,
    pub quantity_before: Option<Decimal>,
    pub change_quantity: Option<Decimal>,
    pub quantity_after: Option<Decimal>,
    pub operator_id: Option<i64>,
    pub operator_name: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<chrono::NaiveDateTime>,
}

impl From<stock_log::Model> for StockLogListItem {
    fn from(m: stock_log::Model) -> Self {
        Self {
            id: m.id,
            product_id: m.product_id,
            product_name: None,
            warehouse_id: m.warehouse_id,
            warehouse_name: None,
            change_type: m.change_type,
            biz_type: m.biz_type,
            biz_id: m.biz_id,
            biz_no: m.biz_no,
            quantity_before: m.quantity_before,
            change_quantity: m.change_quantity,
            quantity_after: m.quantity_after,
            operator_id: m.operator_id,
            operator_name: None,
            remark: m.remark,
            create_time: m.create_time,
        }
    }
}

// DB helper functions
pub async fn select_page<C: ConnectionTrait>(
    db: &C,
    query: &StockLogListQuery,
) -> Result<(Vec<stock_log::Model>, u64), DbErr> {
    let mut q = stock_log::Entity::find();

    if let Some(pid) = query.product_id {
        q = q.filter(stock_log::Column::ProductId.eq(pid));
    }
    if let Some(wid) = query.warehouse_id {
        q = q.filter(stock_log::Column::WarehouseId.eq(wid));
    }
    if let Some(ref ct) = query.change_type {
        q = q.filter(stock_log::Column::ChangeType.eq(ct));
    }
    if let Some(st) = query.start_time {
        q = q.filter(stock_log::Column::CreateTime.gte(st));
    }
    if let Some(et) = query.end_time {
        q = q.filter(stock_log::Column::CreateTime.lte(et));
    }

    let total = q.clone().count(db).await?;
    let rows = q
        .order_by_desc(stock_log::Column::CreateTime)
        .offset((query.page_num - 1) * query.page_size)
        .limit(query.page_size)
        .all(db)
        .await?;

    Ok((rows, total))
}