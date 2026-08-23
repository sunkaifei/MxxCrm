//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 社保管理控制器

use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::core::web::permission_guard::require_permission;
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::InfoId;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::finance::service::insurance_service;

// ==================== 社保政策接口 ====================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyListQuery {
    pub city_code: Option<String>,
    pub year: Option<i32>,
}

pub async fn policy_list(
    state: web::Data<AppState>,
    query: web::Query<PolicyListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    match insurance_service::get_policy_list(db, q.city_code, q.year).await {
        Ok(list) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(list, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}
pub async fn policy_upsert(
    state: web::Data<AppState>,
    form_data: web::Json<insurance_service::InsurancePolicyDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;
    match insurance_service::upsert_policy(db, dto).await {
        Ok(id) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn policy_delete(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;
    if item.id.is_none() {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "政策ID不能为空", "local"));
    }
    match insurance_service::delete_policy(db, item.id.unwrap()).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success("删除成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ==================== 员工社保配置接口 ====================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeConfigQuery {
    pub employee_id: Option<i64>,
}

pub async fn employee_config_list(
    state: web::Data<AppState>,
    query: web::Query<EmployeeConfigQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    match insurance_service::get_all_employee_configs(db).await {
        Ok(list) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(list, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn employee_config_upsert(
    state: web::Data<AppState>,
    form_data: web::Json<insurance_service::EmployeeInsuranceConfigDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;
    match insurance_service::upsert_employee_config(db, dto).await {
        Ok(id) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ==================== 实时预览计算 ====================

pub async fn preview_calc(
    state: web::Data<AppState>,
    form_data: web::Json<insurance_service::PreviewCalcDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;
    match insurance_service::preview_calculation(db, dto).await {
        Ok(r) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(r, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ==================== 路由注册 ====================

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance/insurance")
            .route(
                "/policy/list",
                web::get()
                    .to(policy_list)
                    .wrap(require_permission("finance:insurance:list")),
            )
            .route(
                "/policy/upsert",
                web::post()
                    .to(policy_upsert)
                    .wrap(require_permission("finance:insurance:manage")),
            )
            .route(
                "/policy/delete",
                web::post()
                    .to(policy_delete)
                    .wrap(require_permission("finance:insurance:manage")),
            )
            .route(
                "/employee-config/list",
                web::get()
                    .to(employee_config_list)
                    .wrap(require_permission("finance:insurance:list")),
            )
            .route(
                "/employee-config/upsert",
                web::post()
                    .to(employee_config_upsert)
                    .wrap(require_permission("finance:insurance:manage")),
            )
            .route(
                "/preview-calc",
                web::post()
                    .to(preview_calc)
                    .wrap(require_permission("finance:insurance:list")),
            ),
    );
}
