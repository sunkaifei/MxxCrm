//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::time::Instant;

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::entity::role::Entity as RoleEntity;
use crate::modules::system::model::dept::{DeptDetailVO, DeptModel, DeptSaveDTO, DeptSaveRequest, DeptUpdateRequest, ListQuery};
use crate::modules::system::model::dept_default_role::DeptDefaultRoleModel;
use crate::modules::system::service::{admin_service, dept_service, system_log_service};
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::{DbConn, EntityTrait};

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
    let admin = admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await?;
    let operator = admin.user_name.clone().unwrap_or_else(|| "unknown".to_string());
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

    // 入职默认角色校验（13.3-5）：仅允许启用且未删除的角色
    if let Some(role_id) = sys_dept.default_role_id {
        if role_id > 0 {
            if let Err(msg) = validate_default_role(&db, role_id).await {
                return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &msg, "local")));
            }
        }
    }

    let started = Instant::now();
    match dept_service::insert(&db, &form_data).await {
        Ok(dept_id) => {
            // 入职默认角色映射（13.3-5）：失败不阻塞部门创建
            match apply_default_role_mapping(&db, dept_id, sys_dept.default_role_id, &operator).await {
                Ok(Some(action)) => {
                    log_default_role_change(&db, &req, started, action, dept_id, sys_dept.default_role_id, &operator, "dept_admin_controller::save_dept", "POST").await;
                }
                Ok(None) => {}
                Err(e) => {
                    log::warn!("保存部门默认角色映射失败: {}", e);
                }
            }
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(dept_id, "local")))
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
    let admin = admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await?;
    let operator = admin.user_name.clone().unwrap_or_else(|| "unknown".to_string());

    // 入职默认角色校验（13.3-5）：仅允许启用且未删除的角色，校验失败则部门不更新
    if let Some(role_id) = sys_dept.default_role_id {
        if role_id > 0 {
            if let Err(msg) = validate_default_role(&db, role_id).await {
                return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &msg, "local")));
            }
        }
    }

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
            // 入职默认角色映射（13.3-5）：Some(>0)=设置 Some(0)=清除 None=不处理
            let started = Instant::now();
            match apply_default_role_mapping(&db, dept_id, sys_dept.default_role_id, &operator).await {
                Ok(Some(action)) => {
                    log_default_role_change(&db, &req, started, action, dept_id, sys_dept.default_role_id, &operator, "dept_admin_controller::dept_update", "PUT").await;
                }
                Ok(None) => {}
                Err(e) => {
                    return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &("保存入职默认角色失败,".to_string() + &e.to_string()), "local")));
                }
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
                let mut dept_vo = DeptDetailVO::from(dept_entity);
                // 入职默认角色回显（13.3-5 编辑抽屉）
                if let Ok(Some(mapping)) = DeptDefaultRoleModel::find_by_dept(&db, item.id.unwrap_or_default()).await {
                    dept_vo.default_role_id = Some(mapping.role_id);
                }
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

/// 入职默认角色校验（13.3-5）：仅允许 status=1 且未删除的角色
async fn validate_default_role(db: &DbConn, role_id: i64) -> std::result::Result<(), String> {
    match RoleEntity::find_by_id(role_id).one(db).await {
        Ok(Some(role)) => {
            if role.deleted == Some(2) {
                Err("所选角色已删除，请重新选择".to_string())
            } else if role.status != Some(1) {
                Err("仅可选择启用状态的角色作为入职默认角色".to_string())
            } else {
                Ok(())
            }
        }
        Ok(None) => Err("所选角色不存在，请重新选择".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// 入职默认角色映射处理（13.3-5）：Some(>0)=设置 Some(0)=清除 None=不处理；返回变更动作供审计
async fn apply_default_role_mapping(
    db: &DbConn,
    dept_id: i64,
    default_role_id: Option<i64>,
    operator: &str,
) -> std::result::Result<Option<&'static str>, sea_orm::DbErr> {
    match default_role_id {
        Some(role_id) if role_id > 0 => {
            DeptDefaultRoleModel::upsert(db, dept_id, role_id, operator).await?;
            Ok(Some("set"))
        }
        Some(_) => {
            DeptDefaultRoleModel::delete_by_dept(db, dept_id).await?;
            Ok(Some("clear"))
        }
        None => Ok(None),
    }
}

/// 部门默认角色映射变更审计（方案 10.6-4）：失败仅告警，不影响主流程
#[allow(clippy::too_many_arguments)]
async fn log_default_role_change(
    db: &DbConn,
    req: &HttpRequest,
    started: Instant,
    action: &str,
    dept_id: i64,
    role_id: Option<i64>,
    operator: &str,
    method: &str,
    request_method: &str,
) {
    let ctx = system_log_service::SaveLogContext {
        request: req,
        title: Some("部门默认角色配置".to_string()),
        business_type: Some(2),
        method: Some(method.to_string()),
        request_method: Some(request_method.to_string()),
        operator_type: Some(1),
        oper_name: Some(operator.to_string()),
        dept_name: None,
        oper_param: Some(serde_json::json!({"deptId": dept_id, "defaultRoleId": role_id}).to_string()),
        json_result: Some(format!("{{\"action\":\"{}\"}}", action)),
        status: Some(0),
        error_msg: None,
        status_code: Some(200),
        elapsed: Some(started.elapsed().as_millis() as i64),
    };
    if let Err(e) = system_log_service::save_log(db, ctx).await {
        log::warn!("记录部门默认角色审计日志失败: {}", e);
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





