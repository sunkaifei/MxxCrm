//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::modules::statistics::model::customer_stats::{
    CustomerFunnelStatsVO, CustomerFunnelVO, CustomerIndustryStatsVO, CustomerSourceStatsVO,
    CustomerTypeStatsVO,
};
use crate::modules::statistics::service::stats_range::{
    date_param, ids_param, scope_is_empty, StatsRange, StatsScope,
};
use sea_orm::prelude::Decimal;
use sea_orm::{ConnectionTrait, DbBackend, DbConn, Statement};
use rust_decimal::prelude::RoundingStrategy;

/// 统一百分比保留2位小数（后端兜底）
fn round_pct(d: Decimal) -> Decimal {
    d.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven)
}

fn customer_type_name(t: i32) -> &'static str {
    match t {
        1 => "企业客户",
        2 => "个人客户",
        _ => "未知",
    }
}

fn source_name(s: i32) -> &'static str {
    match s {
        1 => "展会",
        2 => "线上广告",
        3 => "老客户推荐",
        4 => "官网",
        5 => "社交媒体",
        6 => "电话销售",
        7 => "邮件营销",
        8 => "合作伙伴",
        _ => "其他",
    }
}

fn industry_name(i: i32) -> &'static str {
    match i {
        1 => "IT/互联网",
        2 => "制造业",
        3 => "零售业",
        4 => "金融业",
        5 => "医疗健康",
        6 => "教育培训",
        7 => "房地产",
        8 => "交通运输",
        9 => "能源化工",
        10 => "农林牧渔",
        _ => "其他",
    }
}

fn scope_values(scope: &StatsScope) -> sea_orm::Value {
    ids_param(scope)
}

/// 维度分布通用查询（原生参数化 SQL：客户 LEFT JOIN 合同，按维度列分组）
/// 返回 (维度值, 客户数, 合同数, 合同金额)
async fn dimension_stats(
    db: &DbConn,
    dim_column: &str,
    range: &StatsRange,
    scope: &StatsScope,
    with_amount: bool,
) -> Result<Vec<(Option<i32>, i64, i64, Option<Decimal>)>> {
    let amount_expr = if with_amount {
        "COALESCE(SUM(ct.amount), 0) AS contract_amount"
    } else {
        "0::numeric AS contract_amount"
    };
    let sql = format!(
        r#"SELECT cu.{dim} AS dim,
                  COUNT(DISTINCT cu.id)::int8 AS total_count,
                  COUNT(ct.id)::int8 AS contract_count,
                  {amount_expr}
           FROM mxx_crm_customer cu
           LEFT JOIN mxx_crm_contract ct
                  ON ct.customer_id = cu.id AND ct.deleted = 0
                 AND ($1::date IS NULL OR ct.sign_date >= $1::date)
                 AND ($2::date IS NULL OR ct.sign_date <= $2::date)
           WHERE cu.deleted = 0
             AND ($1::date IS NULL OR cu.create_time >= $1::timestamp)
             AND ($2::date IS NULL OR cu.create_time < ($2::date + INTERVAL '1 day'))
             AND ($3::int8[] IS NULL OR cu.assigned_to = ANY($3::int8[]))
           GROUP BY cu.{dim}
               "#,
        dim = dim_column,
        amount_expr = amount_expr
    );
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [
                date_param(range.start),
                date_param(range.end),
                scope_values(scope),
            ],
        ))
        .await?;
    let mut result = Vec::new();
    for row in rows {
        let dim: Option<i32> = row.try_get("", "dim").unwrap_or(None);
        let total: i64 = row.try_get("", "total_count").unwrap_or(0);
        let contracts: i64 = row.try_get("", "contract_count").unwrap_or(0);
        let amount: Option<Decimal> = row.try_get("", "contract_amount").ok();
        result.push((dim, total, contracts, amount));
    }
    Ok(result)
}

/// 客户类型统计
pub async fn get_customer_type_stats(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<Vec<CustomerTypeStatsVO>> {
    if scope_is_empty(scope) {
        return Ok(Vec::new());
    }
    let rows = dimension_stats(db, "customer_type", range, scope, false).await?;
    let mut result: Vec<CustomerTypeStatsVO> = rows
        .into_iter()
        .map(|(t, count, contracts, _)| CustomerTypeStatsVO {
            customer_type: Some(customer_type_name(t.unwrap_or(0)).to_string()),
            total_count: Some(count),
            contract_count: Some(contracts),
            conversion_rate: if count > 0 {
                Some(round_pct(
                    Decimal::from(contracts) / Decimal::from(count) * Decimal::from(100),
                ))
            } else {
                Some(Decimal::ZERO)
            },
        })
        .collect();
    result.sort_by(|a, b| b.total_count.unwrap_or(0).cmp(&a.total_count.unwrap_or(0)));
    Ok(result)
}

/// 客户来源统计
pub async fn get_customer_source_stats(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<Vec<CustomerSourceStatsVO>> {
    if scope_is_empty(scope) {
        return Ok(Vec::new());
    }
    let rows = dimension_stats(db, "source", range, scope, false).await?;
    let mut result: Vec<CustomerSourceStatsVO> = rows
        .into_iter()
        .map(|(s, count, contracts, _)| CustomerSourceStatsVO {
            source: Some(source_name(s.unwrap_or(0)).to_string()),
            total_count: Some(count),
            contract_count: Some(contracts),
            conversion_rate: if count > 0 {
                Some(round_pct(
                    Decimal::from(contracts) / Decimal::from(count) * Decimal::from(100),
                ))
            } else {
                Some(Decimal::ZERO)
            },
        })
        .collect();
    result.sort_by(|a, b| b.total_count.unwrap_or(0).cmp(&a.total_count.unwrap_or(0)));
    Ok(result)
}

/// 客户行业统计
pub async fn get_customer_industry_stats(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<Vec<CustomerIndustryStatsVO>> {
    if scope_is_empty(scope) {
        return Ok(Vec::new());
    }
    let rows = dimension_stats(db, "industry", range, scope, true).await?;
    let mut result: Vec<CustomerIndustryStatsVO> = rows
        .into_iter()
        .map(|(ind, count, contracts, amount)| CustomerIndustryStatsVO {
            industry: Some(industry_name(ind.unwrap_or(0)).to_string()),
            total_count: Some(count),
            contract_count: Some(contracts),
            conversion_rate: if count > 0 {
                Some(round_pct(
                    Decimal::from(contracts) / Decimal::from(count) * Decimal::from(100),
                ))
            } else {
                Some(Decimal::ZERO)
            },
            contract_amount: Some(amount.unwrap_or(Decimal::ZERO)),
        })
        .collect();
    result.sort_by(|a, b| b.total_count.unwrap_or(0).cmp(&a.total_count.unwrap_or(0)));
    Ok(result)
}

/// 客户漏斗（各阶段独立 COUNT 聚合）
pub async fn get_customer_funnel(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<CustomerFunnelStatsVO> {
    let (mut lead_count, mut customer_count, mut opportunity_count, mut contract_count) =
        (0i64, 0i64, 0i64, 0i64);
    let mut opportunity_amount = Decimal::ZERO;
    let mut total_contract_amount = Decimal::ZERO;

    if !scope_is_empty(scope) {
        // 线索（按创建时间；线索无负责人 scope 字段则不做 scope 过滤，保持原口径）
        let row = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT COUNT(*)::int8 AS cnt FROM mxx_crm_lead
                   WHERE deleted = 0
                     AND ($1::date IS NULL OR create_time >= $1::timestamp)
                     AND ($2::date IS NULL OR create_time < ($2::date + INTERVAL '1 day'))"#,
                [
                    date_param(range.start),
                    date_param(range.end),
                ],
            ))
            .await?;
        lead_count = row.and_then(|r| r.try_get("", "cnt").ok()).unwrap_or(0);

        // 客户
        let row = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT COUNT(*)::int8 AS cnt FROM mxx_crm_customer
                   WHERE deleted = 0
                     AND ($1::date IS NULL OR create_time >= $1::timestamp)
                     AND ($2::date IS NULL OR create_time < ($2::date + INTERVAL '1 day'))
                     AND ($3::int8[] IS NULL OR assigned_to = ANY($3::int8[]))"#,
                [
                    date_param(range.start),
                    date_param(range.end),
                    scope_values(scope),
                ],
            ))
            .await?;
        customer_count = row.and_then(|r| r.try_get("", "cnt").ok()).unwrap_or(0);

        // 商机（数量 + 金额）
        let row = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT COUNT(*)::int8 AS cnt, COALESCE(SUM(amount), 0) AS total FROM mxx_crm_opportunity
                   WHERE deleted = 0
                     AND ($1::date IS NULL OR create_time >= $1::timestamp)
                     AND ($2::date IS NULL OR create_time < ($2::date + INTERVAL '1 day'))
                     AND ($3::int8[] IS NULL OR assigned_to = ANY($3::int8[]))"#,
                [
                    date_param(range.start),
                    date_param(range.end),
                    scope_values(scope),
                ],
            ))
            .await?;
        if let Some(r) = row {
            opportunity_count = r.try_get("", "cnt").unwrap_or(0);
            opportunity_amount = r.try_get("", "total").unwrap_or(Decimal::ZERO);
        }

        // 合同（按签约日期）
        let row = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT COUNT(*)::int8 AS cnt, COALESCE(SUM(amount), 0) AS total FROM mxx_crm_contract
                   WHERE deleted = 0
                     AND ($1::date IS NULL OR sign_date >= $1::date)
                     AND ($2::date IS NULL OR sign_date <= $2::date)
                     AND ($3::int8[] IS NULL OR assigned_to = ANY($3::int8[]))"#,
                [
                    date_param(range.start),
                    date_param(range.end),
                    scope_values(scope),
                ],
            ))
            .await?;
        if let Some(r) = row {
            contract_count = r.try_get("", "cnt").unwrap_or(0);
            total_contract_amount = r.try_get("", "total").unwrap_or(Decimal::ZERO);
        }
    }

    let l2c = if lead_count > 0 {
        round_pct(Decimal::from(customer_count) / Decimal::from(lead_count) * Decimal::from(100))
    } else {
        Decimal::ZERO
    };
    let c2o = if customer_count > 0 {
        round_pct(Decimal::from(opportunity_count) / Decimal::from(customer_count) * Decimal::from(100))
    } else {
        Decimal::ZERO
    };
    let o2c = if opportunity_count > 0 {
        round_pct(Decimal::from(contract_count) / Decimal::from(opportunity_count) * Decimal::from(100))
    } else {
        Decimal::ZERO
    };
    let overall = if lead_count > 0 {
        round_pct(Decimal::from(contract_count) / Decimal::from(lead_count) * Decimal::from(100))
    } else {
        Decimal::ZERO
    };

    let funnel = vec![
        CustomerFunnelVO {
            stage: Some("线索".to_string()),
            count: Some(lead_count),
            amount: None,
            rate: Some(Decimal::from(100)),
        },
        CustomerFunnelVO {
            stage: Some("客户".to_string()),
            count: Some(customer_count),
            amount: None,
            rate: Some(l2c),
        },
        CustomerFunnelVO {
            stage: Some("商机".to_string()),
            count: Some(opportunity_count),
            amount: Some(opportunity_amount),
            rate: Some(c2o),
        },
        CustomerFunnelVO {
            stage: Some("合同".to_string()),
            count: Some(contract_count),
            amount: Some(total_contract_amount),
            rate: Some(o2c),
        },
    ];

    Ok(CustomerFunnelStatsVO {
        total_leads: Some(lead_count),
        total_customers: Some(customer_count),
        total_opportunities: Some(opportunity_count),
        total_contracts: Some(contract_count),
        lead_to_customer_rate: Some(l2c),
        customer_to_opportunity_rate: Some(c2o),
        opportunity_to_contract_rate: Some(o2c),
        overall_conversion_rate: Some(overall),
        funnel: Some(funnel),
    })
}
