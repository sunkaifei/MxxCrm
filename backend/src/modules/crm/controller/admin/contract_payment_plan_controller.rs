//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::permission_guard::require_permission;
use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::crm::model::contract_payment_plan::{PaymentPlanListQuery, PaymentPlanSaveRequest};
use crate::modules::crm::service::contract_payment_plan_service;

#[derive(serde::Deserialize)]
#[derive(Debug)]
pub struct ContractIdQuery {
    #[serde(rename = "contractId")]
    pub contract_id: Option<i64>,
}

/// 分页查询回款计划列表
pub async fn payment_plan_page_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PaymentPlanListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();

    match contract_payment_plan_service::page_list(&db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 查询合同回款计划列表
pub async fn payment_plan_list(state: web::Data<AppState>, query: web::Query<ContractIdQuery>) -> HttpResponse {
    let db = &state.db;
    let contract_id = match query.contract_id {
        Some(id) => id,
        None => return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "合同ID不能为空", "local")),
    };

    match contract_payment_plan_service::list(&db, contract_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 批量保存合同回款计划
pub async fn payment_plan_save(state: web::Data<AppState>, form_data: web::Json<PaymentPlanSaveRequest>) -> HttpResponse {
    let db = &state.db;
    let result = contract_payment_plan_service::save(&db, &form_data.0).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

/// 删除合同下所有回款计划
pub async fn payment_plan_delete(state: web::Data<AppState>, query: web::Query<ContractIdQuery>) -> HttpResponse {
    let db = &state.db;
    let contract_id = match query.contract_id {
        Some(id) => id,
        None => return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "合同ID不能为空", "local")),
    };

    match contract_payment_plan_service::delete(&db, contract_id).await {
        Ok(count) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(count, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册合同回款计划模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(contract_payment_plan_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payment-plan")
            // GET /payment-plan/page-list - 分页查询回款计划列表
            .route(
                "/page-list",
                web::get()
                    .to(payment_plan_page_list)
                    .wrap(require_permission("crm:contract:list")),
            )
            // GET /payment-plan/list - 查询合同回款计划列表
            .route(
                "/list",
                web::get()
                    .to(payment_plan_list)
                    .wrap(require_permission("crm:contract:list")),
            )
            // POST /payment-plan/save - 批量保存合同回款计划
            .route(
                "/save",
                web::post()
                    .to(payment_plan_save)
                    .wrap(require_permission("crm:contract:save")),
            )
            // DELETE /payment-plan/delete - 删除合同下所有回款计划
            .route(
                "/delete",
                web::delete()
                    .to(payment_plan_delete)
                    .wrap(require_permission("crm:contract:delete")),
            ),
    );
}
