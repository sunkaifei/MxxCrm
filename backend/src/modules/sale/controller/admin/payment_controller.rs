//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::sale::model::payment::{
    PaymentApplyRequest, PaymentListQuery, PaymentSaveRequest, PaymentUpdateRequest,
};
use crate::modules::sale::service::payment_service;

#[post("/sale/payment/save")]
#[protect("sale:payment:save")]
pub async fn payment_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PaymentSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = payment_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/sale/payment/update")]
#[protect("sale:payment:update")]
pub async fn payment_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PaymentUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "收款记录ID不能为空", "local")));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let result = payment_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/sale/payment/bath_delete")]
#[protect("sale:payment:delete")]
pub async fn bath_delete_payment(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到删除的收款记录ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = payment_service::batch_delete_by_ids(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

#[get("/sale/payment/info")]
#[protect("sale:payment:info")]
pub async fn payment_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "收款记录ID不能为空", "local"));
    }

    match payment_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[get("/sale/payment/list")]
#[protect("sale:payment:list")]
pub async fn payment_list(state: web::Data<AppState>, query: web::Query<PaymentListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    match payment_service::list(&db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 确认回款：status→2，设 confirm_time/confirm_by，联动订单 paid_amount
#[post("/sale/payment/confirm")]
#[protect("sale:payment:confirm")]
pub async fn payment_confirm(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let payment_id = match form_data.0.id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "回款ID不能为空", "local")),
    };

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();

    match payment_service::confirm(db, payment_id, user_id).await {
        Ok(id) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 驳回回款：status→3
#[post("/sale/payment/reject")]
#[protect("sale:payment:confirm")]
pub async fn payment_reject(
    state: web::Data<AppState>,
    form_data: web::Json<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let payment_id = match form_data.0.id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "回款ID不能为空", "local")),
    };

    match payment_service::reject(db, payment_id).await {
        Ok(id) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 核销：将回款金额分配到一个或多个回款计划
#[post("/sale/payment/application/apply")]
#[protect("sale:payment:confirm")]
pub async fn payment_apply(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<PaymentApplyRequest>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();

    match payment_service::apply(db, &dto, user_id).await {
        Ok(id) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 取消核销：回滚 payment 和 plan 金额，软删除核销记录
#[post("/sale/payment/application/cancel")]
#[protect("sale:payment:confirm")]
pub async fn payment_application_cancel(
    state: web::Data<AppState>,
    form_data: web::Json<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let application_id = match form_data.0.id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "核销记录ID不能为空", "local")),
    };

    match payment_service::cancel_apply(db, application_id).await {
        Ok(id) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 查询回款未核销金额及可核销计划列表
#[get("/sale/payment/unapplied")]
#[protect("sale:payment:list")]
pub async fn payment_unapplied(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let payment_id = match query.0.id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "回款ID不能为空", "local")),
    };

    match payment_service::get_unapplied(db, payment_id).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 查询回款的核销明细列表
#[get("/sale/payment/application/list")]
#[protect("sale:payment:list")]
pub async fn payment_application_list(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let payment_id = match query.0.id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "回款ID不能为空", "local")),
    };

    match payment_service::get_applications(db, payment_id).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}