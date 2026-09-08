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
use crate::modules::purchase::model::purchase_requisition::{
    RequisitionItemDTO, RequisitionSaveRequest,
};
use crate::modules::purchase::model::purchase_stock_plan::{
    stock_plan_status, StockPlanDetailVO, StockPlanListQuery, StockPlanListVO, StockPlanModel, StockPlanSaveRequest,
};
use crate::modules::purchase::service::purchase_requisition_service;
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &StockPlanSaveRequest) -> Result<i64> {
    let id = StockPlanModel::insert(db, form_data)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

pub async fn update(db: &DbConn, id: i64, form_data: &StockPlanSaveRequest) -> Result<i64> {
    let existing = StockPlanModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("备货计划不存在"))?;

    if existing.status != Some(stock_plan_status::DRAFT) {
        return Err(Error::from("仅草稿状态的备货计划可编辑"));
    }

    let result = StockPlanModel::update_by_id(db, id, form_data)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

pub async fn batch_delete(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let result = StockPlanModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn get_info(db: &DbConn, id: i64) -> Result<StockPlanDetailVO> {
    let result = StockPlanModel::find_by_id(db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("备货计划不存在")),
    }
}

pub async fn get_list(db: &DbConn, query: &StockPlanListQuery) -> Result<(Vec<StockPlanListVO>, i64, i64)> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);
    let (list, total_pages) = StockPlanModel::select_in_page(
        db,
        page_num,
        page_size,
        query.keywords.clone(),
        query.status,
        query.product_id,
    ).await?;

    let total = StockPlanModel::select_count(
        db,
        query.keywords.clone(),
        query.status,
        query.product_id,
    ).await?;

    let list: Vec<StockPlanListVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((list, total, total_pages))
}

/// 生成采购申请（将备货计划转为采购申请单）
pub async fn generate_pr(db: &DbConn, id: i64) -> Result<i64> {
    let existing = StockPlanModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("备货计划不存在"))?;

    // 构造采购申请单
    let plan_no = existing.plan_no.clone().unwrap_or_default();
    let quantity = existing
        .net_demand
        .or(existing.suggested_quantity.clone())
        .or(existing.demand_quantity.clone());
    let pr_req = RequisitionSaveRequest {
        id: None,
        pr_type: Some("stock_plan".to_string()),
        title: Some(format!("备货计划[{}]生成的采购申请", plan_no)),
        department_id: None,
        requester_id: existing.created_by,
        expected_date: existing.suggested_order_date,
        urgency: Some("normal".to_string()),
        total_amount: None,
        currency: None,
        reason: Some(format!("由备货计划[{}]自动生成", plan_no)),
        remark: existing.remark.clone(),
        items: vec![RequisitionItemDTO {
            product_id: existing.product_id,
            product_name: None,
            product_sku: None,
            spec: None,
            unit: None,
            quantity,
            estimated_price: None,
            estimated_amount: None,
            remark: None,
        }],
    };

    let operator = existing.created_by.unwrap_or(0);

    // 创建采购申请单
    let pr_id = purchase_requisition_service::insert(db, &pr_req, operator).await?;

    // 回填 actual_pr_id 并更新状态为已生成
    StockPlanModel::update_actual_pr_id_and_status(db, id, pr_id, stock_plan_status::GENERATED)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(id)
}

/// 重新计算净需求
pub async fn recalculate(db: &DbConn, id: i64) -> Result<StockPlanDetailVO> {
    let existing = StockPlanModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("备货计划不存在"))?;

    let net_demand = StockPlanModel::calculate_net_demand(existing.demand_quantity, existing.available_quantity);

    let update_req = StockPlanSaveRequest {
        net_demand,
        ..StockPlanSaveRequest {
            plan_no: existing.plan_no.clone(),
            product_id: existing.product_id,
            plan_date: existing.plan_date,
            demand_quantity: existing.demand_quantity,
            demand_source: existing.demand_source.clone(),
            source_type: existing.source_type.clone(),
            source_id: existing.source_id,
            available_quantity: existing.available_quantity,
            net_demand,
            safety_stock: existing.safety_stock,
            suggested_order_date: existing.suggested_order_date,
            suggested_quantity: existing.suggested_quantity,
            supplier_id: existing.supplier_id,
            lead_time_days: existing.lead_time_days,
            status: existing.status,
            actual_pr_id: existing.actual_pr_id,
            remark: existing.remark.clone(),
        }
    };

    StockPlanModel::update_by_id(db, id, &update_req)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    get_info(db, id).await
}

/// 当销售订单变更/取消时，更新关联的备货计划状态
pub async fn sync_order_change(db: &DbConn, order_id: i64, new_status: &str) -> Result<()> {
    use crate::modules::purchase::entity::purchase_stock_plan::{self, Entity as StockPlan};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let plans = StockPlan::find()
        .filter(purchase_stock_plan::Column::SourceType.eq("order"))
        .filter(purchase_stock_plan::Column::SourceId.eq(order_id))
        .filter(purchase_stock_plan::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if plans.is_empty() {
        return Ok(());
    }

    // 订单取消时，将备货计划状态更新为"已取消"
    if new_status == "cancelled" {
        use sea_orm::ActiveModelTrait;
        use sea_orm::ActiveValue::Set;

        for plan in plans {
            let mut active: purchase_stock_plan::ActiveModel = plan.into();
            active.status = Set(Some(stock_plan_status::CANCELLED));
            active.update(db).await.map_err(|e| Error::from(e.to_string()))?;
        }
    }

    Ok(())
}

/// 获取预警列表（需求日期临近且未处理的计划）
pub async fn get_warning_list(db: &DbConn) -> Result<Vec<StockPlanListVO>> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use crate::modules::purchase::entity::purchase_stock_plan;

    let list = purchase_stock_plan::Entity::find()
        .filter(purchase_stock_plan::Column::Deleted.eq(0))
        .filter(purchase_stock_plan::Column::Status.eq(stock_plan_status::DRAFT))
        .filter(purchase_stock_plan::Column::SuggestedOrderDate.is_not_null())
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(list.into_iter().map(|m| m.into()).collect())
}