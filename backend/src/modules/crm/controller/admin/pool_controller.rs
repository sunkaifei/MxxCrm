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
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use actix_web::{web, HttpRequest, HttpResponse};

use crate::modules::crm::model::lead_pool::{
    PoolConfigSaveRequest, PoolIdQuery, PoolListQuery, PoolMemberDeleteRequest, PoolMemberSaveRequest,
    PoolSaveRequest, PoolStatusUpdateQuery,
};
use crate::modules::crm::service::{pool_config_service, pool_member_service, pool_service};

/// 公海池分页
pub async fn pool_page(state: web::Data<AppState>, query: web::Query<PoolListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    match pool_service::page(db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 公海池详情
pub async fn pool_info(state: web::Data<AppState>, query: web::Query<PoolIdQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    if query.pool_id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "池ID不能为空", "local"));
    }

    match pool_service::find_vo_by_id(db, query.pool_id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 新增公海池
pub async fn pool_save(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PoolSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let operator_id = get_current_user_id(&req);

    let result = pool_service::insert(db, &form_data, operator_id).await;
    if result.is_ok() {
        crate::modules::system::service::audit_service::record(
            db,
            &req,
            "pool",
            "create",
            "lead_pool",
            result.as_ref().copied().unwrap_or_default(),
            format!("新增公海池 {}", form_data.name.clone().unwrap_or_default()),
            None,
            None,
        )
        .await;
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 更新公海池
pub async fn pool_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PoolSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let operator_id = get_current_user_id(&req);

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "池ID不能为空", "local")));
    }

    let result = pool_service::update(db, &form_data, operator_id).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 批量删除公海池
pub async fn pool_delete(state: web::Data<AppState>, req: HttpRequest, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;
    let user_id = get_current_user_id(&req);

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到删除的公海池ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = pool_service::batch_delete_by_ids(db, &filtered_ids, user_id).await;
    if result.is_ok() {
        for id in &filtered_ids {
            crate::modules::system::service::audit_service::record(
                db,
                &req,
                "pool",
                "delete",
                "lead_pool",
                *id,
                format!("删除公海池 #{}", id),
                None,
                None,
            )
            .await;
        }
    }
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

/// 启用/停用公海池
pub async fn pool_status(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PoolStatusUpdateQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let operator_id = get_current_user_id(&req);

    if query.id.is_none() || query.status.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "池ID与目标状态不能为空", "local"));
    }

    let result = pool_service::update_status(db, &query, operator_id).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

/// 公海工作台可见池（我的池 ∪ 我管理的池）
pub async fn pool_my(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let db = &state.db;
    let user_id = get_current_user_id(&req);

    match pool_service::my_pools(db, user_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 池成员列表
pub async fn member_list(state: web::Data<AppState>, query: web::Query<PoolIdQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    if query.pool_id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "池ID不能为空", "local"));
    }

    match pool_member_service::list_members(db, query.pool_id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 批量添加池成员
pub async fn member_save(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PoolMemberSaveRequest>) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;
    let operator_id = get_current_user_id(&req);

    let pool_id = match form_data.pool_id {
        Some(pid) => pid,
        None => return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "池ID不能为空", "local")),
    };
    let members: Vec<(i64, i16)> = form_data
        .members
        .unwrap_or_default()
        .into_iter()
        .filter_map(|m| m.user_id.map(|uid| (uid, m.member_type.unwrap_or(2))))
        .collect();
    if members.is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到要添加的成员", "local"));
    }

    let result = pool_member_service::add_members(db, pool_id, &members, operator_id).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

/// 批量移除池成员
pub async fn member_delete(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PoolMemberDeleteRequest>) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;
    let operator_id = get_current_user_id(&req);

    let pool_id = match form_data.pool_id {
        Some(pid) => pid,
        None => return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "池ID不能为空", "local")),
    };
    let user_ids: Vec<i64> = form_data
        .user_ids
        .unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();
    if user_ids.is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到要移除的成员", "local"));
    }

    let result = pool_member_service::remove_members(db, pool_id, &user_ids, operator_id).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

/// 候选成员列表（指派/自动分配下拉，含私海持有量）
pub async fn member_candidates(state: web::Data<AppState>, query: web::Query<PoolIdQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    if query.pool_id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "池ID不能为空", "local"));
    }

    match pool_member_service::list_candidates(db, query.pool_id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 池配置详情
pub async fn config_get(state: web::Data<AppState>, query: web::Query<PoolIdQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    if query.pool_id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "池ID不能为空", "local"));
    }

    match pool_config_service::get_vo(db, query.pool_id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 保存池配置
pub async fn config_save(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PoolConfigSaveRequest>) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;
    let operator_id = get_current_user_id(&req);

    let result = pool_config_service::save(db, &form_data, operator_id).await;
    match result {
        Ok(_) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("ok".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 路由注册
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/pool")
            .route("/page", web::get().to(pool_page).wrap(require_permission("crm:pool:query")))
            .route("/info", web::get().to(pool_info).wrap(require_permission("crm:pool:query")))
            .route("/my", web::get().to(pool_my).wrap(require_permission("crm:lead-pool:list")))
            .route("/save", web::post().to(pool_save).wrap(require_permission("crm:pool:save")))
            .route("/update", web::post().to(pool_update).wrap(require_permission("crm:pool:update")))
            .route("/delete", web::post().to(pool_delete).wrap(require_permission("crm:pool:delete")))
            .route("/status", web::post().to(pool_status).wrap(require_permission("crm:pool:update")))
            .route("/config", web::get().to(config_get).wrap(require_permission("crm:pool:query")))
            .route("/config/save", web::post().to(config_save).wrap(require_permission("crm:pool:update")))
            .route("/member/list", web::get().to(member_list).wrap(require_permission("crm:pool:member")))
            .route("/member/save", web::post().to(member_save).wrap(require_permission("crm:pool:member")))
            .route("/member/delete", web::post().to(member_delete).wrap(require_permission("crm:pool:member")))
            .route("/member/candidates", web::get().to(member_candidates).wrap(require_permission("crm:lead-pool:assign"))),
    );
}
