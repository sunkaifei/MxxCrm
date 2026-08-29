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
use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::perm_set::{ListQuery, PermSetSaveDTO, PermSetSaveRequest, PermSetUpdateRequest, UpdatePermSetMenuRequest};
use crate::modules::system::service::menu_service::contains_all_elements;
use crate::modules::system::service::{admin_service, menu_service, perm_set_service, permission_cache_service};

// 添加权限集信息
pub async fn perm_set_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<PermSetSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    // 检查权限集名称是否为空
    if form_data.perm_set_name.as_ref().map_or(true, |name| name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "权限集名称不能为空", "local")));
    }

    // 检查权限集名称是否唯一
    if perm_set_service::find_by_name_unique(&db, &form_data.perm_set_name, &None).await? {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "权限集名称已存在", "local")));
    }

    // 检查权限集key是否为空
    if form_data.perm_set_key.as_ref().map_or(true, |name| name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "权限集key不能为空", "local")));
    }

    // 获取用户信息
    let admin = match admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await {
        Ok(admin) => admin,
        Err(_) => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "获取当前管理员信息错误", "local"))),
    };

    // 构建权限集数据
    let mut perm_set_data = PermSetSaveDTO::from(form_data);
    perm_set_data.create_by = admin.user_name;

    // 插入权限集信息
    let result = perm_set_service::insert(&db, &perm_set_data).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

// 复制权限集（P3-1 一键复制）：深拷贝源权限集的菜单授权配置
pub async fn perm_set_copy(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let perm_set_id = id.into_inner();

    // 获取用户信息
    let admin = match admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await {
        Ok(admin) => admin,
        Err(_) => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "获取当前管理员信息错误", "local"))),
    };

    // 源权限集快照：用于审计摘要（best-effort，查询失败不阻断复制）
    let source_detail = perm_set_service::get_by_detail(&db, &Some(perm_set_id)).await.ok();

    let result = perm_set_service::copy_perm_set(&db, perm_set_id, admin.user_name).await;

    // P0-3: 复制成功后记录审计（best-effort，不阻断业务）
    if let Ok(new_id) = result.as_ref() {
        let source_name = source_detail.as_ref()
            .and_then(|d| d.perm_set_name.clone())
            .unwrap_or_else(|| format!("#{}", perm_set_id));
        let new_name = perm_set_service::get_by_detail(&db, &Some(*new_id)).await.ok()
            .and_then(|d| d.perm_set_name)
            .unwrap_or_else(|| format!("#{}", new_id));
        let summary = format!("复制权限集[{}] → 新权限集[{}]", source_name, new_name);
        crate::modules::system::service::audit_service::record(
            db,
            &req,
            "auth",
            "copy",
            "perm_set",
            *new_id,
            summary,
            crate::modules::system::service::audit_service::snap(vec![
                ("source_perm_set_id", serde_json::json!(perm_set_id)),
                ("source_perm_set_name", serde_json::json!(source_name)),
            ]),
            crate::modules::system::service::audit_service::snap(vec![
                ("new_perm_set_id", serde_json::json!(new_id)),
                ("new_perm_set_name", serde_json::json!(new_name)),
            ]),
        ).await;
    }

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

// 删除权限集信息
pub async fn bath_delete_perm_set(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_perm_set = item.0;

    // 检查 ids 是否为空
    if delete_perm_set.ids.is_none() || delete_perm_set.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到删除的权限集ID", "local"));
    }

    // 过滤掉空字符串和空白字符串
    let filtered_ids: Vec<i64> = delete_perm_set.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    // 检查过滤后的 ids 是否为空
    if filtered_ids.is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到有效的删除权限集ID", "local"));
    }

    // 执行批量删除
    let result = perm_set_service::batch_delete_by_ids(&db, &filtered_ids).await;
    // 权限集删除后，清除该权限集所有用户的权限缓存
    if result.is_ok() {
        for perm_set_id in &filtered_ids {
            permission_cache_service::invalidate_by_perm_set_id(&db, *perm_set_id).await;
        }
    }
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

// 更新权限集信息
pub async fn update_perm_set(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, form_data: web::Json<PermSetUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let perm_set_id = id.into_inner();

    // 获取用户信息
    let admin = match admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await {
        Ok(admin) => admin,
        Err(_) => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "获取当前管理员信息错误", "local"))),
    };

    // 检查是否是部分更新（只更新status等可选字段，不修改perm_set_name/perm_set_key）
    let is_partial_update = form_data.perm_set_name.is_none() && form_data.perm_set_key.is_none();

    if !is_partial_update {
        // 检查权限集名称是否为空
        if form_data.perm_set_name.as_ref().map_or(true, |name| name.trim().is_empty()) {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "权限集名称不能为空", "local")));
        }

        // 检查权限集名称是否唯一
        if perm_set_service::find_by_name_unique(&db, &form_data.perm_set_name, &Some(perm_set_id)).await? {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "权限集名称已存在", "local")));
        }

        // 检查权限集key是否为空
        if form_data.perm_set_key.as_ref().map_or(true, |name| name.trim().is_empty()) {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "权限集key不能为空", "local")));
        }
    }

    // 构建权限集数据
    let mut perm_set_data = PermSetSaveDTO::from(form_data);
    perm_set_data.id = Some(perm_set_id);
    perm_set_data.update_by = admin.user_name;

    // 更新权限集信息
    let result = perm_set_service::update_by_id(&db, &perm_set_data).await;
    // 状态（停用/启用）或基础信息变更后，清除该权限集所有用户的权限缓存，保证即时生效
    if result.is_ok() {
        permission_cache_service::invalidate_by_perm_set_id(&db, perm_set_id).await;
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 更新权限集关联的菜单
pub async fn update_perm_set_menus(state: web::Data<AppState>, req: HttpRequest, item: web::Json<UpdatePermSetMenuRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let perm_set_form = item.0;

    // 检查权限集ID是否为空
    let perm_set_id = match perm_set_form.perm_set_id {
        Some(id) => id,
        None => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "权限集id不能为空", "local"))),
    };

    // 检查菜单ID是否为空
    let menu_ids = match &perm_set_form.menu_ids {
        Some(ids) => ids,
        None => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "菜单id不能为空", "local"))),
    };

    // 获取用户可授权的菜单ID
    let user_menu_ids = menu_service::get_menu_vec_ids(&db, &Some(get_current_user_id(&req))).await?;

    // 检查用户是否有权限授权这些菜单
    if !contains_all_elements(&user_menu_ids, &menu_ids) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "授权的部分元素不在您的权限之内", "local")));
    }

    // P0-3: 更新前取菜单授权快照与权限集名，用于变更摘要（仅计数与差值，不落全量清单）
    let before_menu_ids: std::collections::HashSet<i64> = perm_set_service::get_perm_set_menu_list_by_perm_set_id(db, &Some(perm_set_id))
        .await
        .unwrap_or_default()
        .iter()
        .filter_map(|s| s.as_ref().and_then(|x| x.parse::<i64>().ok()))
        .collect();
    let perm_set_name = perm_set_service::get_by_detail(db, &Some(perm_set_id))
        .await
        .ok()
        .and_then(|d| d.perm_set_name)
        .unwrap_or_else(|| format!("#{}", perm_set_id));

    // 更新权限集菜单
    let result = perm_set_service::update_perm_set_menus(db, &perm_set_form).await;
    // 权限集菜单权限变更后，清除该权限集所有用户的权限缓存
    if result.is_ok() {
        permission_cache_service::invalidate_by_perm_set_id(db, perm_set_id).await;
        // P0-3 审计摘要：菜单权限前后计数与增减差值（保存成功才记录）
        let after_menu_ids: std::collections::HashSet<i64> = menu_ids.iter().copied().collect();
        let added = after_menu_ids.difference(&before_menu_ids).count();
        let removed = before_menu_ids.difference(&after_menu_ids).count();
        let summary = format!(
            "权限集[{}] 菜单权限 {}→{} 项（+{}/-{}）",
            perm_set_name,
            before_menu_ids.len(),
            after_menu_ids.len(),
            added,
            removed
        );
        crate::modules::system::service::audit_service::record(
            db,
            &req,
            "auth",
            "grant",
            "perm_set",
            perm_set_id,
            summary,
            crate::modules::system::service::audit_service::snap(vec![("menu_count", serde_json::json!(before_menu_ids.len()))]),
            crate::modules::system::service::audit_service::snap(vec![
                ("menu_count", serde_json::json!(after_menu_ids.len())),
                ("added", serde_json::json!(added)),
                ("removed", serde_json::json!(removed)),
            ]),
        ).await;
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let perm_set_id = match item.id {
        Some(id) => id,
        None => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "权限集id不能为空", "local"))),
    };

    let result = perm_set_service::get_by_detail(&db, &Some(perm_set_id)).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

pub async fn get_perm_set_menu_list_by_perm_set_id(state: web::Data<AppState>, perm_set_id: web::Path<String>) -> Result<HttpResponse> {
    let db = &state.db;
    let perm_set_id_str = perm_set_id.into_inner();
    // 验证 perm_set_id 是否为正整数并转换为 Option<i64>
    let perm_set_id: Option<i64> = perm_set_id_str.parse().ok().and_then(|id| if id > 0 { Some(id) } else { None });

    if perm_set_id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "权限集id参数错误", "local")));
    }

    perm_set_service::get_perm_set_menu_list_by_perm_set_id(&db, &perm_set_id).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

pub async fn perm_set_options(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    perm_set_service::get_perm_set_options(&db).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

// 查询权限集列表
pub async fn perm_set_list(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    perm_set_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

// ==================== 路由注册（方案 C：单点维护）====================

/// 注册权限集管理模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(perm_set_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/perm_set")
            // POST /perm_set/save - 添加权限集
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/save",
                web::post()
                    .to(perm_set_insert)
                    .wrap(require_permission("system:perm_set:save")),
            )
            // POST /perm_set/copy/{id} - 复制权限集（P3-1 一键复制，复用新增权限码）
            .route(
                "/copy/{id}",
                web::post()
                    .to(perm_set_copy)
                    .wrap(require_permission("system:perm_set:save")),
            )
            // DELETE /perm_set/bath_delete - 批量删除权限集
            .route(
                "/bath_delete",
                web::delete()
                    .to(bath_delete_perm_set)
                    .wrap(require_permission("system:perm_set:delete")),
            )
            // PUT /perm_set/update/{id} - 更新权限集
            .route(
                "/update/{id}",
                web::put()
                    .to(update_perm_set)
                    .wrap(require_permission("system:perm_set:update")),
            )
            // PUT /perm_set/assign_perm - 更新权限集关联的菜单
            .route(
                "/assign_perm",
                web::put()
                    .to(update_perm_set_menus)
                    .wrap(require_permission("system:perm_set:update")),
            )
            // GET /perm_set/detail/{id} - 权限集详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("system:perm_set:view")),
            )
            // GET /perm_set/{perm_set_id}/menuIds - 获取权限集菜单ID列表
            .route(
                "/{perm_set_id}/menuIds",
                web::get().to(get_perm_set_menu_list_by_perm_set_id),
            )
            // GET /perm_set/options - 权限集下拉选项
            .route("/options", web::get().to(perm_set_options))
            // GET /perm_set/list - 权限集列表
            .route(
                "/list",
                web::get()
                    .to(perm_set_list)
                    .wrap(require_permission("system:perm_set:list")),
            ),
    );
}
