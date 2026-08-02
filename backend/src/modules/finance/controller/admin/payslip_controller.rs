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
use serde::Deserialize;

use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::InfoId;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::finance::service::payslip_service;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayslipListQuery {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub employee_id: Option<i64>,
    pub send_status: Option<i32>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<PayslipListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).max(1);

    match payslip_service::get_payslip_list(
        db,
        q.year,
        q.month,
        q.employee_id,
        q.send_status,
        page,
        page_size,
    ).await {
        Ok((list, total)) => {
            HttpResponse::Ok().content_type("application/msgpack")
                .body(MetaResp::success_with_page(list, "local", page as u32, total as u32))
        }
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateDTO {
    pub year: i32,
    pub month: i32,
}

pub async fn generate(
    state: web::Data<AppState>,
    form_data: web::Json<GenerateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match payslip_service::generate_payslips(db, dto.year, dto.month).await {
        Ok(count) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(count, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendDTO {
    pub id: i64,
    pub channels: Vec<String>,
}

pub async fn send(
    state: web::Data<AppState>,
    form_data: web::Json<SendDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match payslip_service::send_payslip(db, dto.id, dto.channels).await {
        Ok(_) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success("发送成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSendDTO {
    pub ids: Vec<i64>,
    pub channels: Vec<String>,
}

pub async fn batch_send(
    state: web::Data<AppState>,
    form_data: web::Json<BatchSendDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match payslip_service::batch_send_payslips(db, dto.ids, dto.channels).await {
        Ok(count) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(count, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn mark_read(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "工资条ID不能为空", "local"));
    }

    match payslip_service::mark_read(db, item.id.unwrap()).await {
        Ok(_) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success("标记已读成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsQuery {
    pub year: i32,
    pub month: i32,
}

pub async fn statistics(
    state: web::Data<AppState>,
    query: web::Query<StatisticsQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;

    match payslip_service::get_read_statistics(db, q.year, q.month).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ===== V8-4: 工资条密码与撤回 =====

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPasswordDTO {
    pub payslip_id: i64,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyPasswordDTO {
    pub payslip_id: i64,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawDTO {
    pub payslip_id: i64,
    pub reason: String,
}

pub async fn set_password(
    state: web::Data<AppState>,
    dto: web::Json<SetPasswordDTO>,
) -> HttpResponse {
    let db = &state.db;
    match payslip_service::set_payslip_password(db, dto.payslip_id, &dto.password).await {
        Ok(_) => HttpResponse::Ok().content_type("application/json")
            .body(MetaResp::success(serde_json::json!({}), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/json")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn clear_password(
    state: web::Data<AppState>,
    dto: web::Json<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let payslip_id = dto.id.unwrap_or(0);
    match payslip_service::clear_payslip_password(db, payslip_id).await {
        Ok(_) => HttpResponse::Ok().content_type("application/json")
            .body(MetaResp::success(serde_json::json!({}), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/json")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn verify_password(
    state: web::Data<AppState>,
    dto: web::Json<VerifyPasswordDTO>,
) -> HttpResponse {
    let db = &state.db;
    match payslip_service::verify_payslip_password(db, dto.payslip_id, &dto.password).await {
        Ok(ok) => HttpResponse::Ok().content_type("application/json")
            .body(MetaResp::success(serde_json::json!({ "verified": ok }), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/json")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn withdraw(
    state: web::Data<AppState>,
    req: HttpRequest,
    dto: web::Json<WithdrawDTO>,
) -> HttpResponse {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let withdrawn_by = jwt_token.id.unwrap_or(0);
    match payslip_service::withdraw_payslip(db, dto.payslip_id, withdrawn_by, &dto.reason).await {
        Ok(_) => HttpResponse::Ok().content_type("application/json")
            .body(MetaResp::success(serde_json::json!({}), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/json")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance/payslip")
            .route("/list", web::get().to(list).wrap(require_permission("finance:payslip:list")))
            .route("/generate", web::post().to(generate).wrap(require_permission("finance:payslip:manage")))
            .route("/send", web::post().to(send).wrap(require_permission("finance:payslip:manage")))
            .route("/batch-send", web::post().to(batch_send).wrap(require_permission("finance:payslip:manage")))
            .route("/mark-read", web::post().to(mark_read).wrap(require_permission("finance:payslip:list")))
            .route("/statistics", web::get().to(statistics).wrap(require_permission("finance:payslip:list")))
            // V8-4: 密码与撤回
            .route("/set-password", web::post().to(set_password).wrap(require_permission("finance:payslip:manage")))
            .route("/clear-password", web::post().to(clear_password).wrap(require_permission("finance:payslip:manage")))
            .route("/verify-password", web::post().to(verify_password).wrap(require_permission("finance:payslip:list")))
            .route("/withdraw", web::post().to(withdraw).wrap(require_permission("finance:payslip:manage"))),
    );
}
