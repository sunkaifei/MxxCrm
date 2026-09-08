//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! V7-7: 工资单 Excel/PDF 导出服务
//! P2-4: 新增真实 xlsx 导出（基于 rust_xlsxwriter），保留 CSV 作为兼容格式
//! CSV 可被 Excel/WPS 直接打开，xlsx 提供更好的格式化与多 sheet 支持

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use rust_decimal::prelude::ToPrimitive;

use crate::modules::finance::entity::{salary_record, salary_tax_detail};
use crate::modules::finance::service::salary_service::{resolve_data_scope, SalaryDataScope};

/// 根据当前用户数据权限，计算实际可导出的 employee_ids 过滤条件
/// 返回 None 表示可导出全员（All 权限），Some(ids) 表示仅这些员工
async fn resolve_export_scope(
    db: &DatabaseConnection,
    user_id: i64,
    employee_ids: Option<&[i64]>,
) -> Result<Option<Vec<i64>>, String> {
    let (scope, allowed_ids) = resolve_data_scope(db, user_id).await?;
    match scope {
        SalaryDataScope::All => Ok(employee_ids.map(|ids| ids.to_vec())),
        _ => {
            // 非全量权限：取 allowed_ids 与传入 employee_ids 的交集
            let allowed: std::collections::HashSet<i64> = allowed_ids.into_iter().collect();
            let result: Vec<i64> = match employee_ids {
                Some(ids) => ids.iter().filter(|id| allowed.contains(id)).copied().collect(),
                None => allowed.into_iter().collect(),
            };
            Ok(Some(result))
        }
    }
}

/// V7-7: 导出工资单为 CSV（UTF-8 BOM，Excel 兼容）
///
/// - year/month 必填
/// - employee_ids 为空时按当前用户权限导出
pub async fn export_salary_csv(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
    employee_ids: Option<&[i64]>,
    user_id: i64,
) -> Result<Vec<u8>, String> {
    let scoped_ids = resolve_export_scope(db, user_id, employee_ids).await?;

    let mut stmt = salary_record::Entity::find()
        .filter(salary_record::Column::Year.eq(year))
        .filter(salary_record::Column::Month.eq(month))
        .filter(salary_record::Column::Deleted.eq(0));

    if let Some(ref ids) = scoped_ids {
        if !ids.is_empty() {
            stmt = stmt.filter(salary_record::Column::EmployeeId.is_in(ids.clone()));
        } else {
            // 权限范围内无数据，直接返回空表
            let mut buf: Vec<u8> = vec![0xEF, 0xBB, 0xBF];
            buf.extend(b"\xE5\x91\x98\xE5\xB7\xA5ID,\xE5\x91\x98\xE5\xB7\xA5\xE5\xA7\x93\xE5\x90\x8D,\xE9\x83\xA8\xE9\x97\xA8,\xE5\xB9\xB4,\xE6\x9C\x88,\xE5\x9F\xBA\xE6\x9C\xAC\xE5\xB7\xA5\xE8\xB5\x84,\xE6\x8F\x90\xE6\x88\x90\xE9\x87\x91\xE9\xA2\x9D,\xE7\xBB\xA9\xE6\x95\x88\xE5\xA5\x96\xE9\x87\x91,\xE5\x9B\xA2\xE9\x98\x9F\xE6\x8F\x90\xE6\x88\x90,\xE6\x89\xA3\xE6\xAC\xBE\xE9\x87\x91\xE9\xA2\x9D,\xE5\xBA\x94\xE5\x8F\x91\xE5\xB7\xA5\xE8\xB5\x84,\xE4\xB8\xAA\xE4\xBA\xBA\xE7\xA4\xBE\xE4\xBF\x9D,\xE4\xB8\xAA\xE4\xBA\xBA\xE5\x85\xAC\xE7\xA7\xAF\xE9\x87\x91,\xE5\x8D\x95\xE4\xBD\x8D\xE7\xA4\xBE\xE4\xBF\x9D,\xE5\x8D\x95\xE4\xBD\x8D\xE5\x85\xAC\xE7\xA7\xAF\xE9\x87\x91,\xE4\xB8\xAA\xE7\xA8\x8E\xE9\x87\x91\xE9\xA2\x9D,\xE5\xAE\x9E\xE5\x8F\x91\xE5\xB7\xA5\xE8\xB5\x84,\xE7\x8A\xB6\xE6\x80\x81,\xE5\xA4\x87\xE6\xB3\xA8\n");
            return Ok(buf);
        }
    }

    let records = stmt.all(db).await.map_err(|e| e.to_string())?;

    // UTF-8 BOM（让 Excel 正确识别编码）
    let mut buf: Vec<u8> = vec![0xEF, 0xBB, 0xBF];

    // 表头
    let header = "员工ID,员工姓名,部门,年,月,基本工资,提成金额,绩效奖金,团队提成,扣款金额,应发工资,个人社保,个人公积金,单位社保,单位公积金,个税金额,实发工资,状态,备注\n";
    buf.extend(header.as_bytes());

    for r in records {
        let status_str = match r.status.unwrap_or(0) {
            0 => "待审核",
            1 => "已审核",
            2 => "已发放",
            _ => "未知",
        };
        let row = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            r.employee_id,
            csv_escape(r.employee_name.as_deref().unwrap_or("")),
            csv_escape(r.department_name.as_deref().unwrap_or("")),
            r.year,
            r.month,
            r.base_salary,
            r.commission_amount,
            r.performance_bonus,
            r.team_commission_amount,
            r.deduction_amount,
            r.total_salary,
            r.social_insurance_personal,
            r.housing_fund_personal,
            r.social_insurance_company,
            r.housing_fund_company,
            r.tax_amount,
            r.net_salary,
            status_str,
            csv_escape(r.remark.as_deref().unwrap_or("")),
        );
        buf.extend(row.as_bytes());
    }

    Ok(buf)
}

/// V7-8: 导出个税申报数据为 CSV
///
/// 字段：员工ID、年度、月份、本月收入、当月减除费用、当月专项扣除、当月其他扣除、累计收入、累计应纳税所得额、适用税率、速算扣除数、累计应纳税额、累计已预扣、本月应纳税额
pub async fn export_tax_csv(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
    user_id: i64,
) -> Result<Vec<u8>, String> {
    let scoped_ids = resolve_export_scope(db, user_id, None).await?;

    let mut stmt = salary_tax_detail::Entity::find()
        .filter(salary_tax_detail::Column::Year.eq(year))
        .filter(salary_tax_detail::Column::Month.eq(month));

    if let Some(ref ids) = scoped_ids {
        if !ids.is_empty() {
            stmt = stmt.filter(salary_tax_detail::Column::EmployeeId.is_in(ids.clone()));
        } else {
            // 权限范围内无数据，返回空表
            let mut buf: Vec<u8> = vec![0xEF, 0xBB, 0xBF];
            buf.extend(b"\xE5\x91\x98\xE5\xB7\xA5ID,\xE5\xB9\xB4\xE5\xBA\xA6,\xE6\x9C\x88\xE4\xBB\xBD,\xE6\x9C\xAC\xE6\x9C\x88\xE6\x94\xB6\xE5\x85\xA5,\xE5\xBD\x93\xE6\x9C\x88\xE5\x87\x8F\xE9\x99\xA4\xE8\xB4\xB9\xE7\x94\xA8,\xE5\xBD\x93\xE6\x9C\x88\xE4\xB8\x93\xE9\xA1\xB9\xE6\x89\xA3\xE9\x99\xA4,\xE5\xBD\x93\xE6\x9C\x88\xE5\x85\xB6\xE4\xBB\x96\xE6\x89\xA3\xE9\x99\xA4,\xE7\xB4\xAF\xE8\xAE\xA1\xE6\x94\xB6\xE5\x85\xA5,\xE7\xB4\xAF\xE8\xAE\xA1\xE5\xBA\x94\xE7\xBA\xB3\xE7\xA8\x8E\xE6\x89\x80\xE5\xBE\x97\xE9\xA2\x9D,\xE9\x80\x82\xE7\x94\xA8\xE7\xA8\x8E\xE7\x8E\x87,\xE9\x80\x9F\xE7\xAE\x97\xE6\x89\xA3\xE9\x99\xA4\xE6\x95\xB0,\xE7\xB4\xAF\xE8\xAE\xA1\xE5\xBA\x94\xE7\xBA\xB3\xE7\xA8\x8E\xE9\xA2\x9D,\xE7\xB4\xAF\xE8\xAE\xA1\xE5\xB7\xB2\xE9\xA2\x84\xE6\x89\xA3,\xE6\x9C\xAC\xE6\x9C\x88\xE5\xBA\x94\xE7\xBA\xB3\xE7\xA8\x8E\xE9\xA2\x9D\n");
            return Ok(buf);
        }
    }

    let details = stmt.all(db).await.map_err(|e| e.to_string())?;

    let mut buf: Vec<u8> = vec![0xEF, 0xBB, 0xBF];

    let header = "员工ID,年度,月份,本月收入,当月减除费用,当月专项扣除,当月其他扣除,累计收入,累计应纳税所得额,适用税率,速算扣除数,累计应纳税额,累计已预扣,本月应纳税额\n";
    buf.extend(header.as_bytes());

    for d in details {
        let row = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            d.employee_id,
            d.year,
            d.month,
            opt_dec(d.monthly_income),
            opt_dec(d.monthly_threshold),
            opt_dec(d.monthly_special_deduction),
            opt_dec(d.monthly_other_deduction),
            opt_dec(d.cumulative_income),
            opt_dec(d.cumulative_taxable),
            opt_dec(d.applicable_rate),
            opt_dec(d.quick_deduction),
            opt_dec(d.cumulative_tax_should),
            opt_dec(d.cumulative_tax_paid),
            opt_dec(d.monthly_tax),
        );
        buf.extend(row.as_bytes());
    }

    Ok(buf)
}

/// Decimal Option 转 String（None 输出空字符串）
fn opt_dec(v: Option<rust_decimal::Decimal>) -> String {
    v.map(|d| d.to_string()).unwrap_or_default()
}

/// CSV 字段转义：包含逗号/引号/换行时用双引号包裹，内部双引号转义为两个双引号
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

// ==================== P2-4: xlsx 真实 Excel 导出 ====================

/// P2-4: 导出工资单为 xlsx（真实 Excel 格式，含表头样式与金额格式）
///
/// - year/month 必填
/// - employee_ids 为空时导出全员
/// - 返回 xlsx 二进制内容
pub async fn export_salary_xlsx(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
    employee_ids: Option<&[i64]>,
    user_id: i64,
) -> Result<Vec<u8>, String> {
    use rust_xlsxwriter::{Format, Workbook};

    let scoped_ids = resolve_export_scope(db, user_id, employee_ids).await?;

    let mut stmt = salary_record::Entity::find()
        .filter(salary_record::Column::Year.eq(year))
        .filter(salary_record::Column::Month.eq(month))
        .filter(salary_record::Column::Deleted.eq(0));

    if let Some(ref ids) = scoped_ids {
        if !ids.is_empty() {
            stmt = stmt.filter(salary_record::Column::EmployeeId.is_in(ids.clone()));
        } else {
            // 权限范围内无数据，返回空表
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            worksheet.set_name("无数据").map_err(|e| e.to_string())?;
            return workbook.save_to_buffer().map_err(|e| e.to_string());
        }
    }

    let records = stmt.all(db).await.map_err(|e| e.to_string())?;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name(format!("{}年{}月工资单", year, month))
        .map_err(|e| e.to_string())?;

    // 表头格式：加粗、居中、灰色背景
    let header_format = Format::new().set_bold().set_background_color("#D9E1F2").set_align(rust_xlsxwriter::FormatAlign::Center);
    // 金额格式：保留两位小数
    let money_format = Format::new().set_num_format("0.00");

    // 表头
    let headers = ["员工ID", "员工姓名", "部门", "年", "月", "基本工资", "提成金额",
        "绩效奖金", "团队提成", "扣款金额", "应发工资", "个人社保", "个人公积金",
        "单位社保", "单位公积金", "个税金额", "实发工资", "状态", "备注"];
    for (col, h) in headers.iter().enumerate() {
        worksheet.write_with_format(0, col as u16, *h, &header_format)
            .map_err(|e| e.to_string())?;
    }

    // 数据行
    for (row_idx, r) in records.iter().enumerate() {
        let row = (row_idx + 1) as u32;
        let status_str = match r.status.unwrap_or(0) {
            0 => "待审核",
            1 => "已审核",
            2 => "已发放",
            _ => "未知",
        };
        // 文本列
        worksheet.write(row, 0, r.employee_id).map_err(|e| e.to_string())?;
        worksheet.write(row, 1, r.employee_name.as_deref().unwrap_or(""))
            .map_err(|e| e.to_string())?;
        worksheet.write(row, 2, r.department_name.as_deref().unwrap_or(""))
            .map_err(|e| e.to_string())?;
        worksheet.write(row, 3, r.year).map_err(|e| e.to_string())?;
        worksheet.write(row, 4, r.month).map_err(|e| e.to_string())?;
        // 金额列（带格式）
        let money_cols = [
            r.base_salary, r.commission_amount, r.performance_bonus,
            r.team_commission_amount, r.deduction_amount, r.total_salary,
            r.social_insurance_personal, r.housing_fund_personal,
            r.social_insurance_company, r.housing_fund_company,
            r.tax_amount, r.net_salary,
        ];
        for (col_off, val) in money_cols.iter().enumerate() {
            worksheet.write_number_with_format(row, (5 + col_off) as u16, val.to_f64().unwrap_or(0.0), &money_format)
                .map_err(|e| e.to_string())?;
        }
        worksheet.write(row, 17, status_str).map_err(|e| e.to_string())?;
        worksheet.write(row, 18, r.remark.as_deref().unwrap_or(""))
            .map_err(|e| e.to_string())?;
    }

    // 自动列宽（简单估算）
    for col in 0..headers.len() {
        worksheet.set_column_width(col as u16, 14).map_err(|e| e.to_string())?;
    }

    workbook.save_to_buffer().map_err(|e| e.to_string())
}

/// P2-4: 导出个税申报数据为 xlsx
pub async fn export_tax_xlsx(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
    user_id: i64,
) -> Result<Vec<u8>, String> {
    use rust_xlsxwriter::{Format, Workbook};

    let scoped_ids = resolve_export_scope(db, user_id, None).await?;

    let mut stmt = salary_tax_detail::Entity::find()
        .filter(salary_tax_detail::Column::Year.eq(year))
        .filter(salary_tax_detail::Column::Month.eq(month));

    if let Some(ref ids) = scoped_ids {
        if !ids.is_empty() {
            stmt = stmt.filter(salary_tax_detail::Column::EmployeeId.is_in(ids.clone()));
        } else {
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            worksheet.set_name("无数据").map_err(|e| e.to_string())?;
            return workbook.save_to_buffer().map_err(|e| e.to_string());
        }
    }

    let details = stmt.all(db).await.map_err(|e| e.to_string())?;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name(format!("{}年{}月个税申报", year, month))
        .map_err(|e| e.to_string())?;

    let header_format = Format::new().set_bold().set_background_color("#D9E1F2").set_align(rust_xlsxwriter::FormatAlign::Center);
    let money_format = Format::new().set_num_format("0.00");

    let headers = ["员工ID", "年度", "月份", "本月收入", "当月减除费用", "当月专项扣除",
        "当月其他扣除", "累计收入", "累计应纳税所得额", "适用税率", "速算扣除数",
        "累计应纳税额", "累计已预扣", "本月应纳税额"];
    for (col, h) in headers.iter().enumerate() {
        worksheet.write_with_format(0, col as u16, *h, &header_format)
            .map_err(|e| e.to_string())?;
    }

    for (row_idx, d) in details.iter().enumerate() {
        let row = (row_idx + 1) as u32;
        worksheet.write(row, 0, d.employee_id).map_err(|e| e.to_string())?;
        worksheet.write(row, 1, d.year).map_err(|e| e.to_string())?;
        worksheet.write(row, 2, d.month).map_err(|e| e.to_string())?;
        let vals = [
            d.monthly_income, d.monthly_threshold, d.monthly_special_deduction,
            d.monthly_other_deduction, d.cumulative_income, d.cumulative_taxable,
            d.applicable_rate, d.quick_deduction, d.cumulative_tax_should,
            d.cumulative_tax_paid, d.monthly_tax,
        ];
        for (col_off, v) in vals.iter().enumerate() {
            worksheet.write_number_with_format(row, (3 + col_off) as u16, opt_dec_f64(*v), &money_format)
                .map_err(|e| e.to_string())?;
        }
    }

    for col in 0..headers.len() {
        worksheet.set_column_width(col as u16, 16).map_err(|e| e.to_string())?;
    }

    workbook.save_to_buffer().map_err(|e| e.to_string())
}

/// Decimal Option 转 f64（None 输出 0.0）
fn opt_dec_f64(v: Option<rust_decimal::Decimal>) -> f64 {
    use rust_decimal::prelude::ToPrimitive;
    v.map(|d| d.to_f64().unwrap_or(0.0)).unwrap_or(0.0)
}
