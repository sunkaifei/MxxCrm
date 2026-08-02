//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 工资项目自定义引擎控制器
//!

use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::finance::service::salary_item_service;

/// 删除参数
#[derive(Deserialize)]
pub struct DeleteQuery {
    pub id: i64,
}

/// 查询项值参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValuesQuery {
    pub salary_record_id: i64,
}

/// 项目列表
pub async fn list(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;

    match salary_item_service::get_item_list(db).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 新增/更新工资项目
pub async fn upsert(
    state: web::Data<AppState>,
    form_data: web::Json<salary_item_service::SalaryItemUpsertDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match salary_item_service::upsert_item(db, dto).await {
        Ok(id) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 删除工资项目
pub async fn delete(
    state: web::Data<AppState>,
    query: web::Query<DeleteQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;

    match salary_item_service::delete_item(db, q.id).await {
        Ok(_) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success("删除成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 查询某工资记录的自定义项值
pub async fn values(
    state: web::Data<AppState>,
    query: web::Query<ValuesQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;

    match salary_item_service::get_item_values(db, q.salary_record_id).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// 保存自定义项值
pub async fn save_values(
    state: web::Data<AppState>,
    form_data: web::Json<salary_item_service::SaveItemValuesDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    match salary_item_service::save_item_values(db, dto.salary_record_id, dto.values).await {
        Ok(_) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::success("保存成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance/salary-item")
            .route(
                "/list",
                web::get()
                    .to(list)
                    .wrap(require_permission("finance:salary-item:list")),
            )
            .route(
                "/upsert",
                web::post()
                    .to(upsert)
                    .wrap(require_permission("finance:salary-item:manage")),
            )
            .route(
                "/delete",
                web::post()
                    .to(delete)
                    .wrap(require_permission("finance:salary-item:manage")),
            )
            .route(
                "/values",
                web::get()
                    .to(values)
                    .wrap(require_permission("finance:salary-item:list")),
            )
            .route(
                "/values/save",
                web::post()
                    .to(save_values)
                    .wrap(require_permission("finance:salary-item:manage")),
            ),
    );
}
