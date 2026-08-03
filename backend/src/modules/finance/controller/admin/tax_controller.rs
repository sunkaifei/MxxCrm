//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 个税管理控制器

use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::core::web::permission_guard::require_permission;
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::InfoId;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::finance::service::tax_service;

// ==================== 税率表接口 ====================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateListQuery {
    pub tax_type: Option<i32>,
}

pub async fn rate_list(
    state: web::Data<AppState>,
    query: web::Query<RateListQuery>,
) -> HttpResponse {
    let db = &state.db;
    match tax_service::get_tax_rate_list(db, query.tax_type).await {
        Ok(list) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(list, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn rate_upsert(
    state: web::Data<AppState>,
    form_data: web::Json<tax_service::TaxRateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;
    match tax_service::upsert_tax_rate(db, dto).await {
        Ok(id) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn rate_delete(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;
    if item.id.is_none() {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "税率ID不能为空", "local"));
    }
    match tax_service::delete_tax_rate(db, item.id.unwrap()).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success("删除成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ==================== 员工个税配置接口 ====================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeConfigQuery {
    pub employee_id: i64,
    pub year: i32,
}

/// 员工个税配置列表查询参数（支持可选参数，用于列表视图）
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeConfigListQuery {
    pub employee_id: Option<i64>,
    pub year: Option<i32>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn employee_config_list(
    state: web::Data<AppState>,
    query: web::Query<EmployeeConfigListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    // 如果传了 employee_id 和 year，返回单个员工的配置；否则返回所有员工配置列表
    if let (Some(emp_id), Some(year)) = (q.employee_id, q.year) {
        match tax_service::get_employee_tax_config(db, emp_id, year).await {
            Ok(data) => HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::success(vec![data], "local")),
            Err(e) => HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &e, "local")),
        }
    } else {
        match tax_service::get_employee_tax_config_list(db, q.year).await {
            Ok(list) => HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::success(list, "local")),
            Err(e) => HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &e, "local")),
        }
    }
}

pub async fn employee_config_upsert(
    state: web::Data<AppState>,
    form_data: web::Json<tax_service::EmployeeTaxConfigDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;
    match tax_service::upsert_employee_tax_config(db, dto).await {
        Ok(id) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ==================== 个税明细接口 ====================

pub async fn detail_list(
    state: web::Data<AppState>,
    query: web::Query<EmployeeConfigQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    match tax_service::get_tax_detail_list(db, q.employee_id, q.year).await {
        Ok(list) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(list, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ==================== 年终奖计税接口 ====================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnualBonusDTO {
    pub bonus_amount: f64,
}

pub async fn annual_bonus_calculate(
    state: web::Data<AppState>,
    form_data: web::Json<AnnualBonusDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;
    match tax_service::calculate_annual_bonus_tax(db, dto.bonus_amount).await {
        Ok(tax) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(tax, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ==================== 路由注册 ====================

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance/tax")
            .route(
                "/rate/list",
                web::get()
                    .to(rate_list)
                    .wrap(require_permission("finance:tax:list")),
            )
            .route(
                "/rate/upsert",
                web::post()
                    .to(rate_upsert)
                    .wrap(require_permission("finance:tax:manage")),
            )
            .route(
                "/rate/delete",
                web::post()
                    .to(rate_delete)
                    .wrap(require_permission("finance:tax:manage")),
            )
            .route(
                "/employee-config/list",
                web::get()
                    .to(employee_config_list)
                    .wrap(require_permission("finance:tax:list")),
            )
            .route(
                "/employee-config/upsert",
                web::post()
                    .to(employee_config_upsert)
                    .wrap(require_permission("finance:tax:manage")),
            )
            .route(
                "/detail/list",
                web::get()
                    .to(detail_list)
                    .wrap(require_permission("finance:tax:list")),
            )
            .route(
                "/annual-bonus-calculate",
                web::post()
                    .to(annual_bonus_calculate)
                    .wrap(require_permission("finance:tax:list")),
            ),
    );
}
