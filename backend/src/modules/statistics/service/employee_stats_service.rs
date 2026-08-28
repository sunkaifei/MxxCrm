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
use crate::modules::statistics::model::employee_stats::{
    EmployeeConversionVO, EmployeeCustomerCountVO, EmployeeFollowUpVO,
};
use crate::modules::statistics::service::stats_range::{
    date_param, ids_param, scope_is_empty, StatsRange, StatsScope,
};
use crate::modules::system::entity::admin::Entity as Admin;
use crate::modules::system::entity::admin_dept_merge::Entity as AdminDeptMerge;
use crate::modules::system::entity::dept::Entity as Dept;
use sea_orm::prelude::Decimal;
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

/// 员工基础信息（scope 内员工 + 部门名映射）——公共输入，双路径共用
/// admins 为小表，直接全量加载后内存筛选（避免 select_only 的类型转换复杂度）
pub async fn load_admins(
    db: &DbConn,
    scope: &StatsScope,
) -> Result<Vec<(i64, Option<String>, Option<String>)>> {
    let mut query = Admin::find()
        .filter(crate::modules::system::entity::admin::Column::Deleted.eq(0));
    if let Some(ids) = scope {
        if !ids.is_empty() {
            query = query.filter(crate::modules::system::entity::admin::Column::Id.is_in(ids.clone()));
        }
    }
    let admins: Vec<_> = query
        .all(db)
        .await?
        .into_iter()
        .map(|a| (a.id, a.nick_name.or(a.user_name)))
        .collect();

    // 部门映射（小表，全量加载）
    let mut dept_map: HashMap<i64, String> = HashMap::new();
    for d in Dept::find().all(db).await? {
        if let Some(name) = d.dept_name {
            dept_map.insert(d.id, name);
        }
    }
    let mut admin_dept_map: HashMap<i64, String> = HashMap::new();
    for m in AdminDeptMerge::find().all(db).await? {
        if let (Some(aid), Some(did)) = (m.admin_id, m.dept_id) {
            if let Some(name) = dept_map.get(&did) {
                admin_dept_map.insert(aid, name.clone());
            }
        }
    }

    Ok(admins
        .into_iter()
        .map(|(id, name)| (id, name, admin_dept_map.get(&id).cloned()))
        .collect())
}

/// 当年已通过年度销售计划的员工 ID 集合（销售身份的唯一事实源）
/// status=2 已通过；冻结/多级审批不影响该口径
pub async fn load_planned_sales_ids(db: &DbConn, year: i32) -> Result<Vec<i64>> {
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT DISTINCT employee_id FROM mxx_statistics_performance_plan
               WHERE deleted = 0 AND year = $1 AND status = 2 AND employee_id IS NOT NULL"#,
            [year.into()],
        ))
        .await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| r.try_get::<i64>("", "employee_id").ok())
        .collect())
}

/// 员工总客户数（存量指标，公共输入）
pub async fn load_total_customer_map(
    db: &DbConn,
    scope: &StatsScope,
) -> Result<HashMap<i64, i64>> {
    let mut map: HashMap<i64, i64> = HashMap::new();
    if scope_is_empty(scope) {
        return Ok(map);
    }
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT assigned_to AS eid, COUNT(*)::int8 AS cnt
               FROM mxx_crm_customer
               WHERE deleted = 0 AND assigned_to IS NOT NULL
                 AND ($1::int8[] IS NULL OR assigned_to = ANY($1::int8[]))
               GROUP BY assigned_to"#,
            [scope_values(scope)],
        ))
        .await?;
    for row in rows {
        map.insert(row.try_get("", "eid").unwrap_or(0), row.try_get("", "cnt").unwrap_or(0));
    }
    Ok(map)
}

/// 30 天未跟进客户数（公共输入，NOT EXISTS 单趟）
pub async fn load_no_follow_map(
    db: &DbConn,
    scope: &StatsScope,
) -> Result<HashMap<i64, i64>> {
    let mut map: HashMap<i64, i64> = HashMap::new();
    if scope_is_empty(scope) {
        return Ok(map);
    }
    let cutoff = chrono::Local::now().date_naive() - chrono::Duration::days(30);
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT c.assigned_to AS eid, COUNT(*)::int8 AS no_follow_cnt
               FROM mxx_crm_customer c
               WHERE c.deleted = 0 AND c.assigned_to IS NOT NULL
                 AND ($1::int8[] IS NULL OR c.assigned_to = ANY($1::int8[]))
                 AND NOT EXISTS (
                     SELECT 1 FROM mxx_crm_followup f
                     WHERE f.deleted = 0
                       AND f.customer_id = c.id
                       AND COALESCE(f.assigned_to, f.created_by) = c.assigned_to
                       AND f.create_time >= $2::timestamp
                 )
               GROUP BY c.assigned_to"#,
            [scope_values(scope), date_param(Some(cutoff))],
        ))
        .await?;
    for row in rows {
        map.insert(row.try_get("", "eid").unwrap_or(0), row.try_get("", "no_follow_cnt").unwrap_or(0));
    }
    Ok(map)
}

/// 员工客户数统计（实时路径；公共输入由调用方提供）
pub async fn get_employee_customer_count_realtime(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
    _department_id: Option<i64>,
    admins: Vec<(i64, Option<String>, Option<String>)>,
    total_map: HashMap<i64, i64>,
) -> Result<Vec<EmployeeCustomerCountVO>> {
    if admins.is_empty() {
        return Ok(Vec::new());
    }

    // 范围内新增客户
    let mut new_map: HashMap<i64, i64> = HashMap::new();
    // 范围内成交客户（时间范围内有签约合同的客户数）
    let mut contract_map: HashMap<i64, i64> = HashMap::new();

    if !scope_is_empty(scope) {
        let rows = db
            .query_all_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT assigned_to AS eid,
                          COUNT(CASE WHEN ($1::date IS NULL OR create_time >= $1::timestamp)
                                  AND ($2::date IS NULL OR create_time < ($2::date + INTERVAL '1 day'))
                                 THEN 1 END)::int8 AS new_cnt
                   FROM mxx_crm_customer
                   WHERE deleted = 0 AND assigned_to IS NOT NULL
                     AND ($3::int8[] IS NULL OR assigned_to = ANY($3::int8[]))
                   GROUP BY assigned_to"#,
                [
                    date_param(range.start),
                    date_param(range.end),
                    scope_values(scope),
                ],
            ))
            .await?;
        for row in rows {
            let eid: i64 = row.try_get("", "eid").unwrap_or(0);
            new_map.insert(eid, row.try_get("", "new_cnt").unwrap_or(0));
        }

        let rows = db
            .query_all_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT assigned_to AS eid, COUNT(DISTINCT customer_id)::int8 AS contract_cnt
                   FROM mxx_crm_contract
                   WHERE deleted = 0 AND customer_id IS NOT NULL AND assigned_to IS NOT NULL
                     AND ($1::date IS NULL OR sign_date >= $1::date)
                     AND ($2::date IS NULL OR sign_date <= $2::date)
                     AND ($3::int8[] IS NULL OR assigned_to = ANY($3::int8[]))
                   GROUP BY assigned_to"#,
                [
                    date_param(range.start),
                    date_param(range.end),
                    scope_values(scope),
                ],
            ))
            .await?;
        for row in rows {
            contract_map.insert(row.try_get("", "eid").unwrap_or(0), row.try_get("", "contract_cnt").unwrap_or(0));
        }
    }

    let mut result: Vec<EmployeeCustomerCountVO> = admins
        .into_iter()
        .map(|(id, name, dept)| {
            let total = total_map.get(&id).copied().unwrap_or(0);
            let new_cust = new_map.get(&id).copied().unwrap_or(0);
            let contract_cust = contract_map.get(&id).copied().unwrap_or(0);
            let conversion_rate = if total > 0 {
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
                customer_conversion_rate: Some(conversion_rate),
            }
        })
        .collect();

    result.sort_by(|a, b| b.total_customers.unwrap_or(0).cmp(&a.total_customers.unwrap_or(0)));
    Ok(result)
}

/// 员工跟进统计（实时路径；O(n²) 消除：窗口函数 + NOT EXISTS 全部下推数据库；公共输入由调用方提供）
pub async fn get_employee_follow_up_realtime(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
    _department_id: Option<i64>,
    admins: Vec<(i64, Option<String>, Option<String>)>,
    no_follow_map: HashMap<i64, i64>,
) -> Result<Vec<EmployeeFollowUpVO>> {
    if admins.is_empty() {
        return Ok(Vec::new());
    }

    // 跟进次数（总/客户/商机，条件聚合）+ 平均跟进间隔（LAG 窗口函数）
    let mut stats_map: HashMap<i64, (i64, i64, i64, Option<Decimal>)> = HashMap::new();
    if !scope_is_empty(scope) {
        let rows = db
            .query_all_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT agg.eid, agg.total_cnt, agg.customer_cnt, agg.opp_cnt, iv.avg_interval FROM (
                       SELECT COALESCE(assigned_to, created_by) AS eid,
                              COUNT(*)::int8 AS total_cnt,
                              COUNT(CASE WHEN source_type = 2 THEN 1 END)::int8 AS customer_cnt,
                              COUNT(CASE WHEN source_type = 3 THEN 1 END)::int8 AS opp_cnt
                       FROM mxx_crm_followup
                       WHERE deleted = 0 AND COALESCE(assigned_to, created_by) > 0
                         AND ($1::date IS NULL OR create_time >= $1::timestamp)
                         AND ($2::date IS NULL OR create_time < ($2::date + INTERVAL '1 day'))
                         AND ($3::int8[] IS NULL OR COALESCE(assigned_to, created_by) = ANY($3::int8[]))
                       GROUP BY 1
                   ) agg
                   LEFT JOIN (
                       SELECT eid, AVG(days) AS avg_interval FROM (
                           SELECT COALESCE(assigned_to, created_by) AS eid,
                                  EXTRACT(EPOCH FROM (create_time - LAG(create_time)
                                      OVER (PARTITION BY COALESCE(assigned_to, created_by) ORDER BY create_time))) / 86400.0 AS days
                           FROM mxx_crm_followup
                           WHERE deleted = 0 AND COALESCE(assigned_to, created_by) > 0
                             AND ($1::date IS NULL OR create_time >= $1::timestamp)
                             AND ($2::date IS NULL OR create_time < ($2::date + INTERVAL '1 day'))
                             AND ($3::int8[] IS NULL OR COALESCE(assigned_to, created_by) = ANY($3::int8[]))
                       ) t WHERE days IS NOT NULL GROUP BY eid
                   ) iv ON iv.eid = agg.eid"#,
                [
                    date_param(range.start),
                    date_param(range.end),
                    scope_values(scope),
                ],
            ))
            .await?;
        for row in rows {
            let eid: i64 = row.try_get("", "eid").unwrap_or(0);
            let total: i64 = row.try_get("", "total_cnt").unwrap_or(0);
            let customer: i64 = row.try_get("", "customer_cnt").unwrap_or(0);
            let opp: i64 = row.try_get("", "opp_cnt").unwrap_or(0);
            let avg: Option<Decimal> = row.try_get("", "avg_interval").ok().flatten();
            stats_map.insert(eid, (total, customer, opp, avg));
        }
    }

    // 30 天未跟进客户数已由公共输入 no_follow_map 提供（双路径共用）

    let mut result: Vec<EmployeeFollowUpVO> = admins
        .into_iter()
        .map(|(id, name, dept)| {
            let (total, customer, opp, avg) = stats_map
                .get(&id)
                .cloned()
                .unwrap_or((0, 0, 0, Some(Decimal::ZERO)));
            EmployeeFollowUpVO {
                employee_id: Some(id),
                employee_name: name,
                department_name: dept,
                total_follow_up: Some(total),
                customer_follow_up: Some(customer),
                opportunity_follow_up: Some(opp),
                avg_follow_interval: avg.or(Some(Decimal::ZERO)),
                customers_without_follow_30_days: no_follow_map.get(&id).copied(),
            }
        })
        .collect();

    result.sort_by(|a, b| b.total_follow_up.unwrap_or(0).cmp(&a.total_follow_up.unwrap_or(0)));
    Ok(result)
}

/// 员工转化率统计（实时路径；商机/合同聚合下推；公共输入由调用方提供）
pub async fn get_employee_conversion_realtime(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
    _department_id: Option<i64>,
    admins: Vec<(i64, Option<String>, Option<String>)>,
) -> Result<Vec<EmployeeConversionVO>> {
    if admins.is_empty() {
        return Ok(Vec::new());
    }

    // 商机：总/赢单/输单（stage=4 赢单, 5 输单，条件聚合）
    let mut opp_map: HashMap<i64, (i64, i64, i64)> = HashMap::new();
    // 合同：数量/金额
    let mut ct_map: HashMap<i64, (i64, Decimal)> = HashMap::new();
    // 平均赢单日（按员工）
    let mut won_avg_date_map: HashMap<i64, Option<chrono::NaiveDate>> = HashMap::new();
    // 平均销售周期近似值：平均签约日 - 平均赢单日（天，i64）
    let mut cycle_map: HashMap<i64, Option<i64>> = HashMap::new();

    if !scope_is_empty(scope) {
        let rows = db
            .query_all_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT COALESCE(assigned_to, created_by) AS eid,
                          COUNT(*)::int8 AS total_cnt,
                          COUNT(CASE WHEN stage = 4 THEN 1 END)::int8 AS won_cnt,
                          COUNT(CASE WHEN stage = 5 THEN 1 END)::int8 AS lost_cnt,
                          -- PG 无 avg(date)：先减基准日得天数（int，可 AVG），平均后转回 int 再加基准日
                          (DATE '1970-01-01' + AVG(CASE WHEN stage = 4 AND actual_close_date IS NOT NULL THEN actual_close_date - DATE '1970-01-01' END)::int4)::date AS won_avg_date
                   FROM mxx_crm_opportunity
                   WHERE deleted = 0 AND COALESCE(assigned_to, created_by) > 0
                     AND stage <> 6
                     AND ($1::date IS NULL OR create_time >= $1::timestamp)
                     AND ($2::date IS NULL OR create_time < ($2::date + INTERVAL '1 day'))
                     AND ($3::int8[] IS NULL OR COALESCE(assigned_to, created_by) = ANY($3::int8[]))
                   GROUP BY 1"#,
                [
                    date_param(range.start),
                    date_param(range.end),
                    scope_values(scope),
                ],
            ))
            .await?;
        for row in rows {
            let eid: i64 = row.try_get("", "eid").unwrap_or(0);
            let total: i64 = row.try_get("", "total_cnt").unwrap_or(0);
            let won: i64 = row.try_get("", "won_cnt").unwrap_or(0);
            let lost: i64 = row.try_get("", "lost_cnt").unwrap_or(0);
            opp_map.insert(eid, (total, won, lost));
            won_avg_date_map.insert(eid, row.try_get("", "won_avg_date").ok().flatten());
        }

        let rows = db
            .query_all_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT assigned_to AS eid,
                          COUNT(*)::int8 AS contract_cnt,
                          COALESCE(SUM(amount), 0) AS total_amount,
                          (DATE '1970-01-01' + AVG(sign_date - DATE '1970-01-01')::int4)::date AS sign_avg_date
                   FROM mxx_crm_contract
                   WHERE deleted = 0 AND assigned_to IS NOT NULL
                     AND ($1::date IS NULL OR sign_date >= $1::date)
                     AND ($2::date IS NULL OR sign_date <= $2::date)
                     AND ($3::int8[] IS NULL OR assigned_to = ANY($3::int8[]))
                   GROUP BY assigned_to"#,
                [
                    date_param(range.start),
                    date_param(range.end),
                    scope_values(scope),
                ],
            ))
            .await?;
        for row in rows {
            let eid: i64 = row.try_get("", "eid").unwrap_or(0);
            let cnt: i64 = row.try_get("", "contract_cnt").unwrap_or(0);
            let total: Decimal = row.try_get("", "total_amount").unwrap_or(Decimal::ZERO);
            let sign_avg: Option<chrono::NaiveDate> = row.try_get("", "sign_avg_date").ok().flatten();
            // 平均销售周期（近似口径）：平均签约日 - 平均赢单成交日（VO 字段为 i64 天数）
            let cycle = match (sign_avg, won_avg_date_map.get(&eid).cloned().flatten()) {
                (Some(s), Some(w)) => std::cmp::max((s - w).num_days(), 0),
                _ => 0,
            };
            ct_map.insert(eid, (cnt, total));
            cycle_map.insert(eid, Some(cycle));
        }
    }

    let mut result: Vec<EmployeeConversionVO> = admins
        .into_iter()
        .map(|(id, name, dept)| {
            let (to, wo, lo) = opp_map.get(&id).cloned().unwrap_or((0, 0, 0));
            let (tc, ca) = ct_map.get(&id).cloned().unwrap_or((0, Decimal::ZERO));
            let win_rate = if to > 0 {
                round_pct(Decimal::from(wo) / Decimal::from(to) * Decimal::from(100))
            } else {
                Decimal::ZERO
            };
            let avg_amount = if tc > 0 {
                ca / Decimal::from(tc)
            } else {
                Decimal::ZERO
            };
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
                avg_contract_amount: Some(avg_amount),
                avg_sales_cycle_days: cycle_map.get(&id).cloned().flatten().or(Some(0)),
            }
        })
        .collect();

    result.sort_by(|a, b| b.total_contracts.unwrap_or(0).cmp(&a.total_contracts.unwrap_or(0)));
    Ok(result)
}
