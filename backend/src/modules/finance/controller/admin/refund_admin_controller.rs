//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{get, post, put, delete, web, HttpResponse, Result};
use crate::core::kit::global::AppState;
use crate::core::web::response::{MetaResp, ResultPage};
use crate::modules::finance::model::refund_record::{RefundRecordSaveRequest, RefundRecordQuery};
use crate::modules::finance::service::refund_record_service;

#[get("/refund/list")]
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<RefundRecordQuery>
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = refund_record_service::get_list(db, query.into_inner()).await;
    
    match result {
        Ok((list, total)) => {
            let page_data = ResultPage::new(list, total, 1, 20);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local")))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/refund/detail/{id}")]
pub async fn detail(
    state: web::Data<AppState>,
    path: web::Path<i64>
) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();
    
    let result = refund_record_service::get_by_id(db, id).await;
    
    match result {
        Ok(Some(data)) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Ok(None) => Ok(HttpResponse::NotFound().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "记录不存在", "local"))),
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[post("/refund/create")]
pub async fn create(
    state: web::Data<AppState>,
    item: web::Json<RefundRecordSaveRequest>
) -> Result<HttpResponse> {
    let db = &state.db;
    
    let result = refund_record_service::insert(db, item.into_inner()).await;
    
    match result {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[put("/refund/update/{id}")]
pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    item: web::Json<RefundRecordSaveRequest>
) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();
    
    let result = refund_record_service::update(db, id, item.into_inner()).await;
    
    match result {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[delete("/refund/delete/{id}")]
pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>
) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();
    
    let result = refund_record_service::delete(db, id).await;
    
    match result {
        Ok(true) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(200, "删除成功", "local"))),
        Ok(false) => Ok(HttpResponse::NotFound().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "记录不存在", "local"))),
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list);
    cfg.service(detail);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
}
