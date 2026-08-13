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
use actix_web::{HttpResponse, web, HttpRequest};
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::config::{ConfigSaveDTO, ConfigSaveRequest, ConfigUpdateRequest, ListQuery};
use crate::modules::system::service::{admin_service, config_service};

pub async fn insert_config(state: web::Data<AppState>, req: HttpRequest, item: web::Json<ConfigSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.config_key.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置信息key不能为空", "local")));
    }
    if config_service::find_by_key_unique(&db, &item.config_key, &None).await.unwrap_or(false) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置信息key已存在", "local")));
    }
    if item.config_name.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置信息名称不能为空", "local")));
    }
    if config_service::find_by_name_unique(&db, &item.config_name, &None).await.unwrap_or(false) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置信息名称已存在", "local")));
    }
    //获取用户信息
    let admin = admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await?;
    let mut form_data = ConfigSaveDTO::from(item.0);
    form_data.create_by = admin.user_name.clone();
    form_data.update_by = admin.user_name;
    match config_service::insert(&db, &form_data).await {
        Ok(user_op) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(user_op, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        for id_opt in ids_vec.iter() {
            if let Some(id) = id_opt {
                if id == "1" {
                    return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "含有不能删除的超级管理员账户", "local")));
                }
            }
        }

        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let result = config_service::batch_delete_by_ids(&db, &ids_vec).await;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn update_config(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, item: web::Json<ConfigUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let config_id = Some(id.into_inner());
    if item.config_key.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置信息key不能为空", "local")));
    }
    if config_service::find_by_key_unique(&db, &item.config_key, &config_id).await.unwrap_or(false) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置信息key已存在", "local")));
    }
    if item.config_name.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置信息名称不能为空", "local")));
    }
    if config_service::find_by_name_unique(&db, &item.config_name, &config_id).await.unwrap_or(false) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置信息名称已存在", "local")));
    }
    //获取用户信息
    let admin = admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await?;
    let mut form_data = ConfigSaveDTO::from(item.0);
    form_data.id = config_id;
    form_data.update_by = admin.user_name;
    return match config_service::update_by_id(&db, &form_data).await {
        Ok(user_op) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(user_op, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse>  {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置信息id不能为空", "local")));
    }
    match config_service::get_by_detail(&db, &item.id).await {
        Ok(user_op) => match user_op {
            None => {
                Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "查询的配置信息不存在", "local")))
            }
            Some(detail) => {
                Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(detail, "local")))
            }
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

// 分页查询
pub async fn get_by_page(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse>  {
    let db = &state.db;
    config_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

// ==================== 路由注册（方案 C：单点维护）====================

/// 注册系统配置管理模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(config_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/config")
            // POST /config/add - 新增配置
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/add",
                web::post()
                    .to(insert_config)
                    .wrap(require_permission("system:config:create")),
            )
            // DELETE /config/batch_delete - 批量删除配置
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("system:config:delete")),
            )
            // PUT /config/update/{id} - 更新配置
            .route(
                "/update/{id}",
                web::put()
                    .to(update_config)
                    .wrap(require_permission("system:config:update")),
            )
            // GET /config/detail/{id} - 配置详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("system:config:view")),
            )
            // GET /config/list - 配置列表
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("system:config:list")),
            ),
    );
}