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
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::response::MetaResp;
use actix_web::{delete, get, HttpRequest, HttpResponse, post, put, web};
use actix_web_grants::protect;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{InfoId};
use crate::modules::system::model::admin::AdminModel;
use crate::modules::system::model::menu::{ListQuery, ListMeta, MenuAdminVO, MenuModel, MenuSaveRequest, MenuUpdateRequest, MENU_TYPE_MENU};
use crate::modules::system::service::{menu_service};
use crate::modules::system::service::menu_service::{all_menu_list, get_user_router_tree};
use crate::validate;

// 添加菜单
#[post("/menu/add")]
#[protect("system:menu:add")]
pub async fn add_menu(
    state: web::Data<AppState>,
    _req: HttpRequest,
    item: web::Json<MenuSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let menu = item.into_inner(); // 使用 into_inner 获取数据

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
            "路由名称已存在".to_string()
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
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success("添加成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}


// 删除菜单信息
#[delete("/menu/batch_delete")]
#[protect("system:menu:delete")]
pub async fn menu_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let result = menu_service::batch_delete_by_ids(db, ids_vec).await;
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
        }
    }else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

///更新菜单
#[put("/menu/update/{id}")]
#[protect("system:menu:update")]
pub async fn menu_update(state: web::Data<AppState>, path: web::Path<i64>, _req: HttpRequest, item: web::Json<MenuUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let mut sys_menu = item.0;
    
    // 从路径中获取 ID 并设置到请求体中
    sys_menu.id = Some(path.into_inner());

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
            "路由名称已存在".to_string()
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
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success("更新成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "更新失败", "local")))
    }
}


#[get("/menu/detail/{id}")]
#[protect("system:menu:view")]
pub async fn menu_detail(state: web::Data<AppState>, path: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if path.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
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
            params: req.params,
            children: None,
        };

        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(menu_vo, "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未查询到数据", "local")))
    }
}

#[get("/menu/list")]
#[protect("system:menu:list")]
pub async fn menu_list(state: web::Data<AppState>, query: web::Query<ListQuery>) -> HttpResponse {
    let db = &state.db;
    // 菜单是树形结构不需要分页
    let result = all_menu_list(db, query.into_inner()).await;
    match result {
        Ok(router_list) => {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(router_list, "local"))
        }
        Err(err) => {
            log::error!("获取菜单列表错误: {:?}", &err);
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "获取菜单列表错误", "local"))
        }
    }
}

/// 获取菜单下拉列表
#[get("/menu/options")]
#[protect("system:menu:list")]
pub async fn get_menu_options(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    //获取用户信息
    let jwt_token:JWTToken = get_user(&req).unwrap_or_default();
    let menu_result = menu_service::get_menu_options(&db, &jwt_token.id).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(menu_result, "local")))
}


///重新获取用户权限和路由
#[get("/menu/getUserMenus")]
pub async fn get_user_menu(state: web::Data<AppState>,req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    //获取用户信息
    let jwt_token:JWTToken = get_user(&req).unwrap_or_default();
    let user_info = AdminModel::find_by_id(&db,&jwt_token.id).await?.ok_or_else(|| { Error::from(format!("msg={},code={}", "未获取到用户信息".to_string(), 404))})?;

    //判断是否是管理员
    let is_admin = user_info.user_type == Option::from(1);
    //根据id查询路由
    let result = get_user_router_tree(db, &is_admin, &jwt_token.id).await;
    match result {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &("查询菜单异常,".to_string() + &err.to_string()), "local")))
        }
    }
}