//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 业绩统计导出（开发计划书 WP3/WP4）
//!
//! - Excel：rust_xlsxwriter 双 Sheet（业绩排行 + 月度汇总）
//! - PDF：typst 排版（复用 pdf_compiler_service 组装器与 typst_world 字体）
//!
//! 数据复用 get_performance_ranking / get_monthly_performance，
//! 自动继承部门筛选（WP1）、统计口径配置与隐私打码（WP3）。

use rust_decimal::prelude::ToPrimitive;
use rust_xlsxwriter::{Format, FormatAlign, Workbook};
use sea_orm::DbConn;

use crate::core::errors::error::Result;
use crate::modules::statistics::service::performance_overview_service;
use crate::modules::system::service::pdf_compiler_service::{assemble_typst_source, escape_typst, PdfPageOptions};

/// 导出查询参数（与前端 exportPerformanceApi 请求体同构）
#[derive(Debug, Clone)]
pub struct ExportParams {
    pub year: i32,
    pub month: Option<i32>,
    pub department_id: Option<i64>,
    pub order_by: Option<String>,
}

impl ExportParams {
    pub fn period_label(&self) -> String {
        match self.month {
            Some(m) => format!("{} 年 {} 月", self.year, m),
            None => format!("{} 年度", self.year),
        }
    }
}

fn plan_status_label(status: Option<i32>) -> &'static str {
    match status {
        Some(0) => "草稿",
        Some(1) => "审批中",
        Some(2) => "已通过",
        Some(3) => "已驳回",
        _ => "未提交",
    }
}

fn rate_text(value: Option<rust_decimal::Decimal>) -> String {
    value.map(|v| format!("{:.1}%", v.to_f64().unwrap_or(0.0))).unwrap_or_default()
}

fn money_text(value: Option<rust_decimal::Decimal>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

/// Excel 导出：业绩排行 + 月度汇总两个 Sheet
pub async fn export_excel(db: &DbConn, params: &ExportParams, accessible_user_ids: Option<Vec<i64>>, current_user_id: Option<i64>) -> Result<Vec<u8>> {
    let ranking = performance_overview_service::get_performance_ranking(
        db,
        Some(params.year),
        params.month,
        params.order_by.clone(),
        params.department_id,
        accessible_user_ids.clone(),
        current_user_id,
    )
    .await?;
    let monthly = performance_overview_service::get_monthly_performance(
        db,
        Some(params.year),
        params.department_id,
        accessible_user_ids,
    )
    .await?;

    let mut workbook = Workbook::new();

    // ===== Sheet1：业绩排行 =====
    let sheet = workbook.add_worksheet();
    sheet.set_name("业绩排行").map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
    let header_format = Format::new()
        .set_bold()
        .set_background_color("#D9E1F2")
        .set_align(FormatAlign::Center);
    let money_format = Format::new().set_num_format("#,##0.00");
    let rate_format = Format::new().set_num_format("0.0");

    let headers = ["排名", "姓名", "部门", "合同目标", "合同实际", "合同完成率(%)", "回款目标", "回款实际", "回款完成率(%)", "计划状态"];
    for (col, h) in headers.iter().enumerate() {
        sheet.write_with_format(0, col as u16, *h, &header_format)
            .map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
    }
    for (i, row) in ranking.list.iter().enumerate() {
        let r = (i + 1) as u32;
        sheet.write(r, 0, row.rank.unwrap_or(0) as i64).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        sheet.write(r, 1, row.employee_name.as_deref().unwrap_or("")).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        sheet.write(r, 2, row.department_name.as_deref().unwrap_or("")).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        if let Some(v) = row.contract_target {
            sheet.write_number_with_format(r, 3, v.to_f64().unwrap_or(0.0), &money_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        }
        if let Some(v) = row.contract_amount {
            sheet.write_number_with_format(r, 4, v.to_f64().unwrap_or(0.0), &money_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        }
        if let Some(v) = row.contract_completion_rate {
            sheet.write_number_with_format(r, 5, v.to_f64().unwrap_or(0.0), &rate_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        }
        if let Some(v) = row.payment_target {
            sheet.write_number_with_format(r, 6, v.to_f64().unwrap_or(0.0), &money_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        }
        if let Some(v) = row.payment_amount {
            sheet.write_number_with_format(r, 7, v.to_f64().unwrap_or(0.0), &money_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        }
        if let Some(v) = row.payment_completion_rate {
            sheet.write_number_with_format(r, 8, v.to_f64().unwrap_or(0.0), &rate_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        }
        sheet.write(r, 9, plan_status_label(row.plan_status)).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
    }
    // WP4：无负责人金额追加到榜单末尾，形成数据质量提示
    let mut tail_row = ranking.list.len() as u32 + 1;
    if ranking.unassigned_contract_amount.is_some() || ranking.unassigned_payment_amount.is_some() {
        sheet.write(tail_row, 1, "【未分配负责人】").map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        if let Some(v) = ranking.unassigned_contract_amount {
            sheet.write_number_with_format(tail_row, 4, v.to_f64().unwrap_or(0.0), &money_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        }
        if let Some(v) = ranking.unassigned_payment_amount {
            sheet.write_number_with_format(tail_row, 7, v.to_f64().unwrap_or(0.0), &money_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        }
        tail_row += 1;
    }
    for col in 0..headers.len() as u16 {
        sheet.set_column_width(col, 14).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
    }

    // ===== Sheet2：月度汇总 =====
    let sheet2 = workbook.add_worksheet();
    sheet2.set_name("月度汇总").map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
    let headers2 = ["月份", "合同目标", "合同实际", "合同完成率(%)", "回款目标", "回款实际", "回款完成率(%)", "合同数", "回款数"];
    for (col, h) in headers2.iter().enumerate() {
        sheet2.write_with_format(0, col as u16, *h, &header_format)
            .map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
    }
    if let Some(months) = &monthly.months {
        for (i, m) in months.iter().enumerate() {
            let r = (i + 1) as u32;
            sheet2.write(r, 0, format!("{} 月", m.month.unwrap_or(0))).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
            if let Some(v) = m.contract_target {
                sheet2.write_number_with_format(r, 1, v.to_f64().unwrap_or(0.0), &money_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
            }
            if let Some(v) = m.contract_actual {
                sheet2.write_number_with_format(r, 2, v.to_f64().unwrap_or(0.0), &money_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
            }
            if let Some(v) = m.contract_completion_rate {
                sheet2.write_number_with_format(r, 3, v.to_f64().unwrap_or(0.0), &rate_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
            }
            if let Some(v) = m.payment_target {
                sheet2.write_number_with_format(r, 4, v.to_f64().unwrap_or(0.0), &money_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
            }
            if let Some(v) = m.payment_actual {
                sheet2.write_number_with_format(r, 5, v.to_f64().unwrap_or(0.0), &money_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
            }
            if let Some(v) = m.payment_completion_rate {
                sheet2.write_number_with_format(r, 6, v.to_f64().unwrap_or(0.0), &rate_format).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
            }
            sheet2.write(r, 7, m.contract_count.unwrap_or(0)).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
            sheet2.write(r, 8, m.payment_count.unwrap_or(0)).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
        }
    }
    for col in 0..headers2.len() as u16 {
        sheet2.set_column_width(col, 14).map_err(|e| crate::core::errors::error::Error::from(e.to_string()))?;
    }

    workbook.save_to_buffer().map_err(|e| crate::core::errors::error::Error::from(e.to_string()))
}

/// PDF 导出：排行表 + 月度汇总表（typst 排版）
pub async fn export_pdf(db: &DbConn, params: &ExportParams, accessible_user_ids: Option<Vec<i64>>, current_user_id: Option<i64>) -> Result<Vec<u8>> {
    let ranking = performance_overview_service::get_performance_ranking(
        db,
        Some(params.year),
        params.month,
        params.order_by.clone(),
        params.department_id,
        accessible_user_ids.clone(),
        current_user_id,
    )
    .await?;
    let monthly = performance_overview_service::get_monthly_performance(
        db,
        Some(params.year),
        params.department_id,
        accessible_user_ids,
    )
    .await?;

    let mut body = String::new();
    body.push_str(&format!(
        "= 业绩概览导出（{}）\n\n",
        escape_typst(&params.period_label())
    ));

    // 排行表
    body.push_str("== 业绩排行\n\n");
    body.push_str("#table(\n  columns: 8,\n  align: center,\n  table.header([*排名*], [*姓名*], [*部门*], [*合同目标*], [*合同实际*], [*合同完成率*], [*回款实际*], [*回款完成率*]),\n");
    for row in &ranking.list {
        body.push_str(&format!(
            "  [{}], [{}], [{}], [{}], [{}], [{}], [{}], [{}],\n",
            row.rank.unwrap_or(0),
            escape_typst(row.employee_name.as_deref().unwrap_or("-")),
            escape_typst(row.department_name.as_deref().unwrap_or("-")),
            escape_typst(&money_text(row.contract_target)),
            escape_typst(&money_text(row.contract_amount)),
            escape_typst(&rate_text(row.contract_completion_rate)),
            escape_typst(&money_text(row.payment_amount)),
            escape_typst(&rate_text(row.payment_completion_rate)),
        ));
    }
    if ranking.unassigned_contract_amount.is_some() || ranking.unassigned_payment_amount.is_some() {
        body.push_str(&format!(
            "  [-], [【未分配负责人】], [-], [-], [{}], [-], [{}], [-],\n",
            escape_typst(&money_text(ranking.unassigned_contract_amount)),
            escape_typst(&money_text(ranking.unassigned_payment_amount)),
        ));
    }
    body.push_str(")\n\n");

    // 月度汇总表
    body.push_str("== 月度汇总\n\n");
    body.push_str("#table(\n  columns: 7,\n  align: center,\n  table.header([*月份*], [*合同目标*], [*合同实际*], [*合同完成率*], [*回款目标*], [*回款实际*], [*回款完成率*]),\n");
    if let Some(months) = &monthly.months {
        for m in months {
            body.push_str(&format!(
                "  [{} 月], [{}], [{}], [{}], [{}], [{}], [{}],\n",
                m.month.unwrap_or(0),
                escape_typst(&money_text(m.contract_target)),
                escape_typst(&money_text(m.contract_actual)),
                escape_typst(&rate_text(m.contract_completion_rate)),
                escape_typst(&money_text(m.payment_target)),
                escape_typst(&money_text(m.payment_actual)),
                escape_typst(&rate_text(m.payment_completion_rate)),
            ));
        }
    }
    body.push_str(")\n");

    // 横向 A4 更适合宽表
    let opts = PdfPageOptions {
        orientation: "landscape".to_string(),
        ..Default::default()
    };
    let source = assemble_typst_source(&body, &None, &None, &opts);
    crate::modules::system::service::typst_world::compile_to_pdf(&source).map_err(crate::core::errors::error::Error::from)
}
