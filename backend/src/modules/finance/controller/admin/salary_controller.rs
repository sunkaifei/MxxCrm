//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::permission_guard::require_permission;
use serde::Deserialize;

use crate::core::kit::global::AppState;
use crate::core::web::base_controller::{get_current_user, get_current_user_id};
use crate::core::web::entity::common::InfoId;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::finance::model::salary::{
    SalaryQuery, SalaryCalculateDTO, SalaryUpdateDTO, SalaryBatchDTO, SalaryTrendQuery,
};
use crate::modules::finance::service::salary_service;
use crate::modules::finance::service::salary_export_service;

#[derive(Deserialize)]
pub struct SummaryQuery {
    pub year: i32,
    pub month: i32,
}

pub async fn list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<SalaryQuery>,
) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let page = query.page.unwrap_or(1) as u32;

    let user_id = get_current_user_id(&req);

    match salary_service::get_list(db, query, user_id).await {
        Ok((list, total)) => {
            let payload = serde_json::json!({
                "items": list,
                "total": total,
            });
            HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::success_with_page(payload, "local", page, total as u32))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn detail(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;
    let user_id = get_current_user_id(&req);

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "工资记录ID不能为空", "local"));
    }

    match salary_service::get_detail(db, item.id.unwrap(), user_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn calculate(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<SalaryCalculateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    let (operator_id, username) = get_current_user(&req);
    let operator_name: &str = if username.is_empty() { "财务人员" } else { &username };

    match salary_service::calculate(db, dto.year, dto.month, 0, operator_id, operator_name).await {
        Ok(count) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(count, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn update(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<SalaryUpdateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let mut dto = form_data.0;

    dto.updated_by = Some(get_current_user_id(&req));

    match salary_service::update(db, dto).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success("调整成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn approve(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "工资记录ID不能为空", "local"));
    }

    match salary_service::approve(db, item.id.unwrap()).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success("审核成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn batch_approve(
    state: web::Data<AppState>,
    form_data: web::Json<SalaryBatchDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match salary_service::batch_approve(db, dto.ids).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success("批量审核成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn pay(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "工资记录ID不能为空", "local"));
    }

    match salary_service::pay(db, item.id.unwrap()).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success("发放成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn batch_pay(
    state: web::Data<AppState>,
    form_data: web::Json<SalaryBatchDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match salary_service::batch_pay(db, dto.ids).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success("批量发放成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn summary(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<SummaryQuery>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;
    let user_id = get_current_user_id(&req);

    match salary_service::get_summary(db, item.year, item.month, user_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ===== 底薪配置接口 =====

#[derive(Deserialize)]
pub struct ConfigQuery {
    pub employee_id: Option<i64>,
    pub year: Option<i32>,
}

pub async fn config_list(
    state: web::Data<AppState>,
    query: web::Query<ConfigQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    match salary_service::get_config_list(db, q.employee_id, q.year).await {
        Ok(list) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(list, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[derive(Deserialize)]
pub struct ConfigUpsertDTO {
    pub employee_id: i64,
    pub year: i32,
    pub month: Option<i32>,
    pub base_salary: f64,
    pub position_allowance: Option<f64>,
    pub performance_base: Option<f64>,
    pub performance_coefficient: Option<f64>,
}

pub async fn config_upsert(
    state: web::Data<AppState>,
    form_data: web::Json<ConfigUpsertDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;
    match salary_service::upsert_config(
        db, dto.employee_id, dto.year, dto.month,
        dto.base_salary, dto.position_allowance,
        dto.performance_base, dto.performance_coefficient,
    ).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn config_delete(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "配置ID不能为空", "local"));
    }
    match salary_service::delete_config(db, item.id.unwrap()).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success("删除成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ===== 核算日志接口 =====

#[derive(Deserialize)]
pub struct CalcLogQuery {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn calc_log_list(
    state: web::Data<AppState>,
    query: web::Query<CalcLogQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).max(1);
    match salary_service::get_calc_log_list(db, q.year, q.month, page, page_size).await {
        Ok((list, total)) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success_with_page(list, "local", page as u32, total as u32)),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ===== 工资确认/申诉接口 =====

pub async fn confirm(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<salary_service::SalaryConfirmDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    let (user_id, username) = get_current_user(&req);
    let user_name: &str = if username.is_empty() { "员工" } else { &username };

    match salary_service::submit_confirm(db, user_id, user_name, dto).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyConfirmQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    // V7-6: 申诉列表筛选条件（pending_confirm_list 使用）
    pub employee_id: Option<i64>,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub status: Option<i32>,
}

pub async fn my_confirm_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<MyConfirmQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let user_id = get_current_user_id(&req);
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20);

    match salary_service::get_my_confirms(db, user_id, page, page_size).await {
        Ok((list, total)) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success_with_page(list, "local", page as u32, total as u32)),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn pending_confirm_list(
    state: web::Data<AppState>,
    query: web::Query<MyConfirmQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20);

    // V7-6: 透传筛选条件到 service
    match salary_service::get_pending_confirms_filtered(
        db,
        salary_service::PendingConfirmQuery {
            employee_id: q.employee_id,
            year: q.year,
            month: q.month,
            status: q.status,
            page,
            page_size,
        },
    ).await {
        Ok((list, total)) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success_with_page(list, "local", page as u32, total as u32)),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn handle_confirm(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<salary_service::SalaryConfirmHandleDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    let (handler_id, username) = get_current_user(&req);
    let handler_name: &str = if username.is_empty() { "财务" } else { &username };

    match salary_service::handle_confirm(db, handler_id, handler_name, dto).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success("处理成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// V7-7: 导出工资单 CSV
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSalaryQuery {
    pub year: i32,
    pub month: i32,
    pub employee_ids: Option<String>, // 逗号分隔
}

pub async fn export_salary(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ExportSalaryQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let user_id = get_current_user_id(&req);

    let emp_ids: Option<Vec<i64>> = q.employee_ids.as_ref().map(|s| {
        s.split(',').filter_map(|x| x.trim().parse::<i64>().ok()).collect()
    });
    let emp_ids_ref = emp_ids.as_ref().map(|v| v.as_slice());

    match salary_export_service::export_salary_csv(db, q.year, q.month, emp_ids_ref, user_id).await {
        Ok(csv_bytes) => {
            let filename = format!("salary_{}-{}.csv", q.year, q.month);
            HttpResponse::Ok()
                .content_type("text/csv; charset=utf-8")
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                .body(csv_bytes)
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// V7-8: 导出个税申报 CSV
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTaxQuery {
    pub year: i32,
    pub month: i32,
}

pub async fn export_tax(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ExportTaxQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let user_id = get_current_user_id(&req);

    match salary_export_service::export_tax_csv(db, q.year, q.month, user_id).await {
        Ok(csv_bytes) => {
            let filename = format!("tax_{}-{}.csv", q.year, q.month);
            HttpResponse::Ok()
                .content_type("text/csv; charset=utf-8")
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                .body(csv_bytes)
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// P2-4: 导出工资单 xlsx
pub async fn export_salary_xlsx(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ExportSalaryQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let user_id = get_current_user_id(&req);

    let emp_ids: Option<Vec<i64>> = q.employee_ids.as_ref().map(|s| {
        s.split(',').filter_map(|x| x.trim().parse::<i64>().ok()).collect()
    });
    let emp_ids_ref = emp_ids.as_ref().map(|v| v.as_slice());

    match salary_export_service::export_salary_xlsx(db, q.year, q.month, emp_ids_ref, user_id).await {
        Ok(xlsx_bytes) => {
            let filename = format!("salary_{}-{}.xlsx", q.year, q.month);
            HttpResponse::Ok()
                .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                .body(xlsx_bytes)
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// P2-4: 导出个税申报 xlsx
pub async fn export_tax_xlsx(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ExportTaxQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let user_id = get_current_user_id(&req);

    match salary_export_service::export_tax_xlsx(db, q.year, q.month, user_id).await {
        Ok(xlsx_bytes) => {
            let filename = format!("tax_{}-{}.xlsx", q.year, q.month);
            HttpResponse::Ok()
                .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                .body(xlsx_bytes)
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ===== V8-1: 工资单审批流对接 =====

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalaryApprovalQuery {
    pub year: i32,
    pub month: i32,
}

/// 提交月度工资到审批流
pub async fn submit_approval(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<SalaryApprovalQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;

    let (operator_id, operator_name) = get_current_user(&req);

    match salary_service::submit_salary_approval(db, q.year, q.month, operator_id, &operator_name).await {
        Ok((success, failures)) => {
            let msg = if failures.is_empty() {
                format!("成功提交 {} 条工资审批", success)
            } else {
                format!("成功 {} 条，失败 {} 条：{}", success, failures.len(), failures.join("; "))
            };
            HttpResponse::Ok().content_type("application/json")
                .body(MetaResp::success(
                    serde_json::json!({ "success": success, "failures": failures, "message": msg }),
                    "local",
                ))
        }
        Err(e) => HttpResponse::Ok().content_type("application/json")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 同步工资审批状态
pub async fn sync_approval(
    state: web::Data<AppState>,
    query: web::Query<SalaryApprovalQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;

    match salary_service::sync_salary_approval_status(db, q.year, q.month).await {
        Ok((approved, rejected)) => {
            let msg = format!("同步完成：{} 条审批通过，{} 条审批驳回", approved, rejected);
            HttpResponse::Ok().content_type("application/json")
                .body(MetaResp::success(
                    serde_json::json!({ "approved": approved, "rejected": rejected, "message": msg }),
                    "local",
                ))
        }
        Err(e) => HttpResponse::Ok().content_type("application/json")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance/salary")
            .route("/list", web::get().to(list).wrap(require_permission("finance:salary:list")))
            .route("/detail", web::get().to(detail).wrap(require_permission("finance:salary:list")))
            .route("/calculate", web::post().to(calculate).wrap(require_permission("finance:salary:manage")))
            .route("/update", web::post().to(update).wrap(require_permission("finance:salary:manage")))
            .route("/approve", web::post().to(approve).wrap(require_permission("finance:salary:manage")))
            .route("/batch-approve", web::post().to(batch_approve).wrap(require_permission("finance:salary:manage")))
            .route("/pay", web::post().to(pay).wrap(require_permission("finance:salary:manage")))
            .route("/batch-pay", web::post().to(batch_pay).wrap(require_permission("finance:salary:manage")))
            .route("/summary", web::get().to(summary).wrap(require_permission("finance:salary:list")))
            .route("/config/list", web::get().to(config_list).wrap(require_permission("finance:salary:list")))
            .route("/config/upsert", web::post().to(config_upsert).wrap(require_permission("finance:salary:manage")))
            .route("/config/delete", web::post().to(config_delete).wrap(require_permission("finance:salary:manage")))
            .route("/calc-log/list", web::get().to(calc_log_list).wrap(require_permission("finance:salary:list")))
            .route("/confirm", web::post().to(confirm).wrap(require_permission("finance:salary:list")))
            .route("/confirm/my-list", web::get().to(my_confirm_list).wrap(require_permission("finance:salary:list")))
            .route("/confirm/pending-list", web::get().to(pending_confirm_list).wrap(require_permission("finance:salary:list")))
            .route("/confirm/handle", web::post().to(handle_confirm).wrap(require_permission("finance:salary:manage")))
            // V7-7/V7-8: 导出
            .route("/export-salary", web::get().to(export_salary).wrap(require_permission("finance:salary:list")))
            .route("/export-tax", web::get().to(export_tax).wrap(require_permission("finance:salary:list")))
            // P2-4: xlsx 真实 Excel 导出
            .route("/export-salary-xlsx", web::get().to(export_salary_xlsx).wrap(require_permission("finance:salary:list")))
            .route("/export-tax-xlsx", web::get().to(export_tax_xlsx).wrap(require_permission("finance:salary:list")))
            // V8-1: 工资单审批流对接
            .route("/submit-approval", web::post().to(submit_approval).wrap(require_permission("finance:salary:manage")))
            .route("/sync-approval", web::post().to(sync_approval).wrap(require_permission("finance:salary:manage")))
            // P2-2: 工资历史趋势分析（只读，复用 list 权限）
            .route("/trend/monthly", web::get().to(trend_monthly).wrap(require_permission("finance:salary:list")))
            .route("/trend/department", web::get().to(trend_department).wrap(require_permission("finance:salary:list")))
            .route("/trend/employee", web::get().to(trend_employee).wrap(require_permission("finance:salary:list")))
            .route("/trend/summary", web::get().to(trend_summary).wrap(require_permission("finance:salary:list"))),
    );
}

// ===== P2-2: 工资历史趋势分析 =====

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendLimitQuery {
    pub year_start: Option<i32>,
    pub year_end: Option<i32>,
    pub month_start: Option<i32>,
    pub month_end: Option<i32>,
    pub department_name: Option<String>,
    pub employee_id: Option<i64>,
    pub employee_name: Option<String>,
    /// 仅 employee 接口使用，TopN 限制
    pub limit: Option<i64>,
}

fn to_trend_query(q: &TrendLimitQuery) -> SalaryTrendQuery {
    SalaryTrendQuery {
        year_start: q.year_start,
        year_end: q.year_end,
        month_start: q.month_start,
        month_end: q.month_end,
        department_name: q.department_name.clone(),
        employee_id: q.employee_id,
        employee_name: q.employee_name.clone(),
    }
}

/// 月度趋势时间序列
pub async fn trend_monthly(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<TrendLimitQuery>,
) -> HttpResponse {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    let trend_q = to_trend_query(&query.0);

    match salary_service::get_trend_monthly(db, trend_q, user_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 部门维度聚合
pub async fn trend_department(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<TrendLimitQuery>,
) -> HttpResponse {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    let trend_q = to_trend_query(&query.0);

    match salary_service::get_trend_by_department(db, trend_q, user_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 员工排名
pub async fn trend_employee(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<TrendLimitQuery>,
) -> HttpResponse {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    let trend_q = to_trend_query(&query.0);
    let limit = query.limit;

    match salary_service::get_trend_by_employee(db, trend_q, user_id, limit).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 周期汇总
pub async fn trend_summary(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<TrendLimitQuery>,
) -> HttpResponse {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    let trend_q = to_trend_query(&query.0);

    match salary_service::get_trend_summary(db, trend_q, user_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}
