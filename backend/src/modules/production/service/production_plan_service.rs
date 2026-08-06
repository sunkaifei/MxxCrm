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
use crate::modules::production::model::production_order::{
    production_order_status, ProductionOrderSaveRequest,
};
use crate::modules::production::model::production_plan::{
    production_plan_status, ProductionPlanDetailVO, ProductionPlanListQuery, ProductionPlanListVO, ProductionPlanModel, ProductionPlanSaveRequest,
};
use crate::modules::production::service::production_order_service;
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &ProductionPlanSaveRequest) -> Result<i64> {
    let id = ProductionPlanModel::insert(db, form_data)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

pub async fn update(db: &DbConn, id: i64, form_data: &ProductionPlanSaveRequest) -> Result<i64> {
    let existing = ProductionPlanModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("生产计划不存在"))?;

    if existing.status != Some(production_plan_status::PENDING) {
        return Err(Error::from("仅待处理状态的生产计划可编辑"));
    }

    let result = ProductionPlanModel::update_by_id(db, id, form_data)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

pub async fn batch_delete(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let result = ProductionPlanModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn get_info(db: &DbConn, id: i64) -> Result<ProductionPlanDetailVO> {
    let result = ProductionPlanModel::find_by_id(db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("生产计划不存在")),
    }
}

pub async fn get_list(db: &DbConn, query: &ProductionPlanListQuery) -> Result<(Vec<ProductionPlanListVO>, i64, i64)> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);
    let (list, total_pages) = ProductionPlanModel::select_in_page(
        db,
        page_num,
        page_size,
        query.keywords.clone(),
        query.status,
        query.product_id,
    ).await?;

    let total = ProductionPlanModel::select_count(
        db,
        query.keywords.clone(),
        query.status,
        query.product_id,
    ).await?;

    let list: Vec<ProductionPlanListVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((list, total, total_pages))
}

/// 生成生产订单（将生产计划转为生产工单）
pub async fn generate_mo(db: &DbConn, id: i64) -> Result<i64> {
    let existing = ProductionPlanModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("生产计划不存在"))?;

    let plan_no = existing.plan_no.clone().unwrap_or_default();

    // 构造生产工单
    let mo_req = ProductionOrderSaveRequest {
        mo_no: None,
        product_id: existing.product_id,
        product_name: None,
        quantity: existing
            .suggested_quantity
            .clone()
            .or(existing.net_demand.clone())
            .or(existing.demand_quantity.clone()),
        completed_quantity: None,
        plan_start_date: existing.suggested_start_date,
        plan_complete_date: None,
        actual_complete_date: None,
        source_type: Some("plan".to_string()),
        source_id: Some(id),
        source_no: Some(plan_no.clone()),
        workshop_id: None,
        production_lead_time: None,
        status: Some(production_order_status::DRAFT),
        cost_amount: None,
        remark: existing.remark.clone(),
    };

    // 创建生产工单
    let mo_id = production_order_service::insert(db, &mo_req).await?;

    // 回填 actual_mo_id 并更新状态为已生成
    ProductionPlanModel::update_actual_mo_id_and_status(db, id, mo_id, production_plan_status::GENERATED)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(id)
}