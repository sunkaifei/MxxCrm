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

use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user;
use crate::core::web::entity::common::InfoId;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::finance::model::payment::{
    PaymentQuery, PaymentApplyDTO, PaymentApproveDTO, PaymentConfirmDTO, PaymentCancelDTO,
};
use crate::modules::finance::service::payment_service;

pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<PaymentQuery>,
) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let page = query.page.unwrap_or(1) as u32;

    match payment_service::get_list(db, query).await {
        Ok((list, total)) => {
            HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::success_with_page(list, "local", page, total as u32))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn detail(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "付款记录ID不能为空", "local"));
    }

    match payment_service::get_detail(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn apply(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<PaymentApplyDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    let (applicant_id, applicant_name) = get_current_user(&req);

    match payment_service::apply(db, dto, applicant_id, applicant_name).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn approve(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<PaymentApproveDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    let (approver_id, approver_name) = get_current_user(&req);

    match payment_service::approve(db, dto, approver_id, approver_name).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success("审批成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn confirm(
    state: web::Data<AppState>,
    form_data: web::Json<PaymentConfirmDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match payment_service::confirm(db, dto).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success("确认付款成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub async fn cancel(
    state: web::Data<AppState>,
    form_data: web::Json<PaymentCancelDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match payment_service::cancel(db, dto.id, dto.remark).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success("取消成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance/payment")
            .route("/list", web::get().to(list).wrap(require_permission("finance:payment:list")))
            .route("/detail", web::get().to(detail).wrap(require_permission("finance:payment:list")))
            .route("/apply", web::post().to(apply).wrap(require_permission("finance:payment:manage")))
            .route("/approve", web::post().to(approve).wrap(require_permission("finance:payment:manage")))
            .route("/confirm", web::post().to(confirm).wrap(require_permission("finance:payment:manage")))
            .route("/cancel", web::post().to(cancel).wrap(require_permission("finance:payment:manage"))),
    );
}
