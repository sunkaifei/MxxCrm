//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{get, post, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;
use serde::Deserialize;

use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::InfoId;
use crate::core::web::response::MetaResp;
use crate::modules::finance::model::salary::{
    SalaryQuery, SalaryCalculateDTO, SalaryUpdateDTO, SalaryBatchDTO,
};
use crate::modules::finance::service::salary_service;

#[derive(Deserialize)]
pub struct SummaryQuery {
    pub year: i32,
    pub month: i32,
}

#[get("/finance/salary/list")]
#[protect("finance:salary:list")]
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<SalaryQuery>,
) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let page = query.page.unwrap_or(1) as u32;

    match salary_service::get_list(db, query).await {
        Ok((list, total)) => {
            HttpResponse::Ok().content_type("application/msgpack")
                .body(MetaResp::success_with_page(list, "local", page, total as u32))
        }
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[get("/finance/salary/detail")]
#[protect("finance:salary:list")]
pub async fn detail(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "工资记录ID不能为空", "local"));
    }

    match salary_service::get_detail(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[post("/finance/salary/calculate")]
#[protect("finance:salary:manage")]
pub async fn calculate(
    state: web::Data<AppState>,
    form_data: web::Json<SalaryCalculateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match salary_service::calculate(db, dto.year, dto.month).await {
        Ok(count) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(count, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[post("/finance/salary/update")]
#[protect("finance:salary:manage")]
pub async fn update(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<SalaryUpdateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let mut dto = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    dto.updated_by = jwt_token.id;

    match salary_service::update(db, dto).await {
        Ok(_) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success("调整成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[post("/finance/salary/approve")]
#[protect("finance:salary:manage")]
pub async fn approve(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "工资记录ID不能为空", "local"));
    }

    match salary_service::approve(db, item.id.unwrap()).await {
        Ok(_) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success("审核成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[post("/finance/salary/batch-approve")]
#[protect("finance:salary:manage")]
pub async fn batch_approve(
    state: web::Data<AppState>,
    form_data: web::Json<SalaryBatchDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match salary_service::batch_approve(db, dto.ids).await {
        Ok(_) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success("批量审核成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[post("/finance/salary/pay")]
#[protect("finance:salary:manage")]
pub async fn pay(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "工资记录ID不能为空", "local"));
    }

    match salary_service::pay(db, item.id.unwrap()).await {
        Ok(_) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success("发放成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[post("/finance/salary/batch-pay")]
#[protect("finance:salary:manage")]
pub async fn batch_pay(
    state: web::Data<AppState>,
    form_data: web::Json<SalaryBatchDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match salary_service::batch_pay(db, dto.ids).await {
        Ok(_) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success("批量发放成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[get("/finance/salary/summary")]
#[protect("finance:salary:list")]
pub async fn summary(
    state: web::Data<AppState>,
    query: web::Query<SummaryQuery>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;

    match salary_service::get_summary(db, item.year, item.month).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}
