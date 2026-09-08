//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 自定义字段定义管理（混合方案 P0）
//! - 管理侧 6 接口：list/save/update/status/delete/modules（权限码 system:field:*）
//! - 运行侧 1 接口：schema（登录态免权限码，后端按角色过滤下发，P0-23 防串角色）
//! - 管理侧写接口四处审计埋点（save/update/status/delete）

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;

use crate::modules::system::model::field_def::{
    FieldDefSaveRequest, FieldDefStatusRequest, FieldDefUpdateRequest, FieldIndexRequest, ListQuery,
    SchemaListVO, SchemaQuery,
};
use crate::modules::system::service::{admin_service, audit_service, field_def_service, field_index_service};

/// 获取当前操作人用户名（获取失败静默降级为 None，仅影响审计操作人字段）
async fn current_operator(db: &DatabaseConnection, req: &HttpRequest) -> Option<String> {
    admin_service::get_by_detail(db, &Some(get_current_user_id(req)))
        .await
        .ok()
        .and_then(|admin| admin.user_name)
}

// 新增字段定义
pub async fn field_save(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<FieldDefSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form = form_data.0;
    let operator = current_operator(db, &req).await;

    let result = field_def_service::save(db, &form, operator).await;
    // 审计埋点：新增字段定义
    if let Ok(new_id) = &result {
        audit_service::record(
            db,
            &req,
            "field_def",
            "create",
            "field_def",
            *new_id,
            format!("新增自定义字段 {}（{}）", form.field_key.clone().unwrap_or_default(), form.field_label.clone().unwrap_or_default()),
            None,
            audit_service::snap(vec![
                ("module", serde_json::json!(form.module)),
                ("field_key", serde_json::json!(form.field_key)),
                ("field_label", serde_json::json!(form.field_label)),
                ("field_type", serde_json::json!(form.field_type)),
                ("required", serde_json::json!(form.required)),
            ]),
        ).await;
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

// 更新字段定义（module/field_key/field_type 锁定，choices value 不可删改，见 7.4）
pub async fn field_update(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, form_data: web::Json<FieldDefUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let mut form = form_data.0;
    form.id = Some(id.into_inner());
    let operator = current_operator(db, &req).await;

    // 变更前快照（记录不存在时快照为 None，不影响审计语义）
    let before = field_def_service::get_detail(db, form.id.unwrap_or_default())
        .await
        .ok()
        .and_then(|vo| serde_json::to_value(vo).ok());

    let result = field_def_service::update(db, &form, operator).await;
    // 审计埋点：更新字段定义
    if result.is_ok() {
        audit_service::record(
            db,
            &req,
            "field_def",
            "update",
            "field_def",
            form.id.unwrap_or_default(),
            format!("更新自定义字段 #{}", form.id.unwrap_or_default()),
            before,
            audit_service::snap(vec![
                ("field_label", serde_json::json!(form.field_label)),
                ("options", serde_json::json!(form.options)),
                ("required", serde_json::json!(form.required)),
                ("visible_roles", serde_json::json!(form.visible_roles)),
                ("editable_roles", serde_json::json!(form.editable_roles)),
                ("list_visible", serde_json::json!(form.list_visible)),
                ("filterable", serde_json::json!(form.filterable)),
                ("sort", serde_json::json!(form.sort)),
                ("remark", serde_json::json!(form.remark)),
            ]),
        ).await;
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

// 停用/启用字段定义
pub async fn field_update_status(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<FieldDefStatusRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form = form_data.0;

    let field_id = match form.id {
        Some(id) if id > 0 => id,
        _ => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字段ID不能为空", "local"))),
    };
    let status = match form.status {
        Some(s) => s,
        None => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "状态值不能为空", "local"))),
    };

    let result = field_def_service::update_status(db, field_id, status).await;
    // 审计埋点：停用/启用字段定义
    if result.is_ok() {
        audit_service::record(
            db,
            &req,
            "field_def",
            "status",
            "field_def",
            field_id,
            format!("{}自定义字段 #{}", if status == 1 { "启用" } else { "停用" }, field_id),
            None,
            audit_service::snap(vec![("status", serde_json::json!(status))]),
        ).await;
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

// 删除字段定义（逻辑删除，不清理业务表存量数据）
pub async fn field_delete(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let field_id = id.into_inner();
    if field_id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字段ID不能为空", "local")));
    }

    let ids = vec![field_id];
    let result = field_def_service::batch_delete(db, &ids).await;
    // 审计埋点：删除字段定义
    if result.is_ok() {
        audit_service::record(
            db,
            &req,
            "field_def",
            "delete",
            "field_def",
            field_id,
            format!("删除自定义字段 #{}", field_id),
            None,
            audit_service::snap(vec![("ids", serde_json::json!(ids))]),
        ).await;
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

// 查询字段定义列表
pub async fn field_list(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    field_def_service::get_by_page(db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

// 已接入自定义字段的业务模块下拉（前后端同源，数据来自 MODULE_TABLES 常量）
pub async fn field_modules() -> Result<HttpResponse> {
    let options = field_def_service::get_module_options();
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(options, "local")))
}

// 运行侧 schema 下发（登录态免权限码：按当前用户角色过滤后返回，P0-23 防串角色）
pub async fn field_schema(state: web::Data<AppState>, req: HttpRequest, query: web::Query<SchemaQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let module = query.module.clone().unwrap_or_default().trim().to_string();
    if module.is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "module 参数不能为空", "local")));
    }

    let user_id = get_current_user_id(&req);
    let (is_admin, role_keys) = field_def_service::load_user_role(db, user_id).await?;
    let list = field_def_service::get_schema(db, &module, is_admin, &role_keys).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(SchemaListVO { list }, "local")))
}

// 一键加速（P1-4）：对模块下 filterable 字段建表达式索引 + 模块级 GIN
// CONCURRENTLY 无锁表执行，大表期间业务无长阻塞；失败项逐条回传（P1-14 INVALID 自愈在 service 内完成）
pub async fn field_accelerate(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<FieldIndexRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let module = form_data.module.clone().unwrap_or_default().trim().to_string();
    if module.is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "module 参数不能为空", "local")));
    }

    let result = field_index_service::accelerate(db, &module).await;
    // 审计埋点：一键加速（索引属敏感配置变更，与 save/update/status/delete 同链路记录）
    if let Ok(report) = &result {
        audit_service::record(
            db,
            &req,
            "field_def",
            "index",
            "field_def",
            0,
            format!("一键加速模块 {} 自定义字段索引（新建 {} 个）", module, report.created.len()),
            None,
            audit_service::snap(vec![
                ("module", serde_json::json!(module)),
                ("created", serde_json::json!(report.created)),
                ("skipped", serde_json::json!(report.skipped)),
                ("failed", serde_json::json!(report.failed)),
                ("gin_status", serde_json::json!(report.gin_status)),
            ]),
        ).await;
    }
    let body = match result {
        Ok(report) => MetaResp::success(report, "local"),
        Err(e) => MetaResp::<()>::fail(400, &e.to_string(), "local"),
    };
    Ok(HttpResponse::Ok().content_type(MPACK).body(body))
}

// ==================== 路由注册 ====================

/// 注册自定义字段管理模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(field_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/field")
            // GET /field/schema - 运行侧 schema（登录态即可，无权限码；后端按角色过滤 P0-23）
            .route("/schema", web::get().to(field_schema))
            // GET /field/modules - 已接入模块下拉
            .route(
                "/modules",
                web::get()
                    .to(field_modules)
                    .wrap(require_permission("system:field:list")),
            )
            // GET /field/list - 字段定义列表
            .route(
                "/list",
                web::get()
                    .to(field_list)
                    .wrap(require_permission("system:field:list")),
            )
            // POST /field/save - 新增字段定义
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/save",
                web::post()
                    .to(field_save)
                    .wrap(require_permission("system:field:save")),
            )
            // POST /field/status - 停用/启用字段定义
            .route(
                "/status",
                web::post()
                    .to(field_update_status)
                    .wrap(require_permission("system:field:status")),
            )
            // PUT /field/update/{id} - 更新字段定义
            .route(
                "/update/{id}",
                web::put()
                    .to(field_update)
                    .wrap(require_permission("system:field:update")),
            )
            // POST /field/accelerate - 一键加速（filterable 表达式索引 + 模块 GIN，P1-4；复用 update 权限码）
            .route(
                "/accelerate",
                web::post()
                    .to(field_accelerate)
                    .wrap(require_permission("system:field:update")),
            )
            // DELETE /field/{id} - 逻辑删除字段定义（具体路径注册在前，通配 {id} 最后）
            .route(
                "/{id}",
                web::delete()
                    .to(field_delete)
                    .wrap(require_permission("system:field:delete")),
            ),
    );
}
