//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 考勤扣款服务
//! 负责考勤记录的 CRUD 与扣款计算
//!

use sea_orm::*;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};

use crate::modules::finance::entity::{attendance_record, salary_config};

// ==================== 常量 ====================

/// 月计薪天数
fn month_work_days() -> Decimal { Decimal::new(2175, 2) } // 21.75
/// 每日工作小时数
fn daily_work_hours() -> Decimal { Decimal::new(8, 0) } // 8
/// 迟到单次扣款
fn late_penalty() -> Decimal { Decimal::new(50, 0) }
/// 早退单次扣款
fn early_leave_penalty() -> Decimal { Decimal::new(30, 0) }
/// 旷工单次扣款
fn absent_penalty() -> Decimal { Decimal::new(200, 0) }
/// 全勤奖
fn full_attendance_bonus() -> Decimal { Decimal::new(200, 0) }
/// 病假扣款比例（80%）
fn sick_leave_rate() -> Decimal { Decimal::new(8, 1) } // 0.8
/// 工作日加班倍率
fn weekday_overtime_rate() -> Decimal { Decimal::new(15, 1) } // 1.5
/// 周末加班倍率
fn weekend_overtime_rate() -> Decimal { Decimal::new(2, 0) } // 2
/// 法定节假日加班倍率
fn holiday_overtime_rate() -> Decimal { Decimal::new(3, 0) } // 3

// ==================== DTO ====================

/// 考勤新增/更新 DTO
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceUpsertDTO {
    pub id: Option<i64>,
    pub employee_id: i64,
    pub year: i32,
    pub month: i32,
    pub work_days: Option<f64>,
    pub actual_work_days: Option<f64>,
    pub late_count: Option<i32>,
    pub early_leave_count: Option<i32>,
    pub absent_count: Option<i32>,
    pub personal_leave_days: Option<f64>,
    pub sick_leave_days: Option<f64>,
    pub annual_leave_days: Option<f64>,
    pub overtime_hours_weekday: Option<f64>,
    pub overtime_hours_weekend: Option<f64>,
    pub overtime_hours_holiday: Option<f64>,
    pub data_source: Option<i32>,
}

/// 批量导入记录项（与 Upsert 类似，但通常来自外部系统）
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceImportItem {
    pub employee_id: i64,
    pub year: i32,
    pub month: i32,
    pub work_days: Option<f64>,
    pub actual_work_days: Option<f64>,
    pub late_count: Option<i32>,
    pub early_leave_count: Option<i32>,
    pub absent_count: Option<i32>,
    pub personal_leave_days: Option<f64>,
    pub sick_leave_days: Option<f64>,
    pub annual_leave_days: Option<f64>,
    pub overtime_hours_weekday: Option<f64>,
    pub overtime_hours_weekend: Option<f64>,
    pub overtime_hours_holiday: Option<f64>,
    pub data_source: Option<i32>,
}

/// 考勤扣款计算结果
#[derive(serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceDeductionResult {
    /// 扣款总额（迟到+早退+旷工+请假）
    pub deduction_amount: f64,
    /// 加班费
    pub overtime_pay: f64,
    /// 全勤奖
    pub full_attendance_bonus: f64,
    /// 净调整额 = 扣款 - 加班费 - 全勤奖（正数表示净扣款，负数表示净补贴）
    pub total_adjustment: f64,
}

// ==================== CRUD ====================

/// 分页查询考勤记录
pub async fn get_attendance_list(
    db: &DatabaseConnection,
    year: Option<i32>,
    month: Option<i32>,
    employee_id: Option<i64>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<attendance_record::Model>, i64), String> {
    let mut stmt = attendance_record::Entity::find();
    if let Some(y) = year {
        stmt = stmt.filter(attendance_record::Column::Year.eq(y));
    }
    if let Some(m) = month {
        stmt = stmt.filter(attendance_record::Column::Month.eq(m));
    }
    if let Some(eid) = employee_id {
        stmt = stmt.filter(attendance_record::Column::EmployeeId.eq(eid));
    }
    stmt = stmt
        .order_by_desc(attendance_record::Column::Year)
        .order_by_desc(attendance_record::Column::Month)
        .order_by_desc(attendance_record::Column::CreateTime);

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

/// 查询单个员工考勤详情
pub async fn get_attendance_detail(
    db: &DatabaseConnection,
    employee_id: i64,
    year: i32,
    month: i32,
) -> Result<attendance_record::Model, String> {
    attendance_record::Entity::find()
        .filter(attendance_record::Column::EmployeeId.eq(employee_id))
        .filter(attendance_record::Column::Year.eq(year))
        .filter(attendance_record::Column::Month.eq(month))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "考勤记录不存在".to_string())
}

/// 新增/更新考勤记录（按 employee_id + year + month 唯一）
pub async fn upsert_attendance(
    db: &DatabaseConnection,
    dto: AttendanceUpsertDTO,
) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // 查找是否已存在（按 employee_id + year + month）
    let existing = attendance_record::Entity::find()
        .filter(attendance_record::Column::EmployeeId.eq(dto.employee_id))
        .filter(attendance_record::Column::Year.eq(dto.year))
        .filter(attendance_record::Column::Month.eq(dto.month))
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?;

    let work_days = dto.work_days.map(|v| Decimal::from_f64(v).unwrap_or_default());
    let actual_work_days = dto.actual_work_days.map(|v| Decimal::from_f64(v).unwrap_or_default());
    let personal_leave_days = Decimal::from_f64(dto.personal_leave_days.unwrap_or_default()).unwrap_or_default();
    let sick_leave_days = Decimal::from_f64(dto.sick_leave_days.unwrap_or_default()).unwrap_or_default();
    let annual_leave_days = Decimal::from_f64(dto.annual_leave_days.unwrap_or_default()).unwrap_or_default();
    let overtime_weekday = Decimal::from_f64(dto.overtime_hours_weekday.unwrap_or_default()).unwrap_or_default();
    let overtime_weekend = Decimal::from_f64(dto.overtime_hours_weekend.unwrap_or_default()).unwrap_or_default();
    let overtime_holiday = Decimal::from_f64(dto.overtime_hours_holiday.unwrap_or_default()).unwrap_or_default();

    let id = if let Some(model) = existing {
        let mut active: attendance_record::ActiveModel = model.into();
        active.work_days = Set(work_days);
        active.actual_work_days = Set(actual_work_days);
        active.late_count = Set(dto.late_count);
        active.early_leave_count = Set(dto.early_leave_count);
        active.absent_count = Set(dto.absent_count);
        active.personal_leave_days = Set(personal_leave_days);
        active.sick_leave_days = Set(sick_leave_days);
        active.annual_leave_days = Set(annual_leave_days);
        active.overtime_hours_weekday = Set(overtime_weekday);
        active.overtime_hours_weekend = Set(overtime_weekend);
        active.overtime_hours_holiday = Set(overtime_holiday);
        active.data_source = Set(dto.data_source);
        let updated = active.update(&txn).await.map_err(|e| e.to_string())?;
        updated.id
    } else {
        let active = attendance_record::ActiveModel {
            employee_id: Set(dto.employee_id),
            year: Set(dto.year),
            month: Set(dto.month),
            work_days: Set(work_days),
            actual_work_days: Set(actual_work_days),
            late_count: Set(dto.late_count),
            early_leave_count: Set(dto.early_leave_count),
            absent_count: Set(dto.absent_count),
            personal_leave_days: Set(personal_leave_days),
            sick_leave_days: Set(sick_leave_days),
            annual_leave_days: Set(annual_leave_days),
            overtime_hours_weekday: Set(overtime_weekday),
            overtime_hours_weekend: Set(overtime_weekend),
            overtime_hours_holiday: Set(overtime_holiday),
            data_source: Set(dto.data_source),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        let inserted = active.insert(&txn).await.map_err(|e| e.to_string())?;
        inserted.id
    };

    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(id)
}

/// 删除考勤记录
pub async fn delete_attendance(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    attendance_record::Entity::delete_by_id(id)
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 批量导入考勤记录
/// 已存在的（employee_id + year + month）将更新，否则插入
pub async fn batch_import(
    db: &DatabaseConnection,
    records: Vec<AttendanceImportItem>,
) -> Result<i64, String> {
    if records.is_empty() {
        return Ok(0);
    }
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let mut count: i64 = 0;

    for item in records {
        let existing = attendance_record::Entity::find()
            .filter(attendance_record::Column::EmployeeId.eq(item.employee_id))
            .filter(attendance_record::Column::Year.eq(item.year))
            .filter(attendance_record::Column::Month.eq(item.month))
            .one(&txn)
            .await
            .map_err(|e| e.to_string())?;

        let work_days = item.work_days.map(|v| Decimal::from_f64(v).unwrap_or_default());
        let actual_work_days = item.actual_work_days.map(|v| Decimal::from_f64(v).unwrap_or_default());
        let personal_leave_days = Decimal::from_f64(item.personal_leave_days.unwrap_or_default()).unwrap_or_default();
        let sick_leave_days = Decimal::from_f64(item.sick_leave_days.unwrap_or_default()).unwrap_or_default();
        let annual_leave_days = Decimal::from_f64(item.annual_leave_days.unwrap_or_default()).unwrap_or_default();
        let overtime_weekday = Decimal::from_f64(item.overtime_hours_weekday.unwrap_or_default()).unwrap_or_default();
        let overtime_weekend = Decimal::from_f64(item.overtime_hours_weekend.unwrap_or_default()).unwrap_or_default();
        let overtime_holiday = Decimal::from_f64(item.overtime_hours_holiday.unwrap_or_default()).unwrap_or_default();

        if let Some(model) = existing {
            let mut active: attendance_record::ActiveModel = model.into();
            active.work_days = Set(work_days);
            active.actual_work_days = Set(actual_work_days);
            active.late_count = Set(item.late_count);
            active.early_leave_count = Set(item.early_leave_count);
            active.absent_count = Set(item.absent_count);
            active.personal_leave_days = Set(personal_leave_days);
            active.sick_leave_days = Set(sick_leave_days);
            active.annual_leave_days = Set(annual_leave_days);
            active.overtime_hours_weekday = Set(overtime_weekday);
            active.overtime_hours_weekend = Set(overtime_weekend);
            active.overtime_hours_holiday = Set(overtime_holiday);
            active.data_source = Set(item.data_source);
            active.update(&txn).await.map_err(|e| e.to_string())?;
        } else {
            let active = attendance_record::ActiveModel {
                employee_id: Set(item.employee_id),
                year: Set(item.year),
                month: Set(item.month),
                work_days: Set(work_days),
                actual_work_days: Set(actual_work_days),
                late_count: Set(item.late_count),
                early_leave_count: Set(item.early_leave_count),
                absent_count: Set(item.absent_count),
                personal_leave_days: Set(personal_leave_days),
                sick_leave_days: Set(sick_leave_days),
                annual_leave_days: Set(annual_leave_days),
                overtime_hours_weekday: Set(overtime_weekday),
                overtime_hours_weekend: Set(overtime_weekend),
                overtime_hours_holiday: Set(overtime_holiday),
                data_source: Set(item.data_source),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            active.insert(&txn).await.map_err(|e| e.to_string())?;
        }
        count += 1;
    }

    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(count)
}

/// 计算考勤扣款
///
/// - 迟到扣款：late_count × 50
/// - 早退扣款：early_leave_count × 30
/// - 旷工扣款：absent_count × 200
/// - 请假扣款：personal_leave_days × (base_salary/21.75) + sick_leave_days × (base_salary/21.75×0.8)
/// - 加班费：weekday × (base_salary/21.75/8 × 1.5) + weekend × (×2) + holiday × (×3)
/// - 全勤奖：无迟到/早退/请假/旷工 且 actual_work_days >= work_days 则 +200
/// - total_adjustment = deduction_amount - overtime_pay - full_attendance_bonus
pub async fn calculate_deduction(
    db: &DatabaseConnection,
    employee_id: i64,
    year: i32,
    month: i32,
) -> Result<AttendanceDeductionResult, String> {
    // 查考勤记录
    let record = attendance_record::Entity::find()
        .filter(attendance_record::Column::EmployeeId.eq(employee_id))
        .filter(attendance_record::Column::Year.eq(year))
        .filter(attendance_record::Column::Month.eq(month))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "考勤记录不存在".to_string())?;

    // 查员工底薪配置（优先 month 精确匹配，其次 month=null 全年配置）
    let configs = salary_config::Entity::find()
        .filter(salary_config::Column::EmployeeId.eq(employee_id))
        .filter(salary_config::Column::Year.eq(year))
        .filter(salary_config::Column::Status.eq(1))
        .filter(salary_config::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut base_salary = Decimal::ZERO;
    for cfg in configs {
        if let Some(m) = cfg.month {
            if m == month {
                base_salary = cfg.base_salary;
                break;
            }
        } else {
            // month=null 的全年配置作为兜底
            base_salary = cfg.base_salary;
        }
    }

    // 计算各项扣款
    let late_count = record.late_count.unwrap_or(0);
    let early_leave_count = record.early_leave_count.unwrap_or(0);
    let absent_count = record.absent_count.unwrap_or(0);

    let late_deduction = Decimal::from(late_count) * late_penalty();
    let early_deduction = Decimal::from(early_leave_count) * early_leave_penalty();
    let absent_deduction = Decimal::from(absent_count) * absent_penalty();

    // 请假扣款（日薪 = base_salary / 21.75）
    let mwd = month_work_days();
    let daily_salary = if mwd.is_zero() {
        Decimal::ZERO
    } else {
        base_salary / mwd
    };
    let personal_leave_deduction = record.personal_leave_days * daily_salary;
    let sick_leave_deduction = record.sick_leave_days * daily_salary * sick_leave_rate();

    let deduction_amount =
        late_deduction + early_deduction + absent_deduction + personal_leave_deduction + sick_leave_deduction;

    // 加班费（时薪 = base_salary / 21.75 / 8）
    let dwh = daily_work_hours();
    let hourly_salary = if mwd.is_zero() || dwh.is_zero() {
        Decimal::ZERO
    } else {
        base_salary / mwd / dwh
    };
    let weekday_overtime_pay = record.overtime_hours_weekday * hourly_salary * weekday_overtime_rate();
    let weekend_overtime_pay = record.overtime_hours_weekend * hourly_salary * weekend_overtime_rate();
    let holiday_overtime_pay = record.overtime_hours_holiday * hourly_salary * holiday_overtime_rate();
    let overtime_pay = weekday_overtime_pay + weekend_overtime_pay + holiday_overtime_pay;

    // 全勤奖：无迟到/早退/请假/旷工 且 actual_work_days >= work_days
    let no_violations = late_count == 0
        && early_leave_count == 0
        && absent_count == 0
        && record.personal_leave_days.is_zero()
        && record.sick_leave_days.is_zero();
    let work_days = record.work_days.unwrap_or_default();
    let actual_work_days = record.actual_work_days.unwrap_or_default();
    let meets_work_days = actual_work_days >= work_days;
    let full_attendance_bonus = if no_violations && meets_work_days {
        full_attendance_bonus()
    } else {
        Decimal::ZERO
    };

    // 净调整额 = 扣款 - 加班费 - 全勤奖
    let total_adjustment = deduction_amount - overtime_pay - full_attendance_bonus;

    Ok(AttendanceDeductionResult {
        deduction_amount: deduction_amount.to_f64().unwrap_or_default(),
        overtime_pay: overtime_pay.to_f64().unwrap_or_default(),
        full_attendance_bonus: full_attendance_bonus.to_f64().unwrap_or_default(),
        total_adjustment: total_adjustment.to_f64().unwrap_or_default(),
    })
}
