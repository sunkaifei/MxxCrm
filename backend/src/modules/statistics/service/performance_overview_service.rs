//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter};
use std::collections::{HashMap, HashSet};

use crate::core::errors::error::Result;
use crate::modules::crm::entity::contract::{self, Entity as Contract};
use crate::modules::crm::entity::customer::{self, Entity as Customer};
use crate::modules::crm::entity::followup::{self, Entity as Followup};
use crate::modules::crm::entity::lead::{self, Entity as Lead};
use crate::modules::crm::entity::opportunity::{self, Entity as Opportunity};
use crate::modules::sale::entity::order::{self, Entity as Order};
use crate::modules::sale::entity::order_item::{self, Entity as OrderItem};
use crate::modules::sale::entity::payment::{self, Entity as Payment};
use crate::modules::sale::entity::quotation::{self, Entity as Quotation};
use crate::modules::statistics::entity::performance_target::{self, Entity as PerformanceTarget};
use crate::modules::statistics::model::performance_overview::*;
use crate::modules::system::entity::admin::{self, Entity as Admin};

// ===================== Helper functions =====================

/// 根据年/月/时间维度计算日期范围（闭区间）
fn range_for(y: i32, m: Option<i32>, dim: &str) -> (NaiveDate, NaiveDate) {
    let now = Local::now();
    let today = now.date_naive();
    match dim {
        "year" => {
            let start = NaiveDate::from_ymd_opt(y, 1, 1).unwrap_or(today);
            let end = NaiveDate::from_ymd_opt(y, 12, 31).unwrap_or(today);
            (start, end)
        }
        "month" => {
            let m_u32 = m.unwrap_or(now.month() as i32) as u32;
            let start = NaiveDate::from_ymd_opt(y, m_u32, 1).unwrap_or(today);
            let next = if m_u32 == 12 {
                NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap_or(today)
            } else {
                NaiveDate::from_ymd_opt(y, m_u32 + 1, 1).unwrap_or(today)
            };
            let end = next.pred_opt().unwrap_or(start);
            (start, end)
        }
        _ => {
            // 默认：当年1月1日至今
            let start = NaiveDate::from_ymd_opt(y, 1, 1).unwrap_or(today);
            (start, today)
        }
    }
}

/// 计算查询日期范围（闭区间）
fn compute_date_range(
    year: Option<i32>,
    month: Option<i32>,
    time_dim: Option<String>,
) -> (NaiveDate, NaiveDate) {
    let now = Local::now();
    let y = year.unwrap_or(now.year());
    let dim = time_dim.unwrap_or_else(|| "day".to_string());
    range_for(y, month, &dim)
}

fn date_to_start_dt(date: NaiveDate) -> NaiveDateTime {
    date.and_hms_opt(0, 0, 0).unwrap()
}

fn date_to_end_dt(date: NaiveDate) -> NaiveDateTime {
    date.and_hms_opt(23, 59, 59).unwrap()
}

/// 同比/环比增长率（Decimal）
fn calc_rate(cur: Decimal, prev: Decimal) -> f64 {
    if prev == Decimal::from(0) {
        return 0.0;
    }
    ((cur - prev) / prev * Decimal::from(100))
        .to_f64()
        .unwrap_or(0.0)
}

/// 同比/环比增长率（i64）
fn calc_rate_i64(cur: i64, prev: i64) -> f64 {
    if prev == 0 {
        return 0.0;
    }
    (cur - prev) as f64 / prev as f64 * 100.0
}

/// 已签订及之后状态的合同 ID 列表
fn signed_status_values() -> Vec<i32> {
    vec![2, 3, 4, 5]
}

/// 范围内合同金额求和（已签订及之后，按签订日期过滤）
async fn sum_contract_amount(
    db: &DbConn,
    start: NaiveDate,
    end: NaiveDate,
    accessible_user_ids: &Option<Vec<i64>>,
) -> Result<Decimal> {
    if let Some(ids) = accessible_user_ids {
        if ids.is_empty() {
            return Ok(Decimal::from(0));
        }
    }
    let mut query = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::Status.is_in(signed_status_values()))
        .filter(contract::Column::SignDate.between(start, end));
    if let Some(ids) = accessible_user_ids {
        query = query.filter(contract::Column::AssignedTo.is_in(ids.clone()));
    }
    let rows = query.all(db).await?;
    let sum: Decimal = rows
        .into_iter()
        .map(|c| c.amount.unwrap_or(Decimal::from(0)))
        .sum();
    Ok(sum)
}

/// 范围内合同数量（已签订及之后）
async fn count_contracts_in_range(
    db: &DbConn,
    start: NaiveDate,
    end: NaiveDate,
    accessible_user_ids: &Option<Vec<i64>>,
) -> Result<i64> {
    if let Some(ids) = accessible_user_ids {
        if ids.is_empty() {
            return Ok(0);
        }
    }
    let mut query = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::Status.is_in(signed_status_values()))
        .filter(contract::Column::SignDate.between(start, end));
    if let Some(ids) = accessible_user_ids {
        query = query.filter(contract::Column::AssignedTo.is_in(ids.clone()));
    }
    let count = query.count(db).await?;
    Ok(count as i64)
}

/// 范围内回款金额求和（status=2 已确认，按 payment_date 过滤）
async fn sum_payment_amount(
    db: &DbConn,
    start: NaiveDate,
    end: NaiveDate,
    accessible_user_ids: &Option<Vec<i64>>,
) -> Result<Decimal> {
    if let Some(ids) = accessible_user_ids {
        if ids.is_empty() {
            return Ok(Decimal::from(0));
        }
    }
    let mut query = Payment::find()
        .filter(payment::Column::Deleted.eq(0))
        .filter(payment::Column::Status.eq(2))
        .filter(payment::Column::PaymentDate.between(start, end));
    if let Some(ids) = accessible_user_ids {
        query = query.filter(payment::Column::OwnerUserId.is_in(ids.clone()));
    }
    let rows = query.all(db).await?;
    let sum: Decimal = rows
        .into_iter()
        .map(|p| p.amount.unwrap_or(Decimal::from(0)))
        .sum();
    Ok(sum)
}

// ===================== 1. 业绩对比（同比/环比） =====================

pub async fn get_comparison(
    db: &DbConn,
    year: Option<i32>,
    month: Option<i32>,
    time_dim: Option<String>,
    accessible_user_ids: Option<Vec<i64>>,
) -> Result<PerformanceComparisonVO> {
    let now = Local::now();
    let y = year.unwrap_or(now.year());
    let dim = time_dim.clone().unwrap_or_else(|| "day".to_string());

    let (cs, ce) = range_for(y, month, &dim);
    let (ys, ye) = range_for(y - 1, month, &dim);
    let (ms, me) = if dim == "month" {
        let m = month.unwrap_or(now.month() as i32);
        if m == 1 {
            range_for(y - 1, Some(12), &dim)
        } else {
            range_for(y, Some(m - 1), &dim)
        }
    } else {
        range_for(y - 1, month, &dim)
    };

    let cur_contract = sum_contract_amount(db, cs, ce, &accessible_user_ids).await?;
    let yoy_contract = sum_contract_amount(db, ys, ye, &accessible_user_ids).await?;
    let mom_contract = sum_contract_amount(db, ms, me, &accessible_user_ids).await?;

    let cur_payment = sum_payment_amount(db, cs, ce, &accessible_user_ids).await?;
    let yoy_payment = sum_payment_amount(db, ys, ye, &accessible_user_ids).await?;
    let mom_payment = sum_payment_amount(db, ms, me, &accessible_user_ids).await?;

    let cur_count = count_contracts_in_range(db, cs, ce, &accessible_user_ids).await?;
    let yoy_count = count_contracts_in_range(db, ys, ye, &accessible_user_ids).await?;
    let mom_count = count_contracts_in_range(db, ms, me, &accessible_user_ids).await?;

    let cur_avg = if cur_count > 0 {
        cur_contract / Decimal::from(cur_count)
    } else {
        Decimal::from(0)
    };
    let yoy_avg = if yoy_count > 0 {
        yoy_contract / Decimal::from(yoy_count)
    } else {
        Decimal::from(0)
    };
    let mom_avg = if mom_count > 0 {
        mom_contract / Decimal::from(mom_count)
    } else {
        Decimal::from(0)
    };

    Ok(PerformanceComparisonVO {
        contract: ComparisonItemVO {
            yoy: Some(calc_rate(cur_contract, yoy_contract)),
            mom: Some(calc_rate(cur_contract, mom_contract)),
        },
        payment: ComparisonItemVO {
            yoy: Some(calc_rate(cur_payment, yoy_payment)),
            mom: Some(calc_rate(cur_payment, mom_payment)),
        },
        contract_count: ComparisonItemVO {
            yoy: Some(calc_rate_i64(cur_count, yoy_count)),
            mom: Some(calc_rate_i64(cur_count, mom_count)),
        },
        avg_deal_size: ComparisonItemVO {
            yoy: Some(calc_rate(cur_avg, yoy_avg)),
            mom: Some(calc_rate(cur_avg, mom_avg)),
        },
    })
}

// ===================== 2. 业绩预测 =====================

pub async fn get_forecast(
    db: &DbConn,
    year: Option<i32>,
    month: Option<i32>,
    time_dim: Option<String>,
    accessible_user_ids: Option<Vec<i64>>,
) -> Result<PerformanceForecastVO> {
    let now = Local::now();
    let y = year.unwrap_or(now.year());
    let (start, end) = compute_date_range(year, month, time_dim);

    // 无可访问用户：直接返回零值结果
    if let Some(ref ids) = accessible_user_ids {
        if ids.is_empty() {
            return Ok(PerformanceForecastVO {
                completed_amount: Decimal::from(0),
                pipeline_amount: Decimal::from(0),
                historical_win_rate: 0.0,
                forecast_amount: Decimal::from(0),
                target_amount: Decimal::from(0),
                gap_amount: Decimal::from(0),
                status: "red".to_string(),
                pipeline_coverage: 999.0,
            });
        }
    }

    let completed_amount = sum_contract_amount(db, start, end, &accessible_user_ids).await?;

    let start_dt = date_to_start_dt(start);
    let end_dt = date_to_end_dt(end);
    let mut pipeline_query = Opportunity::find()
        .filter(opportunity::Column::Deleted.eq(0))
        .filter(opportunity::Column::Stage.lt(5))
        .filter(opportunity::Column::CreateTime.between(start_dt, end_dt));
    if let Some(ref ids) = accessible_user_ids {
        pipeline_query = pipeline_query.filter(opportunity::Column::AssignedTo.is_in(ids.clone()));
    }
    let pipeline_rows = pipeline_query.all(db).await?;
    let pipeline_amount: Decimal = pipeline_rows
        .into_iter()
        .map(|o| o.amount.unwrap_or(Decimal::from(0)))
        .sum();

    // 历史成交率 = 历史成交商机数 / 历史总商机数
    let mut all_opps_query = Opportunity::find()
        .filter(opportunity::Column::Deleted.eq(0));
    if let Some(ref ids) = accessible_user_ids {
        all_opps_query = all_opps_query.filter(opportunity::Column::AssignedTo.is_in(ids.clone()));
    }
    let all_opps = all_opps_query.all(db).await?;
    let total_count = all_opps.len() as i64;
    let won_count = all_opps
        .iter()
        .filter(|o| o.stage.unwrap_or(0) == 5 && o.actual_close_date.is_some())
        .count() as i64;
    let win_rate_decimal = if total_count > 0 {
        Decimal::from(won_count) / Decimal::from(total_count)
    } else {
        Decimal::from(0)
    };
    let historical_win_rate = win_rate_decimal.to_f64().unwrap_or(0.0);

    let forecast_amount = completed_amount + pipeline_amount * win_rate_decimal;

    // 当年业绩目标（按可访问用户过滤 employee_id）
    let mut target_query = PerformanceTarget::find()
        .filter(performance_target::Column::Year.eq(y));
    if let Some(ref ids) = accessible_user_ids {
        target_query = target_query.filter(performance_target::Column::EmployeeId.is_in(ids.clone()));
    }
    let target_rows = target_query.all(db).await?;
    let target_amount: Decimal = target_rows
        .into_iter()
        .map(|t| t.contract_target_amount.unwrap_or(Decimal::from(0)))
        .sum();

    let gap_amount = if target_amount > forecast_amount {
        target_amount - forecast_amount
    } else {
        Decimal::from(0)
    };

    let threshold_80 = target_amount * Decimal::from(8) / Decimal::from(10);
    let threshold_60 = target_amount * Decimal::from(6) / Decimal::from(10);
    let status = if forecast_amount >= target_amount {
        "green"
    } else if forecast_amount >= threshold_80 {
        "yellow"
    } else if forecast_amount >= threshold_60 {
        "warning"
    } else {
        "red"
    }
    .to_string();

    let pipeline_coverage = if gap_amount == Decimal::from(0) {
        999.0
    } else {
        (pipeline_amount / gap_amount).to_f64().unwrap_or(999.0)
    };

    Ok(PerformanceForecastVO {
        completed_amount,
        pipeline_amount,
        historical_win_rate,
        forecast_amount,
        target_amount,
        gap_amount,
        status,
        pipeline_coverage,
    })
}

// ===================== 3. 销售漏斗 =====================

pub async fn get_funnel(
    db: &DbConn,
    year: Option<i32>,
    month: Option<i32>,
    time_dim: Option<String>,
    accessible_user_ids: Option<Vec<i64>>,
) -> Result<SalesFunnelVO> {
    let (start, end) = compute_date_range(year, month, time_dim);
    let start_dt = date_to_start_dt(start);
    let end_dt = date_to_end_dt(end);

    // 无可访问用户：返回空漏斗
    if let Some(ref ids) = accessible_user_ids {
        if ids.is_empty() {
            return Ok(SalesFunnelVO {
                stages: vec![
                    FunnelStageVO { stage: "线索".to_string(), count: 0, amount: Decimal::from(0), conversion_rate: 100.0 },
                    FunnelStageVO { stage: "客户".to_string(), count: 0, amount: Decimal::from(0), conversion_rate: 0.0 },
                    FunnelStageVO { stage: "商机".to_string(), count: 0, amount: Decimal::from(0), conversion_rate: 0.0 },
                    FunnelStageVO { stage: "报价".to_string(), count: 0, amount: Decimal::from(0), conversion_rate: 0.0 },
                    FunnelStageVO { stage: "订单".to_string(), count: 0, amount: Decimal::from(0), conversion_rate: 0.0 },
                    FunnelStageVO { stage: "合同".to_string(), count: 0, amount: Decimal::from(0), conversion_rate: 0.0 },
                ],
                avg_cycle_days: 0,
                win_rate: 0.0,
                total_leads: 0,
            });
        }
    }

    // 阶段 1: 线索（Lead）
    let mut lead_query = Lead::find()
        .filter(lead::Column::Deleted.eq(0))
        .filter(lead::Column::CreateTime.between(start_dt, end_dt));
    if let Some(ref ids) = accessible_user_ids {
        lead_query = lead_query.filter(lead::Column::AssignedTo.is_in(ids.clone()));
    }
    let leads = lead_query.all(db).await?;
    let lead_count = leads.len() as i64;
    let lead_amount: Decimal = leads
        .iter()
        .map(|l| l.budget.unwrap_or(Decimal::from(0)))
        .sum();

    // 阶段 2: 客户（Customer）
    let mut customer_query = Customer::find()
        .filter(customer::Column::Deleted.eq(0))
        .filter(customer::Column::CreateTime.between(start_dt, end_dt));
    if let Some(ref ids) = accessible_user_ids {
        customer_query = customer_query.filter(customer::Column::AssignedTo.is_in(ids.clone()));
    }
    let customers = customer_query.all(db).await?;
    let customer_count = customers.len() as i64;
    let customer_amount: Decimal = customers
        .iter()
        .map(|c| c.total_deal_amount.unwrap_or(Decimal::from(0)))
        .sum();

    // 阶段 3: 商机（Opportunity）
    let mut opp_query = Opportunity::find()
        .filter(opportunity::Column::Deleted.eq(0))
        .filter(opportunity::Column::CreateTime.between(start_dt, end_dt));
    if let Some(ref ids) = accessible_user_ids {
        opp_query = opp_query.filter(opportunity::Column::AssignedTo.is_in(ids.clone()));
    }
    let opps = opp_query.all(db).await?;
    let opp_count = opps.len() as i64;
    let opp_amount: Decimal = opps
        .iter()
        .map(|o| o.amount.unwrap_or(Decimal::from(0)))
        .sum();

    // 阶段 4: 报价（按商机ID去重：一条商机可能有多条报价单，漏斗只统计"有报价的商机数"）
    let mut quotation_query = Quotation::find()
        .filter(quotation::Column::Deleted.eq(0))
        .filter(quotation::Column::CreateTime.between(start_dt, end_dt));
    if let Some(ref ids) = accessible_user_ids {
        quotation_query = quotation_query.filter(quotation::Column::OwnerUserId.is_in(ids.clone()));
    }
    let quotations = quotation_query.all(db).await?;
    // 按 opportunity_id 分组，每个商机取 grand_total 最大的一条作为该阶段价值
    let mut quote_by_opp: HashMap<i64, Decimal> = HashMap::new();
    for q in &quotations {
        if let Some(opp_id) = q.opportunity_id {
            let amt = q.grand_total.unwrap_or(Decimal::from(0));
            let entry = quote_by_opp.entry(opp_id).or_insert(Decimal::from(0));
            if amt > *entry {
                *entry = amt;
            }
        }
    }
    let quotation_count = quote_by_opp.len() as i64;
    let quotation_amount: Decimal = quote_by_opp.values().copied().sum();

    // 阶段 5: 订单（同样按商机ID去重：一条商机可能有多条订单，漏斗只统计"有订单的商机数"）
    let mut order_query = Order::find()
        .filter(order::Column::Deleted.eq(0))
        .filter(order::Column::CreateTime.between(start_dt, end_dt));
    if let Some(ref ids) = accessible_user_ids {
        order_query = order_query.filter(order::Column::OwnerUserId.is_in(ids.clone()));
    }
    let orders = order_query.all(db).await?;
    let mut order_by_opp: HashMap<i64, Decimal> = HashMap::new();
    for o in &orders {
        if let Some(opp_id) = o.opportunity_id {
            let amt = o.total_amount.unwrap_or(Decimal::from(0));
            let entry = order_by_opp.entry(opp_id).or_insert(Decimal::from(0));
            if amt > *entry {
                *entry = amt;
            }
        }
    }
    let order_count = order_by_opp.len() as i64;
    let order_amount: Decimal = order_by_opp.values().copied().sum();

    // 阶段 6: 合同（Contract，已签订及之后状态：2=已签订, 3=执行中, 4=已完成, 5=已终止）
    // 合同按客户去重：一个客户可能签多个合同，漏斗统计"有成交合同的客户数"
    let mut contract_query = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::Status.is_in(signed_status_values()))
        .filter(contract::Column::SignDate.between(start, end));
    if let Some(ref ids) = accessible_user_ids {
        contract_query = contract_query.filter(contract::Column::AssignedTo.is_in(ids.clone()));
    }
    let contracts = contract_query.all(db).await?;
    let mut contract_by_customer: HashMap<i64, Decimal> = HashMap::new();
    for c in &contracts {
        if let Some(cid) = c.customer_id {
            let amt = c.amount.unwrap_or(Decimal::from(0));
            let entry = contract_by_customer.entry(cid).or_insert(Decimal::from(0));
            if amt > *entry {
                *entry = amt;
            }
        }
    }
    let contract_count = contract_by_customer.len() as i64;
    let contract_amount: Decimal = contract_by_customer.values().copied().sum();

    // 各阶段转化率（相对上一阶段）
    let cr1 = if lead_count > 0 {
        customer_count as f64 / lead_count as f64 * 100.0
    } else {
        0.0
    };
    let cr2 = if customer_count > 0 {
        opp_count as f64 / customer_count as f64 * 100.0
    } else {
        0.0
    };
    let cr3 = if opp_count > 0 {
        quotation_count as f64 / opp_count as f64 * 100.0
    } else {
        0.0
    };
    let cr4 = if quotation_count > 0 {
        order_count as f64 / quotation_count as f64 * 100.0
    } else {
        0.0
    };
    let cr5 = if order_count > 0 {
        contract_count as f64 / order_count as f64 * 100.0
    } else {
        0.0
    };

    let stages = vec![
        FunnelStageVO {
            stage: "线索".to_string(),
            count: lead_count,
            amount: lead_amount,
            conversion_rate: 100.0,
        },
        FunnelStageVO {
            stage: "客户".to_string(),
            count: customer_count,
            amount: customer_amount,
            conversion_rate: cr1,
        },
        FunnelStageVO {
            stage: "商机".to_string(),
            count: opp_count,
            amount: opp_amount,
            conversion_rate: cr2,
        },
        FunnelStageVO {
            stage: "报价".to_string(),
            count: quotation_count,
            amount: quotation_amount,
            conversion_rate: cr3,
        },
        FunnelStageVO {
            stage: "订单".to_string(),
            count: order_count,
            amount: order_amount,
            conversion_rate: cr4,
        },
        FunnelStageVO {
            stage: "合同".to_string(),
            count: contract_count,
            amount: contract_amount,
            conversion_rate: cr5,
        },
    ];

    // 平均成交周期：从商机创建到合同签订的天数
    let mut total_days = 0i64;
    let mut cycle_count = 0i64;
    let contract_customer_ids: HashSet<i64> = contracts
        .iter()
        .filter_map(|c| c.customer_id)
        .collect();
    for o in &opps {
        if let Some(cid) = o.customer_id {
            if contract_customer_ids.contains(&cid) {
                if let Some(create_time) = o.create_time {
                    // 用合同签订日期 - 商机创建时间 作为成交周期近似
                    if let Some(contract) = contracts
                        .iter()
                        .find(|c| c.customer_id == Some(cid))
                        .and_then(|c| c.sign_date)
                    {
                        let diff = contract - create_time.date();
                        let days = diff.num_days();
                        if days >= 0 {
                            total_days += days;
                            cycle_count += 1;
                        }
                    }
                }
            }
        }
    }
    let avg_cycle_days = if cycle_count > 0 {
        total_days / cycle_count
    } else {
        0
    };

    // 赢单率：合同数 / 商机数
    let win_rate = if opp_count > 0 {
        contract_count as f64 / opp_count as f64 * 100.0
    } else {
        0.0
    };

    Ok(SalesFunnelVO {
        stages,
        avg_cycle_days,
        win_rate,
        total_leads: lead_count,
    })
}

// ===================== 4. 客户拆解 =====================

pub async fn get_customer_breakdown(
    db: &DbConn,
    year: Option<i32>,
    month: Option<i32>,
    time_dim: Option<String>,
    accessible_user_ids: Option<Vec<i64>>,
) -> Result<CustomerBreakdownVO> {
    let now = Local::now();
    let y = year.unwrap_or(now.year());
    let (start, end) = compute_date_range(year, month, time_dim);

    // 无可访问用户：返回空结果
    if let Some(ref ids) = accessible_user_ids {
        if ids.is_empty() {
            return Ok(CustomerBreakdownVO {
                new_vs_old: vec![],
                abc_distribution: vec![],
                top10: vec![],
            });
        }
    }

    let mut contract_query = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::Status.is_in(signed_status_values()))
        .filter(contract::Column::SignDate.between(start, end));
    if let Some(ref ids) = accessible_user_ids {
        contract_query = contract_query.filter(contract::Column::AssignedTo.is_in(ids.clone()));
    }
    let contracts = contract_query.all(db).await?;

    let mut customer_amounts: HashMap<i64, Decimal> = HashMap::new();
    for c in &contracts {
        if let Some(cid) = c.customer_id {
            *customer_amounts
                .entry(cid)
                .or_insert(Decimal::from(0)) += c.amount.unwrap_or(Decimal::from(0));
        }
    }

    let customer_ids: Vec<i64> = customer_amounts.keys().copied().collect();
    let customers = if customer_ids.is_empty() {
        vec![]
    } else {
        Customer::find()
            .filter(customer::Column::Deleted.eq(0))
            .filter(customer::Column::Id.is_in(customer_ids))
            .all(db)
            .await?
    };

    // 新老客户：当年 create_time 为新客户
    let year_start = NaiveDate::from_ymd_opt(y, 1, 1).unwrap_or(start);
    let year_end = NaiveDate::from_ymd_opt(y, 12, 31).unwrap_or(end);

    let mut new_count = 0i64;
    let mut old_count = 0i64;
    let mut new_amount = Decimal::from(0);
    let mut old_amount = Decimal::from(0);

    for cust in &customers {
        let amount = customer_amounts
            .get(&cust.id)
            .copied()
            .unwrap_or(Decimal::from(0));
        let is_new = cust
            .create_time
            .map(|ct| {
                let cd = ct.date();
                cd >= year_start && cd <= year_end
            })
            .unwrap_or(false);
        if is_new {
            new_count += 1;
            new_amount += amount;
        } else {
            old_count += 1;
            old_amount += amount;
        }
    }

    let new_vs_old = vec![
        BreakdownItemVO {
            name: "新客户".to_string(),
            value: new_count,
            amount: new_amount,
        },
        BreakdownItemVO {
            name: "老客户".to_string(),
            value: old_count,
            amount: old_amount,
        },
    ];

    // ABC 分级：按合同金额降序排列
    let mut sorted_customers: Vec<(i64, Decimal)> = customer_amounts
        .iter()
        .map(|(k, v)| (*k, *v))
        .collect();
    sorted_customers.sort_by(|a, b| b.1.cmp(&a.1));

    let total = sorted_customers.len();
    let a_threshold = ((total as f64) * 0.2).ceil() as usize;
    let b_threshold = ((total as f64) * 0.5).ceil() as usize;

    let mut a_count = 0i64;
    let mut a_amount = Decimal::from(0);
    let mut b_count = 0i64;
    let mut b_amount = Decimal::from(0);
    let mut c_count = 0i64;
    let mut c_amount = Decimal::from(0);

    for (i, (_, amount)) in sorted_customers.iter().enumerate() {
        if i < a_threshold && *amount > Decimal::from(0) {
            a_count += 1;
            a_amount += *amount;
        } else if i < b_threshold {
            b_count += 1;
            b_amount += *amount;
        } else {
            c_count += 1;
            c_amount += *amount;
        }
    }

    let abc_distribution = vec![
        BreakdownItemVO {
            name: "A级".to_string(),
            value: a_count,
            amount: a_amount,
        },
        BreakdownItemVO {
            name: "B级".to_string(),
            value: b_count,
            amount: b_amount,
        },
        BreakdownItemVO {
            name: "C级".to_string(),
            value: c_count,
            amount: c_amount,
        },
    ];

    // Top10 + growth
    let last_year_start = NaiveDate::from_ymd_opt(y - 1, 1, 1).unwrap_or(start);
    let last_year_end = NaiveDate::from_ymd_opt(y - 1, 12, 31).unwrap_or(end);
    let mut last_year_query = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::Status.is_in(signed_status_values()))
        .filter(contract::Column::SignDate.between(last_year_start, last_year_end));
    if let Some(ref ids) = accessible_user_ids {
        last_year_query = last_year_query.filter(contract::Column::AssignedTo.is_in(ids.clone()));
    }
    let last_year_contracts = last_year_query.all(db).await?;

    let mut last_year_amounts: HashMap<i64, Decimal> = HashMap::new();
    for c in &last_year_contracts {
        if let Some(cid) = c.customer_id {
            *last_year_amounts
                .entry(cid)
                .or_insert(Decimal::from(0)) += c.amount.unwrap_or(Decimal::from(0));
        }
    }

    let customer_names: HashMap<i64, String> = customers
        .iter()
        .map(|c| {
            (
                c.id,
                c.company_name
                    .clone()
                    .or_else(|| c.short_name.clone())
                    .or_else(|| c.person_name.clone())
                    .unwrap_or_else(|| format!("客户-{}", c.id)),
            )
        })
        .collect();

    let mut top10: Vec<TopCustomerVO> = Vec::new();
    for (i, (cid, amount)) in sorted_customers.iter().take(10).enumerate() {
        let last_amount = last_year_amounts.get(cid).copied().unwrap_or(Decimal::from(0));
        let growth = if last_amount > Decimal::from(0) {
            ((*amount - last_amount) / last_amount * Decimal::from(100))
                .to_f64()
                .unwrap_or(0.0)
        } else {
            0.0
        };
        top10.push(TopCustomerVO {
            rank: (i + 1) as i32,
            customer_name: customer_names
                .get(cid)
                .cloned()
                .unwrap_or_else(|| format!("客户-{}", cid)),
            amount: *amount,
            growth,
        });
    }

    Ok(CustomerBreakdownVO {
        new_vs_old,
        abc_distribution,
        top10,
    })
}

// ===================== 5. 产品拆解 =====================

pub async fn get_product_breakdown(
    db: &DbConn,
    year: Option<i32>,
    month: Option<i32>,
    time_dim: Option<String>,
    accessible_user_ids: Option<Vec<i64>>,
) -> Result<ProductBreakdownVO> {
    let (start, end) = compute_date_range(year, month, time_dim);
    let start_dt = date_to_start_dt(start);
    let end_dt = date_to_end_dt(end);

    // 无可访问用户：返回空结果
    if let Some(ref ids) = accessible_user_ids {
        if ids.is_empty() {
            return Ok(ProductBreakdownVO {
                products: vec![],
                categories: vec![],
            });
        }
    }

    let mut order_query = Order::find()
        .filter(order::Column::Deleted.eq(0))
        .filter(order::Column::CreateTime.between(start_dt, end_dt));
    if let Some(ref ids) = accessible_user_ids {
        order_query = order_query.filter(order::Column::OwnerUserId.is_in(ids.clone()));
    }
    let orders = order_query.all(db).await?;

    let order_ids: Vec<i64> = orders.iter().map(|o| o.id).collect();
    if order_ids.is_empty() {
        return Ok(ProductBreakdownVO {
            products: vec![],
            categories: vec![],
        });
    }

    let items = OrderItem::find()
        .filter(order_item::Column::Deleted.eq(0))
        .filter(order_item::Column::OrderId.is_in(order_ids))
        .all(db)
        .await?;

    let mut product_map: HashMap<String, (Decimal, i64)> = HashMap::new();
    for item in &items {
        let name = item
            .product_name
            .clone()
            .unwrap_or_else(|| "未知产品".to_string());
        let amount = item
            .total_amount
            .or(item.amount)
            .unwrap_or(Decimal::from(0));
        let entry = product_map.entry(name).or_insert((Decimal::from(0), 0));
        entry.0 += amount;
        entry.1 += 1;
    }

    let total_amount: Decimal = product_map.values().map(|(a, _)| *a).sum();

    let mut products: Vec<ProductRankVO> = product_map
        .into_iter()
        .map(|(name, (amount, count))| {
            let share = if total_amount > Decimal::from(0) {
                (amount * Decimal::from(100) / total_amount)
                    .to_f64()
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            ProductRankVO {
                rank: 0,
                product_name: name,
                amount,
                count,
                share,
            }
        })
        .collect();

    products.sort_by(|a, b| b.amount.cmp(&a.amount));
    for (i, p) in products.iter_mut().enumerate() {
        p.rank = (i + 1) as i32;
    }

    // 简单分类
    let mut category_map: HashMap<String, (Decimal, i64)> = HashMap::new();
    for p in &products {
        let name_lower = p.product_name.to_lowercase();
        let category = if name_lower.contains("硬件") || name_lower.contains("hardware") {
            "硬件"
        } else if name_lower.contains("软件") || name_lower.contains("software") {
            "软件"
        } else if name_lower.contains("服务") || name_lower.contains("service") {
            "服务"
        } else {
            "其他"
        };
        let entry = category_map
            .entry(category.to_string())
            .or_insert((Decimal::from(0), 0));
        entry.0 += p.amount;
        entry.1 += 1;
    }

    let categories: Vec<BreakdownItemVO> = category_map
        .into_iter()
        .map(|(name, (amount, count))| BreakdownItemVO {
            name,
            value: count,
            amount,
        })
        .collect();

    Ok(ProductBreakdownVO { products, categories })
}

// ===================== 6. 行为指标 =====================

pub async fn get_behavior_metrics(
    db: &DbConn,
    year: Option<i32>,
    month: Option<i32>,
    time_dim: Option<String>,
    accessible_user_ids: Option<Vec<i64>>,
) -> Result<BehaviorMetricsVO> {
    let now = Local::now();
    let y = year.unwrap_or(now.year());
    let (start, end) = compute_date_range(year, month, time_dim);
    let start_dt = date_to_start_dt(start);
    let end_dt = date_to_end_dt(end);

    // 无可访问用户：返回空结果
    if let Some(ref ids) = accessible_user_ids {
        if ids.is_empty() {
            return Ok(BehaviorMetricsVO {
                summary: BehaviorSummaryVO {
                    visit_count: 0,
                    phone_count: 0,
                    follow_up_count: 0,
                    conversion_rate: 0.0,
                },
                trend: vec![],
            });
        }
    }

    let mut followup_query = Followup::find()
        .filter(followup::Column::Deleted.eq(0))
        .filter(followup::Column::CreateTime.between(start_dt, end_dt));
    if let Some(ref ids) = accessible_user_ids {
        followup_query = followup_query.filter(followup::Column::CreatedBy.is_in(ids.clone()));
    }
    let followups = followup_query.all(db).await?;

    let mut visit_count = 0i64;
    let mut phone_count = 0i64;
    let mut follow_up_count = 0i64;
    for f in &followups {
        match f.activity_type.unwrap_or(0) {
            1 => phone_count += 1,
            2 => visit_count += 1,
            _ => follow_up_count += 1,
        }
    }

    // 转化率：当期成交客户数 / 当期跟进客户数 * 100
    let followed_customer_ids: HashSet<i64> =
        followups.iter().filter_map(|f| f.customer_id).collect();

    let mut contract_query = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::Status.is_in(signed_status_values()))
        .filter(contract::Column::SignDate.between(start, end));
    if let Some(ref ids) = accessible_user_ids {
        contract_query = contract_query.filter(contract::Column::AssignedTo.is_in(ids.clone()));
    }
    let contracts = contract_query.all(db).await?;
    let won_customer_ids: HashSet<i64> =
        contracts.iter().filter_map(|c| c.customer_id).collect();

    let conversion_rate = if !followed_customer_ids.is_empty() {
        let won_followed = won_customer_ids
            .intersection(&followed_customer_ids)
            .count() as f64;
        won_followed / followed_customer_ids.len() as f64 * 100.0
    } else {
        0.0
    };

    let summary = BehaviorSummaryVO {
        visit_count,
        phone_count,
        follow_up_count,
        conversion_rate,
    };

    // 按月趋势
    let year_start = NaiveDate::from_ymd_opt(y, 1, 1).unwrap_or(start);
    let year_end = NaiveDate::from_ymd_opt(y, 12, 31).unwrap_or(end);
    let year_start_dt = date_to_start_dt(year_start);
    let year_end_dt = date_to_end_dt(year_end);

    let mut year_followup_query = Followup::find()
        .filter(followup::Column::Deleted.eq(0))
        .filter(followup::Column::CreateTime.between(year_start_dt, year_end_dt));
    if let Some(ref ids) = accessible_user_ids {
        year_followup_query = year_followup_query.filter(followup::Column::CreatedBy.is_in(ids.clone()));
    }
    let year_followups = year_followup_query.all(db).await?;

    let mut year_contract_query = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::Status.is_in(signed_status_values()))
        .filter(contract::Column::SignDate.between(year_start, year_end));
    if let Some(ref ids) = accessible_user_ids {
        year_contract_query = year_contract_query.filter(contract::Column::AssignedTo.is_in(ids.clone()));
    }
    let year_contracts = year_contract_query.all(db).await?;

    let mut monthly_contracts: HashMap<String, Decimal> = HashMap::new();
    for c in &year_contracts {
        if let Some(sd) = c.sign_date {
            let key = sd.format("%Y-%m").to_string();
            *monthly_contracts
                .entry(key)
                .or_insert(Decimal::from(0)) += c.amount.unwrap_or(Decimal::from(0));
        }
    }

    let mut monthly_data: HashMap<String, (i64, i64, i64)> = HashMap::new();
    for f in &year_followups {
        if let Some(ct) = f.create_time {
            let key = ct.format("%Y-%m").to_string();
            let entry = monthly_data.entry(key).or_insert((0, 0, 0));
            match f.activity_type.unwrap_or(0) {
                1 => entry.1 += 1,
                2 => entry.0 += 1,
                _ => entry.2 += 1,
            }
        }
    }

    let mut trend: Vec<BehaviorTrendItemVO> = monthly_data
        .into_iter()
        .map(|(period, (visit, phone, followup))| BehaviorTrendItemVO {
            amount: monthly_contracts
                .get(&period)
                .copied()
                .unwrap_or(Decimal::from(0)),
            period,
            visit_count: visit,
            phone_count: phone,
            follow_up_count: followup,
        })
        .collect();
    trend.sort_by(|a, b| a.period.cmp(&b.period));

    Ok(BehaviorMetricsVO { summary, trend })
}

// ===================== 7. 区域拆解 =====================

pub async fn get_region_breakdown(
    db: &DbConn,
    year: Option<i32>,
    month: Option<i32>,
    time_dim: Option<String>,
    accessible_user_ids: Option<Vec<i64>>,
) -> Result<Vec<RegionItemVO>> {
    let (start, end) = compute_date_range(year, month, time_dim);

    // 无可访问用户：返回空结果
    if let Some(ref ids) = accessible_user_ids {
        if ids.is_empty() {
            return Ok(vec![]);
        }
    }

    let mut contract_query = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::Status.is_in(signed_status_values()))
        .filter(contract::Column::SignDate.between(start, end));
    if let Some(ref ids) = accessible_user_ids {
        contract_query = contract_query.filter(contract::Column::AssignedTo.is_in(ids.clone()));
    }
    let contracts = contract_query.all(db).await?;

    let mut customer_amounts: HashMap<i64, Decimal> = HashMap::new();
    for c in &contracts {
        if let Some(cid) = c.customer_id {
            *customer_amounts
                .entry(cid)
                .or_insert(Decimal::from(0)) += c.amount.unwrap_or(Decimal::from(0));
        }
    }

    let customer_ids: Vec<i64> = customer_amounts.keys().copied().collect();
    let customers = if customer_ids.is_empty() {
        vec![]
    } else {
        Customer::find()
            .filter(customer::Column::Deleted.eq(0))
            .filter(customer::Column::Id.is_in(customer_ids))
            .all(db)
            .await?
    };

    let mut region_map: HashMap<String, (Decimal, i64)> = HashMap::new();
    for cust in &customers {
        let region = cust
            .region
            .clone()
            .filter(|r| !r.is_empty())
            .unwrap_or_else(|| "未知".to_string());
        let amount = customer_amounts
            .get(&cust.id)
            .copied()
            .unwrap_or(Decimal::from(0));
        let entry = region_map
            .entry(region)
            .or_insert((Decimal::from(0), 0));
        entry.0 += amount;
        entry.1 += 1;
    }

    let total_amount: Decimal = region_map.values().map(|(a, _)| *a).sum();

    let mut result: Vec<RegionItemVO> = region_map
        .into_iter()
        .map(|(province, (amount, customer_count))| {
            let share = if total_amount > Decimal::from(0) {
                (amount * Decimal::from(100) / total_amount)
                    .to_f64()
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            RegionItemVO {
                rank: 0,
                province,
                amount,
                customer_count,
                share,
            }
        })
        .collect();

    result.sort_by(|a, b| b.amount.cmp(&a.amount));
    for (i, r) in result.iter_mut().enumerate() {
        r.rank = (i + 1) as i32;
    }

    Ok(result)
}

// ===================== 8. 个人成长 =====================

pub async fn get_personal_growth(
    db: &DbConn,
    employee_id: Option<i64>,
) -> Result<PersonalGrowthVO> {
    let eid = match employee_id {
        Some(id) => id,
        None => {
            return Ok(PersonalGrowthVO {
                hire_date: None,
                total_amount: Decimal::from(0),
                total_contract_count: 0,
                best_month: BestMonthVO {
                    month: String::new(),
                    amount: Decimal::from(0),
                },
                monthly_trend: vec![],
            });
        }
    };

    let admin = Admin::find()
        .filter(admin::Column::Id.eq(eid))
        .filter(admin::Column::Deleted.eq(0))
        .one(db)
        .await?;
    let hire_date = admin
        .and_then(|a| a.create_time)
        .map(|ct| ct.format("%Y-%m-%d").to_string());

    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::AssignedTo.eq(eid))
        .filter(contract::Column::Status.is_in(signed_status_values()))
        .all(db)
        .await?;

    let total_amount: Decimal = contracts
        .iter()
        .map(|c| c.amount.unwrap_or(Decimal::from(0)))
        .sum();
    let total_contract_count = contracts.len() as i64;

    let mut monthly_map: HashMap<String, Decimal> = HashMap::new();
    for c in &contracts {
        if let Some(sd) = c.sign_date {
            let key = sd.format("%Y-%m").to_string();
            *monthly_map
                .entry(key)
                .or_insert(Decimal::from(0)) += c.amount.unwrap_or(Decimal::from(0));
        }
    }

    let mut monthly_trend: Vec<GrowthTrendItemVO> = monthly_map
        .iter()
        .map(|(month, amount)| GrowthTrendItemVO {
            month: month.clone(),
            amount: *amount,
        })
        .collect();
    monthly_trend.sort_by(|a, b| a.month.cmp(&b.month));

    let best_month = monthly_map
        .iter()
        .max_by(|a, b| a.1.cmp(b.1))
        .map(|(m, a)| BestMonthVO {
            month: m.clone(),
            amount: *a,
        })
        .unwrap_or(BestMonthVO {
            month: String::new(),
            amount: Decimal::from(0),
        });

    Ok(PersonalGrowthVO {
        hire_date,
        total_amount,
        total_contract_count,
        best_month,
        monthly_trend,
    })
}

// ===================== 9. 里程碑 =====================

pub async fn get_milestone(
    db: &DbConn,
    year: Option<i32>,
    employee_id: Option<i64>,
    current_user_id: i64,
) -> Result<PerformanceMilestoneVO> {
    let now = Local::now();
    let y = year.unwrap_or(now.year());
    let year_start = NaiveDate::from_ymd_opt(y, 1, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    let year_end = NaiveDate::from_ymd_opt(y, 12, 31)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

    // 默认档位
    let levels: Vec<(String, Decimal)> = vec![
        ("100万".to_string(), Decimal::from(1000000)),
        ("500万".to_string(), Decimal::from(5000000)),
        ("1000万".to_string(), Decimal::from(10000000)),
        ("2000万".to_string(), Decimal::from(20000000)),
    ];

    // 未指定 employee_id 时回退到当前用户ID，确保始终按个人维度过滤
    let eid = employee_id.unwrap_or(current_user_id);
    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::Status.is_in(signed_status_values()))
        .filter(contract::Column::SignDate.between(year_start, year_end))
        .filter(contract::Column::AssignedTo.eq(eid))
        .all(db)
        .await?;

    // 按签订日期升序
    let mut sorted_contracts: Vec<contract::Model> = contracts
        .into_iter()
        .filter(|c| c.sign_date.is_some())
        .collect();
    sorted_contracts.sort_by_key(|c| c.sign_date);

    let current_amount: Decimal = sorted_contracts
        .iter()
        .map(|c| c.amount.unwrap_or(Decimal::from(0)))
        .sum();

    // 计算每个档位的达成日期
    let mut milestones: Vec<MilestoneItemVO> = Vec::new();
    for (label, amount) in &levels {
        let mut cumulative = Decimal::from(0);
        let mut achieved_date: Option<String> = None;
        for c in &sorted_contracts {
            cumulative += c.amount.unwrap_or(Decimal::from(0));
            if cumulative >= *amount {
                achieved_date = c
                    .sign_date
                    .map(|d| d.format("%Y-%m-%d").to_string());
                break;
            }
        }
        let achieved = achieved_date.is_some();
        milestones.push(MilestoneItemVO {
            label: label.clone(),
            amount: *amount,
            achieved,
            achieved_date,
        });
    }

    // current_milestone：最后一个已达成档位
    let current_idx = milestones.iter().rposition(|m| m.achieved);
    let current_milestone = current_idx.map(|i| milestones[i].clone());

    // next_milestone：current_milestone 之后第一个档位
    let next_milestone = match current_idx {
        Some(i) => {
            if i + 1 < milestones.len() {
                Some(milestones[i + 1].clone())
            } else {
                None
            }
        }
        None => None,
    };

    // future_milestone：第一个未达成档位
    let future_milestone = milestones.iter().find(|m| !m.achieved).cloned();

    // remaining = future_milestone.amount - current_amount（>0）
    let remaining = future_milestone.as_ref().map(|fm| {
        let r = fm.amount - current_amount;
        if r > Decimal::from(0) {
            r
        } else {
            Decimal::from(0)
        }
    });

    Ok(PerformanceMilestoneVO {
        current_milestone,
        next_milestone,
        future_milestone,
        remaining,
        milestones,
    })
}
