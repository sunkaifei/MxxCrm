//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 服务权益 Service
//!

use chrono::{Datelike, NaiveDate};
use sea_orm::{ColumnTrait, Condition, ConnectionTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait};

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::entity::entitlement::{Column as EntColumn, Entity as EntEntity};
use crate::modules::sale::model::entitlement::{
    entitlement_status_name, entitlement_type_name, remaining_days, EntitlementListQuery,
    EntitlementListVO, EntitlementModel, EntitlementRenewRequest, EntitlementSaveRequest,
};
use crate::modules::sale::model::order::{
    needs_entitlement, OrderItemModel, OrderModel, PRODUCT_TYPE_PHYSICAL, PRODUCT_TYPE_SERVICE,
    PRODUCT_TYPE_SUBSCRIPTION,
};

/// 列表
pub async fn get_list(
    db: &DbConn, query: &EntitlementListQuery
) -> Result<ResultPage<Vec<EntitlementListVO>>> {
    let page = query.page_num.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1);

    let mut cond = Condition::all();
    cond = cond.add(EntColumn::Deleted.eq(0));
    if let Some(cid) = query.customer_id { cond = cond.add(EntColumn::CustomerId.eq(cid)); }
    if let Some(oid) = query.order_id { cond = cond.add(EntColumn::OrderId.eq(oid)); }
    if let Some(s) = query.status { cond = cond.add(EntColumn::Status.eq(s)); }
    if let Some(t) = query.entitlement_type { cond = cond.add(EntColumn::EntitlementType.eq(t)); }

    let paginator = EntEntity::find()
        .filter(cond)
        .order_by_desc(EntColumn::Id)
        .paginate(db, page_size as u64);

    let total = paginator.num_items().await.map_err(|e| Error::from(e.to_string()))? as i64;
    let rows = paginator.fetch_page((page - 1) as u64).await
        .map_err(|e| Error::from(e.to_string()))?;

    let items: Vec<EntitlementListVO> = rows.into_iter().map(|m| {
        let end_date_naive = m.end_date;
        EntitlementListVO {
            id: m.id,
            entitlement_no: m.entitlement_no,
            order_id: m.order_id,
            customer_id: m.customer_id,
            product_id: m.product_id,
            product_name: m.product_name,
            entitlement_type: m.entitlement_type,
            entitlement_type_name: m.entitlement_type.map(|v| entitlement_type_name(v).to_string()),
            status: m.status,
            status_name: m.status.map(|v| entitlement_status_name(v).to_string()),
            start_date: m.start_date.map(|d| d.format("%Y-%m-%d").to_string()),
            end_date: m.end_date.map(|d| d.format("%Y-%m-%d").to_string()),
            duration_months: m.duration_months,
            auto_renew: m.auto_renew,
            renew_count: m.renew_count,
            remaining_days: remaining_days(end_date_naive),
            total_quota: m.total_quota,
            used_quota: m.used_quota,
            remaining_quota: m.remaining_quota,
            sla_level: m.sla_level,
            parent_entitlement_id: m.parent_entitlement_id,
            create_time: m.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }).collect();

    Ok(ResultPage::new(items, total, page, page_size))
}

/// 新建权益（手动）
pub async fn create(db: &DbConn, req: EntitlementSaveRequest, user_id: i64) -> Result<i64> {
    if req.customer_id.is_none() { return Err(Error::from("客户ID不能为空")); }
    if req.entitlement_type.is_none() { return Err(Error::from("权益类型不能为空")); }

    let date_prefix = format!("ENT{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = EntitlementModel::get_max_entitlement_no_today(db, &date_prefix).await
        .map_err(|e| Error::from(e.to_string()))?;
    let seq = max_seq.unwrap_or(0) + 1;
    let entitlement_no = format!("{}{:04}", date_prefix, seq);

    EntitlementModel::insert(db, &req, Some(entitlement_no), Some(user_id)).await
        .map_err(|e| Error::from(e.to_string()))
}

/// 订单确认时自动创建权益
pub async fn create_for_order<C: ConnectionTrait>(
    db: &C, order_id: i64
) -> Result<Vec<i64>> {
    let order = OrderModel::find_by_id(db, order_id).await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("订单不存在"))?;
    let items = OrderItemModel::find_by_order_id(db, order_id).await
        .map_err(|e| Error::from(e.to_string()))?;
    let mut entitlement_ids = Vec::new();

    for item in items.iter() {
        let product_type = item.product_type.unwrap_or(PRODUCT_TYPE_PHYSICAL);
        if !needs_entitlement(product_type) { continue; }

        let item_id = item.id;
        let duration = item.service_duration.unwrap_or(12);
        let start_date = item.service_start_date
            .or(Some(chrono::Local::now().date_naive()));
        let end_date = start_date.map(|s| add_months(s, duration));

        let date_prefix = format!("ENT{}", chrono::Local::now().format("%Y%m%d"));
        let max_seq = EntitlementModel::get_max_entitlement_no_today(db, &date_prefix).await
            .map_err(|e| Error::from(e.to_string()))?;
        let seq = max_seq.unwrap_or(0) + 1 + entitlement_ids.len() as i64;
        let entitlement_no = format!("{}{:04}", date_prefix, seq);

        let req = EntitlementSaveRequest {
            order_id: Some(order_id),
            order_item_id: Some(item_id),
            customer_id: order.customer_id,
            product_id: item.product_id,
            product_name: item.product_name.clone(),
            entitlement_type: Some(match product_type {
                PRODUCT_TYPE_SERVICE => 1,
                PRODUCT_TYPE_SUBSCRIPTION => 2,
                _ => 1,
            }),
            start_date,
            end_date,
            duration_months: Some(duration),
            auto_renew: order.auto_renew,
            total_quota: None,
            sla_level: None,
            response_time_hours: None,
            resolution_time_hours: None,
            remark: Some("订单确认自动创建".to_string()),
        };
        let eid = EntitlementModel::insert(db, &req, Some(entitlement_no), None).await
            .map_err(|e| Error::from(e.to_string()))?;
        entitlement_ids.push(eid);
    }

    Ok(entitlement_ids)
}

/// 增加 N 月（处理跨年/月末）
fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let total = date.year() * 12 + (date.month() as i32 - 1) + months;
    let new_year = total.div_euclid(12);
    let new_month = total.rem_euclid(12) + 1;
    let last_day_of_month = NaiveDate::from_ymd_opt(
        if new_month == 12 { new_year + 1 } else { new_year },
        if new_month == 12 { 1u32 } else { (new_month + 1) as u32 },
        1
    ).map(|d| d.pred_opt().unwrap_or(d).day())
     .unwrap_or(28);
    let day = date.day().min(last_day_of_month);
    NaiveDate::from_ymd_opt(new_year, new_month as u32, day).unwrap_or(date)
}

/// 到期检查
pub async fn check_expiring(db: &DbConn, days_before: i64) -> Result<u64> {
    let target_date = chrono::Local::now().date_naive() + chrono::Duration::days(days_before);
    let expiring = EntitlementModel::find_expiring(db, target_date).await
        .map_err(|e| Error::from(e.to_string()))?;
    let count = expiring.len() as u64;
    // TODO: 集成通知服务
    Ok(count)
}

/// 续约
pub async fn renew(db: &DbConn, req: EntitlementRenewRequest, user_id: i64) -> Result<i64> {
    let old = EntitlementModel::find_by_id(db, req.old_entitlement_id).await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("权益不存在"))?;

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    EntitlementModel::update_status(&txn, req.old_entitlement_id, 4).await
        .map_err(|e| Error::from(e.to_string()))?;

    let date_prefix = format!("ENT{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = EntitlementModel::get_max_entitlement_no_today(&txn, &date_prefix).await
        .map_err(|e| Error::from(e.to_string()))?;
    let seq = max_seq.unwrap_or(0) + 1;
    let entitlement_no = format!("{}{:04}", date_prefix, seq);

    let new_start = chrono::Local::now().date_naive();
    let duration = old.duration_months.unwrap_or(12);
    let new_end = add_months(new_start, duration);

    let new_req = EntitlementSaveRequest {
        order_id: Some(req.new_order_id),
        order_item_id: old.order_item_id,
        customer_id: old.customer_id,
        product_id: old.product_id,
        product_name: old.product_name.clone(),
        entitlement_type: old.entitlement_type,
        start_date: Some(new_start),
        end_date: Some(new_end),
        duration_months: old.duration_months,
        auto_renew: old.auto_renew,
        total_quota: old.total_quota,
        sla_level: old.sla_level.clone(),
        response_time_hours: old.response_time_hours,
        resolution_time_hours: old.resolution_time_hours,
        remark: Some(format!("续约自权益 #{}", req.old_entitlement_id)),
    };
    let new_id = EntitlementModel::insert(&txn, &new_req, Some(entitlement_no), Some(user_id)).await
        .map_err(|e| Error::from(e.to_string()))?;

    EntitlementModel::link_renewal(&txn, new_id, req.old_entitlement_id).await
        .map_err(|e| Error::from(e.to_string()))?;

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;

    Ok(new_id)
}

/// 修改状态
pub async fn update_status(db: &DbConn, id: i64, status: i32) -> Result<i64> {
    EntitlementModel::update_status(db, id, status).await
        .map_err(|e| Error::from(e.to_string()))
}

/// 按客户查询
pub async fn find_by_customer(db: &DbConn, customer_id: i64) -> Result<Vec<EntitlementListVO>> {
    let ents = EntitlementModel::find_by_customer(db, customer_id).await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(ents.into_iter().map(|m| {
        let end_date_naive = m.end_date;
        EntitlementListVO {
            id: m.id,
            entitlement_no: m.entitlement_no,
            order_id: m.order_id,
            customer_id: m.customer_id,
            product_id: m.product_id,
            product_name: m.product_name,
            entitlement_type: m.entitlement_type,
            entitlement_type_name: m.entitlement_type.map(|v| entitlement_type_name(v).to_string()),
            status: m.status,
            status_name: m.status.map(|v| entitlement_status_name(v).to_string()),
            start_date: m.start_date.map(|d| d.format("%Y-%m-%d").to_string()),
            end_date: m.end_date.map(|d| d.format("%Y-%m-%d").to_string()),
            duration_months: m.duration_months,
            auto_renew: m.auto_renew,
            renew_count: m.renew_count,
            remaining_days: remaining_days(end_date_naive),
            total_quota: m.total_quota,
            used_quota: m.used_quota,
            remaining_quota: m.remaining_quota,
            sla_level: m.sla_level,
            parent_entitlement_id: m.parent_entitlement_id,
            create_time: m.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }).collect())
}
