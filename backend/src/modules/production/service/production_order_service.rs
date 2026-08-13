//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::model::inbound::{InboundItemRequest, InboundSaveRequest};
use crate::modules::inventory::service::inbound_service;
use crate::modules::inventory::entity::warehouse::{self, Entity as Warehouse};
use crate::modules::production::model::production_order::{
    production_order_status, ProductionOrderDetailVO, ProductionOrderListQuery, ProductionOrderListVO, ProductionOrderModel, ProductionOrderSaveRequest,
};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, QueryOrder};

pub async fn insert(db: &DbConn, form_data: &ProductionOrderSaveRequest) -> Result<i64> {
    let id = ProductionOrderModel::insert(db, form_data)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

pub async fn update(db: &DbConn, id: i64, form_data: &ProductionOrderSaveRequest) -> Result<i64> {
    let existing = ProductionOrderModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("生产工单不存在"))?;

    if existing.status != Some(production_order_status::DRAFT) {
        return Err(Error::from("仅草稿状态的生产工单可编辑"));
    }

    let result = ProductionOrderModel::update_by_id(db, id, form_data)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

pub async fn batch_delete(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let result = ProductionOrderModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn get_info(db: &DbConn, id: i64) -> Result<ProductionOrderDetailVO> {
    let result = ProductionOrderModel::find_by_id(db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("生产工单不存在")),
    }
}

pub async fn get_list(db: &DbConn, query: &ProductionOrderListQuery) -> Result<(Vec<ProductionOrderListVO>, i64, i64)> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);
    let (list, total_pages) = ProductionOrderModel::select_in_page(
        db,
        page_num,
        page_size,
        query.keywords.clone(),
        query.status,
        query.product_id,
    ).await?;

    let total = ProductionOrderModel::select_count(
        db,
        query.keywords.clone(),
        query.status,
        query.product_id,
    ).await?;

    let list: Vec<ProductionOrderListVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((list, total, total_pages))
}

/// 下达生产工单
pub async fn release(db: &DbConn, id: i64) -> Result<()> {
    let existing = ProductionOrderModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("生产工单不存在"))?;

    if existing.status != Some(production_order_status::DRAFT) {
        return Err(Error::from("仅草稿状态的生产工单可下达"));
    }

    ProductionOrderModel::update_status(db, id, production_order_status::RELEASED)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 开始生产
pub async fn start(db: &DbConn, id: i64) -> Result<()> {
    let existing = ProductionOrderModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("生产工单不存在"))?;

    if existing.status != Some(production_order_status::RELEASED) {
        return Err(Error::from("仅已下达状态的生产工单可开始生产"));
    }

    ProductionOrderModel::update_status(db, id, production_order_status::IN_PROGRESS)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 完成生产
pub async fn complete(db: &DbConn, id: i64) -> Result<()> {
    let existing = ProductionOrderModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("生产工单不存在"))?;

    if existing.status != Some(production_order_status::IN_PROGRESS) {
        return Err(Error::from("仅生产中的工单可完成"));
    }

    ProductionOrderModel::update_status(db, id, production_order_status::COMPLETED)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 入库（完工入库：创建入库单并自动审核，然后更新工单状态）
pub async fn inbound(db: &DbConn, id: i64) -> Result<()> {
    let existing = ProductionOrderModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("生产工单不存在"))?;

    if existing.status != Some(production_order_status::COMPLETED) {
        return Err(Error::from("仅已完成状态的工单可入库"));
    }

    let product_id = existing.product_id
        .ok_or_else(|| Error::from("生产工单缺少产品信息，无法入库"))?;

    // 入库数量优先使用完工数量，回退到计划数量
    let inbound_qty = existing
        .completed_quantity
        .filter(|q| *q > rust_decimal::Decimal::ZERO)
        .or(existing.quantity.clone())
        .ok_or_else(|| Error::from("生产工单缺少入库数量"))?;

    // 查询可用仓库（优先取启用的第一个仓库）
    let warehouse = Warehouse::find()
        .filter(warehouse::Column::Deleted.eq(0))
        .filter(warehouse::Column::IsActive.eq(true))
        .order_by_asc(warehouse::Column::Id)
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("未找到可用仓库，无法入库"))?;
    let warehouse_id = warehouse.id;

    let mo_no = existing.mo_no.clone().unwrap_or_default();
    let operator = existing.created_by.unwrap_or(0);

    // 构造入库单请求
    let inbound_req = InboundSaveRequest {
        inbound_type: "production".to_string(),
        warehouse_id,
        source_order_id: Some(id),
        source_order_no: Some(mo_no.clone()),
        total_quantity: Some(inbound_qty),
        total_amount: existing.cost_amount.clone(),
        remark: Some(format!("由生产工单[{}]完工入库", mo_no)),
        items: vec![InboundItemRequest {
            product_id,
            product_sku: None,
            quantity: inbound_qty,
            unit_price: None,
            amount: None,
            batch_no: None,
            remark: None,
        }],
    };

    // 创建入库单
    let inbound_id = inbound_service::create(db, &inbound_req, operator).await?;

    // 自动审核入库单（更新库存 + 写流水，系统自动完成不走审批引擎）
    inbound_service::do_complete_audit(db, inbound_id, operator).await?;

    // 更新工单状态为已入库
    ProductionOrderModel::update_status(db, id, production_order_status::INBOUNDED)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 关闭生产工单
pub async fn close(db: &DbConn, id: i64) -> Result<()> {
    let existing = ProductionOrderModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("生产工单不存在"))?;

    if existing.status == Some(production_order_status::CLOSED) || existing.status == Some(production_order_status::CANCELLED) {
        return Err(Error::from("生产工单已关闭或已取消"));
    }

    ProductionOrderModel::update_status(db, id, production_order_status::CLOSED)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}