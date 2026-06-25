use chrono::Datelike;
use crate::core::errors::error::{Error, Result};
use crate::modules::statistics::entity::performance_target::{self, Entity as PerformanceTarget};
use crate::modules::statistics::model::performance_target::{PerformanceTargetVO, PerformanceTargetSaveRequest, MonthlyPerformanceStatsVO, MonthlyPerformanceVO, PerformanceRankingVO};
use crate::modules::system::entity::admin::Entity as Admin;
use sea_orm::{DbConn, DbErr, QuerySelect, ColumnTrait, QueryFilter, EntityTrait, QueryOrder, ActiveModelTrait, Set, IntoActiveModel, TransactionTrait};
use sea_orm::prelude::Decimal;

pub async fn get_targets(db: &DbConn, employee_id: Option<i64>, year: Option<i32>, month: Option<i32>) -> Result<Vec<PerformanceTargetVO>> {
    let mut query = PerformanceTarget::find();
    
    if let Some(eid) = employee_id {
        query = query.filter(performance_target::Column::EmployeeId.eq(eid));
    }
    if let Some(y) = year {
        query = query.filter(performance_target::Column::Year.eq(y));
    }
    if let Some(m) = month {
        query = query.filter(performance_target::Column::Month.eq(m));
    }
    
    query = query.order_by(performance_target::Column::EmployeeId, sea_orm::Order::Asc)
        .order_by(performance_target::Column::Year, sea_orm::Order::Desc)
        .order_by(performance_target::Column::Month, sea_orm::Order::Asc);
    
    let targets = query.all(db).await?;
    
    let admin_map = Admin::find()
        .filter(crate::modules::system::entity::admin::Column::Deleted.eq(0))
        .all(db)
        .await?
        .into_iter()
        .map(|a| (a.id, a.user_name))
        .collect::<std::collections::HashMap<i64, Option<String>>>();
    
    let result = targets.into_iter()
        .map(|t| {
            let mut vo: PerformanceTargetVO = t.into();
            vo.employee_name = admin_map.get(&vo.employee_id.unwrap_or(0)).cloned().flatten();
            vo
        })
        .collect();
    
    Ok(result)
}

pub async fn save_targets(db: &DbConn, targets: &Vec<PerformanceTargetSaveRequest>) -> Result<(i64, i64)> {
    let targets_clone = targets.clone();

    // 批量 upsert 业绩目标需原子执行，避免部分失败导致同一批数据不一致
    let (saved_count, updated_count) = db
        .transaction::<_, (i64, i64), DbErr>(|txn| {
            Box::pin(async move {
                let mut saved_count = 0;
                let mut updated_count = 0;

                for target in &targets_clone {
                    if target.employee_id.is_none() || target.year.is_none() || target.month.is_none() {
                        continue;
                    }

                    let existing = PerformanceTarget::find()
                        .filter(performance_target::Column::EmployeeId.eq(target.employee_id.unwrap()))
                        .filter(performance_target::Column::Year.eq(target.year.unwrap()))
                        .filter(performance_target::Column::Month.eq(target.month.unwrap()))
                        .one(txn)
                        .await?;

                    let mut active_model: performance_target::ActiveModel = if let Some(existing) = existing {
                        existing.into_active_model()
                    } else {
                        <performance_target::ActiveModel as std::default::Default>::default()
                    };

                    active_model.employee_id = Set(target.employee_id);
                    active_model.year = Set(target.year);
                    active_model.month = Set(target.month);
                    active_model.contract_target_amount = Set(target.contract_target_amount);
                    active_model.payment_target_amount = Set(target.payment_target_amount);
                    active_model.contract_target_count = Set(target.contract_target_count);

                    if active_model.id.is_set() {
                        let _ = active_model.update(txn).await?;
                        updated_count += 1;
                    } else {
                        let _ = active_model.insert(txn).await?;
                        saved_count += 1;
                    }
                }

                Ok((saved_count, updated_count))
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok((saved_count, updated_count))
}

pub async fn get_monthly_performance(db: &DbConn, year: Option<i32>, _department_id: Option<i64>) -> Result<MonthlyPerformanceStatsVO> {
    let year = year.unwrap_or(chrono::Local::now().year() as i32);
    
    let targets = PerformanceTarget::find()
        .filter(performance_target::Column::Year.eq(year))
        .all(db)
        .await?;
    
    let mut month_targets: std::collections::HashMap<i32, (Decimal, Decimal)> = std::collections::HashMap::new();
    for t in &targets {
        let m = t.month.unwrap_or(0);
        let (c, p) = month_targets.entry(m).or_insert((Decimal::from(0), Decimal::from(0)));
        *c += t.contract_target_amount.unwrap_or(Decimal::from(0));
        *p += t.payment_target_amount.unwrap_or(Decimal::from(0));
    }
    
    let mut months = Vec::new();
    for m in 1..=12 {
        let (contract_target, payment_target) = month_targets.get(&m).copied().unwrap_or((Decimal::from(0), Decimal::from(0)));
        
        months.push(MonthlyPerformanceVO {
            month: Some(m),
            contract_target: Some(contract_target),
            payment_target: Some(payment_target),
            contract_actual: Some(Decimal::from(0)),
            payment_actual: Some(Decimal::from(0)),
            contract_completion_rate: Some(Decimal::from(0)),
            payment_completion_rate: Some(Decimal::from(0)),
            contract_count: Some(0),
            payment_count: Some(0),
        });
    }
    
    let total_contract_target: Decimal = month_targets.values().map(|(c, _)| *c).sum();
    let total_payment_target: Decimal = month_targets.values().map(|(_, p)| *p).sum();
    
    Ok(MonthlyPerformanceStatsVO {
        year: Some(year),
        total_contract_target: Some(total_contract_target),
        total_payment_target: Some(total_payment_target),
        total_contract_actual: Some(Decimal::from(0)),
        total_payment_actual: Some(Decimal::from(0)),
        contract_completion_rate: Some(Decimal::from(0)),
        payment_completion_rate: Some(Decimal::from(0)),
        months: Some(months),
    })
}

pub async fn get_performance_ranking(db: &DbConn, year: Option<i32>, month: Option<i32>, order_by: Option<String>, _department_id: Option<i64>) -> Result<Vec<PerformanceRankingVO>> {
    let year = year.unwrap_or(chrono::Local::now().year() as i32);
    
    let targets = PerformanceTarget::find()
        .filter(performance_target::Column::Year.eq(year))
        .all(db)
        .await?;
    
    let admin_map = Admin::find()
        .filter(crate::modules::system::entity::admin::Column::Deleted.eq(0))
        .all(db)
        .await?
        .into_iter()
        .map(|a| (a.id, a.user_name))
        .collect::<std::collections::HashMap<i64, Option<String>>>();
    
    let mut employee_stats: std::collections::HashMap<i64, (Decimal, Decimal, Decimal, Decimal)> = std::collections::HashMap::new();
    
    for t in targets {
        let eid = t.employee_id.unwrap_or(0);
        let (contract_target, payment_target, contract_actual, payment_actual) = 
            employee_stats.entry(eid).or_insert((Decimal::from(0), Decimal::from(0), Decimal::from(0), Decimal::from(0)));
        *contract_target += t.contract_target_amount.unwrap_or(Decimal::from(0));
        *payment_target += t.payment_target_amount.unwrap_or(Decimal::from(0));
    }
    
    let mut ranking: Vec<PerformanceRankingVO> = employee_stats.into_iter()
        .map(|(eid, (contract_target, payment_target, contract_actual, payment_actual))| {
            let contract_completion_rate = if contract_target > Decimal::from(0) {
                contract_actual / contract_target * Decimal::from(100)
            } else {
                Decimal::from(0)
            };
            let payment_completion_rate = if payment_target > Decimal::from(0) {
                payment_actual / payment_target * Decimal::from(100)
            } else {
                Decimal::from(0)
            };
            
            PerformanceRankingVO {
                rank: None,
                employee_id: Some(eid),
                employee_name: admin_map.get(&eid).cloned().flatten(),
                department_name: None,
                contract_amount: Some(contract_actual),
                contract_count: Some(0),
                payment_amount: Some(payment_actual),
                payment_count: Some(0),
                contract_target: Some(contract_target),
                payment_target: Some(payment_target),
                contract_completion_rate: Some(contract_completion_rate),
                payment_completion_rate: Some(payment_completion_rate),
            }
        })
        .collect();
    
    let order_by = order_by.unwrap_or("contract_amount".to_string());
    if order_by == "payment_amount" {
        ranking.sort_by(|a, b| b.payment_amount.unwrap_or(Decimal::from(0)).cmp(&a.payment_amount.unwrap_or(Decimal::from(0))));
    } else {
        ranking.sort_by(|a, b| b.contract_amount.unwrap_or(Decimal::from(0)).cmp(&a.contract_amount.unwrap_or(Decimal::from(0))));
    }
    
    for (i, item) in ranking.iter_mut().enumerate() {
        item.rank = Some((i + 1) as i32);
    }
    
    Ok(ranking)
}