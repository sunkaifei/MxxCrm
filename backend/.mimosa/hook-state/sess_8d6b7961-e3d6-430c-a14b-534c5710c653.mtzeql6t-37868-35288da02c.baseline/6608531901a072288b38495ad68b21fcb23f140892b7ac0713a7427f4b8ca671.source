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
use crate::modules::statistics::model::contract_stats::{
    ContractRankingVO, ContractStatusAnalysisVO, ContractTypeDistributionVO,
};
use crate::modules::statistics::service::stats_range::{
    date_cond, date_param, ids_param, scope_cond, scope_is_empty, StatsRange, StatsScope,
};
use crate::modules::system::entity::admin::Entity as Admin;
use sea_orm::prelude::Decimal;
use sea_orm::prelude::Expr;
use sea_orm::*;
use std::collections::HashMap;
use rust_decimal::prelude::RoundingStrategy;

/// 统一百分比保留2位小数（后端兜底）
fn round_pct(d: Decimal) -> Decimal {
    d.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven)
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

/// 构建参数化原生 SQL 的公共 WHERE 片段参数（PG 占位符风格由调用方拼接）
/// 返回 (start, end, scope_ids) 三个 Value
fn range_scope_values(range: &StatsRange, scope: &StatsScope) -> [sea_orm::Value; 3] {
    [
        date_param(range.start),
        date_param(range.end),
        ids_param(scope),
    ]
}

/// 合同排行（P1 SQL 聚合下推版）
///
/// SQL 路径：
/// 1. 按客户聚合合同（count/sum）
/// 2. 按员工聚合合同（count/sum）
/// 3. 回款计划 JOIN 合同后按 (customer_id, assigned_to) 聚合，内存分摊到客户/员工桶
pub async fn get_contract_ranking(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
    order_by: Option<String>,
    _order_type: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<ContractRankingVO>> {
    if scope_is_empty(scope) {
        return Ok(Vec::new());
    }
    let order_by = order_by.unwrap_or_else(|| "amount".to_string());
    let limit = limit.unwrap_or(10) as usize;

    // ---- 1. 按客户聚合 ----
    let mut by_customer = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(scope_cond(contract::Column::AssignedTo, scope))
        .filter(date_cond(contract::Column::SignDate, range))
        .select_only()
        .column(contract::Column::CustomerId)
        .column_as(Expr::col(contract::Column::Id).count(), "cnt")
        .column_as(Expr::col(contract::Column::Amount).sum(), "total")
        .group_by(contract::Column::CustomerId)
        .into_tuple::<(Option<i64>, i64, Option<Decimal>)>()
        .all(db)
        .await?;

    // ---- 2. 按员工聚合 ----
    let mut by_employee = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(scope_cond(contract::Column::AssignedTo, scope))
        .filter(date_cond(contract::Column::SignDate, range))
        .select_only()
        .column(contract::Column::AssignedTo)
        .column_as(Expr::col(contract::Column::Id).count(), "cnt")
        .column_as(Expr::col(contract::Column::Amount).sum(), "total")
        .group_by(contract::Column::AssignedTo)
        .into_tuple::<(Option<i64>, i64, Option<Decimal>)>()
        .all(db)
        .await?;

    // ---- 3. 回款金额分摊（原生参数化 SQL：JOIN 后按客户+员工分组）----
    let mut payment_by_customer: HashMap<i64, Decimal> = HashMap::new();
    let mut payment_by_employee: HashMap<i64, Decimal> = HashMap::new();
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT c.customer_id AS customer_id, c.assigned_to AS assigned_to,
                      COALESCE(SUM(p.received_amount), 0) AS total
               FROM mxx_crm_contract_payment_plan p
               JOIN mxx_crm_contract c ON p.contract_id = c.id
               WHERE p.deleted = 0
                 AND ($1::date IS NULL OR c.sign_date >= $1::date)
                 AND ($2::date IS NULL OR c.sign_date <= $2::date)
                 AND ($3::int8[] IS NULL OR c.assigned_to = ANY($3::int8[]))
               GROUP BY c.customer_id, c.assigned_to"#,
            range_scope_values(range, scope),
        ))
        .await?;
    for row in rows {
        let cid: Option<i64> = row.try_get("", "customer_id").unwrap_or(None);
        let eid: Option<i64> = row.try_get("", "assigned_to").unwrap_or(None);
        let total: Decimal = row.try_get::<Decimal>("", "total").unwrap_or(Decimal::ZERO);
        if let Some(cid) = cid {
            *payment_by_customer.entry(cid).or_insert(Decimal::ZERO) += total;
        }
        if let Some(eid) = eid {
            *payment_by_employee.entry(eid).or_insert(Decimal::ZERO) += total;
        }
    }

    // ---- 名称映射（批量 in 查询）----
    let cids: Vec<i64> = by_customer.iter().filter_map(|(c, _, _)| *c).collect();
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
    let eids: Vec<i64> = by_employee.iter().filter_map(|(e, _, _)| *e).collect();
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

    // ---- 组装结果 ----
    let mut result: Vec<ContractRankingVO> = Vec::new();
    for (cid, count, amount) in by_customer.drain(..) {
        let Some(cid) = cid else { continue };
        let amount = amount.unwrap_or(Decimal::ZERO);
        let payment = payment_by_customer.get(&cid).copied().unwrap_or(Decimal::ZERO);
        let payment_rate = if amount > Decimal::ZERO {
            round_pct(payment / amount * Decimal::from(100))
        } else {
            Decimal::ZERO
        };
        result.push(ContractRankingVO {
            rank: None,
            target_type: Some("customer".to_string()),
            target_id: Some(cid),
            target_name: customer_name_map.get(&cid).cloned(),
            contract_count: Some(count),
            contract_amount: Some(amount),
            payment_amount: Some(payment),
            payment_rate: Some(payment_rate),
        });
    }
    for (eid, count, amount) in by_employee.drain(..) {
        let Some(eid) = eid else { continue };
        let amount = amount.unwrap_or(Decimal::ZERO);
        let payment = payment_by_employee.get(&eid).copied().unwrap_or(Decimal::ZERO);
        let payment_rate = if amount > Decimal::ZERO {
            round_pct(payment / amount * Decimal::from(100))
        } else {
            Decimal::ZERO
        };
        result.push(ContractRankingVO {
            rank: None,
            target_type: Some("employee".to_string()),
            target_id: Some(eid),
            target_name: admin_name_map.get(&eid).cloned(),
            contract_count: Some(count),
            contract_amount: Some(amount),
            payment_amount: Some(payment),
            payment_rate: Some(payment_rate),
        });
    }

    // ---- 排序 + 截断 ----
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

/// 合同类型分布（单表聚合下推）
pub async fn get_contract_type_distribution(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<Vec<ContractTypeDistributionVO>> {
    if scope_is_empty(scope) {
        return Ok(Vec::new());
    }
    let rows = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(scope_cond(contract::Column::AssignedTo, scope))
        .filter(date_cond(contract::Column::SignDate, range))
        .select_only()
        .column_as(Expr::expr(Expr::cust("contract_type::int")), "contract_type")
        .column_as(Expr::col(contract::Column::Id).count(), "cnt")
        .column_as(Expr::col(contract::Column::Amount).sum(), "total")
        .group_by(contract::Column::ContractType)
        .into_tuple::<(Option<i32>, i64, Option<Decimal>)>()
        .all(db)
        .await?;

    let total_amount: Decimal = rows.iter().map(|(_, _, a)| a.unwrap_or(Decimal::ZERO)).sum();

    let mut result: Vec<ContractTypeDistributionVO> = rows
        .into_iter()
        .map(|(t, count, amount)| {
            let t = t.unwrap_or(0);
            let amount = amount.unwrap_or(Decimal::ZERO);
            ContractTypeDistributionVO {
                contract_type: Some(type_name(t).to_string()),
                contract_count: Some(count),
                contract_amount: Some(amount),
                percentage: if total_amount > Decimal::ZERO {
                    Some(round_pct(amount / total_amount * Decimal::from(100)))
                } else {
                    Some(Decimal::ZERO)
                },
            }
        })
        .collect();

    result.sort_by(|a, b| b.contract_count.unwrap_or(0).cmp(&a.contract_count.unwrap_or(0)));
    Ok(result)
}

/// 合同状态分析（单表聚合下推）
pub async fn get_contract_status_analysis(
    db: &DbConn,
    range: &StatsRange,
    scope: &StatsScope,
) -> Result<Vec<ContractStatusAnalysisVO>> {
    if scope_is_empty(scope) {
        return Ok(Vec::new());
    }
    let rows = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(scope_cond(contract::Column::AssignedTo, scope))
        .filter(date_cond(contract::Column::SignDate, range))
        .select_only()
        .column_as(Expr::expr(Expr::cust("status::int")), "status")
        .column_as(Expr::col(contract::Column::Id).count(), "cnt")
        .column_as(Expr::col(contract::Column::Amount).sum(), "total")
        .group_by(contract::Column::Status)
        .into_tuple::<(Option<i32>, i64, Option<Decimal>)>()
        .all(db)
        .await?;

    let total_amount: Decimal = rows.iter().map(|(_, _, a)| a.unwrap_or(Decimal::ZERO)).sum();

    let mut result: Vec<ContractStatusAnalysisVO> = rows
        .into_iter()
        .map(|(s, count, amount)| {
            let s = s.unwrap_or(0);
            let amount = amount.unwrap_or(Decimal::ZERO);
            ContractStatusAnalysisVO {
                status: Some(format!("{}", s)),
                status_name: Some(status_name(s).to_string()),
                contract_count: Some(count),
                contract_amount: Some(amount),
                percentage: if total_amount > Decimal::ZERO {
                    Some(round_pct(amount / total_amount * Decimal::from(100)))
                } else {
                    Some(Decimal::ZERO)
                },
            }
        })
        .collect();

    result.sort_by(|a, b| b.contract_count.unwrap_or(0).cmp(&a.contract_count.unwrap_or(0)));
    Ok(result)
}
