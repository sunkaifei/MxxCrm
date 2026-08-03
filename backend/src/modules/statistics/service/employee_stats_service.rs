use chrono::Datelike;
use crate::core::errors::error::Result;
use crate::modules::crm::entity::contract::{self, Entity as Contract};
use crate::modules::crm::entity::customer::{self, Entity as Customer};
use crate::modules::crm::entity::followup::{self, Entity as FollowUp};
use crate::modules::crm::entity::opportunity::{self, Entity as Opportunity};
use crate::modules::statistics::model::employee_stats::{EmployeeCustomerCountVO, EmployeeFollowUpVO, EmployeeConversionVO};
use crate::modules::system::entity::admin::Entity as Admin;
use crate::modules::system::entity::admin_dept_merge::Entity as AdminDeptMerge;
use crate::modules::system::entity::dept::Entity as Dept;
use sea_orm::prelude::Decimal;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};
use std::collections::HashMap;

pub async fn get_employee_customer_count(db: &DbConn, _department_id: Option<i64>) -> Result<Vec<EmployeeCustomerCountVO>> {
    let admins = Admin::find()
        .filter(crate::modules::system::entity::admin::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let now = chrono::Local::now().date_naive();
    let month_start = now.with_day(1).unwrap_or(now);

    // 客户按负责人统计
    let customers = Customer::find()
        .filter(customer::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut customer_count: HashMap<i64, i64> = HashMap::new();
    let mut new_customer_count: HashMap<i64, i64> = HashMap::new();
    let mut contract_customer_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();

    for c in &customers {
        if let Some(eid) = c.assigned_to {
            *customer_count.entry(eid).or_insert(0) += 1;
            if let Some(ct) = c.create_time {
                if ct.date() >= month_start {
                    *new_customer_count.entry(eid).or_insert(0) += 1;
                }
            }
        }
    }

    // 有合同的客户
    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;
    for c in &contracts {
        if let Some(cid) = c.customer_id {
            contract_customer_ids.insert(cid);
        }
    }

    let mut contract_customer_count: HashMap<i64, i64> = HashMap::new();
    for c in &customers {
        if contract_customer_ids.contains(&c.id) {
            if let Some(eid) = c.assigned_to {
                *contract_customer_count.entry(eid).or_insert(0) += 1;
            }
        }
    }

    // 部门映射
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

    let mut result: Vec<EmployeeCustomerCountVO> = Vec::new();
    for a in &admins {
        let total = customer_count.get(&a.id).copied().unwrap_or(0);
        let new_cust = new_customer_count.get(&a.id).copied().unwrap_or(0);
        let contract_cust = contract_customer_count.get(&a.id).copied().unwrap_or(0);
        let conversion_rate = if total > 0 {
            Decimal::from(contract_cust) / Decimal::from(total) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };
        result.push(EmployeeCustomerCountVO {
            employee_id: Some(a.id),
            employee_name: a.nick_name.clone().or(a.user_name.clone()),
            department_name: admin_dept_map.get(&a.id).cloned(),
            total_customers: Some(total),
            new_customers_this_month: Some(new_cust),
            contract_customers: Some(contract_cust),
            customer_conversion_rate: Some(conversion_rate),
        });
    }

    result.sort_by(|a, b| b.total_customers.unwrap_or(0).cmp(&a.total_customers.unwrap_or(0)));
    Ok(result)
}

pub async fn get_employee_follow_up(db: &DbConn, _year: Option<i32>, _month: Option<i32>, _department_id: Option<i64>) -> Result<Vec<EmployeeFollowUpVO>> {
    // 获取所有员工和部门信息
    let admins = Admin::find()
        .filter(crate::modules::system::entity::admin::Column::Deleted.eq(0))
        .all(db)
        .await?;

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

    // 按负责人获取跟进记录
    let followups = FollowUp::find()
        .filter(followup::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut total_follow_up: HashMap<i64, i64> = HashMap::new();
    let mut customer_follow_up: HashMap<i64, i64> = HashMap::new();
    let mut opportunity_follow_up: HashMap<i64, i64> = HashMap::new();
    let mut followup_times: HashMap<i64, Vec<chrono::NaiveDateTime>> = HashMap::new();

    for f in &followups {
        let eid = f.assigned_to.or(f.created_by).unwrap_or(0);
        if eid == 0 { continue; }

        *total_follow_up.entry(eid).or_insert(0) += 1;
        if f.source_type == Some(2) {
            *customer_follow_up.entry(eid).or_insert(0) += 1;
        } else if f.source_type == Some(3) {
            *opportunity_follow_up.entry(eid).or_insert(0) += 1;
        }

        if let Some(ct) = f.create_time {
            followup_times.entry(eid).or_default().push(ct);
        }
    }

    // 30天未跟进的客户数
    let thirty_days_ago = chrono::Local::now().date_naive() - chrono::Duration::days(30);
    let mut customers_without_follow: HashMap<i64, i64> = HashMap::new();

    let customers = Customer::find()
        .filter(customer::Column::Deleted.eq(0))
        .all(db)
        .await?;

    for c in &customers {
        if let Some(eid) = c.assigned_to {
            let has_recent_follow = followups.iter().any(|f| {
                let feid = f.assigned_to.or(f.created_by).unwrap_or(0);
                feid == eid && f.customer_id == Some(c.id) && f.create_time.map_or(false, |t| t.date() >= thirty_days_ago)
            });
            if !has_recent_follow {
                *customers_without_follow.entry(eid).or_insert(0) += 1;
            }
        }
    }

    let mut result: Vec<EmployeeFollowUpVO> = Vec::new();
    for a in &admins {
        let total = total_follow_up.get(&a.id).copied().unwrap_or(0);
        let cf = customer_follow_up.get(&a.id).copied().unwrap_or(0);
        let of = opportunity_follow_up.get(&a.id).copied().unwrap_or(0);

        // 平均跟进间隔（天）
        let avg_interval = if let Some(times) = followup_times.get(&a.id) {
            let mut sorted = times.clone();
            sorted.sort();
            if sorted.len() >= 2 {
                let total_diff: i64 = sorted.windows(2)
                    .map(|w| (w[1] - w[0]).num_days())
                    .sum();
                let avg = Decimal::from(total_diff) / Decimal::from((sorted.len() - 1) as i64);
                Some(avg)
            } else {
                Some(Decimal::ZERO)
            }
        } else {
            Some(Decimal::ZERO)
        };

        result.push(EmployeeFollowUpVO {
            employee_id: Some(a.id),
            employee_name: a.nick_name.clone().or(a.user_name.clone()),
            department_name: admin_dept_map.get(&a.id).cloned(),
            total_follow_up: Some(total),
            customer_follow_up: Some(cf),
            opportunity_follow_up: Some(of),
            avg_follow_interval: avg_interval,
            customers_without_follow_30_days: customers_without_follow.get(&a.id).copied(),
        });
    }

    result.sort_by(|a, b| b.total_follow_up.unwrap_or(0).cmp(&a.total_follow_up.unwrap_or(0)));
    Ok(result)
}

pub async fn get_employee_conversion(db: &DbConn, _year: Option<i32>, _month: Option<i32>, _department_id: Option<i64>) -> Result<Vec<EmployeeConversionVO>> {
    let admins = Admin::find()
        .filter(crate::modules::system::entity::admin::Column::Deleted.eq(0))
        .all(db)
        .await?;

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

    // 商机统计
    let opportunities = Opportunity::find()
        .filter(opportunity::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut total_opp: HashMap<i64, i64> = HashMap::new();
    let mut won_opp: HashMap<i64, i64> = HashMap::new();
    let mut lost_opp: HashMap<i64, i64> = HashMap::new();

    for o in &opportunities {
        let eid = o.assigned_to.or(o.created_by).unwrap_or(0);
        if eid == 0 { continue; }
        *total_opp.entry(eid).or_insert(0) += 1;
        if o.stage == Some(4) {
            *won_opp.entry(eid).or_insert(0) += 1;
        } else if o.stage == Some(5) {
            *lost_opp.entry(eid).or_insert(0) += 1;
        }
    }

    // 合同统计
    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut total_contracts: HashMap<i64, i64> = HashMap::new();
    let mut contract_amount: HashMap<i64, Decimal> = HashMap::new();
    let mut contract_dates: HashMap<i64, Vec<chrono::NaiveDate>> = HashMap::new();

    for c in &contracts {
        let eid = c.assigned_to.or(c.created_by).unwrap_or(0);
        if eid == 0 { continue; }
        *total_contracts.entry(eid).or_insert(0) += 1;
        *contract_amount.entry(eid).or_insert(Decimal::ZERO) += c.amount.unwrap_or(Decimal::ZERO);
        if let Some(sd) = c.sign_date {
            contract_dates.entry(eid).or_default().push(sd);
        }
    }

    let mut result: Vec<EmployeeConversionVO> = Vec::new();
    for a in &admins {
        let to = total_opp.get(&a.id).copied().unwrap_or(0);
        let wo = won_opp.get(&a.id).copied().unwrap_or(0);
        let lo = lost_opp.get(&a.id).copied().unwrap_or(0);
        let tc = total_contracts.get(&a.id).copied().unwrap_or(0);
        let ca = contract_amount.get(&a.id).copied().unwrap_or(Decimal::ZERO);

        let win_rate = if to > 0 {
            Decimal::from(wo) / Decimal::from(to) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        let avg_amount = if tc > 0 {
            ca / Decimal::from(tc)
        } else {
            Decimal::ZERO
        };

        // 平均销售周期（天）
        let avg_cycle = if let Some(dates) = contract_dates.get(&a.id) {
            let opp_dates: Vec<_> = opportunities.iter()
                .filter(|o| o.assigned_to == Some(a.id) && o.stage == Some(4) && o.actual_close_date.is_some())
                .filter_map(|o| o.actual_close_date)
                .collect();
            if !opp_dates.is_empty() && !dates.is_empty() {
                let total_days: i64 = dates.iter().zip(opp_dates.iter())
                    .map(|(c, o)| (*c - *o).num_days().abs())
                    .sum();
                let avg = total_days / std::cmp::max(dates.len(), opp_dates.len()) as i64;
                Some(avg)
            } else {
                Some(0)
            }
        } else {
            Some(0)
        };

        result.push(EmployeeConversionVO {
            employee_id: Some(a.id),
            employee_name: a.nick_name.clone().or(a.user_name.clone()),
            department_name: admin_dept_map.get(&a.id).cloned(),
            total_opportunities: Some(to),
            won_opportunities: Some(wo),
            lost_opportunities: Some(lo),
            opportunity_win_rate: Some(win_rate),
            total_contracts: Some(tc),
            contract_amount: Some(ca),
            avg_contract_amount: Some(avg_amount),
            avg_sales_cycle_days: avg_cycle,
        });
    }

    result.sort_by(|a, b| b.total_contracts.unwrap_or(0).cmp(&a.total_contracts.unwrap_or(0)));
    Ok(result)
}