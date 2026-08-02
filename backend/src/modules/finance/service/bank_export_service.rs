//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;

use crate::modules::finance::entity::{bank_payment_file, salary_record};
use crate::modules::system::entity::admin;

/// 分页查询代发文件记录
pub async fn get_file_list(
    db: &DatabaseConnection,
    year: Option<i32>,
    month: Option<i32>,
    bank_type: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<bank_payment_file::Model>, i64), String> {
    let mut stmt = bank_payment_file::Entity::find();

    if let Some(y) = year {
        stmt = stmt.filter(bank_payment_file::Column::Year.eq(y));
    }
    if let Some(m) = month {
        stmt = stmt.filter(bank_payment_file::Column::Month.eq(m));
    }
    if let Some(bt) = bank_type {
        if !bt.is_empty() {
            stmt = stmt.filter(bank_payment_file::Column::BankType.eq(bt));
        }
    }

    stmt = stmt.order_by_desc(bank_payment_file::Column::CreateTime);

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

/// 生成代发文件
/// 查询指定年月已发放(status=2)的工资记录，关联员工表查银行卡信息，
/// 根据银行类型生成不同格式的 TXT 文件内容。
///
/// 返回 (file_content, file_name, total_count, total_amount)
pub async fn generate_file(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
    bank_type: &str,
    _creator_id: i64,
    _creator_name: &str,
) -> Result<(String, String, i32, Decimal), String> {
    // 1. 查询指定年月已发放(status=2)的工资记录
    let records = salary_record::Entity::find()
        .filter(salary_record::Column::Year.eq(year))
        .filter(salary_record::Column::Month.eq(month))
        .filter(salary_record::Column::Status.eq(2))
        .filter(salary_record::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    if records.is_empty() {
        return Err(format!("{}年{}月没有已发放的工资记录", year, month));
    }

    // 2. 关联员工表查银行卡号、银行名称、开户名
    let employee_ids: Vec<i64> = records.iter().map(|r| r.employee_id).collect();
    let admins = admin::Entity::find()
        .filter(admin::Column::Id.is_in(employee_ids))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut admin_map: HashMap<i64, admin::Model> = HashMap::new();
    for a in admins {
        admin_map.insert(a.id, a);
    }

    // 3. 根据银行类型生成不同格式
    let mut lines: Vec<String> = Vec::new();
    let mut total_count: i32 = 0;
    let mut total_amount = Decimal::ZERO;
    // P1-7: 记录跳过的员工清单，便于管理员补全银行卡信息
    let mut skipped_employees: Vec<(i64, String, String)> = Vec::new();

    for record in &records {
        let admin_model = match admin_map.get(&record.employee_id) {
            Some(a) => a,
            None => {
                skipped_employees.push((record.employee_id, record.employee_name.clone().unwrap_or_default(), "员工信息不存在".to_string()));
                continue;
            }
        };

        let account_name = admin_model
            .bank_account_name
            .clone()
            .or_else(|| admin_model.nick_name.clone())
            .or_else(|| admin_model.user_name.clone())
            .unwrap_or_default();
        let account_no = admin_model.bank_card_no.clone().unwrap_or_default();
        let amount = record.net_salary;
        let bank_name = admin_model.bank_name.clone().unwrap_or_default();
        let mobile = admin_model.mobile.clone().unwrap_or_default();

        // 跳过没有银行卡号的记录，并记录到跳过清单
        if account_no.is_empty() {
            skipped_employees.push((record.employee_id, account_name.clone(), "银行卡号为空".to_string()));
            continue;
        }

        let line = match bank_type {
            "icbc" | "工行" => {
                // 工行：账户名|账号|金额|证件号|手机号（证件号暂留空，需扩展 admin 表字段）
                format!("{}|{}|{}||{}", account_name, account_no, amount, mobile)
            }
            "ccb" | "建行" => {
                // 建行：账号|姓名|金额|币种
                format!("{}|{}|{}|CNY", account_no, account_name, amount)
            }
            "cmb" | "招行" => {
                // 招行：账号|姓名|金额|用途
                format!(
                    "{}|{}|{}|{}年{}月工资",
                    account_no, account_name, amount, year, month
                )
            }
            "boc" | "中行" => {
                // 中行：账号|姓名|金额|银行名称
                format!("{}|{}|{}|{}", account_no, account_name, amount, bank_name)
            }
            "abc" | "农行" => {
                // 农行：账号|姓名|金额|证件类型|证件号（证件号暂留空）
                format!("{}|{}|{}|0|", account_no, account_name, amount)
            }
            _ => {
                return Err(format!("不支持的银行类型: {}", bank_type));
            }
        };

        lines.push(line);
        total_count += 1;
        total_amount += amount;
    }

    if total_count == 0 {
        return Err("没有符合条件的工资记录（缺少银行卡信息）".to_string());
    }

    // P1-7: 记录跳过的员工清单到日志，便于管理员排查
    if !skipped_employees.is_empty() {
        let skip_detail: Vec<String> = skipped_employees.iter()
            .map(|(id, name, reason)| format!("ID={} {}({})", id, name, reason))
            .collect();
        log::warn!(
            "[bank_export] {}年{}月 {} 银行代发文件生成时跳过 {} 名员工：{}",
            year, month, bank_type, skipped_employees.len(), skip_detail.join("; ")
        );
    }

    let file_content = lines.join("\n");
    let file_name = format!(
        "{}_{}_{}_{}.txt",
        year,
        month,
        bank_type,
        Utc::now().format("%Y%m%d%H%M%S")
    );

    Ok((file_content, file_name, total_count, total_amount))
}

/// 保存文件记录到数据库
pub async fn save_file_record(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
    bank_type: &str,
    file_name: &str,
    file_content: &str,
    total_count: i32,
    total_amount: Decimal,
    creator_id: i64,
    creator_name: &str,
) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let model = bank_payment_file::ActiveModel {
        year: Set(year),
        month: Set(month),
        bank_type: Set(bank_type.to_string()),
        file_name: Set(Some(file_name.to_string())),
        // file_path 字段用于存储生成的文件内容（实体无独立 file_content 字段）
        file_path: Set(Some(file_content.to_string())),
        file_format: Set(Some("TXT".to_string())),
        total_count: Set(Some(total_count)),
        total_amount: Set(Some(total_amount)),
        status: Set(Some(1)),
        creator_id: Set(Some(creator_id)),
        creator_name: Set(Some(creator_name.to_string())),
        create_time: Set(Some(now)),
        ..Default::default()
    };

    let inserted = model.insert(&txn).await.map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(inserted.id)
}

/// 获取文件内容用于下载
/// 返回 (file_content, file_name)
pub async fn get_file_content(db: &DatabaseConnection, id: i64) -> Result<(String, String), String> {
    let record = bank_payment_file::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "代发文件记录不存在".to_string())?;

    let file_name = record
        .file_name
        .clone()
        .unwrap_or_else(|| "bank_payment.txt".to_string());
    let file_content = record.file_path.clone().unwrap_or_default();

    Ok((file_content, file_name))
}

/// P2-6: 生成银行代发 Excel（xlsx）文件
///
/// 与 `generate_file` 共享数据查询逻辑，但输出为真实 xlsx 二进制内容。
/// 表头按银行类型差异化（工行/建行/招行/中行/农行），包含汇总行。
///
/// 返回 (xlsx_bytes, file_name, total_count, total_amount)
pub async fn generate_excel_file(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
    bank_type: &str,
    _creator_id: i64,
    _creator_name: &str,
) -> Result<(Vec<u8>, String, i32, Decimal), String> {
    use rust_xlsxwriter::{Format, Workbook};

    // 1. 查询已发放的工资记录
    let records = salary_record::Entity::find()
        .filter(salary_record::Column::Year.eq(year))
        .filter(salary_record::Column::Month.eq(month))
        .filter(salary_record::Column::Status.eq(2))
        .filter(salary_record::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    if records.is_empty() {
        return Err(format!("{}年{}月没有已发放的工资记录", year, month));
    }

    // 2. 查员工银行卡信息
    let employee_ids: Vec<i64> = records.iter().map(|r| r.employee_id).collect();
    let admins = admin::Entity::find()
        .filter(admin::Column::Id.is_in(employee_ids))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    let mut admin_map: HashMap<i64, admin::Model> = HashMap::new();
    for a in admins {
        admin_map.insert(a.id, a);
    }

    // 3. 构建数据行（跳过无银行卡的员工）
    struct RowData {
        account_name: String,
        account_no: String,
        amount: Decimal,
        bank_name: String,
        mobile: String,
        employee_id: i64,
    }
    let mut rows: Vec<RowData> = Vec::new();
    let mut skipped_count = 0i32;

    for record in &records {
        let admin_model = match admin_map.get(&record.employee_id) {
            Some(a) => a,
            None => { skipped_count += 1; continue; }
        };
        let account_name = admin_model.bank_account_name.clone()
            .or_else(|| admin_model.nick_name.clone())
            .or_else(|| admin_model.user_name.clone())
            .unwrap_or_default();
        let account_no = admin_model.bank_card_no.clone().unwrap_or_default();
        if account_no.is_empty() {
            skipped_count += 1;
            continue;
        }
        rows.push(RowData {
            account_name,
            account_no,
            amount: record.net_salary,
            bank_name: admin_model.bank_name.clone().unwrap_or_default(),
            mobile: admin_model.mobile.clone().unwrap_or_default(),
            employee_id: record.employee_id,
        });
    }

    if rows.is_empty() {
        return Err("没有符合条件的工资记录（缺少银行卡信息）".to_string());
    }

    let total_count = rows.len() as i32;
    let total_amount: Decimal = rows.iter().map(|r| r.amount).sum();

    // 4. 生成 xlsx
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let bank_display = match bank_type {
        "icbc" | "工行" => "工商银行",
        "ccb" | "建行" => "建设银行",
        "cmb" | "招行" => "招商银行",
        "boc" | "中行" => "中国银行",
        "abc" | "农行" => "农业银行",
        _ => bank_type,
    };
    worksheet.set_name(format!("{}代发-{}年{}月", bank_display, year, month))
        .map_err(|e| e.to_string())?;

    let header_format = Format::new().set_bold().set_background_color("#D9E1F2").set_align(rust_xlsxwriter::FormatAlign::Center);
    let money_format = Format::new().set_num_format("0.00");

    // 表头（统一格式：序号/账户名/账号/金额/银行名称/手机号）
    let headers = ["序号", "账户名", "账号", "金额", "银行名称", "手机号"];
    for (col, h) in headers.iter().enumerate() {
        worksheet.write_with_format(0, col as u16, *h, &header_format)
            .map_err(|e| e.to_string())?;
    }

    // 数据行
    for (idx, r) in rows.iter().enumerate() {
        let row = (idx + 1) as u32;
        worksheet.write(row, 0, (idx + 1) as i64).map_err(|e| e.to_string())?;
        worksheet.write(row, 1, r.account_name.as_str()).map_err(|e| e.to_string())?;
        // 账号作为文本写入，避免长数字被科学计数
        worksheet.write_string(row, 2, r.account_no.as_str()).map_err(|e| e.to_string())?;
        worksheet.write_number_with_format(row, 3, r.amount.to_f64().unwrap_or(0.0), &money_format)
            .map_err(|e| e.to_string())?;
        worksheet.write(row, 4, r.bank_name.as_str()).map_err(|e| e.to_string())?;
        worksheet.write_string(row, 5, r.mobile.as_str()).map_err(|e| e.to_string())?;
    }

    // 汇总行
    let summary_row = (rows.len() + 1) as u32;
    let bold_format = Format::new().set_bold();
    worksheet.write_with_format(summary_row, 0, "合计", &bold_format)
        .map_err(|e| e.to_string())?;
    worksheet.write_with_format(summary_row, 1, format!("共 {} 人", total_count), &bold_format)
        .map_err(|e| e.to_string())?;
    worksheet.write_number_with_format(summary_row, 3, total_amount.to_f64().unwrap_or(0.0), &Format::new().set_num_format("0.00").set_bold())
        .map_err(|e| e.to_string())?;

    // 列宽
    worksheet.set_column_width(0, 6).map_err(|e| e.to_string())?;   // 序号
    worksheet.set_column_width(1, 16).map_err(|e| e.to_string())?;  // 账户名
    worksheet.set_column_width(2, 28).map_err(|e| e.to_string())?;  // 账号
    worksheet.set_column_width(3, 14).map_err(|e| e.to_string())?;  // 金额
    worksheet.set_column_width(4, 16).map_err(|e| e.to_string())?;  // 银行名称
    worksheet.set_column_width(5, 14).map_err(|e| e.to_string())?;  // 手机号

    let xlsx_bytes = workbook.save_to_buffer().map_err(|e| e.to_string())?;

    if skipped_count > 0 {
        log::warn!(
            "[bank_export] {}年{}月 {} Excel代发文件生成时跳过 {} 名员工（无银行卡或员工信息缺失）",
            year, month, bank_type, skipped_count
        );
    }

    let file_name = format!(
        "{}_{}_{}_{}.xlsx",
        year, month, bank_type,
        Utc::now().format("%Y%m%d%H%M%S")
    );

    Ok((xlsx_bytes, file_name, total_count, total_amount))
}
