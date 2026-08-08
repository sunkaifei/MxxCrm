use crate::core::errors::error::Result;
use crate::modules::crm::entity::contract::{self, Entity as Contract};
use crate::modules::crm::entity::contract_payment_plan::{self, Entity as PaymentPlan};
use crate::modules::crm::entity::customer::Entity as Customer;
use crate::modules::system::entity::admin::Entity as Admin;
use crate::modules::statistics::model::contract_stats::{ContractRankingVO, ContractTypeDistributionVO, ContractStatusAnalysisVO};
use sea_orm::prelude::Decimal;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};
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

pub async fn get_contract_ranking(db: &DbConn, _year: Option<i32>, _month: Option<i32>, order_by: Option<String>, _order_type: Option<String>, _limit: Option<i64>) -> Result<Vec<ContractRankingVO>> {
    let order_by = order_by.unwrap_or("amount".to_string());
    let limit = _limit.unwrap_or(10) as usize;

    // 获取所有未删除合同
    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;

    // 按客户分组
    let mut customer_map: HashMap<i64, (i64, Decimal)> = HashMap::new();
    for c in &contracts {
        if let Some(cid) = c.customer_id {
            let e = customer_map.entry(cid).or_insert((0, Decimal::ZERO));
            e.0 += 1;
            e.1 += c.amount.unwrap_or(Decimal::ZERO);
        }
    }

    // 按员工分组
    let mut employee_map: HashMap<i64, (i64, Decimal)> = HashMap::new();
    for c in &contracts {
        if let Some(eid) = c.assigned_to {
            let e = employee_map.entry(eid).or_insert((0, Decimal::ZERO));
            e.0 += 1;
            e.1 += c.amount.unwrap_or(Decimal::ZERO);
        }
    }

    // 客户名称映射
    let cids: Vec<i64> = customer_map.keys().copied().collect();
    let mut customer_name_map: HashMap<i64, String> = HashMap::new();
    if !cids.is_empty() {
        for c in Customer::find().filter(crate::modules::crm::entity::customer::Column::Id.is_in(cids)).all(db).await? {
            let name = c.company_name.or(c.person_name).or(c.nickname).unwrap_or_default();
            if !name.is_empty() {
                customer_name_map.insert(c.id, name);
            }
        }
    }

    // 员工名称映射
    let eids: Vec<i64> = employee_map.keys().copied().collect();
    let mut admin_name_map: HashMap<i64, String> = HashMap::new();
    if !eids.is_empty() {
        for a in Admin::find().filter(crate::modules::system::entity::admin::Column::Id.is_in(eids)).all(db).await? {
            let name = a.nick_name.or(a.user_name).unwrap_or_default();
            if !name.is_empty() {
                admin_name_map.insert(a.id, name);
            }
        }
    }

    // 回款金额
    let all_cids: Vec<i64> = contracts.iter().map(|c| c.id).collect();
    let mut payment_by_contract: HashMap<i64, Decimal> = HashMap::new();
    if !all_cids.is_empty() {
        for p in PaymentPlan::find()
            .filter(contract_payment_plan::Column::ContractId.is_in(all_cids))
            .filter(contract_payment_plan::Column::Deleted.eq(0))
            .all(db).await?
        {
            if let Some(cid) = p.contract_id {
                *payment_by_contract.entry(cid).or_insert(Decimal::ZERO) += p.received_amount.unwrap_or(Decimal::ZERO);
            }
        }
    }

    let mut result: Vec<ContractRankingVO> = Vec::new();

    // 客户排名
    for (cid, (count, amount)) in &customer_map {
        let payment: Decimal = contracts.iter()
            .filter(|c| c.customer_id == Some(*cid))
            .filter_map(|c| payment_by_contract.get(&c.id))
            .sum();
        let payment_rate = if *amount > Decimal::ZERO {
            round_pct(payment / *amount * Decimal::from(100))
        } else {
            Decimal::ZERO
        };
        result.push(ContractRankingVO {
            rank: None,
            target_type: Some("customer".to_string()),
            target_id: Some(*cid),
            target_name: customer_name_map.get(cid).cloned(),
            contract_count: Some(*count),
            contract_amount: Some(*amount),
            payment_amount: Some(payment),
            payment_rate: Some(payment_rate),
        });
    }

    // 员工排名
    for (eid, (count, amount)) in &employee_map {
        let payment: Decimal = contracts.iter()
            .filter(|c| c.assigned_to == Some(*eid))
            .filter_map(|c| payment_by_contract.get(&c.id))
            .sum();
        let payment_rate = if *amount > Decimal::ZERO {
            round_pct(payment / *amount * Decimal::from(100))
        } else {
            Decimal::ZERO
        };
        result.push(ContractRankingVO {
            rank: None,
            target_type: Some("employee".to_string()),
            target_id: Some(*eid),
            target_name: admin_name_map.get(eid).cloned(),
            contract_count: Some(*count),
            contract_amount: Some(*amount),
            payment_amount: Some(payment),
            payment_rate: Some(payment_rate),
        });
    }

    // 排序
    if order_by == "count" {
        result.sort_by(|a, b| b.contract_count.unwrap_or(0).cmp(&a.contract_count.unwrap_or(0)));
    } else {
        result.sort_by(|a, b| b.contract_amount.unwrap_or(Decimal::ZERO).cmp(&a.contract_amount.unwrap_or(Decimal::ZERO)));
    }

    result.truncate(limit);

    for (i, item) in result.iter_mut().enumerate() {
        item.rank = Some((i + 1) as i32);
    }

    Ok(result)
}

pub async fn get_contract_type_distribution(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<Vec<ContractTypeDistributionVO>> {
    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut type_map: HashMap<i32, (i64, Decimal)> = HashMap::new();
    for c in &contracts {
        let t = c.contract_type.map(|ct| ct as i32).unwrap_or(0);
        let e = type_map.entry(t).or_insert((0, Decimal::ZERO));
        e.0 += 1;
        e.1 += c.amount.unwrap_or(Decimal::ZERO);
    }

    let total_amount: Decimal = type_map.values().map(|(_, a)| a).sum();

    let mut result: Vec<ContractTypeDistributionVO> = type_map.into_iter()
        .map(|(t, (count, amount))| ContractTypeDistributionVO {
            contract_type: Some(type_name(t).to_string()),
            contract_count: Some(count),
            contract_amount: Some(amount),
            percentage: if total_amount > Decimal::ZERO {
                Some(round_pct(amount / total_amount * Decimal::from(100)))
            } else {
                Some(Decimal::ZERO)
            },
        })
        .collect();

    result.sort_by(|a, b| b.contract_count.unwrap_or(0).cmp(&a.contract_count.unwrap_or(0)));
    Ok(result)
}

pub async fn get_contract_status_analysis(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<Vec<ContractStatusAnalysisVO>> {
    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut status_map: HashMap<i32, (i64, Decimal)> = HashMap::new();
    for c in &contracts {
        let s = c.status.map(|cs| cs as i32).unwrap_or(0);
        let e = status_map.entry(s).or_insert((0, Decimal::ZERO));
        e.0 += 1;
        e.1 += c.amount.unwrap_or(Decimal::ZERO);
    }

    let total_amount: Decimal = status_map.values().map(|(_, a)| a).sum();

    let mut result: Vec<ContractStatusAnalysisVO> = status_map.into_iter()
        .map(|(s, (count, amount))| ContractStatusAnalysisVO {
            status: Some(format!("{}", s)),
            status_name: Some(status_name(s).to_string()),
            contract_count: Some(count),
            contract_amount: Some(amount),
            percentage: if total_amount > Decimal::ZERO {
                Some(round_pct(amount / total_amount * Decimal::from(100)))
            } else {
                Some(Decimal::ZERO)
            },
        })
        .collect();

    result.sort_by(|a, b| b.contract_count.unwrap_or(0).cmp(&a.contract_count.unwrap_or(0)));
    Ok(result)
}