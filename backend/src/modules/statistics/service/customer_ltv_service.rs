//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 客户 LTV 分析业务逻辑层
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::crm::entity::customer::{self, Entity as CustomerEntity};
use crate::modules::sale::entity::order::{Entity as OrderEntity, Column as OrderColumn};
use rust_decimal::Decimal;
use rust_decimal::prelude::RoundingStrategy;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};
use serde::Serialize;
use std::collections::HashMap;

/// 保留2位小数
fn round2(d: Decimal) -> Decimal {
    d.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven)
}

#[derive(Debug, Serialize)]
pub struct CustomerLtvVO {
    pub customer_id: i64,
    pub customer_name: Option<String>,
    pub total_amount: Decimal,
    pub estimated_profit: Decimal,
    pub first_purchase_date: Option<String>,
    pub last_purchase_date: Option<String>,
    pub purchase_count: i64,
    pub avg_order_amount: Decimal,
    pub ltv_score: Decimal,
}

#[derive(Debug, Serialize)]
pub struct RepurchaseRateVO {
    pub year: i32,
    pub month: i32,
    pub total_customers: i64,
    pub repurchase_customers: i64,
    pub repurchase_rate: Decimal,
}

#[derive(Debug, Serialize)]
pub struct TopLtvCustomerVO {
    pub customer_id: i64,
    pub customer_name: Option<String>,
    pub total_amount: Decimal,
    pub purchase_count: i64,
    pub avg_order_amount: Decimal,
    pub ltv_score: Decimal,
}

/// 返回单个客户的 LTV：总成交额、总利润(估)、首次购买日、最近购买日、购买次数、平均订单金额、LTV 评分
pub async fn get_customer_ltv(db: &DbConn, customer_id: i64) -> Result<CustomerLtvVO> {
    // 查询客户信息
    let customer = CustomerEntity::find_by_id(customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("客户不存在"))?;

    // 查询该客户的所有有效订单（排除草稿0、已取消9）
    let orders = OrderEntity::find()
        .filter(OrderColumn::CustomerId.eq(customer_id))
        .filter(OrderColumn::Deleted.eq(0))
        .filter(OrderColumn::OrderStatus.ne(0))
        .all(db)
        .await?;

    let purchase_count = orders.len() as i64;
    let total_amount: Decimal = orders.iter()
        .map(|o| o.total_amount.unwrap_or(Decimal::ZERO))
        .sum();

    // 总利润(估)：按 15% 毛利率估算
    let estimated_profit = round2(total_amount * Decimal::new(15, 2));

    let first_purchase_date = orders.iter()
        .filter_map(|o| o.create_time)
        .min()
        .map(|dt| dt.format("%Y-%m-%d").to_string());

    let last_purchase_date = orders.iter()
        .filter_map(|o| o.create_time)
        .max()
        .map(|dt| dt.format("%Y-%m-%d").to_string());

    let avg_order_amount = if purchase_count > 0 {
        round2(total_amount / Decimal::from(purchase_count))
    } else {
        Decimal::ZERO
    };

    // LTV 评分：总成交额权重 70% + 购买次数权重 30%，归一化到 0-100
    // 评分 = (总成交额 / 10000) * 0.7 + (购买次数 * 5) * 0.3，上限 100
    let amount_score = total_amount / Decimal::from(10000);
    let frequency_score = Decimal::from(purchase_count * 5);
    let ltv_score = round2(
        (amount_score * Decimal::new(7, 1) + frequency_score * Decimal::new(3, 1))
            .min(Decimal::from(100))
    );

    let customer_name = customer.company_name.or(customer.short_name).or(customer.person_name);

    Ok(CustomerLtvVO {
        customer_id,
        customer_name,
        total_amount: round2(total_amount),
        estimated_profit,
        first_purchase_date,
        last_purchase_date,
        purchase_count,
        avg_order_amount,
        ltv_score,
    })
}

/// 复购率：当月有2+订单的客户数 / 当月有订单的客户数
pub async fn get_repurchase_rate(db: &DbConn, year: i32, month: i32) -> Result<RepurchaseRateVO> {
    let (start, end) = month_range(year, month);

    let orders = OrderEntity::find()
        .filter(OrderColumn::Deleted.eq(0))
        .filter(OrderColumn::OrderStatus.ne(0))
        .filter(OrderColumn::CreateTime.gte(start))
        .filter(OrderColumn::CreateTime.lt(end))
        .all(db)
        .await?;

    let mut order_count_by_customer: HashMap<i64, i64> = HashMap::new();
    for o in &orders {
        if let Some(cid) = o.customer_id {
            *order_count_by_customer.entry(cid).or_insert(0) += 1;
        }
    }

    let total_customers = order_count_by_customer.len() as i64;
    let repurchase_customers = order_count_by_customer.values()
        .filter(|&&c| c >= 2)
        .count() as i64;

    let repurchase_rate = if total_customers > 0 {
        round2(Decimal::from(repurchase_customers) / Decimal::from(total_customers) * Decimal::from(100))
    } else {
        Decimal::ZERO
    };

    Ok(RepurchaseRateVO {
        year,
        month,
        total_customers,
        repurchase_customers,
        repurchase_rate,
    })
}

/// 最近N月复购率趋势
pub async fn get_repurchase_analysis(db: &DbConn, months: i32) -> Result<Vec<RepurchaseRateVO>> {
    let mut result = Vec::new();
    let now = chrono::Local::now();
    let count = months.max(1).min(24) as i32;

    for i in (0..count).rev() {
        let date = now - chrono::Duration::days(i as i64 * 30);
        let year = date.format("%Y").to_string().parse::<i32>().unwrap_or(now.format("%Y").to_string().parse().unwrap_or(2026));
        let month = date.format("%m").to_string().parse::<i32>().unwrap_or(1);
        let rate = get_repurchase_rate(db, year, month).await?;
        result.push(rate);
    }

    Ok(result)
}

/// LTV TOP N 客户列表
pub async fn get_top_ltv_customers(db: &DbConn, limit: i32) -> Result<Vec<TopLtvCustomerVO>> {
    let limit = limit.max(1).min(100) as u64;

    // 查询所有有效订单
    let orders = OrderEntity::find()
        .filter(OrderColumn::Deleted.eq(0))
        .filter(OrderColumn::OrderStatus.ne(0))
        .all(db)
        .await?;

    // 按客户聚合
    let mut stats: HashMap<i64, (Decimal, i64)> = HashMap::new();
    for o in &orders {
        if let Some(cid) = o.customer_id {
            let entry = stats.entry(cid).or_insert((Decimal::ZERO, 0));
            entry.0 += o.total_amount.unwrap_or(Decimal::ZERO);
            entry.1 += 1;
        }
    }

    // 排序取 TOP N
    let mut sorted: Vec<(i64, Decimal, i64)> = stats.into_iter()
        .map(|(cid, (amount, count))| (cid, amount, count))
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(limit as usize);

    let customer_ids: Vec<i64> = sorted.iter().map(|(id, _, _)| *id).collect();
    let customer_name_map: HashMap<i64, Option<String>> = if !customer_ids.is_empty() {
        CustomerEntity::find()
            .filter(customer::Column::Id.is_in(customer_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|c| (c.id, c.company_name.or(c.short_name).or(c.person_name)))
            .collect()
    } else {
        HashMap::new()
    };

    let result: Vec<TopLtvCustomerVO> = sorted.into_iter()
        .map(|(cid, amount, count)| {
            let avg = if count > 0 {
                round2(amount / Decimal::from(count))
            } else {
                Decimal::ZERO
            };
            let amount_score = amount / Decimal::from(10000);
            let frequency_score = Decimal::from(count * 5);
            let ltv_score = round2(
                (amount_score * Decimal::new(7, 1) + frequency_score * Decimal::new(3, 1))
                    .min(Decimal::from(100))
            );
            TopLtvCustomerVO {
                customer_id: cid,
                customer_name: customer_name_map.get(&cid).cloned().flatten(),
                total_amount: round2(amount),
                purchase_count: count,
                avg_order_amount: avg,
                ltv_score,
            }
        })
        .collect();

    Ok(result)
}

/// 计算指定月份的起止时间（NaiveDateTime）
fn month_range(year: i32, month: i32) -> (chrono::NaiveDateTime, chrono::NaiveDateTime) {
    let start = chrono::NaiveDate::from_ymd_opt(year, month as u32, 1)
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let next_month = if month >= 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1)
    };
    let end = next_month
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
        .and_hms_opt(0, 0, 0)
        .unwrap();
    (start, end)
}
