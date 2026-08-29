//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 标准敏感字段权限管理（FLS 配置入口，P2-1）
//! - 管理侧 2 接口：list/save（权限码 system:field_perm:list / system:field_perm:save）
//! - save 带审计埋点（P0-3 摘要风格：记录可见/可编辑角色配置变更前后状态）

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;

use crate::modules::system::model::field_perm::{FieldPermQuery, FieldPermSaveRequest};
use crate::modules::system::service::{admin_service, audit_service, field_perm_service};

/// 模块标识 → 中文名（审计摘要展示用）
fn module_label(module: &str) -> &str {
    match module {
        "crm_customer" => "客户",
        "crm_contact" => "联系人",
        "crm_contract" => "合同",
        _ => "未知模块",
    }
}

/// 角色配置转可读文案（None=未限制；数组=[a, b]）
fn roles_text(roles: &Option<serde_json::Value>) -> String {
    match roles {
        Some(serde_json::Value::Array(arr)) => {
            let names: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
            if names.is_empty() { "全部".to_string() } else { format!("[{}]", names.join(", ")) }
        },
        _ => "全部".to_string(),
    }
}

/// 获取当前操作人用户名（获取失败静默降级为 None，仅影响审计操作人字段）
async fn current_operator(db: &DatabaseConnection, req: &HttpRequest) -> Option<String> {
    admin_service::get_by_detail(db, &Some(get_current_user_id(req)))
        .await
        .ok()
        .and_then(|admin| admin.user_name)
}

// 查询标准字段权限清单（可按模块过滤）
pub async fn field_perm_list(state: web::Data<AppState>, query: web::Query<FieldPermQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let list = field_perm_service::get_list(db, query.module.clone()).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(list, "local")))
}

// 保存标准字段权限（仅白名单内字段；审计埋点）
pub async fn field_perm_save(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<FieldPermSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form = form_data.0;
    let module = form.module.clone().unwrap_or_default();
    let field_key = form.field_key.clone().unwrap_or_default();
    if module.is_empty() || field_key.is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "模块标识与字段键不能为空", "local")));
    }

    // 变更前快照（行不存在时快照为 None，不影响审计语义）
    let before = field_perm_service::get_list(db, Some(module.clone())).await
        .ok()
        .and_then(|list| list.into_iter().find(|v| v.field_key.as_deref() == Some(field_key.as_str())))
        .and_then(|vo| serde_json::to_value(vo).ok());

    let operator = current_operator(db, &req).await;
    let result = field_perm_service::save(db, &form, operator).await;
    // 审计埋点：标准字段权限配置变更（P0-3 摘要风格）
    if result.is_ok() {
        let label = module_label(&module);
        // 行不存在（首次配置）时前后均按"全部"展示
        let before_visible = before.as_ref().map(|b| b.get("visibleRoles").cloned().unwrap_or(serde_json::Value::Null));
        let before_editable = before.as_ref().map(|b| b.get("editableRoles").cloned().unwrap_or(serde_json::Value::Null));
        let summary = format!(
            "字段[{}.{}] 权限配置更新（可见: {} → {}; 可编辑: {} → {}）",
            label,
            field_key,
            roles_text(&before_visible),
            roles_text(&form.visible_roles),
            roles_text(&before_editable),
            roles_text(&form.editable_roles),
        );
        audit_service::record(
            db,
            &req,
            "field_perm",
            "update",
            "field_perm",
            result.clone().unwrap_or_default(),
            summary,
            before,
            audit_service::snap(vec![
                ("module", serde_json::json!(form.module)),
                ("field_key", serde_json::json!(form.field_key)),
                ("visible_roles", serde_json::json!(form.visible_roles)),
                ("editable_roles", serde_json::json!(form.editable_roles)),
            ]),
        ).await;
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

// ==================== 路由注册 ====================

/// 注册标准字段权限管理路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(field_perm_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/field-perm")
            // GET /field-perm/list - 白名单字段权限清单
            .route(
                "/list",
                web::get()
                    .to(field_perm_list)
                    .wrap(require_permission("system:field_perm:list")),
            )
            // POST /field-perm/save - 保存字段角色配置（注意：先 to() 再 wrap()）
            .route(
                "/save",
                web::post()
                    .to(field_perm_save)
                    .wrap(require_permission("system:field_perm:save")),
            ),
    );
}
