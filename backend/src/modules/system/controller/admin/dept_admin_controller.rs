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
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::dept::{DeptDetailVO, DeptModel, DeptSaveDTO, DeptSaveRequest, DeptUpdateRequest, ListQuery};
use crate::modules::system::service::{admin_service, dept_service};
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn save_dept(state: web::Data<AppState>, req: HttpRequest, item: web::Json<DeptSaveRequest>) -> Result<HttpResponse> {
    //log::info!("dept_save params: {:?}", &item);
    let db = &state.db;
    let sys_dept = item.0;
    if let Some(dept_name) = sys_dept.dept_name.as_ref() {
        if dept_name.trim().is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "部门名称不能为空", "local")));
        }
        if dept_name.len() > 30 {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "部门名称不能超过30个字符", "local")));
        }
    } else {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "部门名称不能为空", "local")));
    }
    
    //获取用户信息
    let jwt_token:JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;
    let mut form_data = DeptSaveDTO::from(sys_dept.clone());

    if let Some(leader_id) = form_data.leader_id {
        if leader_id > 0 {
            if let Ok(Some(leader_admin)) = admin_service::find_by_id(&db, &Some(leader_id)).await {
                form_data.leader = leader_admin.nick_name
                    .filter(|s| !s.is_empty())
                    .or(leader_admin.user_name);
            }
        }
    }

    form_data.create_by = admin.user_name.clone();
    form_data.update_by = admin.user_name;
    match dept_service::insert(&db, &form_data).await {
        Ok(user_op) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(user_op, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

// 删除部门信息
pub async fn dept_batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
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
        let result = dept_service::batch_delete_by_ids(&db, &ids_vec).await;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

///更新部门信息
pub async fn dept_update(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, item: web::Json<DeptUpdateRequest>) ->  Result<HttpResponse> {
    let db = &state.db;
    let sys_dept = item.0;
    let dept_id = id.into_inner();

    if let Some(dept_name) = sys_dept.dept_name.as_ref() {
        if dept_name.trim().is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "部门名称不能为空", "local")));
        }
        if dept_name.len() > 30 {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "部门名称不能超过30个字符", "local")));
        }
    } else {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "部门名称不能为空", "local")));
    }
    
    //获取用户信息
    let jwt_token:JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;

    let mut form_data = DeptSaveDTO::from(sys_dept.clone());
    form_data.id = Some(dept_id);

    if let Some(leader_id) = form_data.leader_id {
        if leader_id > 0 {
            if let Ok(Some(leader_admin)) = admin_service::find_by_id(&db, &Some(leader_id)).await {
                form_data.leader = leader_admin.nick_name
                    .filter(|s| !s.is_empty())
                    .or(leader_admin.user_name);
            }
        } else {
            form_data.leader = None;
        }
    } else {
        form_data.leader = None;
    }

    form_data.update_by = admin.user_name;
    let result = dept_service::update_by_id(&db, &form_data).await;
    match result {
        Ok(v) => {
            if v == 0 {
                return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "更新部门信息异常", "local")));
            }
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &("更新部门信息异常,".to_string() + &err.to_string()), "local")))
        }
    }
}

pub async fn get_dept_options(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = dept_service::get_dept_options(db).await;
    match result {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &("查询部门列表异常,".to_string() + &err.to_string()), "local")))
        }
    }
}

pub async fn get_dept_tree(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = dept_service::get_dept_tree(db).await;
    match result {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &("查询部门列表树异常,".to_string() + &err.to_string()), "local")))
        }
    }
}

pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> HttpResponse {
    let db = &state.db;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "部门id不能为空", "local"));
    }
    return match DeptModel::find_by_id(&db, item.id.unwrap_or_default()).await {
        Ok(dept_op) => match dept_op {
            None => {
                HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "部门信息不存在", "local"))
            }
            Some(dept_entity) => {
                let dept_vo = DeptDetailVO::from(dept_entity);
                HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(dept_vo, "local"))
            }
        }
        Err(err) => {
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local"))
        }
    }
}

// 查询用户列表
pub async fn dept_list(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = dept_service::get_all_tree(&db,query.into_inner()).await;
    match result {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &("查询部门列表树异常,".to_string() + &err.to_string()), "local")))
        }
    }
}

// ==================== 路由注册（方案 C：单点维护）====================

/// 注册部门管理模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(dept_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dept")
            // POST /dept/save - 添加部门
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/save",
                web::post()
                    .to(save_dept)
                    .wrap(require_permission("system:dept:save")),
            )
            // DELETE /dept/batch_delete - 批量删除部门
            .route(
                "/batch_delete",
                web::delete()
                    .to(dept_batch_delete)
                    .wrap(require_permission("system:dept:delete")),
            )
            // PUT /dept/update/{id} - 更新部门
            .route(
                "/update/{id}",
                web::put()
                    .to(dept_update)
                    .wrap(require_permission("system:dept:update")),
            )
            // GET /dept/options - 部门下拉选项
            .route("/options", web::get().to(get_dept_options))
            // GET /dept/tree - 部门树
            .route("/tree", web::get().to(get_dept_tree))
            // GET /dept/detail/{id} - 部门详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("system:dept:view")),
            )
            // GET /dept/list - 部门列表
            .route(
                "/list",
                web::get()
                    .to(dept_list)
                    .wrap(require_permission("system:dept:list")),
            ),
    );
}





