//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 提成分配服务
//! 处理 category=5（总提成再分配）的待分配列表、手动分配、分配记录查询
//!

use sea_orm::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::modules::finance::entity::{commission_result, commission_allocation, salary_record};
use crate::modules::system::entity::admin;

/// 待分配提成 VO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingCommissionVO {
    pub id: i64,
    pub rule_id: i64,
    pub rule_name: Option<String>,
    pub contract_id: Option<i64>,
    pub contract_name: Option<String>,
    pub user_id: i64,
    pub user_name: Option<String>,
    pub commission_amount: f64,
    pub allocated_amount: f64,
    pub remaining_amount: f64,
    pub period_year: i32,
    pub period_month: i32,
    pub allocate_status: i16,
    pub create_time: Option<String>,
}

/// 分配成员项 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocateMemberItem {
    pub employee_id: i64,
    pub employee_name: Option<String>,
    /// 当期业绩(回款额)，按业绩比例分配时必填
    pub employee_payment: Option<f64>,
    /// 手动分配时必填的金额
    pub amount: Option<f64>,
}

/// 提交分配 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocateDTO {
    /// 待分配的 commission_result ID
    pub commission_result_id: i64,
    /// 分配方式: 1=平均 2=按业绩比例 3=手动
    pub allocate_method: i16,
    /// 参与分配的成员列表
    pub members: Vec<AllocateMemberItem>,
    /// 分配人(管理者)ID
    pub allocator_id: i64,
    pub remark: Option<String>,
}

/// 分配记录 VO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationLogVO {
    pub id: i64,
    pub commission_result_id: i64,
    pub allocator_id: i64,
    pub allocator_name: Option<String>,
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub amount: f64,
    pub allocate_method: i16,
    pub allocate_method_name: String,
    pub employee_payment: Option<f64>,
    pub team_total_payment: Option<f64>,
    pub salary_record_id: Option<i64>,
    pub year: i32,
    pub month: i32,
    pub remark: Option<String>,
    pub create_time: Option<String>,
}

/// 待分配列表查询参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub allocator_id: Option<i64>,
}

/// 查询待分配列表（allocate_status=1）
pub async fn get_pending_list(
    db: &DatabaseConnection,
    query: PendingQuery,
) -> Result<(Vec<PendingCommissionVO>, i64), String> {
    let page = std::cmp::max(query.page.unwrap_or(1), 1);
    let page_size = std::cmp::max(query.page_size.unwrap_or(20), 1);

    let mut stmt = commission_result::Entity::find()
        .filter(commission_result::Column::AllocateStatus.eq(1));

    if let Some(y) = query.year {
        stmt = stmt.filter(commission_result::Column::PeriodYear.eq(y));
    }
    if let Some(m) = query.month {
        stmt = stmt.filter(commission_result::Column::PeriodMonth.eq(m));
    }
    if let Some(uid) = query.allocator_id {
        stmt = stmt.filter(commission_result::Column::UserId.eq(uid));
    }

    stmt = stmt.order_by_desc(commission_result::Column::CreateTime);

    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?;

    let vo_list = items
        .into_iter()
        .map(|m| {
            let allocated = m.allocated_amount.unwrap_or(Decimal::ZERO);
            let remaining = m.commission_amount - allocated;
            PendingCommissionVO {
                id: m.id,
                rule_id: m.rule_id,
                rule_name: m.rule_name,
                contract_id: m.contract_id,
                contract_name: m.contract_name,
                user_id: m.user_id,
                user_name: m.user_name,
                commission_amount: m.commission_amount.to_f64().unwrap_or_default(),
                allocated_amount: allocated.to_f64().unwrap_or_default(),
                remaining_amount: remaining.to_f64().unwrap_or_default(),
                period_year: m.period_year,
                period_month: m.period_month,
                allocate_status: m.allocate_status.unwrap_or(0),
                create_time: m.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            }
        })
        .collect();

    Ok((vo_list, total))
}

/// 提交分配
///
/// 三种分配方式：
/// 1. 平均分配：总额 ÷ 人数
/// 2. 按业绩比例：每人 = 总额 × (个人业绩 / 团队总业绩)
/// 3. 手动填写：每人金额由前端传入
///
/// 流程：
/// 1. 校验 commission_result 存在且为待分配状态
/// 2. 按分配方式计算每人金额
/// 3. 校验合计不超过待分配金额
/// 4. 写入 commission_allocation 记录
/// 5. 写入各成员 salary_record.allocated_commission
/// 6. 更新 commission_result.allocate_status=2 和 allocated_amount
pub async fn allocate(db: &DatabaseConnection, dto: AllocateDTO) -> Result<i64, String> {
    if dto.members.is_empty() {
        return Err("分配成员列表不能为空".to_string());
    }

    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // 1. 查询待分配的提成结果
    let result = commission_result::Entity::find_by_id(dto.commission_result_id)
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "提成记录不存在".to_string())?;

    if result.allocate_status.unwrap_or(0) != 1 {
        return Err("该提成记录不是待分配状态".to_string());
    }

    let total_amount = result.commission_amount;
    let already_allocated = result.allocated_amount.unwrap_or(Decimal::ZERO);
    let remaining = total_amount - already_allocated;
    if remaining <= Decimal::ZERO {
        return Err("待分配余额为0，无需分配".to_string());
    }

    let year = result.period_year;
    let month = result.period_month;
    let now = chrono::Utc::now().naive_utc();

    // 2. 按分配方式计算每人金额
    let allocations = compute_allocations(&dto, remaining)?;

    // 3. 校验合计不超过待分配金额
    let sum: Decimal = allocations.iter().map(|(_, amount)| *amount).sum();
    if sum > remaining {
        return Err(format!(
            "分配总额 {:.2} 超过待分配金额 {:.2}",
            sum, remaining
        ));
    }

    // 4. 写入分配记录并更新工资条
    let mut allocation_ids = Vec::new();
    for (member, amount) in allocations {
        // 查询或创建该员工的 salary_record
        let salary_record_id = ensure_salary_record(&txn, member.employee_id, year, month).await?;

        // 累加 allocated_commission 到 salary_record
        if let Some(sid) = salary_record_id {
            let sr = salary_record::Entity::find_by_id(sid)
                .one(&txn)
                .await
                .map_err(|e| e.to_string())?;
            if let Some(sr_model) = sr {
                let current = sr_model.allocated_commission;
                let mut sr_active: salary_record::ActiveModel = sr_model.into();
                sr_active.allocated_commission = Set(current + amount);
                sr_active.update_time = Set(Some(now));
                sr_active.update(&txn).await.map_err(|e| e.to_string())?;
            }
        }

        let allocation = commission_allocation::ActiveModel {
            commission_result_id: Set(dto.commission_result_id),
            allocator_id: Set(dto.allocator_id),
            employee_id: Set(member.employee_id),
            employee_name: Set(member.employee_name),
            amount: Set(amount),
            allocate_method: Set(dto.allocate_method),
            employee_payment: Set(member.employee_payment.and_then(|v| {
                Decimal::from_f64(v)
            })),
            team_total_payment: Set(None),
            salary_record_id: Set(salary_record_id),
            year: Set(year),
            month: Set(month),
            remark: Set(dto.remark.clone()),
            create_time: Set(now),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        let inserted = allocation.insert(&txn).await.map_err(|e| e.to_string())?;
        allocation_ids.push(inserted.id);
    }

    // 5. 更新 commission_result 状态
    let mut result_active: commission_result::ActiveModel = result.into();
    let new_allocated = already_allocated + sum;
    result_active.allocated_amount = Set(Some(new_allocated));
    // 若全部分配完，状态改为已分配
    if new_allocated >= total_amount {
        result_active.allocate_status = Set(Some(2));
    }
    result_active.update(&txn).await.map_err(|e| e.to_string())?;

    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(allocation_ids.len() as i64)
}

/// 按分配方式计算每人金额
fn compute_allocations(
    dto: &AllocateDTO,
    total: Decimal,
) -> Result<Vec<(AllocateMemberItem, Decimal)>, String> {
    match dto.allocate_method {
        1 => {
            // 平均分配
            let count = Decimal::from(dto.members.len() as i64);
            if count.is_zero() {
                return Err("成员数为0".to_string());
            }
            let per_person = total / count;
            Ok(dto.members.iter().map(|m| (m.clone(), per_person)).collect())
        }
        2 => {
            // 按业绩比例
            let total_payment: f64 = dto.members.iter()
                .map(|m| m.employee_payment.unwrap_or(0.0))
                .sum();
            if total_payment <= 0.0 {
                return Err("按业绩比例分配时，成员业绩合计必须大于0".to_string());
            }
            let total_payment_dec = Decimal::from_f64(total_payment)
                .ok_or_else(|| "业绩金额格式错误".to_string())?;
            let mut result = Vec::new();
            for m in &dto.members {
                let payment = m.employee_payment.unwrap_or(0.0);
                let payment_dec = Decimal::from_f64(payment)
                    .ok_or_else(|| "业绩金额格式错误".to_string())?;
                let amount = total * (payment_dec / total_payment_dec);
                result.push((m.clone(), amount));
            }
            Ok(result)
        }
        3 => {
            // 手动填写
            let mut result = Vec::new();
            for m in &dto.members {
                let amount = m.amount.ok_or_else(|| "手动分配时每人金额必填".to_string())?;
                let amount_dec = Decimal::from_f64(amount)
                    .ok_or_else(|| "分配金额格式错误".to_string())?;
                if amount_dec < Decimal::ZERO {
                    return Err(format!("员工 {} 的分配金额不能为负", m.employee_id));
                }
                result.push((m.clone(), amount_dec));
            }
            Ok(result)
        }
        _ => Err(format!("不支持的分配方式: {}", dto.allocate_method)),
    }
}

/// 确保员工在该年月有 salary_record，返回其 ID
///
/// 如果不存在则不创建（返回 None），因为工资核算应由 salary_service 统一处理
/// 分配金额仅在 salary_record 已存在时累加，否则只记录分配历史
async fn ensure_salary_record<C: ConnectionTrait>(
    db: &C,
    employee_id: i64,
    year: i32,
    month: i32,
) -> Result<Option<i64>, String> {
    let sr = salary_record::Entity::find()
        .filter(salary_record::Column::EmployeeId.eq(employee_id))
        .filter(salary_record::Column::Year.eq(year))
        .filter(salary_record::Column::Month.eq(month))
        .filter(salary_record::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(sr.map(|s| s.id))
}

/// 查询分配记录
pub async fn get_allocation_log(
    db: &DatabaseConnection,
    year: Option<i32>,
    month: Option<i32>,
    allocator_id: Option<i64>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<AllocationLogVO>, i64), String> {
    let page = std::cmp::max(page, 1);
    let page_size = std::cmp::max(page_size, 1);

    let mut stmt = commission_allocation::Entity::find()
        .filter(commission_allocation::Column::Deleted.eq(0));

    if let Some(y) = year {
        stmt = stmt.filter(commission_allocation::Column::Year.eq(y));
    }
    if let Some(m) = month {
        stmt = stmt.filter(commission_allocation::Column::Month.eq(m));
    }
    if let Some(aid) = allocator_id {
        stmt = stmt.filter(commission_allocation::Column::AllocatorId.eq(aid));
    }

    stmt = stmt.order_by_desc(commission_allocation::Column::CreateTime);

    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?;

    // 批量查询分配人姓名
    let allocator_ids: Vec<i64> = items.iter().map(|a| a.allocator_id).collect::<HashSet<_>>().into_iter().collect();
    let mut allocator_map: HashMap<i64, String> = HashMap::new();
    if !allocator_ids.is_empty() {
        let admins = admin::Entity::find()
            .filter(admin::Column::Id.is_in(allocator_ids))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for a in admins {
            if let Some(n) = a.nick_name.or(a.user_name) {
                allocator_map.insert(a.id, n);
            }
        }
    }

    let vo_list = items
        .into_iter()
        .map(|m| {
            let method_name = match m.allocate_method {
                1 => "平均分配".to_string(),
                2 => "按业绩比例".to_string(),
                3 => "手动填写".to_string(),
                _ => "未知".to_string(),
            };
            AllocationLogVO {
                id: m.id,
                commission_result_id: m.commission_result_id,
                allocator_id: m.allocator_id,
                allocator_name: allocator_map.get(&m.allocator_id).cloned(),
                employee_id: m.employee_id,
                employee_name: m.employee_name,
                amount: m.amount.to_f64().unwrap_or_default(),
                allocate_method: m.allocate_method,
                allocate_method_name: method_name,
                employee_payment: m.employee_payment.and_then(|d| d.to_f64()),
                team_total_payment: m.team_total_payment.and_then(|d| d.to_f64()),
                salary_record_id: m.salary_record_id,
                year: m.year,
                month: m.month,
                remark: m.remark,
                create_time: Some(m.create_time.format("%Y-%m-%d %H:%M:%S").to_string()),
            }
        })
        .collect();

    Ok((vo_list, total))
}
