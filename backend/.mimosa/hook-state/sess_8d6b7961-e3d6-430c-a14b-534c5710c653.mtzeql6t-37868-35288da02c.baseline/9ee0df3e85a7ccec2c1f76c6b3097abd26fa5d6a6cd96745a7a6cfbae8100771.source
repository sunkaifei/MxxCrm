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
use crate::modules::crm::entity::customer::Entity as Customer;
use crate::modules::statistics::model::contract_stats::{
    ContractRankingVO, ContractStatusAnalysisVO, ContractTypeDistributionVO,
};
use crate::modules::statistics::model::employee_stats::{
    EmployeeConversionVO, EmployeeCustomerCountVO, EmployeeFollowUpVO,
};
use crate::modules::statistics::model::payment_stats::{PaymentCompletionVO, PaymentRankingVO};
use crate::modules::statistics::service::stats_range::{
    date_param, ids_param, scope_is_empty, StatsRange, StatsScope,
};
use crate::modules::system::entity::admin::Entity as Admin;
use chrono::Datelike;
use sea_orm::prelude::Decimal;
use sea_orm::*;
use std::collections::HashMap;
use rust_decimal::prelude::RoundingStrategy;

fn round_pct(d: Decimal) -> Decimal {
    d.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven)
}

fn scope_values(scope: &StatsScope) -> sea_orm::Value {
    ids_param(scope)
}

fn type_name(t: i32) -> &'static str {
    match t {
        1 => "销售合同",
        2 => "采购合同",
        3 => "服务合同",
        4 => "合作协议",
        5 => "其他",
        _ => "未知",
    }
}

fn status_name(s: i32) -> &'static str {
    match s {
        1 => "草稿",
        2 => "已签订",
        3 => "执行中",
        4 => "已完成",
        5 => "已终止",
        _ => "未知",
    }
}

// ============ 合同 ============

/// 合同排行（汇总表）
pub async fn contract_ranking(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
    order_by: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<ContractRankingVO>> {
    if scope_is_empty(scope) {
        return Ok(Vec::new());
    }
    let order_by = order_by.unwrap_or_else(|| "amount".to_string());
    let limit = limit.unwrap_or(10) as usize;

    // 按客户
    let by_customer = db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT customer_id AS id, SUM(contract_count)::int8 AS cnt, SUM(contract_amount) AS total
               FROM mxx_statistics_daily_contract
               WHERE stat_date BETWEEN $1::date AND $2::date AND customer_id > 0
                 AND ($3::int8[] IS NULL OR employee_id = ANY($3::int8[]))
               GROUP BY customer_id"#,
            [range.start.into(), range.end.into(), scope_values(scope)],
        ))
        .await?;
    // 按员工
    let by_employee = db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT employee_id AS id, SUM(contract_count)::int8 AS cnt, SUM(contract_amount) AS total
               FROM mxx_statistics_daily_contract
               WHERE stat_date BETWEEN $1::date AND $2::date AND employee_id > 0
                 AND ($3::int8[] IS NULL OR employee_id = ANY($3::int8[]))
               GROUP BY employee_id"#,
            [range.start.into(), range.end.into(), scope_values(scope)],
        ))
        .await?;

    let parse = |rows: Vec<sea_orm::QueryResult>| -> Vec<(i64, i64, Decimal)> {
        rows.into_iter()
            .filter_map(|r| {
                let id: i64 = r.try_get("", "id").ok()?;
                let cnt: i64 = r.try_get("", "cnt").unwrap_or(0);
                let total: Decimal = r.try_get("", "total").unwrap_or(Decimal::ZERO);
                Some((id, cnt, total))
            })
            .collect()
    };

    let c_rows = parse(by_customer);
    let e_rows = parse(by_employee);

    // 名称映射
    let cids: Vec<i64> = c_rows.iter().map(|(id, _, _)| *id).collect();
    let mut cname: HashMap<i64, String> = HashMap::new();
    if !cids.is_empty() {
        for c in Customer::find()
            .filter(crate::modules::crm::entity::customer::Column::Id.is_in(cids))
            .all(db)
            .await?
        {
            let n = c.company_name.or(c.person_name).or(c.nickname).unwrap_or_default();
            if !n.is_empty() {
                cname.insert(c.id, n);
            }
        }
    }
    let eids: Vec<i64> = e_rows.iter().map(|(id, _, _)| *id).collect();
    let mut ename: HashMap<i64, String> = HashMap::new();
    if !eids.is_empty() {
        for a in Admin::find()
            .filter(crate::modules::system::entity::admin::Column::Id.is_in(eids))
            .all(db)
            .await?
        {
            let n = a.nick_name.or(a.user_name).unwrap_or_default();
            if !n.is_empty() {
                ename.insert(a.id, n);
            }
        }
    }

    let mut result: Vec<ContractRankingVO> = Vec::new();
    for (cid, cnt, total) in c_rows {
        result.push(ContractRankingVO {
            rank: None,
            target_type: Some("customer".to_string()),
            target_id: Some(cid),
            target_name: cname.get(&cid).cloned(),
            contract_count: Some(cnt),
            contract_amount: Some(total),
            payment_amount: Some(Decimal::ZERO),
            payment_rate: Some(Decimal::ZERO),
        });
    }
    for (eid, cnt, total) in e_rows {
        result.push(ContractRankingVO {
            rank: None,
            target_type: Some("employee".to_string()),
            target_id: Some(eid),
            target_name: ename.get(&eid).cloned(),
            contract_count: Some(cnt),
            contract_amount: Some(total),
            payment_amount: Some(Decimal::ZERO),
            payment_rate: Some(Decimal::ZERO),
        });
    }

    if order_by == "count" {
        result.sort_by(|a, b| b.contract_count.unwrap_or(0).cmp(&a.contract_count.unwrap_or(0)));
    } else {
        result.sort_by(|a, b| {
            b.contract_amount
                .unwrap_or(Decimal::ZERO)
                .cmp(&a.contract_amount.unwrap_or(Decimal::ZERO))
        });
    }
    result.truncate(limit);
    for (i, item) in result.iter_mut().enumerate() {
        item.rank = Some((i + 1) as i32);
    }
    Ok(result)
}

/// 合同类型/状态分布（汇总表通用分组）
async fn contract_group(
    db: &DbConn,
    dim: &str,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<Vec<(i32, i64, Decimal)>> {
    let sql = format!(
        r#"SELECT {dim}::int AS dim, SUM(contract_count)::int8 AS cnt, SUM(contract_amount) AS total
           FROM mxx_statistics_daily_contract
           WHERE stat_date BETWEEN $1::date AND $2::date
             AND ($3::int8[] IS NULL OR employee_id = ANY($3::int8[]))
           GROUP BY 1"#,
        dim = dim
    );
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [range.start.into(), range.end.into(), scope_values(scope)],
        ))
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.try_get::<i32>("", "dim").unwrap_or(0),
                r.try_get::<i64>("", "cnt").unwrap_or(0),
                r.try_get::<Decimal>("", "total").unwrap_or(Decimal::ZERO),
            )
        })
        .collect())
}

pub async fn contract_type_distribution(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<Vec<ContractTypeDistributionVO>> {
    if scope_is_empty(scope) {
        return Ok(Vec::new());
    }
    let rows = contract_group(db, "contract_type", range, scope).await?;
    let total: Decimal = rows.iter().map(|(_, _, a)| *a).sum();
    let mut result: Vec<ContractTypeDistributionVO> = rows
        .into_iter()
        .map(|(t, cnt, amount)| ContractTypeDistributionVO {
            contract_type: Some(type_name(t).to_string()),
            contract_count: Some(cnt),
            contract_amount: Some(amount),
            percentage: Some(if total > Decimal::ZERO {
                round_pct(amount / total * Decimal::from(100))
            } else {
                Decimal::ZERO
            }),
        })
        .collect();
    result.sort_by(|a, b| b.contract_count.unwrap_or(0).cmp(&a.contract_count.unwrap_or(0)));
    Ok(result)
}

pub async fn contract_status_analysis(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<Vec<ContractStatusAnalysisVO>> {
    if scope_is_empty(scope) {
        return Ok(Vec::new());
    }
    let rows = contract_group(db, "status", range, scope).await?;
    let total: Decimal = rows.iter().map(|(_, _, a)| *a).sum();
    let mut result: Vec<ContractStatusAnalysisVO> = rows
        .into_iter()
        .map(|(s, cnt, amount)| ContractStatusAnalysisVO {
            status: Some(format!("{}", s)),
            status_name: Some(status_name(s).to_string()),
            contract_count: Some(cnt),
            contract_amount: Some(amount),
            percentage: Some(if total > Decimal::ZERO {
                round_pct(amount / total * Decimal::from(100))
            } else {
                Decimal::ZERO
            }),
        })
        .collect();
    result.sort_by(|a, b| b.contract_count.unwrap_or(0).cmp(&a.contract_count.unwrap_or(0)));
    Ok(result)
}

// ============ 回款 ============

pub async fn payment_completion(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<PaymentCompletionVO> {
    let year = range
        .start
        .map_or(chrono::Local::now().year(), |s| s.year());

    let mut contract_total = Decimal::ZERO;
    let mut payment_total = Decimal::ZERO;

    if !scope_is_empty(scope) {
        let r = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT COALESCE(SUM(contract_amount), 0) AS total FROM mxx_statistics_daily_contract
                   WHERE stat_date BETWEEN $1::date AND $2::date
                     AND ($3::int8[] IS NULL OR employee_id = ANY($3::int8[]))"#,
                [range.start.into(), range.end.into(), scope_values(scope)],
            ))
            .await?;
        if let Some(r) = r {
            contract_total = r.try_get("", "total").unwrap_or(Decimal::ZERO);
        }
        let r = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT COALESCE(SUM(payment_amount), 0) AS total FROM mxx_statistics_daily_payment
                   WHERE stat_date BETWEEN $1::date AND $2::date
                     AND ($3::int8[] IS NULL OR employee_id = ANY($3::int8[]))"#,
                [range.start.into(), range.end.into(), scope_values(scope)],
            ))
            .await?;
        if let Some(r) = r {
            payment_total = r.try_get("", "total").unwrap_or(Decimal::ZERO);
        }
    }

    let unpaid = contract_total - payment_total;
    let rate = |a: Decimal, b: Decimal| if b > Decimal::ZERO { round_pct(a / b * Decimal::from(100)) } else { Decimal::ZERO };

    Ok(PaymentCompletionVO {
        year: Some(year),
        total_contract_amount: Some(contract_total),
        total_payment_amount: Some(payment_total),
        completion_rate: Some(rate(payment_total, contract_total)),
        overdue_amount: Some(Decimal::ZERO),
        overdue_rate: Some(Decimal::ZERO),
        unpaid_amount: Some(unpaid),
        unpaid_rate: Some(rate(unpaid, contract_total)),
    })
}

/// 回款状态分析：汇总表仅有回款总额维度，状态分桶依赖明细 → 该接口历史区间也走实时（有索引），此处返回 None 表示不支持
pub async fn payment_ranking(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
    order_by: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<PaymentRankingVO>> {
    if scope_is_empty(scope) {
        return Ok(Vec::new());
    }
    let order_by = order_by.unwrap_or_else(|| "payment_amount".to_string());
    let limit = limit.unwrap_or(10) as usize;

    let by_customer = db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT customer_id AS id, SUM(payment_amount) AS total
               FROM mxx_statistics_daily_payment
               WHERE stat_date BETWEEN $1::date AND $2::date AND customer_id > 0
                 AND ($3::int8[] IS NULL OR employee_id = ANY($3::int8[]))
               GROUP BY customer_id"#,
            [range.start.into(), range.end.into(), scope_values(scope)],
        ))
        .await?;
    let by_employee = db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT employee_id AS id, SUM(payment_amount) AS total
               FROM mxx_statistics_daily_payment
               WHERE stat_date BETWEEN $1::date AND $2::date AND employee_id > 0
                 AND ($3::int8[] IS NULL OR employee_id = ANY($3::int8[]))
               GROUP BY employee_id"#,
            [range.start.into(), range.end.into(), scope_values(scope)],
        ))
        .await?;

    // 对应合同金额（同样从汇总表取）
    let c_contracts: HashMap<i64, Decimal> = db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT customer_id AS id, SUM(contract_amount) AS total
               FROM mxx_statistics_daily_contract
               WHERE stat_date BETWEEN $1::date AND $2::date AND customer_id > 0
                 AND ($3::int8[] IS NULL OR employee_id = ANY($3::int8[]))
               GROUP BY customer_id"#,
            [range.start.into(), range.end.into(), scope_values(scope)],
        ))
        .await?
        .into_iter()
        .filter_map(|r| {
            let id: i64 = r.try_get("", "id").ok()?;
            let total: Decimal = r.try_get("", "total").unwrap_or(Decimal::ZERO);
            Some((id, total))
        })
        .collect();
    let e_contracts: HashMap<i64, Decimal> = db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT employee_id AS id, SUM(contract_amount) AS total
               FROM mxx_statistics_daily_contract
               WHERE stat_date BETWEEN $1::date AND $2::date AND employee_id > 0
                 AND ($3::int8[] IS NULL OR employee_id = ANY($3::int8[]))
               GROUP BY employee_id"#,
            [range.start.into(), range.end.into(), scope_values(scope)],
        ))
        .await?
        .into_iter()
        .filter_map(|r| {
            let id: i64 = r.try_get("", "id").ok()?;
            let total: Decimal = r.try_get("", "total").unwrap_or(Decimal::ZERO);
            Some((id, total))
        })
        .collect();

    // 名称映射
    let cids: Vec<i64> = by_customer.iter().filter_map(|r| r.try_get::<i64>("", "id").ok()).collect();
    let mut cname: HashMap<i64, String> = HashMap::new();
    if !cids.is_empty() {
        for c in Customer::find()
            .filter(crate::modules::crm::entity::customer::Column::Id.is_in(cids))
            .all(db)
            .await?
        {
            let n = c.company_name.or(c.person_name).or(c.nickname).unwrap_or_default();
            if !n.is_empty() {
                cname.insert(c.id, n);
            }
        }
    }
    let eids: Vec<i64> = by_employee.iter().filter_map(|r| r.try_get::<i64>("", "id").ok()).collect();
    let mut ename: HashMap<i64, String> = HashMap::new();
    if !eids.is_empty() {
        for a in Admin::find()
            .filter(crate::modules::system::entity::admin::Column::Id.is_in(eids))
            .all(db)
            .await?
        {
            let n = a.nick_name.or(a.user_name).unwrap_or_default();
            if !n.is_empty() {
                ename.insert(a.id, n);
            }
        }
    }

    let mut result: Vec<PaymentRankingVO> = Vec::new();
    for row in by_customer {
        let cid: i64 = row.try_get("", "id").unwrap_or(0);
        let pa: Decimal = row.try_get("", "total").unwrap_or(Decimal::ZERO);
        let ca = c_contracts.get(&cid).copied().unwrap_or(Decimal::ZERO);
        result.push(PaymentRankingVO {
            rank: None,
            target_type: Some("customer".to_string()),
            target_id: Some(cid),
            target_name: cname.get(&cid).cloned(),
            contract_amount: Some(ca),
            payment_amount: Some(pa),
            completion_rate: Some(if ca > Decimal::ZERO { round_pct(pa / ca * Decimal::from(100)) } else { Decimal::ZERO }),
            overdue_amount: Some(Decimal::ZERO),
        });
    }
    for row in by_employee {
        let eid: i64 = row.try_get("", "id").unwrap_or(0);
        let pa: Decimal = row.try_get("", "total").unwrap_or(Decimal::ZERO);
        let ca = e_contracts.get(&eid).copied().unwrap_or(Decimal::ZERO);
        result.push(PaymentRankingVO {
            rank: None,
            target_type: Some("employee".to_string()),
            target_id: Some(eid),
            target_name: ename.get(&eid).cloned(),
            contract_amount: Some(ca),
            payment_amount: Some(pa),
            completion_rate: Some(if ca > Decimal::ZERO { round_pct(pa / ca * Decimal::from(100)) } else { Decimal::ZERO }),
            overdue_amount: Some(Decimal::ZERO),
        });
    }

    if order_by == "completion_rate" {
        result.sort_by(|a, b| {
            b.completion_rate
                .unwrap_or(Decimal::ZERO)
                .cmp(&a.completion_rate.unwrap_or(Decimal::ZERO))
        });
    } else {
        result.sort_by(|a, b| {
            b.payment_amount
                .unwrap_or(Decimal::ZERO)
                .cmp(&a.payment_amount.unwrap_or(Decimal::ZERO))
        });
    }
    result.truncate(limit);
    for (i, item) in result.iter_mut().enumerate() {
        item.rank = Some((i + 1) as i32);
    }
    Ok(result)
}

// ============ 客户 ============
// 客户维度（类型/来源/行业）与漏斗：
// 线索不在汇总表、合同数需 join 客户维度，汇总表口径无法完全对齐 → 始终走实时路径（P1 已下推 + 索引）。
// 汇总表服务：合同 / 回款 / 员工 三个 topic 的查询加速。

// ============ 员工 ============

/// 员工汇总行
async fn employee_agg_rows(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<Vec<sea_orm::QueryResult>> {
    Ok(db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT employee_id AS eid,
                      SUM(new_customers)::int8 AS nc,
                      SUM(contract_customers)::int8 AS cc,
                      SUM(followup_total)::int8 AS ft,
                      SUM(followup_customer)::int8 AS fc,
                      SUM(followup_opportunity)::int8 AS fo,
                      SUM(new_opportunities)::int8 AS no_,
                      SUM(won_opportunities)::int8 AS wo,
                      SUM(lost_opportunities)::int8 AS lo,
                      SUM(contract_count)::int8 AS cnt,
                      SUM(contract_amount) AS amt
               FROM mxx_statistics_daily_employee
               WHERE stat_date BETWEEN $1::date AND $2::date
                 AND ($3::int8[] IS NULL OR employee_id = ANY($3::int8[]))
               GROUP BY employee_id"#,
            [range.start.into(), range.end.into(), scope_values(scope)],
        ))
        .await?)
}

pub async fn employee_customer_count(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
    admins: Vec<(i64, Option<String>, Option<String>)>,
    total_map: &HashMap<i64, i64>,
) -> Result<Vec<EmployeeCustomerCountVO>> {
    let mut cc_map: HashMap<i64, i64> = HashMap::new();
    let mut nc_map: HashMap<i64, i64> = HashMap::new();
    if !scope_is_empty(scope) {
        for row in employee_agg_rows(db, range, scope).await? {
            let eid: i64 = row.try_get("", "eid").unwrap_or(0);
            cc_map.insert(eid, row.try_get("", "cc").unwrap_or(0));
            nc_map.insert(eid, row.try_get("", "nc").unwrap_or(0));
        }
    }

    let mut result: Vec<EmployeeCustomerCountVO> = admins
        .into_iter()
        .map(|(id, name, dept)| {
            let total = total_map.get(&id).copied().unwrap_or(0);
            let contract_cust = cc_map.get(&id).copied().unwrap_or(0);
            let new_cust = nc_map.get(&id).copied().unwrap_or(0);
            let rate = if total > 0 {
                round_pct(Decimal::from(contract_cust) / Decimal::from(total) * Decimal::from(100))
            } else {
                Decimal::ZERO
            };
            EmployeeCustomerCountVO {
                employee_id: Some(id),
                employee_name: name,
                department_name: dept,
                total_customers: Some(total),
                new_customers_this_month: Some(new_cust),
                contract_customers: Some(contract_cust),
                customer_conversion_rate: Some(rate),
            }
        })
        .collect();
    result.sort_by(|a, b| b.total_customers.unwrap_or(0).cmp(&a.total_customers.unwrap_or(0)));
    Ok(result)
}

pub async fn employee_follow_up(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
    admins: Vec<(i64, Option<String>, Option<String>)>,
    no_follow_map: &HashMap<i64, i64>,
) -> Result<Vec<EmployeeFollowUpVO>> {
    let mut stats: HashMap<i64, (i64, i64, i64)> = HashMap::new();
    if !scope_is_empty(scope) {
        for row in employee_agg_rows(db, range, scope).await? {
            let eid: i64 = row.try_get("", "eid").unwrap_or(0);
            stats.insert(
                eid,
                (
                    row.try_get("", "ft").unwrap_or(0),
                    row.try_get("", "fc").unwrap_or(0),
                    row.try_get("", "fo").unwrap_or(0),
                ),
            );
        }
    }

    let mut result: Vec<EmployeeFollowUpVO> = admins
        .into_iter()
        .map(|(id, name, dept)| {
            let (ft, fc, fo) = stats.get(&id).cloned().unwrap_or((0, 0, 0));
            EmployeeFollowUpVO {
                employee_id: Some(id),
                employee_name: name,
                department_name: dept,
                total_follow_up: Some(ft),
                customer_follow_up: Some(fc),
                opportunity_follow_up: Some(fo),
                avg_follow_interval: Some(Decimal::ZERO),
                customers_without_follow_30_days: no_follow_map.get(&id).copied(),
            }
        })
        .collect();
    result.sort_by(|a, b| b.total_follow_up.unwrap_or(0).cmp(&a.total_follow_up.unwrap_or(0)));
    Ok(result)
}

pub async fn employee_conversion(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
    admins: Vec<(i64, Option<String>, Option<String>)>,
) -> Result<Vec<EmployeeConversionVO>> {
    let mut opp: HashMap<i64, (i64, i64, i64)> = HashMap::new();
    let mut ct: HashMap<i64, (i64, Decimal)> = HashMap::new();
    if !scope_is_empty(scope) {
        for row in employee_agg_rows(db, range, scope).await? {
            let eid: i64 = row.try_get("", "eid").unwrap_or(0);
            opp.insert(
                eid,
                (
                    row.try_get("", "no_").unwrap_or(0),
                    row.try_get("", "wo").unwrap_or(0),
                    row.try_get("", "lo").unwrap_or(0),
                ),
            );
            ct.insert(
                eid,
                (
                    row.try_get("", "cnt").unwrap_or(0),
                    row.try_get("", "amt").unwrap_or(Decimal::ZERO),
                ),
            );
        }
    }

    let mut result: Vec<EmployeeConversionVO> = admins
        .into_iter()
        .map(|(id, name, dept)| {
            let (to, wo, lo) = opp.get(&id).cloned().unwrap_or((0, 0, 0));
            let (tc, ca) = ct.get(&id).cloned().unwrap_or((0, Decimal::ZERO));
            let win_rate = if to > 0 {
                round_pct(Decimal::from(wo) / Decimal::from(to) * Decimal::from(100))
            } else {
                Decimal::ZERO
            };
            let avg = if tc > 0 { ca / Decimal::from(tc) } else { Decimal::ZERO };
            EmployeeConversionVO {
                employee_id: Some(id),
                employee_name: name,
                department_name: dept,
                total_opportunities: Some(to),
                won_opportunities: Some(wo),
                lost_opportunities: Some(lo),
                opportunity_win_rate: Some(win_rate),
                total_contracts: Some(tc),
                contract_amount: Some(ca),
                avg_contract_amount: Some(avg),
                avg_sales_cycle_days: Some(0),
            }
        })
        .collect();
    result.sort_by(|a, b| b.total_contracts.unwrap_or(0).cmp(&a.total_contracts.unwrap_or(0)));
    Ok(result)
}
