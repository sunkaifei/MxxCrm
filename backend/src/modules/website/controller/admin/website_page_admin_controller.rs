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
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::website::model::website_page::{PageSaveDTO, PageSaveRequest, PageUpdateRequest, ListQuery};
use crate::modules::website::service::{website_page_service};

/// 新增页面
pub async fn add(state: web::Data<AppState>, _req: HttpRequest, item: web::Json<PageSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    if payload.page_code.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "页面编码不能为空", "local")));
    }
    if payload.page_name.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "页面名称不能为空", "local")));
    }
    let form_data = PageSaveDTO::from(payload);
    let result = website_page_service::insert(db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("添加成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}

/// 批量删除页面
pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let result = website_page_service::batch_delete_by_ids(db, &ids_vec).await?;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// 修改页面
pub async fn update_by_id(state: web::Data<AppState>, _req: HttpRequest, id: web::Path<i64>, item: web::Json<PageUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    let mut form_data = PageSaveDTO::from(payload);
    form_data.id = Some(id.into_inner());
    let result = website_page_service::update_by_id(db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("修改成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "修改失败", "local")))
    }
}

/// 页面详情
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = website_page_service::get_by_detail(db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

/// 页面分页列表
pub async fn get_by_page(state: web::Data<AppState>, _req: HttpRequest, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    website_page_service::get_by_page(db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

/// 按 page_code 获取页面内容
pub async fn get_by_code(state: web::Data<AppState>, code: web::Path<String>) -> Result<HttpResponse> {
    let db = &state.db;
    let code = Some(code.into_inner());
    let result = website_page_service::get_by_code(db, &code).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

// ==================== 路由注册（单点维护）====================

/// 注册网站页面模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/website/page")
            // POST /website/page/add - 新增页面
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("website:page:add")),
            )
            // DELETE /website/page/batch_delete - 批量删除页面
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("website:page:delete")),
            )
            // PUT /website/page/update/{id} - 修改页面
            .route(
                "/update/{id}",
                web::put()
                    .to(update_by_id)
                    .wrap(require_permission("website:page:update")),
            )
            // GET /website/page/detail/{id} - 页面详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("website:page:view")),
            )
            // GET /website/page/list - 页面分页
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("website:page:list")),
            )
            // GET /website/page/code/{code} - 按code获取页面
            .route(
                "/code/{code}",
                web::get()
                    .to(get_by_code)
                    .wrap(require_permission("website:page:list")),
            ),
    );
}
