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
use crate::modules::crm::entity::contract::{self, Entity as Contract};
use crate::modules::crm::entity::customer::Entity as Customer;
use crate::modules::sale::entity::payment::{self, Entity as SalePayment};
use crate::modules::statistics::model::payment_stats::{
    PaymentCompletionVO, PaymentMonthlyTrendStatsVO, PaymentMonthlyTrendVO,
    PaymentRankingVO, PaymentStatusAnalysisVO,
};
use crate::modules::statistics::service::stats_range::{
    date_cond, date_param, ids_param, scope_cond, scope_is_empty, StatsRange, StatsScope,
};
use crate::modules::system::entity::admin::Entity as Admin;
use chrono::Datelike;
use sea_orm::prelude::Decimal;
use sea_orm::prelude::Expr;
use sea_orm::*;
use std::collections::HashMap;
use rust_decimal::prelude::RoundingStrategy;

/// 统一百分比保留2位小数（后端兜底）
fn round_pct(d: Decimal) -> Decimal {
    d.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven)
}

fn scope_values(scope: &StatsScope) -> sea_orm::Value {
    ids_param(scope)
}

/// 回款完成情况（双聚合下推：合同按签约日 + 回款按回款日）
pub async fn get_payment_completion(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<PaymentCompletionVO> {
    let year = range
        .start
        .map_or(chrono::Local::now().year(), |s| s.year());

    if scope_is_empty(scope) {
        return Ok(PaymentCompletionVO {
            year: Some(year),
            total_contract_amount: Some(Decimal::ZERO),
            total_payment_amount: Some(Decimal::ZERO),
            completion_rate: Some(Decimal::ZERO),
            overdue_amount: Some(Decimal::ZERO),
            overdue_rate: Some(Decimal::ZERO),
            unpaid_amount: Some(Decimal::ZERO),
            unpaid_rate: Some(Decimal::ZERO),
        });
    }

    // 范围内合同总金额（按签约日期过滤）
    let contract_total = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(scope_cond(contract::Column::AssignedTo, scope))
        .filter(date_cond(contract::Column::SignDate, range))
        .select_only()
        .column_as(Expr::col(contract::Column::Amount).sum(), "total")
        .into_tuple::<Option<Decimal>>()
        .one(db)
        .await?
        .flatten()
        .unwrap_or(Decimal::ZERO);

    // 范围内回款总金额（已确认，按回款日期过滤）
    let payment_total = SalePayment::find()
        .filter(payment::Column::Deleted.eq(0))
        .filter(payment::Column::Status.eq(2))
        .filter(scope_cond(payment::Column::OwnerUserId, scope))
        .filter(date_cond(payment::Column::PaymentDate, range))
        .select_only()
        .column_as(Expr::col(payment::Column::Amount).sum(), "total")
        .into_tuple::<Option<Decimal>>()
        .one(db)
        .await?
        .flatten()
        .unwrap_or(Decimal::ZERO);

    // 逾期金额：简化处理，暂不计算
    let overdue = Decimal::ZERO;
    let unpaid_amount = contract_total - payment_total;
    let completion_rate = if contract_total > Decimal::ZERO {
        round_pct(payment_total / contract_total * Decimal::from(100))
    } else {
        Decimal::ZERO
    };
    let unpaid_rate = if contract_total > Decimal::ZERO {
        round_pct(unpaid_amount / contract_total * Decimal::from(100))
    } else {
        Decimal::ZERO
    };

    Ok(PaymentCompletionVO {
        year: Some(year),
        total_contract_amount: Some(contract_total),
        total_payment_amount: Some(payment_total),
        completion_rate: Some(completion_rate),
        overdue_amount: Some(overdue),
        overdue_rate: Some(Decimal::ZERO),
        unpaid_amount: Some(unpaid_amount),
        unpaid_rate: Some(unpaid_rate),
    })
}

/// 回款月度趋势（原生参数化 SQL：EXTRACT(MONTH) 分组）
pub async fn get_payment_monthly_trend(
    db: &DbConn,
    year: Option<i32>,
    scope: &StatsScope,
) -> Result<PaymentMonthlyTrendStatsVO> {
    let year = year.unwrap_or(chrono::Local::now().year());

    let mut contract_by_month: HashMap<i32, Decimal> = HashMap::new();
    let mut payment_by_month: HashMap<i32, Decimal> = HashMap::new();

    if !scope_is_empty(scope) {
        let rows = db
            .query_all_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT EXTRACT(MONTH FROM sign_date)::int AS m, COALESCE(SUM(amount), 0) AS total
                   FROM mxx_crm_contract
                   WHERE deleted = 0 AND sign_date IS NOT NULL
                     AND EXTRACT(YEAR FROM sign_date) = $1::int
                     AND ($2::int8[] IS NULL OR assigned_to = ANY($2::int8[]))
                   GROUP BY 1"#,
                [year.into(), scope_values(scope)],
            ))
            .await?;
        for row in rows {
            let m: i32 = row.try_get("", "m").unwrap_or(0);
            let total: Decimal = row.try_get("", "total").unwrap_or(Decimal::ZERO);
            contract_by_month.insert(m, total);
        }

        let rows = db
            .query_all_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT EXTRACT(MONTH FROM payment_date)::int AS m, COALESCE(SUM(amount), 0) AS total
                   FROM mxx_sale_payment
                   WHERE deleted = 0 AND status = 2 AND payment_date IS NOT NULL
                     AND EXTRACT(YEAR FROM payment_date) = $1::int
                     AND ($2::int8[] IS NULL OR owner_user_id = ANY($2::int8[]))
                   GROUP BY 1"#,
                [year.into(), scope_values(scope)],
            ))
            .await?;
        for row in rows {
            let m: i32 = row.try_get("", "m").unwrap_or(0);
            let total: Decimal = row.try_get("", "total").unwrap_or(Decimal::ZERO);
            payment_by_month.insert(m, total);
        }
    }

    let mut months = Vec::new();
    for m in 1..=12 {
        let ca = contract_by_month.get(&m).copied().unwrap_or(Decimal::ZERO);
        let pa = payment_by_month.get(&m).copied().unwrap_or(Decimal::ZERO);
        let rate = if ca > Decimal::ZERO {
            round_pct(pa / ca * Decimal::from(100))
        } else {
            Decimal::ZERO
        };
        months.push(PaymentMonthlyTrendVO {
            month: Some(m),
            contract_amount: Some(ca),
            payment_amount: Some(pa),
            completion_rate: Some(rate),
            overdue_amount: Some(Decimal::ZERO),
        });
    }

    Ok(PaymentMonthlyTrendStatsVO {
        year: Some(year),
        months: Some(months),
    })
}

/// 回款状态分析（原生参数化 SQL：LEFT JOIN + CASE 分组，分桶在数据库端完成，仅返回 4 行）
pub async fn get_payment_status_analysis(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<Vec<PaymentStatusAnalysisVO>> {
    let mut buckets: HashMap<String, (i64, Decimal, Decimal, i64, Decimal)> = HashMap::new();
    let mut total_amount = Decimal::ZERO;

    if !scope_is_empty(scope) {
        let rows = db
            .query_all_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT bucket, cnt, amount_sum, paid_sum, overdue_cnt, overdue_amount FROM (
                       SELECT
                         CASE WHEN c.amount > 0 AND COALESCE(p.paid, 0) >= c.amount THEN 'paid'
                              WHEN COALESCE(p.paid, 0) > 0 THEN 'partial'
                              ELSE 'unpaid' END AS bucket,
                         COUNT(*)::int8 AS cnt,
                         COALESCE(SUM(c.amount), 0) AS amount_sum,
                         COALESCE(SUM(COALESCE(p.paid, 0)), 0) AS paid_sum,
                         COALESCE(SUM(CASE WHEN COALESCE(p.paid, 0) < c.amount AND c.sign_date < CURRENT_DATE THEN 1 ELSE 0 END), 0)::int8 AS overdue_cnt,
                         COALESCE(SUM(CASE WHEN COALESCE(p.paid, 0) < c.amount AND c.sign_date < CURRENT_DATE THEN c.amount ELSE 0 END), 0) AS overdue_amount
                       FROM mxx_crm_contract c
                       LEFT JOIN (
                           SELECT contract_id, SUM(amount) AS paid
                           FROM mxx_sale_payment
                           WHERE deleted = 0 AND status = 2
                           GROUP BY contract_id
                       ) p ON p.contract_id = c.id
                       WHERE c.deleted = 0
                         AND ($1::date IS NULL OR c.sign_date >= $1::date)
                         AND ($2::date IS NULL OR c.sign_date <= $2::date)
                         AND ($3::int8[] IS NULL OR c.assigned_to = ANY($3::int8[]))
                       GROUP BY 1
                   ) t"#,
                [
                    date_param(range.start),
                    date_param(range.end),
                    scope_values(scope),
                ],
            ))
            .await?;

        for row in rows {
            let bucket: String = row.try_get("", "bucket").unwrap_or_default();
            let cnt: i64 = row.try_get("", "cnt").unwrap_or(0);
            let amount_sum: Decimal = row.try_get("", "amount_sum").unwrap_or(Decimal::ZERO);
            let paid_sum: Decimal = row.try_get("", "paid_sum").unwrap_or(Decimal::ZERO);
            let overdue_cnt: i64 = row.try_get("", "overdue_cnt").unwrap_or(0);
            let overdue_amount: Decimal = row.try_get("", "overdue_amount").unwrap_or(Decimal::ZERO);
            total_amount += amount_sum;
            buckets.insert(bucket, (cnt, amount_sum, paid_sum, overdue_cnt, overdue_amount));
        }
    }

    let get = |k: &str| buckets.get(k).cloned().unwrap_or((0, Decimal::ZERO, Decimal::ZERO, 0, Decimal::ZERO));
    let pct = |v: Decimal| -> Decimal {
        if total_amount > Decimal::ZERO {
            round_pct(v / total_amount * Decimal::from(100))
        } else {
            Decimal::ZERO
        }
    };

    let unpaid = get("unpaid");
    let partial = get("partial");
    let paid = get("paid");
    // 逾期桶：合并三个分桶的逾期计数
    let (mut o_cnt, mut o_amount) = (0i64, Decimal::ZERO);
    for (_, v) in buckets.iter() {
        o_cnt += v.3;
        o_amount += v.4;
    }

    Ok(vec![
        PaymentStatusAnalysisVO {
            status: Some("unpaid".to_string()),
            status_name: Some("未回款".to_string()),
            contract_count: Some(unpaid.0),
            contract_amount: Some(unpaid.1),
            paid_amount: Some(Decimal::ZERO),
            percentage: Some(pct(unpaid.1)),
        },
        PaymentStatusAnalysisVO {
            status: Some("partial".to_string()),
            status_name: Some("部分回款".to_string()),
            contract_count: Some(partial.0),
            contract_amount: Some(partial.1),
            paid_amount: Some(partial.2),
            percentage: Some(pct(partial.1)),
        },
        PaymentStatusAnalysisVO {
            status: Some("paid".to_string()),
            status_name: Some("已回款".to_string()),
            contract_count: Some(paid.0),
            contract_amount: Some(paid.1),
            paid_amount: Some(paid.2),
            percentage: Some(pct(paid.1)),
        },
        PaymentStatusAnalysisVO {
            status: Some("overdue".to_string()),
            status_name: Some("逾期".to_string()),
            contract_count: Some(o_cnt),
            contract_amount: Some(o_amount),
            paid_amount: Some(Decimal::ZERO),
            percentage: Some(pct(o_amount)),
        },
    ])
}

/// 回款排行（四路聚合：回款/合同 × 客户/员工）
pub async fn get_payment_ranking(
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

    // 回款按客户聚合
    let payment_by_customer = SalePayment::find()
        .filter(payment::Column::Deleted.eq(0))
        .filter(payment::Column::Status.eq(2))
        .filter(scope_cond(payment::Column::OwnerUserId, scope))
        .filter(date_cond(payment::Column::PaymentDate, range))
        .select_only()
        .column(payment::Column::CustomerId)
        .column_as(Expr::col(payment::Column::Amount).sum(), "total")
        .group_by(payment::Column::CustomerId)
        .into_tuple::<(Option<i64>, Option<Decimal>)>()
        .all(db)
        .await?;

    // 回款按员工聚合
    let payment_by_employee = SalePayment::find()
        .filter(payment::Column::Deleted.eq(0))
        .filter(payment::Column::Status.eq(2))
        .filter(scope_cond(payment::Column::OwnerUserId, scope))
        .filter(date_cond(payment::Column::PaymentDate, range))
        .select_only()
        .column(payment::Column::OwnerUserId)
        .column_as(Expr::col(payment::Column::Amount).sum(), "total")
        .group_by(payment::Column::OwnerUserId)
        .into_tuple::<(Option<i64>, Option<Decimal>)>()
        .all(db)
        .await?;

    // 合同金额按客户聚合
    let contract_by_customer = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(scope_cond(contract::Column::AssignedTo, scope))
        .filter(date_cond(contract::Column::SignDate, range))
        .select_only()
        .column(contract::Column::CustomerId)
        .column_as(Expr::col(contract::Column::Amount).sum(), "total")
        .group_by(contract::Column::CustomerId)
        .into_tuple::<(Option<i64>, Option<Decimal>)>()
        .all(db)
        .await?;

    // 合同金额按员工聚合
    let contract_by_employee = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(scope_cond(contract::Column::AssignedTo, scope))
        .filter(date_cond(contract::Column::SignDate, range))
        .select_only()
        .column(contract::Column::AssignedTo)
        .column_as(Expr::col(contract::Column::Amount).sum(), "total")
        .group_by(contract::Column::AssignedTo)
        .into_tuple::<(Option<i64>, Option<Decimal>)>()
        .all(db)
        .await?;

    let c_map: HashMap<i64, Decimal> = contract_by_customer
        .into_iter()
        .filter_map(|(id, amt)| id.map(|id| (id, amt.unwrap_or(Decimal::ZERO))))
        .collect();
    let e_map: HashMap<i64, Decimal> = contract_by_employee
        .into_iter()
        .filter_map(|(id, amt)| id.map(|id| (id, amt.unwrap_or(Decimal::ZERO))))
        .collect();

    // 名称映射
    let cids: Vec<i64> = payment_by_customer.iter().filter_map(|(c, _)| *c).collect();
    let mut customer_name_map: HashMap<i64, String> = HashMap::new();
    if !cids.is_empty() {
        for c in Customer::find()
            .filter(crate::modules::crm::entity::customer::Column::Id.is_in(cids))
            .all(db)
            .await?
        {
            let name = c.company_name.or(c.person_name).or(c.nickname).unwrap_or_default();
            if !name.is_empty() {
                customer_name_map.insert(c.id, name);
            }
        }
    }
    let eids: Vec<i64> = payment_by_employee.iter().filter_map(|(e, _)| *e).collect();
    let mut admin_name_map: HashMap<i64, String> = HashMap::new();
    if !eids.is_empty() {
        for a in Admin::find()
            .filter(crate::modules::system::entity::admin::Column::Id.is_in(eids))
            .all(db)
            .await?
        {
            let name = a.nick_name.or(a.user_name).unwrap_or_default();
            if !name.is_empty() {
                admin_name_map.insert(a.id, name);
            }
        }
    }

    let mut result: Vec<PaymentRankingVO> = Vec::new();
    for (cid, pa) in payment_by_customer {
        let Some(cid) = cid else { continue };
        let pa = pa.unwrap_or(Decimal::ZERO);
        let ca = c_map.get(&cid).copied().unwrap_or(Decimal::ZERO);
        let rate = if ca > Decimal::ZERO {
            round_pct(pa / ca * Decimal::from(100))
        } else {
            Decimal::ZERO
        };
        result.push(PaymentRankingVO {
            rank: None,
            target_type: Some("customer".to_string()),
            target_id: Some(cid),
            target_name: customer_name_map.get(&cid).cloned(),
            contract_amount: Some(ca),
            payment_amount: Some(pa),
            completion_rate: Some(rate),
            overdue_amount: Some(Decimal::ZERO),
        });
    }
    for (eid, pa) in payment_by_employee {
        let Some(eid) = eid else { continue };
        let pa = pa.unwrap_or(Decimal::ZERO);
        let ca = e_map.get(&eid).copied().unwrap_or(Decimal::ZERO);
        let rate = if ca > Decimal::ZERO {
            round_pct(pa / ca * Decimal::from(100))
        } else {
            Decimal::ZERO
        };
        result.push(PaymentRankingVO {
            rank: None,
            target_type: Some("employee".to_string()),
            target_id: Some(eid),
            target_name: admin_name_map.get(&eid).cloned(),
            contract_amount: Some(ca),
            payment_amount: Some(pa),
            completion_rate: Some(rate),
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
