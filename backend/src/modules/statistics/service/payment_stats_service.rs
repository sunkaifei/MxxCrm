use chrono::Datelike;
use crate::core::errors::error::Result;
use crate::modules::crm::entity::contract::{self, Entity as Contract};
use crate::modules::crm::entity::customer::Entity as Customer;
use crate::modules::sale::entity::payment::{self, Entity as SalePayment};
use crate::modules::statistics::model::payment_stats::{PaymentCompletionVO, PaymentMonthlyTrendStatsVO, PaymentMonthlyTrendVO, PaymentStatusAnalysisVO, PaymentRankingVO};
use crate::modules::system::entity::admin::Entity as Admin;
use sea_orm::prelude::Decimal;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};
use std::collections::HashMap;
use rust_decimal::prelude::RoundingStrategy;

/// 统一百分比保留2位小数（后端兜底）
fn round_pct(d: Decimal) -> Decimal {
    d.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven)
}

pub async fn get_payment_completion(db: &DbConn, year: Option<i32>, _month: Option<i32>) -> Result<PaymentCompletionVO> {
    let year = year.unwrap_or(chrono::Local::now().year() as i32);

    // 当年合同总金额
    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut total_contract_amount = Decimal::ZERO;
    for c in &contracts {
        if let Some(sd) = c.sign_date {
            if sd.year() == year {
                total_contract_amount += c.amount.unwrap_or(Decimal::ZERO);
            }
        }
    }

    // 当年回款总金额
    let payments = SalePayment::find()
        .filter(payment::Column::Deleted.eq(0))
        .filter(payment::Column::Status.eq(2))
        .all(db)
        .await?;

    let mut total_payment_amount = Decimal::ZERO;

    for p in &payments {
        if let Some(pd) = p.payment_date {
            if pd.year() == year {
                total_payment_amount += p.amount.unwrap_or(Decimal::ZERO);
            }
        }
    }

    // 逾期金额：简化处理，暂不计算
    let overdue = Decimal::ZERO;

    let unpaid_amount = total_contract_amount - total_payment_amount;
    let completion_rate = if total_contract_amount > Decimal::ZERO {
        round_pct(total_payment_amount / total_contract_amount * Decimal::from(100))
    } else {
        Decimal::ZERO
    };
    let unpaid_rate = if total_contract_amount > Decimal::ZERO {
        round_pct(unpaid_amount / total_contract_amount * Decimal::from(100))
    } else {
        Decimal::ZERO
    };

    Ok(PaymentCompletionVO {
        year: Some(year),
        total_contract_amount: Some(total_contract_amount),
        total_payment_amount: Some(total_payment_amount),
        completion_rate: Some(completion_rate),
        overdue_amount: Some(overdue),
        overdue_rate: Some(Decimal::ZERO),
        unpaid_amount: Some(unpaid_amount),
        unpaid_rate: Some(unpaid_rate),
    })
}

pub async fn get_payment_monthly_trend(db: &DbConn, year: Option<i32>) -> Result<PaymentMonthlyTrendStatsVO> {
    let year = year.unwrap_or(chrono::Local::now().year() as i32);

    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut contract_by_month: HashMap<i32, Decimal> = HashMap::new();
    for c in &contracts {
        if let Some(sd) = c.sign_date {
            if sd.year() == year {
                *contract_by_month.entry(sd.month() as i32).or_insert(Decimal::ZERO) += c.amount.unwrap_or(Decimal::ZERO);
            }
        }
    }

    let payments = SalePayment::find()
        .filter(payment::Column::Deleted.eq(0))
        .filter(payment::Column::Status.eq(2))
        .all(db)
        .await?;

    let mut payment_by_month: HashMap<i32, Decimal> = HashMap::new();
    for p in &payments {
        if let Some(pd) = p.payment_date {
            if pd.year() == year {
                *payment_by_month.entry(pd.month() as i32).or_insert(Decimal::ZERO) += p.amount.unwrap_or(Decimal::ZERO);
            }
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

pub async fn get_payment_status_analysis(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<Vec<PaymentStatusAnalysisVO>> {
    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let total_contract_amount: Decimal = contracts.iter()
        .map(|c| c.amount.unwrap_or(Decimal::ZERO))
        .sum();

    // 获取所有回款记录，按合同分组
    let payments = SalePayment::find()
        .filter(payment::Column::Deleted.eq(0))
        .filter(payment::Column::Status.eq(2))
        .all(db)
        .await?;

    let mut paid_by_contract: HashMap<i64, Decimal> = HashMap::new();
    for p in &payments {
        if let Some(cid) = p.contract_id {
            *paid_by_contract.entry(cid).or_insert(Decimal::ZERO) += p.amount.unwrap_or(Decimal::ZERO);
        }
    }

    let mut paid_count: i64 = 0;
    let mut paid_amount: Decimal = Decimal::ZERO;
    let mut partial_count: i64 = 0;
    let mut partial_paid_amount: Decimal = Decimal::ZERO;
    let mut unpaid_count: i64 = 0;
    let mut overdue_count: i64 = 0;
    let mut overdue_paid_amount: Decimal = Decimal::ZERO;

    let now = chrono::Local::now().date_naive();

    for c in &contracts {
        let paid = paid_by_contract.get(&c.id).copied().unwrap_or(Decimal::ZERO);
        let amount = c.amount.unwrap_or(Decimal::ZERO);

        if amount > Decimal::ZERO && paid >= amount {
            paid_count += 1;
            paid_amount += paid;
        } else if paid > Decimal::ZERO {
            partial_count += 1;
            partial_paid_amount += paid;
            // 如果有部分回款但已过期
            if let Some(sd) = c.sign_date {
                if sd < now {
                    overdue_count += 1;
                    overdue_paid_amount += paid;
                }
            }
        } else {
            unpaid_count += 1;
            // 无回款且已过期
            if let Some(sd) = c.sign_date {
                if sd < now {
                    overdue_count += 1;
                }
            }
        }
    }

    let result = vec![
        PaymentStatusAnalysisVO {
            status: Some("unpaid".to_string()),
            status_name: Some("未回款".to_string()),
            contract_count: Some(unpaid_count),
            contract_amount: Some(contracts.iter().filter(|c| paid_by_contract.get(&c.id).copied().unwrap_or(Decimal::ZERO) == Decimal::ZERO).map(|c| c.amount.unwrap_or(Decimal::ZERO)).sum()),
            paid_amount: Some(Decimal::ZERO),
            percentage: if total_contract_amount > Decimal::ZERO { Some(round_pct(contracts.iter().filter(|c| paid_by_contract.get(&c.id).copied().unwrap_or(Decimal::ZERO) == Decimal::ZERO).map(|c| c.amount.unwrap_or(Decimal::ZERO)).sum::<Decimal>() / total_contract_amount * Decimal::from(100))) } else { Some(Decimal::ZERO) },
        },
        PaymentStatusAnalysisVO {
            status: Some("partial".to_string()),
            status_name: Some("部分回款".to_string()),
            contract_count: Some(partial_count),
            contract_amount: Some(contracts.iter().filter(|c| {
                let p = paid_by_contract.get(&c.id).copied().unwrap_or(Decimal::ZERO);
                p > Decimal::ZERO && p < c.amount.unwrap_or(Decimal::ZERO)
            }).map(|c| c.amount.unwrap_or(Decimal::ZERO)).sum()),
            paid_amount: Some(partial_paid_amount),
            percentage: if total_contract_amount > Decimal::ZERO { Some(round_pct(contracts.iter().filter(|c| {
                let p = paid_by_contract.get(&c.id).copied().unwrap_or(Decimal::ZERO);
                p > Decimal::ZERO && p < c.amount.unwrap_or(Decimal::ZERO)
            }).map(|c| c.amount.unwrap_or(Decimal::ZERO)).sum::<Decimal>() / total_contract_amount * Decimal::from(100))) } else { Some(Decimal::ZERO) },
        },
        PaymentStatusAnalysisVO {
            status: Some("paid".to_string()),
            status_name: Some("已回款".to_string()),
            contract_count: Some(paid_count),
            contract_amount: Some(contracts.iter().filter(|c| {
                let p = paid_by_contract.get(&c.id).copied().unwrap_or(Decimal::ZERO);
                p >= c.amount.unwrap_or(Decimal::ZERO)
            }).map(|c| c.amount.unwrap_or(Decimal::ZERO)).sum()),
            paid_amount: Some(paid_amount),
            percentage: if total_contract_amount > Decimal::ZERO { Some(round_pct(contracts.iter().filter(|c| {
                let p = paid_by_contract.get(&c.id).copied().unwrap_or(Decimal::ZERO);
                p >= c.amount.unwrap_or(Decimal::ZERO)
            }).map(|c| c.amount.unwrap_or(Decimal::ZERO)).sum::<Decimal>() / total_contract_amount * Decimal::from(100))) } else { Some(Decimal::ZERO) },
        },
        PaymentStatusAnalysisVO {
            status: Some("overdue".to_string()),
            status_name: Some("逾期".to_string()),
            contract_count: Some(overdue_count),
            contract_amount: Some(contracts.iter().filter(|c| {
                let p = paid_by_contract.get(&c.id).copied().unwrap_or(Decimal::ZERO);
                let amount = c.amount.unwrap_or(Decimal::ZERO);
                (p < amount) && c.sign_date.map_or(false, |sd| sd < now)
            }).map(|c| c.amount.unwrap_or(Decimal::ZERO)).sum()),
            paid_amount: Some(overdue_paid_amount),
            percentage: if total_contract_amount > Decimal::ZERO { Some(round_pct(contracts.iter().filter(|c| {
                let p = paid_by_contract.get(&c.id).copied().unwrap_or(Decimal::ZERO);
                let amount = c.amount.unwrap_or(Decimal::ZERO);
                (p < amount) && c.sign_date.map_or(false, |sd| sd < now)
            }).map(|c| c.amount.unwrap_or(Decimal::ZERO)).sum::<Decimal>() / total_contract_amount * Decimal::from(100))) } else { Some(Decimal::ZERO) },
        },
    ];

    Ok(result)
}

pub async fn get_payment_ranking(db: &DbConn, _year: Option<i32>, _month: Option<i32>, order_by: Option<String>, _limit: Option<i64>) -> Result<Vec<PaymentRankingVO>> {
    let order_by = order_by.unwrap_or("payment_amount".to_string());
    let limit = _limit.unwrap_or(10) as usize;

    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let payments = SalePayment::find()
        .filter(payment::Column::Deleted.eq(0))
        .filter(payment::Column::Status.eq(2))
        .all(db)
        .await?;

    // 按客户聚合回款
    let mut payment_by_customer: HashMap<i64, Decimal> = HashMap::new();
    let mut contract_by_customer: HashMap<i64, Vec<i64>> = HashMap::new();
    for c in &contracts {
        if let Some(cid) = c.customer_id {
            contract_by_customer.entry(cid).or_default().push(c.id);
        }
    }
    for p in &payments {
        if let Some(cid) = p.customer_id {
            *payment_by_customer.entry(cid).or_insert(Decimal::ZERO) += p.amount.unwrap_or(Decimal::ZERO);
        }
    }

    // 按员工聚合回款
    let mut payment_by_employee: HashMap<i64, Decimal> = HashMap::new();
    let mut contract_by_employee: HashMap<i64, Vec<i64>> = HashMap::new();
    for c in &contracts {
        if let Some(eid) = c.assigned_to {
            contract_by_employee.entry(eid).or_default().push(c.id);
        }
    }
    for p in &payments {
        if let Some(eid) = p.owner_user_id {
            *payment_by_employee.entry(eid).or_insert(Decimal::ZERO) += p.amount.unwrap_or(Decimal::ZERO);
        }
    }

    // 客户名称映射
    let cids: Vec<i64> = payment_by_customer.keys().copied().collect();
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
    let eids: Vec<i64> = payment_by_employee.keys().copied().collect();
    let mut admin_name_map: HashMap<i64, String> = HashMap::new();
    if !eids.is_empty() {
        for a in Admin::find().filter(crate::modules::system::entity::admin::Column::Id.is_in(eids)).all(db).await? {
            let name = a.nick_name.or(a.user_name).unwrap_or_default();
            if !name.is_empty() {
                admin_name_map.insert(a.id, name);
            }
        }
    }

    let mut result: Vec<PaymentRankingVO> = Vec::new();

    for (cid, pa) in &payment_by_customer {
        let contract_amount: Decimal = contract_by_customer.get(cid).map(|ids| {
            contracts.iter().filter(|c| ids.contains(&c.id)).map(|c| c.amount.unwrap_or(Decimal::ZERO)).sum()
        }).unwrap_or(Decimal::ZERO);
        let rate = if contract_amount > Decimal::ZERO {
            round_pct(*pa / contract_amount * Decimal::from(100))
        } else {
            Decimal::ZERO
        };
        result.push(PaymentRankingVO {
            rank: None,
            target_type: Some("customer".to_string()),
            target_id: Some(*cid),
            target_name: customer_name_map.get(cid).cloned(),
            contract_amount: Some(contract_amount),
            payment_amount: Some(*pa),
            completion_rate: Some(rate),
            overdue_amount: Some(Decimal::ZERO),
        });
    }

    for (eid, pa) in &payment_by_employee {
        let contract_amount: Decimal = contract_by_employee.get(eid).map(|ids| {
            contracts.iter().filter(|c| ids.contains(&c.id)).map(|c| c.amount.unwrap_or(Decimal::ZERO)).sum()
        }).unwrap_or(Decimal::ZERO);
        let rate = if contract_amount > Decimal::ZERO {
            round_pct(*pa / contract_amount * Decimal::from(100))
        } else {
            Decimal::ZERO
        };
        result.push(PaymentRankingVO {
            rank: None,
            target_type: Some("employee".to_string()),
            target_id: Some(*eid),
            target_name: admin_name_map.get(eid).cloned(),
            contract_amount: Some(contract_amount),
            payment_amount: Some(*pa),
            completion_rate: Some(rate),
            overdue_amount: Some(Decimal::ZERO),
        });
    }

    if order_by == "completion_rate" {
        result.sort_by(|a, b| b.completion_rate.unwrap_or(Decimal::ZERO).cmp(&a.completion_rate.unwrap_or(Decimal::ZERO)));
    } else {
        result.sort_by(|a, b| b.payment_amount.unwrap_or(Decimal::ZERO).cmp(&a.payment_amount.unwrap_or(Decimal::ZERO)));
    }

    result.truncate(limit);

    for (i, item) in result.iter_mut().enumerate() {
        item.rank = Some((i + 1) as i32);
    }

    Ok(result)
}