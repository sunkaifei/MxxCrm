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
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::system::model::role::{ListQuery, RoleSaveDTO, RoleSaveRequest, RoleUpdateRequest, UpdateRoleDeptRequest, UpdateRoleMenuRequest};
use crate::modules::system::service::menu_service::contains_all_elements;
use crate::modules::system::service::{admin_service, menu_service, role_service};

// 添加角色信息
#[post("/role/save")]
#[protect("system:role:save")]
pub async fn role_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<RoleSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    // 检查角色名称是否为空
    if form_data.role_name.as_ref().map_or(true, |name| name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "角色名称不能为空", "local")));
    }

    // 检查角色名称是否唯一
    if role_service::find_by_name_unique(&db, &form_data.role_name, &None).await? {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "角色名称已存在", "local")));
    }

    // 检查角色key是否为空
    if form_data.role_key.as_ref().map_or(true, |name| name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "角色key不能为空", "local")));
    }

    // 获取用户信息
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let admin = match admin_service::get_by_detail(&db, &jwt_token.id).await {
        Ok(admin) => admin,
        Err(_) => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "获取当前管理员信息错误", "local"))),
    };

    // 构建角色数据
    let mut role_data = RoleSaveDTO::from(form_data);
    role_data.create_by = admin.user_name;

    // 插入角色信息
    let result = role_service::insert(&db, &role_data).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

// 删除角色信息
#[delete("/role/bath_delete")]
#[protect("system:role:delete")]
pub async fn bath_delete_role(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_role = item.0;

    // 检查 ids 是否为空
    if delete_role.ids.is_none() || delete_role.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到删除的角色ID", "local"));
    }

    // 过滤掉空字符串和空白字符串
    let filtered_ids: Vec<i64> = delete_role.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    // 检查是否包含超级管理员角色ID (1)
    if filtered_ids.contains(&1) {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "不能删除超级管理员角色", "local"));
    }

    // 检查过滤后的 ids 是否为空
    if filtered_ids.is_empty() {
        return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到有效的删除角色ID", "local"));
    }

    // 执行批量删除
    let result = role_service::batch_delete_by_ids(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}

// 更新角色信息
#[put("/role/update/{id}")]
#[protect("system:role:update")]
pub async fn update_role(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, form_data: web::Json<RoleUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let role_id = id.into_inner();

    // 获取用户信息
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let admin = match admin_service::get_by_detail(&db, &jwt_token.id).await {
        Ok(admin) => admin,
        Err(_) => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "获取当前管理员信息错误", "local"))),
    };

    // 超级管理员角色只允许修改数据权限、备注、排序、状态，不允许修改角色名和key
    if role_id == 1 {
        // 构建只包含安全字段的更新数据
        let role_data = RoleSaveDTO {
            id: Some(role_id),
            role_name: None,
            role_key: None,
            data_scope: form_data.data_scope,
            sort: form_data.sort,
            status: form_data.status,
            remark: form_data.remark,
            deleted: None,
            create_by: None,
            create_time: None,
            update_by: admin.user_name,
            update_time: None,
        };
        let result = role_service::update_by_id(&db, &role_data).await;
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)));
    }

    // 检查是否是部分更新（只更新dataScope等可选字段，不修改role_name/role_key）
    let is_partial_update = form_data.role_name.is_none() && form_data.role_key.is_none();
    
    if !is_partial_update {
        // 检查角色名称是否为空
        if form_data.role_name.as_ref().map_or(true, |name| name.trim().is_empty()) {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "角色名称不能为空", "local")));
        }

        // 检查角色名称是否唯一
        if role_service::find_by_name_unique(&db, &form_data.role_name, &Some(role_id)).await? {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "角色名称已存在", "local")));
        }

        // 检查角色key是否为空
        if form_data.role_key.as_ref().map_or(true, |name| name.trim().is_empty()) {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "角色key不能为空", "local")));
        }
    }

    // 构建角色数据
    let mut role_data = RoleSaveDTO::from(form_data);
    role_data.id = Some(role_id);
    role_data.update_by = admin.user_name;

    // 更新角色信息
    let result = role_service::update_by_id(&db, &role_data).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

/// 更新角色关联的菜单
#[put("/role/assign_perm")]
pub async fn update_role_menus(state: web::Data<AppState>, req: HttpRequest, item: web::Json<UpdateRoleMenuRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let sys_role = item.0;

    // 获取用户信息
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    // 检查角色ID是否为空
    let role_id = match sys_role.role_id {
        Some(id) => id,
        None => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "角色id不能为空", "local"))),
    };

    // 超级管理员角色无需修改菜单权限（默认拥有所有权限），直接返回成功
    if role_id == 1 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(1i64, "local")));
    }

    // 检查菜单ID是否为空
    let menu_ids = match &sys_role.menu_ids {
        Some(ids) => ids,
        None => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "菜单id不能为空", "local"))),
    };

    // 获取用户可授权的菜单ID
    let user_menu_ids = menu_service::get_menu_vec_ids(&db, &jwt_token.id).await?;

    // 检查用户是否有权限授权这些菜单
    if !contains_all_elements(&user_menu_ids, &menu_ids) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "授权的部分元素不在您的权限之内", "local")));
    }

    // 更新角色菜单
    let result = role_service::update_role_menus(db, &sys_role).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}
#[get("/role/detail/{id}")]
#[protect("system:role:view")]
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let role_id = match item.id {
        Some(id) => id,
        None => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "角色id不能为空", "local"))),
    };

    let result = role_service::get_by_detail(&db, &Some(role_id)).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

#[get("/role/{role_id}/menuIds")]
pub async fn get_role_menu_list_by_role_id(state: web::Data<AppState>, role_id: web::Path<String>) -> Result<HttpResponse> {
    let db = &state.db;
    let role_id_str = role_id.into_inner();
    // 验证 role_id 是否为正整数并转换为 Option<i64>
    let role_id: Option<i64> = role_id_str.parse().ok().and_then(|id| if id > 0 { Some(id) } else { None });

    if role_id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "角色id参数错误", "local")));
    }

    let detail = role_service::get_by_detail(&db,&role_id).await?;
    
    role_service::get_role_menu_list_by_role_id(&db,&detail.id).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

#[put("/role/assign_data_scope")]
pub async fn update_role_depts(state: web::Data<AppState>, req: HttpRequest, item: web::Json<UpdateRoleDeptRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let sys_role = item.0;

    let _jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    let role_id = match sys_role.role_id {
        Some(id) => id,
        None => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "角色id不能为空", "local"))),
    };

    if role_id == 1 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(1i64, "local")));
    }

    let result = role_service::update_role_depts(db, &sys_role).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[get("/role/{role_id}/deptIds")]
pub async fn get_role_dept_list_by_role_id(state: web::Data<AppState>, role_id: web::Path<String>) -> Result<HttpResponse> {
    let db = &state.db;
    let role_id_str = role_id.into_inner();
    let role_id: Option<i64> = role_id_str.parse().ok().and_then(|id| if id > 0 { Some(id) } else { None });

    if role_id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "角色id参数错误", "local")));
    }

    role_service::get_role_dept_list_by_role_id(&db, &role_id).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

#[get("/role/options")]
pub async fn role_options(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    role_service::get_role_options(&db).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

// 查询角色列表
#[get("/role/list")]
#[protect("system:role:list")]
pub async fn role_list(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    role_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}
