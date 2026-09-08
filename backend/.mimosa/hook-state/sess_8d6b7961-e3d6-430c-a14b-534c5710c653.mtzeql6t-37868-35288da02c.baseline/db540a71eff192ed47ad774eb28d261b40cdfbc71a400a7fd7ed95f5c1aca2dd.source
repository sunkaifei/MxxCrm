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
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::salary_band::{ListQuery, SalaryBandSaveRequest, SalaryBandUpdateRequest};
use crate::modules::system::service::salary_band_service;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn save_band(state: web::Data<AppState>, _req: HttpRequest, item: web::Json<SalaryBandSaveRequest>) -> HttpResponse {
    let db = &state.db;
    if item.post_id.is_none() || item.post_id == Some(0) {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "岗位不能为空", "local"));
    }
    let Some(min_salary) = item.min_salary else {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "带宽下限不能为空", "local"));
    };
    let Some(max_salary) = item.max_salary else {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "带宽上限不能为空", "local"));
    };
    if max_salary < min_salary {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "带宽上限不能小于下限", "local"));
    }
    match salary_band_service::save(&db, &item.0).await {
        Ok(v) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::success(v, "local")),
        Err(err) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")),
    }
}

pub async fn bath_delete_band(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local"))
        } else {
            match salary_band_service::batch_delete_by_ids(&db, &ids_vec).await {
                Ok(v) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::success(v, "local")),
                Err(err) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")),
            }
        }
    } else {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local"))
    }
}

pub async fn update_band(state: web::Data<AppState>, _req: HttpRequest, id: web::Path<i64>, item: web::Json<SalaryBandUpdateRequest>) -> HttpResponse {
    let db = &state.db;
    let mut data = item.into_inner();
    let the_id = id.into_inner();
    if data.post_id.is_none() || data.post_id == Some(0) {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "岗位不能为空", "local"));
    }
    if let (Some(min_salary), Some(max_salary)) = (data.min_salary, data.max_salary) {
        if max_salary < min_salary {
            return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "带宽上限不能小于下限", "local"));
        }
    }
    data.id = Some(the_id);
    match salary_band_service::update_by_id(&db, &data).await {
        Ok(v) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::success(v, "local")),
        Err(err) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")),
    }
}

pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    match salary_band_service::get_by_detail(&db, &item.id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(err) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local"))),
    }
}

pub async fn get_by_page(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    salary_band_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

/// 按岗位ID查询启用的带宽（供入职定薪参照；内嵌岗位管理页/审批详情使用）
pub async fn get_band_by_post(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    match salary_band_service::get_band_by_post(&db, item.id.unwrap_or_default()).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(err) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册岗位薪资带宽模块所有路由
///
/// 带宽入口内嵌岗位管理页，权限码复用 system:post:*（避免额外菜单种子数据）。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(salary_band_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/salary-band")
            .route(
                "/save",
                web::post()
                    .to(save_band)
                    .wrap(require_permission("system:post:save")),
            )
            .route(
                "/bath_delete",
                web::delete()
                    .to(bath_delete_band)
                    .wrap(require_permission("system:post:delete")),
            )
            .route(
                "/update/{id}",
                web::put()
                    .to(update_band)
                    .wrap(require_permission("system:post:update")),
            )
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("system:post:view")),
            )
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("system:post:list")),
            )
            // 按岗位查带宽：与审批详情共用，权限同岗位列表
            .route(
                "/by-post/{id}",
                web::get()
                    .to(get_band_by_post)
                    .wrap(require_permission("system:post:list")),
            ),
    );
}
