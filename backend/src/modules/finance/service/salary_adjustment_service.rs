//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 调薪记录服务
//! 负责调薪记录的 CRUD、审批流，审批通过后同步更新 salary_config
//!

use sea_orm::*;
use chrono::Utc;
use chrono::Datelike;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

use crate::modules::finance::entity::{salary_adjustment, salary_config};

// ==================== DTO ====================

/// 创建调薪记录 DTO
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalaryAdjustmentCreateDTO {
    pub employee_id: i64,
    /// 调薪日期（ISO 8601 字符串，如 "2026-07-01T00:00:00"）。若不传则取当前时间
    pub adjustment_date: Option<String>,
    pub adjustment_type: Option<i32>,
    pub old_base_salary: Option<f64>,
    pub new_base_salary: Option<f64>,
    pub old_position_allowance: Option<f64>,
    pub new_position_allowance: Option<f64>,
    pub old_performance_base: Option<f64>,
    pub new_performance_base: Option<f64>,
    pub adjustment_reason: Option<String>,
}

/// 驳回请求 DTO
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectDTO {
    pub id: i64,
    pub reason: String,
}

// ==================== CRUD ====================

/// 分页查询调薪记录
pub async fn get_adjustment_list(
    db: &DatabaseConnection,
    employee_id: Option<i64>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<salary_adjustment::Model>, i64), String> {
    let mut stmt = salary_adjustment::Entity::find();
    if let Some(eid) = employee_id {
        stmt = stmt.filter(salary_adjustment::Column::EmployeeId.eq(eid));
    }
    stmt = stmt
        .order_by_desc(salary_adjustment::Column::AdjustmentDate)
        .order_by_desc(salary_adjustment::Column::CreateTime);

    let page = std::cmp::max(page, 1);
    let page_size = std::cmp::max(page_size, 1);
    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?;
    Ok((items, total))
}

/// 查员工调薪历史（时间轴，按调薪日期倒序）
pub async fn get_employee_history(
    db: &DatabaseConnection,
    employee_id: i64,
) -> Result<Vec<salary_adjustment::Model>, String> {
    salary_adjustment::Entity::find()
        .filter(salary_adjustment::Column::EmployeeId.eq(employee_id))
        .filter(salary_adjustment::Column::Status.eq(1)) // 只看已通过的
        .order_by_desc(salary_adjustment::Column::AdjustmentDate)
        .all(db)
        .await
        .map_err(|e| e.to_string())
}

/// 创建调薪记录（状态默认 0=待审批）
pub async fn create_adjustment(
    db: &DatabaseConnection,
    dto: SalaryAdjustmentCreateDTO,
) -> Result<i64, String> {
    let now = Utc::now().naive_utc();

    // 解析调薪日期，未传则用当前时间
    let adjustment_date = match dto.adjustment_date.as_deref() {
        Some(s) => {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
                .or_else(|_| {
                    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                        .map(|d| d.and_hms_opt(0, 0, 0).unwrap_or(now))
                })
                .map_err(|e| format!("调薪日期格式错误: {}", e))?
        }
        None => now,
    };

    // 自动从 salary_config 读取 old 值（如果未提供）
    let (old_base_salary, old_position_allowance, old_performance_base) =
        if dto.old_base_salary.is_some()
            && dto.old_position_allowance.is_some()
            && dto.old_performance_base.is_some()
        {
            (
                dto.old_base_salary,
                dto.old_position_allowance,
                dto.old_performance_base,
            )
        } else {
            // 查 salary_config 当前生效配置
            let current = get_current_salary_config(db, dto.employee_id, adjustment_date.year()).await?;
            let cfg_old_base = current.as_ref().map(|c| c.base_salary);
            let cfg_old_position = current.as_ref().and_then(|c| c.position_allowance);
            let cfg_old_perf = current.as_ref().and_then(|c| c.performance_base);
            (
                dto.old_base_salary.or_else(|| {
                    cfg_old_base.map(|v| v.to_f64().unwrap_or_default())
                }),
                dto.old_position_allowance.or_else(|| {
                    cfg_old_position.map(|v| v.to_f64().unwrap_or_default())
                }),
                dto.old_performance_base.or_else(|| {
                    cfg_old_perf.map(|v| v.to_f64().unwrap_or_default())
                }),
            )
        };

    let active = salary_adjustment::ActiveModel {
        employee_id: Set(dto.employee_id),
        adjustment_date: Set(Some(adjustment_date)),
        adjustment_type: Set(dto.adjustment_type),
        old_base_salary: Set(old_base_salary.map(|v| Decimal::from_f64(v).unwrap_or_default())),
        new_base_salary: Set(dto.new_base_salary.map(|v| Decimal::from_f64(v).unwrap_or_default())),
        old_position_allowance: Set(old_position_allowance.map(|v| Decimal::from_f64(v).unwrap_or_default())),
        new_position_allowance: Set(dto.new_position_allowance.map(|v| Decimal::from_f64(v).unwrap_or_default())),
        old_performance_base: Set(old_performance_base.map(|v| Decimal::from_f64(v).unwrap_or_default())),
        new_performance_base: Set(dto.new_performance_base.map(|v| Decimal::from_f64(v).unwrap_or_default())),
        adjustment_reason: Set(dto.adjustment_reason),
        approver_id: Set(None),
        approver_name: Set(None),
        approve_time: Set(None),
        status: Set(Some(0)),
        create_time: Set(Some(now)),
        ..Default::default()
    };

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let inserted = active.insert(&txn).await.map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(inserted.id)
}

/// 审批通过
/// - 将调薪记录状态置为 1
/// - 同步更新 salary_config（按生效年月）
pub async fn approve_adjustment(
    db: &DatabaseConnection,
    id: i64,
    approver_id: i64,
    approver_name: &str,
) -> Result<(), String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let record = salary_adjustment::Entity::find_by_id(id)
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "调薪记录不存在".to_string())?;

    let status = record.status.unwrap_or(0);
    if status != 0 {
        return Err("只有待审批状态的调薪记录才能审批".to_string());
    }

    // 更新调薪记录
    let mut active: salary_adjustment::ActiveModel = record.clone().into();
    active.status = Set(Some(1));
    active.approver_id = Set(Some(approver_id));
    active.approver_name = Set(Some(approver_name.to_string()));
    active.approve_time = Set(Some(now));
    active.update(&txn).await.map_err(|e| e.to_string())?;

    // 同步到 salary_config
    // 取生效年月（adjustment_date 的年月）
    let adjustment_date = record.adjustment_date.unwrap_or(now);
    let year = adjustment_date.year();
    let month = Some(adjustment_date.month() as i32);

    let new_base = record.new_base_salary.unwrap_or_else(|| {
        record.old_base_salary.unwrap_or(Decimal::ZERO)
    });
    let new_position = record.new_position_allowance.or(record.old_position_allowance);
    let new_perf = record.new_performance_base.or(record.old_performance_base);

    upsert_salary_config(
        &txn,
        record.employee_id,
        year,
        month,
        new_base,
        new_position,
        new_perf,
        now,
    )
    .await?;

    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 审批驳回
pub async fn reject_adjustment(
    db: &DatabaseConnection,
    id: i64,
    approver_id: i64,
    approver_name: &str,
    reason: &str,
) -> Result<(), String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let record = salary_adjustment::Entity::find_by_id(id)
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "调薪记录不存在".to_string())?;

    let status = record.status.unwrap_or(0);
    if status != 0 {
        return Err("只有待审批状态的调薪记录才能驳回".to_string());
    }

    let mut active: salary_adjustment::ActiveModel = record.into();
    active.status = Set(Some(2));
    active.approver_id = Set(Some(approver_id));
    active.approver_name = Set(Some(approver_name.to_string()));
    active.approve_time = Set(Some(now));
    active.adjustment_reason = Set(Some(reason.to_string()));
    active.update(&txn).await.map_err(|e| e.to_string())?;

    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 调薪前后对比（最近一次调薪）
pub async fn get_comparison(
    db: &DatabaseConnection,
    employee_id: i64,
) -> Result<salary_adjustment::Model, String> {
    salary_adjustment::Entity::find()
        .filter(salary_adjustment::Column::EmployeeId.eq(employee_id))
        .order_by_desc(salary_adjustment::Column::AdjustmentDate)
        .order_by_desc(salary_adjustment::Column::CreateTime)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "暂无调薪记录".to_string())
}

// ==================== 内部工具 ====================

/// 查询员工当前生效的 salary_config
/// 优先匹配 month，其次 month=null 的全年配置
async fn get_current_salary_config(
    db: &DatabaseConnection,
    employee_id: i64,
    year: i32,
) -> Result<Option<salary_config::Model>, String> {
    let configs = salary_config::Entity::find()
        .filter(salary_config::Column::EmployeeId.eq(employee_id))
        .filter(salary_config::Column::Year.eq(year))
        .filter(salary_config::Column::Status.eq(1))
        .filter(salary_config::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    // 优先 month=当前月份 的，其次 month=null 的
    let now = Utc::now().naive_utc();
    let current_month = now.month() as i32;
    if let Some(c) = configs.iter().find(|c| c.month == Some(current_month)) {
        return Ok(Some(c.clone()));
    }
    if let Some(c) = configs.iter().find(|c| c.month.is_none()) {
        return Ok(Some(c.clone()));
    }
    Ok(None)
}

/// 新增或更新 salary_config
async fn upsert_salary_config<C: ConnectionTrait>(
    db: &C,
    employee_id: i64,
    year: i32,
    month: Option<i32>,
    base_salary: Decimal,
    position_allowance: Option<Decimal>,
    performance_base: Option<Decimal>,
    now: chrono::NaiveDateTime,
) -> Result<(), String> {
    // 查找现有配置（同 employee_id + year + month）
    let existing = salary_config::Entity::find()
        .filter(salary_config::Column::EmployeeId.eq(employee_id))
        .filter(salary_config::Column::Year.eq(year))
        .filter(salary_config::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let matched = existing.iter().find(|c| c.month == month).cloned();

    if let Some(model) = matched {
        let mut active: salary_config::ActiveModel = model.into();
        active.base_salary = Set(base_salary);
        active.position_allowance = Set(position_allowance);
        active.performance_base = Set(performance_base);
        active.status = Set(Some(1));
        active.update_time = Set(Some(now));
        active.update(db).await.map_err(|e| e.to_string())?;
    } else {
        let active = salary_config::ActiveModel {
            employee_id: Set(employee_id),
            year: Set(year),
            month: Set(month),
            base_salary: Set(base_salary),
            position_allowance: Set(position_allowance),
            performance_base: Set(performance_base),
            status: Set(Some(1)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        active.insert(db).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}
