//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use crate::core::kit::app::is_demo_mode;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::response::{MetaResp, MPACK};
use actix_web::{HttpRequest, HttpResponse, web};
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::entity::common::{InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::modules::system::model::admin::AdminModel;
use crate::modules::system::model::menu::{ListQuery, ListMeta, MenuAdminVO, MenuModel, MenuSaveRequest, MenuUpdateRequest, MENU_TYPE_MENU};
use crate::modules::system::service::{menu_service};
use crate::modules::system::service::menu_service::{all_menu_list, get_user_router_tree};
use crate::validate;

// 添加菜单
pub async fn add_menu(
    state: web::Data<AppState>,
    _req: HttpRequest,
    item: web::Json<MenuSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let menu = item.into_inner(); // 使用 into_inner 获取数据

    // 演示站（测试系统）模式下禁止新增菜单
    validate!(is_demo_mode(), "演示站模式下禁止新增菜单".to_string());

    // 从 meta 中获取 name
    let name = menu.meta.as_ref().and_then(|m| m.name.clone());
    
    // 基础字段校验
    validate!(menu.parent_id.is_none(), t!("system.menu.parent_id_empty", locale = "zh-CN").to_string());
    validate!(name.is_none(), t!("system.menu.name_empty", locale = "zh-CN").to_string());

    // 名称唯一性校验
    validate!(
        menu_service::find_by_name_unique(db, &name, &menu.parent_id, &None).await?,
        t!("system.menu.name_exists", locale = "zh-CN").to_string()
    );

    // 类型相关校验
    if menu.r#type == Some(MENU_TYPE_MENU.to_string()) {
        validate!(menu.route_name.is_none(), t!("system.menu.route_name_empty", locale = "zh-CN").to_string());
        validate!(menu.path.is_none(), t!("system.menu.route_path_empty", locale = "zh-CN").to_string());

        validate!(
            menu_service::find_by_path_unique(db, &menu.path, &menu.parent_id, &None).await?,
            t!("system.menu.route_path_exists", locale = "zh-CN").to_string()
        );

        validate!(
            menu_service::find_by_route_name_unique(db, &menu.route_name, &None).await?,
            t!("system.menu.route_name_exists", locale = "zh-CN").to_string()
        );
    }

    // 权限标识校验
    validate!(menu.perm.is_none(), t!("system.menu.permission_empty", locale = "zh-CN").to_string());
    validate!(
        menu_service::find_by_perms_unique(db, &menu.perm, &None).await?,
        t!("system.menu.permission_exists", locale = "zh-CN").to_string()
    );

    // 执行插入操作
    let result = menu_service::insert(db, menu).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success("添加成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}


// 删除菜单信息
pub async fn menu_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    // 演示站（测试系统）模式下禁止删除菜单
    validate!(is_demo_mode(), "演示站模式下禁止删除菜单".to_string());
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let result = menu_service::batch_delete_by_ids(db, ids_vec).await;
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
        }
    }else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

///更新菜单
pub async fn menu_update(state: web::Data<AppState>, path: web::Path<i64>, _req: HttpRequest, item: web::Json<MenuUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let mut sys_menu = item.0;

    // 从路径中获取 ID 并设置到请求体中
    sys_menu.id = Some(path.into_inner());

    // 演示站（测试系统）模式下禁止修改菜单
    validate!(is_demo_mode(), "演示站模式下禁止修改菜单".to_string());

    // 从 meta 中获取 name
    let name = sys_menu.meta.as_ref().and_then(|m| m.name.clone());

    validate!(sys_menu.parent_id.is_none(), t!("system.menu.parent_id_empty", locale = "zh-CN").to_string());
    validate!(name.is_none(), t!("system.menu.name_empty", locale = "zh-CN").to_string());
    validate!(
        menu_service::find_by_name_unique(&db, &name, &sys_menu.parent_id, &sys_menu.id).await?,
        t!("system.menu.name_exists", locale = "zh-CN").to_string()
    );

    // 类型相关校验
    if sys_menu.r#type == Some(MENU_TYPE_MENU.to_string()) {
        validate!(sys_menu.route_name.is_none(), t!("system.menu.route_name_empty", locale = "zh-CN").to_string());
        validate!(sys_menu.path.is_none(), t!("system.menu.route_path_empty", locale = "zh-CN").to_string());

        validate!(
            menu_service::find_by_path_unique(db, &sys_menu.path, &sys_menu.parent_id, &sys_menu.id).await?,
            t!("system.menu.route_path_exists", locale = "zh-CN").to_string()
        );

        validate!(
            menu_service::find_by_route_name_unique(db, &sys_menu.route_name, &sys_menu.id).await?,
            t!("system.menu.route_name_exists", locale = "zh-CN").to_string()
        );
    }
    
    // 权限标识校验
    validate!(sys_menu.perm.is_none(), t!("system.menu.permission_empty", locale = "zh-CN").to_string());
    validate!(
        menu_service::find_by_perms_unique(db, &sys_menu.perm, &sys_menu.id).await?,
        t!("system.menu.permission_exists", locale = "zh-CN").to_string()
    );

    let result = menu_service::update_by_id(&db, &sys_menu).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success("更新成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "更新失败", "local")))
    }
}


pub async fn menu_detail(state: web::Data<AppState>, path: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if path.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = MenuModel::find_by_id(db,&path.id).await?;
    if let Some(req) = result {
        let meta = ListMeta {
            name: req.name.clone(),
            affix_tab: Some(req.affix_tab),
            hide_children_in_menu: Some(req.hide_children_in_menu),
            hide_in_breadcrumb: Some(req.hide_in_breadcrumb),
            hide_in_menu: Some(req.hide_in_menu),
            hide_in_tab: Some(req.hide_in_tab),
            keep_alive: Some(req.keep_alive),
            sort: req.sort,
            icon: req.icon.clone(),
        };
        
        let menu_vo = MenuAdminVO {
            id: req.id,
            parent_id: Some(req.parent_id),
            tree_path: req.tree_path,
            name: req.name,
            meta: Some(meta),
            r#type: req.r#type,
            route_name: req.route_name,
            path: req.path,
            component: req.component,
            perm: req.perm,
            status: Some(req.status),
            redirect: req.redirect,
            create_time: None,
            update_time: None,
            params: req.params,
            children: Vec::new(),
        };

        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(menu_vo, "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未查询到数据", "local")))
    }
}

pub async fn menu_list(state: web::Data<AppState>, query: web::Query<ListQuery>) -> HttpResponse {
    let db = &state.db;
    // 菜单是树形结构不需要分页
    let result = all_menu_list(db, query.into_inner()).await;
    match result {
        Ok(router_list) => {
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(router_list, "local"))
        }
        Err(err) => {
            log::error!("获取菜单列表错误: {:?}", &err);
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "获取菜单列表错误", "local"))
        }
    }
}

/// 获取菜单下拉列表
pub async fn get_menu_options(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    //获取用户信息
    let menu_result = menu_service::get_menu_options(&db, &Some(get_current_user_id(&req))).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(menu_result, "local")))
}


///重新获取用户权限和路由
pub async fn get_user_menu(state: web::Data<AppState>,req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    //获取用户信息
    let current_user_id = get_current_user_id(&req);
    let user_info = AdminModel::find_by_id(&db,&Some(current_user_id)).await?.ok_or_else(|| { Error::from(format!("msg={},code={}", "未获取到用户信息".to_string(), 404))})?;

    //判断是否是管理员
    let is_admin = user_info.user_type == Option::from(1);
    log::info!("[菜单] getUserMenus: user_id={:?}, user_type={:?}, is_admin={}", current_user_id, user_info.user_type, is_admin);
    //根据id查询路由
    let result = get_user_router_tree(db, &is_admin, &Some(current_user_id)).await;
    match result {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &("查询菜单异常,".to_string() + &err.to_string()), "local")))
        }
    }
}

// ==================== 路由注册（方案 C：单点维护）====================

/// 注册菜单管理模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(menu_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/menu")
            // POST /menu/add - 添加菜单
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/add",
                web::post()
                    .to(add_menu)
                    .wrap(require_permission("system:menu:add")),
            )
            // DELETE /menu/batch_delete - 批量删除菜单
            .route(
                "/batch_delete",
                web::delete()
                    .to(menu_delete)
                    .wrap(require_permission("system:menu:delete")),
            )
            // PUT /menu/update/{id} - 更新菜单
            .route(
                "/update/{id}",
                web::put()
                    .to(menu_update)
                    .wrap(require_permission("system:menu:update")),
            )
            // GET /menu/detail/{id} - 菜单详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(menu_detail)
                    .wrap(require_permission("system:menu:view")),
            )
            // GET /menu/list - 菜单列表
            .route(
                "/list",
                web::get()
                    .to(menu_list)
                    .wrap(require_permission("system:menu:list")),
            )
            // GET /menu/options - 菜单下拉列表
            .route(
                "/options",
                web::get()
                    .to(get_menu_options)
                    .wrap(require_permission("system:menu:list")),
            )
            // GET /menu/getUserMenus - 获取当前用户路由
            .route("/getUserMenus", web::get().to(get_user_menu)),
    );
}