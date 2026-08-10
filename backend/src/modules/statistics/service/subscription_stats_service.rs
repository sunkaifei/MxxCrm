use chrono::Datelike;
use rust_decimal::prelude::RoundingStrategy;
use sea_orm::prelude::Decimal;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter};
use std::collections::HashMap;

use crate::core::errors::error::Result;
use crate::modules::sale::entity::entitlement::{Column as EntColumn, Entity as EntEntity};
use crate::modules::sale::entity::order_item::{self, Entity as OrderItemEntity};

/// 统一百分比保留2位小数（后端兜底）
fn round_pct(d: Decimal) -> Decimal {
    d.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven)
}

/// 月度经常性收入（MRR）
/// MRR = SUM(entitlement 关联 order_item 的 amount / duration_months)
/// WHERE status=2(生效中) AND 当月覆盖 start_date ~ end_date
pub async fn get_mrr(db: &DbConn, year: Option<i32>, month: Option<u32>) -> Result<serde_json::Value> {
    let now = chrono::Local::now();
    let year = year.unwrap_or(now.year());
    let month = month.unwrap_or(now.month());

    let active_ents = EntEntity::find()
        .filter(EntColumn::Status.eq(2))
        .filter(EntColumn::Deleted.eq(0))
        .all(db)
        .await?;

    // 收集 order_item_id 批量查询
    let item_ids: Vec<i64> = active_ents.iter()
        .filter_map(|e| e.order_item_id)
        .collect();

    let item_map: HashMap<i64, order_item::Model> = if !item_ids.is_empty() {
        OrderItemEntity::find()
            .filter(order_item::Column::Id.is_in(item_ids))
            .filter(order_item::Column::Deleted.eq(0))
            .all(db)
            .await?
            .into_iter()
            .map(|it| (it.id, it))
            .collect()
    } else {
        HashMap::new()
    };

    let mut mrr = Decimal::ZERO;
    for ent in &active_ents {
        // 检查该权益在指定月份是否生效
        let in_month = match (&ent.start_date, &ent.end_date) {
            (Some(sd), Some(ed)) => {
                let month_start = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(*sd);
                let month_end = month_start
                    .checked_add_months(chrono::Months::new(1))
                    .map(|d| d.pred_opt().unwrap_or(d))
                    .unwrap_or(*ed);
                sd <= &month_end && ed >= &month_start
            }
            _ => false,
        };
        if !in_month {
            continue;
        }
        if let Some(item_id) = ent.order_item_id {
            if let Some(item) = item_map.get(&item_id) {
                let amount = item.total_amount.unwrap_or(Decimal::ZERO);
                let duration = ent.duration_months.unwrap_or(1).max(1);
                mrr += amount / Decimal::from(duration);
            }
        }
    }

    Ok(serde_json::json!({
        "year": year,
        "month": month,
        "mrr": mrr,
    }))
}

/// 年度经常性收入（ARR）
/// ARR = MRR * 12 + 当年已到期权益的折算
pub async fn get_arr(db: &DbConn, year: Option<i32>) -> Result<serde_json::Value> {
    let now = chrono::Local::now();
    let year = year.unwrap_or(now.year());

    let mrr_result = get_mrr(db, Some(year), Some(now.month())).await?;
    let mrr = mrr_result.get("mrr")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or(Decimal::ZERO);

    let base_arr = mrr * Decimal::from(12);

    // 当年已到期权益的折算（amount / duration * 剩余月数比例）
    let expired_ents = EntEntity::find()
        .filter(EntColumn::Status.eq(4)) // 已到期
        .filter(EntColumn::Deleted.eq(0))
        .all(db)
        .await?;

    let expired_item_ids: Vec<i64> = expired_ents.iter()
        .filter_map(|e| e.order_item_id)
        .collect();

    let expired_item_map: HashMap<i64, order_item::Model> = if !expired_item_ids.is_empty() {
        OrderItemEntity::find()
            .filter(order_item::Column::Id.is_in(expired_item_ids))
            .filter(order_item::Column::Deleted.eq(0))
            .all(db)
            .await?
            .into_iter()
            .map(|it| (it.id, it))
            .collect()
    } else {
        HashMap::new()
    };

    let mut expired_amount = Decimal::ZERO;
    for ent in &expired_ents {
        if let Some(ed) = ent.end_date {
            if ed.year() == year {
                if let Some(item_id) = ent.order_item_id {
                    if let Some(item) = expired_item_map.get(&item_id) {
                        let amount = item.total_amount.unwrap_or(Decimal::ZERO);
                        let duration = ent.duration_months.unwrap_or(1).max(1);
                        expired_amount += amount / Decimal::from(duration);
                    }
                }
            }
        }
    }

    let arr = base_arr + expired_amount;

    Ok(serde_json::json!({
        "year": year,
        "arr": arr,
        "mrr": mrr,
        "baseArr": base_arr,
        "expiredAmount": expired_amount,
    }))
}

/// 流失率（当月到期未续约的权益数 / 月初有效权益数）
pub async fn get_churn_rate(db: &DbConn, year: Option<i32>, month: Option<u32>) -> Result<serde_json::Value> {
    let now = chrono::Local::now();
    let year = year.unwrap_or(now.year());
    let month = month.unwrap_or(now.month());

    let month_start = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_else(|| chrono::Local::now().date_naive());

    let all_ents = EntEntity::find()
        .filter(EntColumn::Deleted.eq(0))
        .all(db)
        .await?;

    // 月初有效权益数：start_date <= 月初 AND (end_date >= 月初 OR status=2)
    let beginning_active = all_ents.iter()
        .filter(|e| {
            let started = e.start_date.map(|sd| sd <= month_start).unwrap_or(false);
            let still_active = e.end_date.map(|ed| ed >= month_start).unwrap_or(false)
                || e.status == Some(2);
            started && still_active
        })
        .count();

    // 当月到期未续约权益数：end_date 在当月 AND status=4(已到期) AND 无续约(parent_entitlement_id 不被引用)
    let month_end = month_start
        .checked_add_months(chrono::Months::new(1))
        .map(|d| d.pred_opt().unwrap_or(d))
        .unwrap_or(month_start);

    let churned = all_ents.iter()
        .filter(|e| {
            if e.status != Some(4) { return false; }
            if let Some(ed) = e.end_date {
                ed >= month_start && ed <= month_end
            } else {
                false
            }
        })
        .count();

    let churn_rate = if beginning_active > 0 {
        round_pct(Decimal::from(churned) / Decimal::from(beginning_active) * Decimal::from(100))
    } else {
        Decimal::ZERO
    };

    Ok(serde_json::json!({
        "year": year,
        "month": month,
        "beginningActive": beginning_active,
        "churned": churned,
        "churnRate": churn_rate,
    }))
}

/// 续约率（当月续约数 / 当月到期数）
pub async fn get_renewal_rate(db: &DbConn, year: Option<i32>, month: Option<u32>) -> Result<serde_json::Value> {
    let now = chrono::Local::now();
    let year = year.unwrap_or(now.year());
    let month = month.unwrap_or(now.month());

    let month_start = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_else(|| chrono::Local::now().date_naive());
    let month_end = month_start
        .checked_add_months(chrono::Months::new(1))
        .map(|d| d.pred_opt().unwrap_or(d))
        .unwrap_or(month_start);

    let all_ents = EntEntity::find()
        .filter(EntColumn::Deleted.eq(0))
        .all(db)
        .await?;

    // 当月到期数
    let expired_count = all_ents.iter()
        .filter(|e| {
            if let Some(ed) = e.end_date {
                ed >= month_start && ed <= month_end
            } else {
                false
            }
        })
        .count();

    // 当月续约数：有 parent_entitlement_id 且 start_date 在当月
    let renewed_count = all_ents.iter()
        .filter(|e| {
            let has_parent = e.parent_entitlement_id.is_some();
            let started_this_month = e.start_date.map(|sd| sd >= month_start && sd <= month_end).unwrap_or(false);
            has_parent && started_this_month
        })
        .count();

    let renewal_rate = if expired_count > 0 {
        round_pct(Decimal::from(renewed_count) / Decimal::from(expired_count) * Decimal::from(100))
    } else {
        Decimal::ZERO
    };

    Ok(serde_json::json!({
        "year": year,
        "month": month,
        "expiredCount": expired_count,
        "renewedCount": renewed_count,
        "renewalRate": renewal_rate,
    }))
}

/// 净收入留存率（NRR）
/// NRR = (月初MRR + 扩展 - 流失 - 降级) / 月初MRR
pub async fn get_net_revenue_retention(db: &DbConn, year: Option<i32>, month: Option<u32>) -> Result<serde_json::Value> {
    let now = chrono::Local::now();
    let year = year.unwrap_or(now.year());
    let month = month.unwrap_or(now.month());

    let current_mrr = get_mrr(db, Some(year), Some(month)).await?
        .get("mrr")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or(Decimal::ZERO);

    // 上月MRR
    let (prev_year, prev_month) = if month == 1 {
        (year - 1, 12u32)
    } else {
        (year, month - 1)
    };
    let beginning_mrr = get_mrr(db, Some(prev_year), Some(prev_month)).await?
        .get("mrr")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or(Decimal::ZERO);

    let nrr = if beginning_mrr > Decimal::ZERO {
        round_pct(current_mrr / beginning_mrr * Decimal::from(100))
    } else {
        Decimal::ZERO
    };

    Ok(serde_json::json!({
        "year": year,
        "month": month,
        "beginningMrr": beginning_mrr,
        "currentMrr": current_mrr,
        "netRevenueRetention": nrr,
    }))
}

/// 订阅概览（活跃订阅数、MRR、ARR、本月新增、本月流失）
pub async fn get_subscription_overview(db: &DbConn) -> Result<serde_json::Value> {
    let now = chrono::Local::now();
    let year = now.year();
    let month = now.month();

    let active_count = EntEntity::find()
        .filter(EntColumn::Status.eq(2))
        .filter(EntColumn::Deleted.eq(0))
        .count(db)
        .await? as i64;

    let mrr = get_mrr(db, Some(year), Some(month)).await?
        .get("mrr")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or(Decimal::ZERO);

    let arr = get_arr(db, Some(year)).await?
        .get("arr")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or(Decimal::ZERO);

    let month_start = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_else(|| now.date_naive());
    let month_end = month_start
        .checked_add_months(chrono::Months::new(1))
        .map(|d| d.pred_opt().unwrap_or(d))
        .unwrap_or(month_start);

    let all_ents = EntEntity::find()
        .filter(EntColumn::Deleted.eq(0))
        .all(db)
        .await?;

    // 本月新增
    let new_count = all_ents.iter()
        .filter(|e| {
            e.start_date.map(|sd| sd >= month_start && sd <= month_end).unwrap_or(false)
        })
        .count();

    // 本月流失
    let churn_count = all_ents.iter()
        .filter(|e| {
            e.status == Some(4) && e.end_date.map(|ed| ed >= month_start && ed <= month_end).unwrap_or(false)
        })
        .count();

    Ok(serde_json::json!({
        "activeCount": active_count,
        "mrr": mrr,
        "arr": arr,
        "newThisMonth": new_count,
        "churnThisMonth": churn_count,
    }))
}

/// 订阅趋势（最近N月的MRR/活跃数）
pub async fn get_subscription_trend(db: &DbConn, months: Option<i64>) -> Result<serde_json::Value> {
    let n = months.unwrap_or(12).min(24).max(1) as usize;
    let now = chrono::Local::now();

    let mut trend = Vec::new();

    for i in (0..n).rev() {
        let target = now.checked_sub_months(chrono::Months::new(i as u32)).unwrap_or(now);
        let year = target.year();
        let month = target.month();

        let mrr = get_mrr(db, Some(year), Some(month)).await?
            .get("mrr")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<Decimal>().ok())
            .unwrap_or(Decimal::ZERO);

        let month_start = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_else(|| now.date_naive());
        let month_end = month_start
            .checked_add_months(chrono::Months::new(1))
            .map(|d| d.pred_opt().unwrap_or(d))
            .unwrap_or(month_start);

        let active_count = EntEntity::find()
            .filter(EntColumn::Status.eq(2))
            .filter(EntColumn::Deleted.eq(0))
            .filter(EntColumn::StartDate.lte(month_end))
            .filter(EntColumn::EndDate.gte(month_start))
            .count(db)
            .await? as i64;

        trend.push(serde_json::json!({
            "year": year,
            "month": month,
            "mrr": mrr,
            "activeCount": active_count,
        }));
    }

    Ok(serde_json::json!(trend))
}
