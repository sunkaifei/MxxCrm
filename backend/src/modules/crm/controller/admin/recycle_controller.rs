//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::response::{MetaResp, MPACK};
use actix_web::{web, HttpRequest, HttpResponse};

use crate::modules::crm::model::recycle::{RecycleActionRequest, RecycleListQuery};
use crate::modules::crm::service::{recycle_service, recycle_service::module_label};
use crate::modules::system::service::audit_service;

/// GET /recycle/list - 回收站分页列表（普通用户仅见自己删除的，超管见全部）
pub async fn recycle_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<RecycleListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let current_user_id = get_current_user_id(&req);

    match recycle_service::list(db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /recycle/restore - 还原回收站数据（本人可还原自己删的，管理员可还原任何人的）
pub async fn recycle_restore(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<RecycleActionRequest>) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.module.trim().is_empty() || form_data.id <= 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "数据模块和ID不能为空", "local"));
    }
    let current_user_id = get_current_user_id(&req);

    let result = recycle_service::restore(db, &form_data.module, form_data.id, current_user_id).await;
    if result.is_ok() {
        audit_service::record(
            db, &req, "recycle", "restore", &form_data.module, form_data.id,
            format!("从回收站还原{} #{}", module_label(&form_data.module), form_data.id),
            None,
            None,
        ).await;
    }
    match result {
        Ok(_) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(true, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /recycle/purge - 彻底删除回收站数据（仅超管，前端二次确认）
pub async fn recycle_purge(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<RecycleActionRequest>) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.module.trim().is_empty() || form_data.id <= 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "数据模块和ID不能为空", "local"));
    }
    let current_user_id = get_current_user_id(&req);

    let result = recycle_service::purge(db, &form_data.module, form_data.id, current_user_id).await;
    if result.is_ok() {
        audit_service::record(
            db, &req, "recycle", "purge", &form_data.module, form_data.id,
            format!("彻底删除回收站{} #{}", module_label(&form_data.module), form_data.id),
            None,
            None,
        ).await;
    }
    match result {
        Ok(_) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(true, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册回收站模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(recycle_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/recycle")
            // GET /recycle/list - 回收站分页列表（权限由 service 层校验：超管见全部，普通用户仅见自己删的）
            .route("/list", web::get().to(recycle_list))
            // POST /recycle/restore - 还原（service 层校验：本人可还原自己删的，管理员可还原任何人的）
            .route("/restore", web::post().to(recycle_restore))
            // POST /recycle/purge - 彻底删除（service 层校验：仅超管）
            .route("/purge", web::post().to(recycle_purge)),
    );
}
