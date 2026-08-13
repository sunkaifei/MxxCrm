//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{web, HttpRequest, HttpResponse, Result};
use crate::core::web::permission_guard::require_permission;

use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user;
use crate::core::web::response::{MetaResp, ResultPage, MPACK};
use crate::modules::crm::model::work_log::WorkLogCreateDTO;
use crate::modules::crm::service::work_log_service;
use crate::modules::finance::model::payment_record::{PaymentRecordSaveRequest, PaymentRecordQuery};
use crate::modules::finance::service::payment_record_service;

pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<PaymentRecordQuery>
) -> Result<HttpResponse> {
    let db = &state.db;
    let query_inner = query.into_inner();
    let page = query_inner.page.unwrap_or(1);
    let result = payment_record_service::get_list(db, query_inner).await;

    match result {
        Ok((list, total)) => {
            let page_data = ResultPage::new(list, total, page, 20);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local")))
        }
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn detail(
    state: web::Data<AppState>,
    path: web::Path<i64>
) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();

    let result = payment_record_service::get_by_id(db, id).await;

    match result {
        Ok(Some(data)) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Ok(None) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "记录不存在", "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn create(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<PaymentRecordSaveRequest>
) -> Result<HttpResponse> {
    let db = &state.db;
    let req_data = item.into_inner();

    let result = payment_record_service::insert(db, req_data).await;

    match result {
        Ok(data) => {
            // 工作日志埋点（回款登记），不影响主业务
            let (operator_id, username) = get_current_user(&req);
            if operator_id > 0 {
                let log_dto = WorkLogCreateDTO {
                    user_id: operator_id,
                    user_name: Some(username),
                    action_type: Some(3),
                    action_name: Some("回款登记".to_string()),
                    business_type: Some("payment".to_string()),
                    business_id: Some(data.id),
                    business_title: data.order_id.clone(),
                    description: data.remark.clone(),
                    result: Some(1),
                    work_date: Some(chrono::Local::now().naive_local().date()),
                };
                let _ = work_log_service::insert(db, &log_dto).await;
            }
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")))
        }
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    item: web::Json<PaymentRecordSaveRequest>
) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();

    let result = payment_record_service::update(db, id, item.into_inner()).await;

    match result {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>
) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();

    let result = payment_record_service::delete(db, id).await;

    match result {
        Ok(true) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success("删除成功", "local"))),
        Ok(false) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "记录不存在", "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance/payment-record")
            .route("/list", web::get().to(list).wrap(require_permission("finance:payment-record:list")))
            .route("/detail/{id}", web::get().to(detail).wrap(require_permission("finance:payment-record:list")))
            .route("/create", web::post().to(create).wrap(require_permission("finance:payment-record:save")))
            .route("/update/{id}", web::put().to(update).wrap(require_permission("finance:payment-record:update")))
            .route("/delete/{id}", web::delete().to(delete).wrap(require_permission("finance:payment-record:delete"))),
    );
}
