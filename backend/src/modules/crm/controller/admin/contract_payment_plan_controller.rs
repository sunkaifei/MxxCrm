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
use actix_web::{delete, get, post, web, HttpResponse};
use actix_web_grants::protect;

use crate::core::web::response::MetaResp;
use crate::modules::crm::model::contract_payment_plan::PaymentPlanSaveRequest;
use crate::modules::crm::service::contract_payment_plan_service;

#[derive(serde::Deserialize)]
#[derive(Debug)]
pub struct ContractIdQuery {
    #[serde(rename = "contractId")]
    pub contract_id: Option<i64>,
}

/// 查询合同回款计划列表
#[get("/contract/payment-plan/list")]
#[protect("crm:contract:list")]
pub async fn payment_plan_list(state: web::Data<AppState>, query: web::Query<ContractIdQuery>) -> HttpResponse {
    let db = &state.db;
    let contract_id = match query.contract_id {
        Some(id) => id,
        None => return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "合同ID不能为空", "local")),
    };

    match contract_payment_plan_service::list(&db, contract_id).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 批量保存合同回款计划
#[post("/contract/payment-plan/save")]
#[protect("crm:contract:save")]
pub async fn payment_plan_save(state: web::Data<AppState>, form_data: web::Json<PaymentPlanSaveRequest>) -> HttpResponse {
    let db = &state.db;
    let result = contract_payment_plan_service::save(&db, &form_data.0).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

/// 删除合同下所有回款计划
#[delete("/contract/payment-plan/delete")]
#[protect("crm:contract:delete")]
pub async fn payment_plan_delete(state: web::Data<AppState>, query: web::Query<ContractIdQuery>) -> HttpResponse {
    let db = &state.db;
    let contract_id = match query.contract_id {
        Some(id) => id,
        None => return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "合同ID不能为空", "local")),
    };

    match contract_payment_plan_service::delete(&db, contract_id).await {
        Ok(count) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(count, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}
