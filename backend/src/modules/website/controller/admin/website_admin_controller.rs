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
use crate::core::web::entity::common::{BathDeleteIdRequest, BathIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::website::model::website::{ListQuery, SiteModel, SiteSaveDTO, SiteSaveRequest, SiteUpdateRequest, UpdateDefaultDTO, UpdateDefaultRequest, UpdateStatusDTO, UpdateStatusRequest};
use crate::modules::website::service::website_service;
use crate::modules::website::service::website_service::can_remove_default_site;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn add_site(state: web::Data<AppState>, req: HttpRequest, item: web::Json<SiteSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let mut  payload = item.0;
    //获取当前用户id
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    payload.user_id = admin_token.id;
    
    if payload.site_name.as_ref().map_or(true, |s| s.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"网站名称不能为空", "local")))
    }
    let name = SiteModel::find_by_name_count(&db, &payload.site_name, &payload.user_id, &None).await?;
    if name > 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"网站名称已存在", "local")))
    }

    if payload.template_id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"网站模板id不能为空", "local")))
    }

    if payload.domain.as_ref().map_or(true, |s| s.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"二级域名不能为空", "local")))
    }
    let domain = SiteModel::find_by_domain_count(&db, &payload.domain, &None).await?;
    if domain > 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"该二级域名已被占用", "local")))
    }
    
    if payload.bind_domain.as_ref().map_or(true, |s| s.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"绑定的域名不能为空", "local")))
    }

    let bind_domain = SiteModel::find_by_bind_domain_count(&db, &payload.bind_domain, &None).await?;
    if bind_domain > 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"该绑定域名已被占用", "local")))
    }
    let mut form_data = SiteSaveDTO::from(payload);
    form_data.user_id = admin_token.id;
    let result = website_service::save_site(&db, &form_data).await?;

    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success("添加成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"添加失败", "local")))
    }
}

/// 批量删除网站
pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"删除的ID不能为空", "local")))
        }

        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let result = website_service::batch_delete_by_ids(&db, ids).await?;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"删除的ID不能为空", "local")))
    }
}

pub async fn update_site(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, item: web::Json<SiteUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let mut payload = item.0;
    let site_id = id.into_inner();
    //获取当前用户id
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    payload.user_id = admin_token.id;

    if payload.site_name.as_ref().map_or(true, |s| s.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"网站名称不能为空", "local")))
    }
    let name = SiteModel::find_by_name_count(&db, &payload.site_name, &payload.user_id, &Some(site_id)).await?;
    if name > 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"网站名称已存在", "local")))
    }

    if payload.template_id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"网站模板名称不能为空", "local")))
    }

    if payload.domain.as_ref().map_or(true, |s| s.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"二级域名不能为空", "local")))
    }
    let domain = SiteModel::find_by_domain_count(&db, &payload.domain, &Some(site_id)).await?;
    if domain > 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"该二级域名已被占用", "local")))
    }

    if payload.bind_domain.as_ref().map_or(true, |s| s.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"绑定的域名不能为空", "local")))
    }

    let bind_domain = SiteModel::find_by_bind_domain_count(&db, &payload.bind_domain, &Some(site_id)).await?;
    if bind_domain > 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"该绑定域名已被占用", "local")))
    }

    let mut form_data = SiteSaveDTO::from(payload);
    form_data.user_id = admin_token.id;
    let result = website_service::update_by_id(&db, &form_data).await?;

    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success("添加成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"添加失败", "local")))
    }
}

pub async fn update_by_status(state: web::Data<AppState>, req: HttpRequest, item: web::Json<UpdateStatusRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let mut  payload = item.0;
    //获取当前用户id
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();

    payload.user_id = admin_token.id;

    if payload.id.unwrap_or(0) == 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"删除的ID不能为空", "local")))
    }
    
    if !website_service::find_by_id_unique(&db, &payload.id, &payload.user_id).await? {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"该网站不存在或者无权限", "local")))
    }
    
    let status_dto = UpdateStatusDTO::from(payload);
    
    let site_data = SiteModel::update_status_by_id(&db, status_dto).await;
    if site_data.is_err() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"更新失败", "local")))
    }
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success("修改成功", "local")))
}

pub async fn update_by_default(state: web::Data<AppState>, req: HttpRequest, item: web::Json<UpdateDefaultRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.0;
    let mut form_data = UpdateDefaultDTO::from(payload);

    // 获取当前用户ID
    let admin_token: JWTToken = get_user(&req).unwrap_or_default();
    form_data.user_id = admin_token.id;

    // 验证ID是否有效
    if form_data.id.unwrap_or(0) == 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"ID不能为空", "local")))
    }

    // 检查网站是否存在
    if !website_service::find_by_id_unique(&db, &form_data.id, &form_data.user_id).await? {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"该网站不存在或者无权限", "local")))
    }

    // 检查是否需要保留一个默认站点
    if form_data.is_default.unwrap_or_default() == 0 {
        if !can_remove_default_site(&db, &form_data.user_id, &form_data.id).await? {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"必须保留一个默认站点方便查看数据", "local")))
        }
    }

    // 更新默认站点
    let rows = website_service::update_by_default_id(&db, &form_data).await?;
    if rows == 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"更新失败", "local")))
    }

    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success("修改成功", "local")))
}

pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400,"ID不能为空", "local")))
    }
    let result = website_service::find_by_id(&db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

pub async fn get_by_page(state: web::Data<AppState>, _req: HttpRequest, query: web::Query<ListQuery>,) -> Result<HttpResponse> {
    let db = &state.db;

    website_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

// ==================== 路由注册（单点维护）====================

/// 注册网站模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(website_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/site")
            // POST /site/add - 新建网站
            .route(
                "/add",
                web::post()
                    .to(add_site)
                    .wrap(require_permission("system:site:add")),
            )
            // DELETE /site/batch_delete - 批量删除网站
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("system:site:delete")),
            )
            // PUT /site/update/{id} - 修改网站
            .route(
                "/update/{id}",
                web::put()
                    .to(update_site)
                    .wrap(require_permission("system:site:update")),
            )
            // PUT /site/update_status - 修改网站状态
            .route("/update_status", web::put().to(update_by_status))
            // PUT /site/update_default - 修改默认网站
            .route("/update_default", web::put().to(update_by_default))
            // GET /site/detail/{id} - 网站详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("system:site:view")),
            )
            // GET /site/list - 网站列表
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("system:website:list")),
            ),
    );
}