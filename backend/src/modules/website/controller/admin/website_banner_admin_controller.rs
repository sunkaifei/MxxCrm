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
use crate::modules::website::model::website_banner::{BannerSaveDTO, BannerSaveRequest, BannerUpdateRequest, ListQuery};
use crate::modules::website::service::{website_banner_service};

/// 新增Banner
pub async fn add(state: web::Data<AppState>, _req: HttpRequest, item: web::Json<BannerSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    if payload.title.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "标题不能为空", "local")));
    }
    if payload.image_url.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "图片地址不能为空", "local")));
    }
    let form_data = BannerSaveDTO::from(payload);
    let result = website_banner_service::insert(db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("添加成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}

/// 批量删除Banner
pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let result = website_banner_service::batch_delete_by_ids(db, &ids_vec).await?;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// 修改Banner
pub async fn update_by_id(state: web::Data<AppState>, _req: HttpRequest, id: web::Path<i64>, item: web::Json<BannerUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    let mut form_data = BannerSaveDTO::from(payload);
    form_data.id = Some(id.into_inner());
    let result = website_banner_service::update_by_id(db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("修改成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "修改失败", "local")))
    }
}

/// Banner详情
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = website_banner_service::get_by_detail(db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

/// Banner分页列表
pub async fn get_by_page(state: web::Data<AppState>, _req: HttpRequest, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    website_banner_service::get_by_page(db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

/// 获取指定位置的Banner列表
pub async fn get_by_position(state: web::Data<AppState>, position: web::Path<String>) -> Result<HttpResponse> {
    let db = &state.db;
    let position = Some(position.into_inner());
    let result = website_banner_service::get_by_position(db, &position).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

// ==================== 路由注册（单点维护）====================

/// 注册网站Banner模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/website/banner")
            // POST /website/banner/add - 新增Banner
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("website:banner:add")),
            )
            // DELETE /website/banner/batch_delete - 批量删除Banner
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("website:banner:delete")),
            )
            // PUT /website/banner/update/{id} - 修改Banner
            .route(
                "/update/{id}",
                web::put()
                    .to(update_by_id)
                    .wrap(require_permission("website:banner:update")),
            )
            // GET /website/banner/detail/{id} - Banner详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("website:banner:view")),
            )
            // GET /website/banner/list - Banner分页
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("website:banner:list")),
            )
            // GET /website/banner/position/{position} - 获取指定位置Banner
            .route(
                "/position/{position}",
                web::get()
                    .to(get_by_position)
                    .wrap(require_permission("website:banner:list")),
            ),
    );
}
