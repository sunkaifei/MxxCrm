//!
//! 客户公海（v57）HTTP 接口层：工作台 + 池管理 + 成员 + 池配置
//!
//! 挂载点：`customer_controller.rs` 内 `/customer-pool` scope 的 `.configure(register)`。
//! 存量 `/customer-pool/list` 与 `/customer/claim`、`/customer/add-to-pool` 保持兼容不迁。

use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::crm::model::customer_pool::{
    CustomerPoolAuditPageQuery, CustomerPoolAuditRequest, CustomerPoolConfigSaveRequest,
    CustomerPoolIdQuery, CustomerPoolListQuery, CustomerPoolMemberDeleteRequest,
    CustomerPoolMemberSaveRequest, CustomerPoolSaveRequest, CustomerPoolStatusUpdateQuery,
    CustomerPoolWorkbenchQuery, CustomerWorkbenchApplyRequest, CustomerWorkbenchClaimRequest,
    CustomerWorkbenchReleaseRequest, CustomerWorkbenchRetainRequest,
};
use crate::modules::crm::service::customer_pool_assign_service::{
    assign as pool_assign_service, frozen_page as pool_frozen_page, restore as pool_frozen_restore,
    CustomerPoolAssignRequest, CustomerPoolFreezePageQuery,
};
use crate::modules::crm::service::customer_pool_assign_service;
use crate::modules::crm::service::{
    customer_pool_config_service, customer_pool_dup_rule_service,
    customer_pool_member_service, customer_pool_service, customer_pool_stats_service,
    customer_pool_workbench_service,
};
use crate::utils::string_utils::deserialize_string_or_number_to_i64;

/// 流水轨迹查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerTraceQuery {
    #[serde(deserialize_with = "deserialize_string_or_number_to_i64")]
    pub customer_id: i64,
}

// ==================== 工作台 ====================

/// 我可操作的池列表
pub async fn workbench_pools(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_service::my_pools(db, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 池内客户分页（脱敏 + 可见性过滤）
pub async fn workbench_customers(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<CustomerPoolWorkbenchQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_workbench_service::pool_customers(db, &query.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 领取（单个/批量，逐条返回明细）
pub async fn workbench_claim(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerWorkbenchClaimRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_workbench_service::claim(db, &item.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 提交申领
pub async fn workbench_apply(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerWorkbenchApplyRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_workbench_service::apply(db, &item.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 退回公海（批量）
pub async fn workbench_release(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerWorkbenchReleaseRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_workbench_service::release(db, &item.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 申请延期
pub async fn workbench_retain_apply(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerWorkbenchRetainRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_workbench_service::retain_apply(db, &item.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 公海流水轨迹
pub async fn workbench_trace(
    state: web::Data<AppState>,
    query: web::Query<CustomerTraceQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match customer_pool_workbench_service::trace(db, query.customer_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 管理端：池 CRUD ====================

/// 池分页
pub async fn pool_page(
    state: web::Data<AppState>,
    query: web::Query<CustomerPoolListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match customer_pool_service::page(db, &query.0).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 池详情
pub async fn pool_detail(
    state: web::Data<AppState>,
    query: web::Query<CustomerPoolIdQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match customer_pool_service::find_vo_by_query(db, &query.0).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 新建池
pub async fn pool_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerPoolSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_service::insert(db, &item.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 更新池
pub async fn pool_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerPoolSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_service::update(db, &item.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 池删除请求（按池 ID 列表）
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PoolDeleteRequest {
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_or_num_vec_to_i64_vec")]
    pub ids: Vec<i64>,
}

/// 删除池（默认池禁删；需池内无客户）
pub async fn pool_delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<PoolDeleteRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_service::batch_delete_by_ids(db, &item.ids, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 启用/停用池
pub async fn pool_status(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerPoolStatusUpdateQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_service::update_status(db, &item.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 管理端：成员 ====================

/// 池成员列表
pub async fn member_list(
    state: web::Data<AppState>,
    query: web::Query<CustomerPoolIdQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match customer_pool_member_service::list_members(db, query.pool_id.unwrap_or(0)).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 添加池成员
pub async fn member_add(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerPoolMemberSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    let pool_id = item.pool_id.unwrap_or(0);
    // 含 member_type=1 的条目走 upsert_admin（池管理员），其余批量 add_members
    let (admins, normals): (Vec<_>, Vec<_>) = item
        .members
        .clone()
        .unwrap_or_default()
        .into_iter()
        .partition(|m| m.member_type == 1);
    let mut first_err: Option<String> = None;
    for m in admins {
        if let Err(e) =
            customer_pool_member_service::upsert_admin(db, pool_id, m.user_id, user_id).await
        {
            first_err = Some(e.to_string());
        }
    }
    if !normals.is_empty() {
        if let Err(e) =
            customer_pool_member_service::add_members(db, pool_id, &normals, user_id).await
        {
            first_err = Some(e.to_string());
        }
    }
    match first_err {
        None => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(true, "local"))),
        Some(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e, "local"))),
    }
}

/// 移除池成员
pub async fn member_remove(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerPoolMemberDeleteRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_member_service::remove_members(
        db,
        item.pool_id.unwrap_or(0),
        &item.user_ids.clone().unwrap_or_default(),
        user_id,
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 管理端：池配置 ====================

/// 池配置读取
pub async fn config_get(
    state: web::Data<AppState>,
    query: web::Query<CustomerPoolIdQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match customer_pool_config_service::get_vo(db, query.pool_id.unwrap_or(0)).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 池配置保存
pub async fn config_save(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerPoolConfigSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_config_service::save(db, &item.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 管理端：审批（申领/延期） ====================

/// 申领/延期待办、已办分页（type=apply|retain）
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct AuditPageQuery {
    pub r#type: Option<String>,
    pub status: Option<i16>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 申领分页
pub async fn apply_page(
    state: web::Data<AppState>,
    query: web::Query<CustomerPoolAuditPageQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match customer_pool_workbench_service::apply_page(db, &query.0).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 延期分页
pub async fn retain_page(
    state: web::Data<AppState>,
    query: web::Query<CustomerPoolAuditPageQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match customer_pool_workbench_service::retain_page(db, &query.0).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 申领审批
pub async fn apply_audit(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerPoolAuditRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_workbench_service::audit_apply(db, &item.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 延期审批
pub async fn retain_audit(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerPoolAuditRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_workbench_service::audit_retain(db, &item.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 管理端：指派 / 冻结 ====================

/// 管理员指派（池内客户 → 指定池成员，批量逐条返回明细）
pub async fn pool_assign(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CustomerPoolAssignRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match pool_assign_service(db, &item.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 冻结客户分页
pub async fn freeze_page(
    state: web::Data<AppState>,
    query: web::Query<CustomerPoolFreezePageQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match pool_frozen_page(db, &query.0).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 恢复冻结客户请求体
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FreezeRestoreRequest {
    #[serde(deserialize_with = "deserialize_string_or_number_to_i64")]
    pub customer_id: i64,
}

/// 恢复冻结客户
pub async fn freeze_restore(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<FreezeRestoreRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match pool_frozen_restore(db, item.customer_id, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}


// ==================== 管理端：查重规则 ====================

/// 查重规则保存请求
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DupRuleSaveBody {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub fields: Option<Vec<String>>,
    pub strength: Option<i16>,
    pub valid_days: Option<i32>,
    pub match_ratio: Option<i32>,
    pub enabled: Option<i16>,
}

/// 查重规则删除请求
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DupRuleDeleteBody {
    #[serde(default)]
    pub ids: Vec<i64>,
}

/// 查重检查请求（新建客户前端提示）
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DupCheckQuery {
    pub name: Option<String>,
    pub mobile: Option<String>,
    pub phone: Option<String>,
    pub wechat: Option<String>,
    pub email: Option<String>,
}

/// 查重规则分页
pub async fn dup_rule_page(state: web::Data<AppState>, query: web::Query<CustomerPoolAuditPageQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    match customer_pool_dup_rule_service::page(db, query.page.unwrap_or(1), query.page_size.unwrap_or(20)).await {
        Ok((total, items)) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(serde_json::json!({ "total": total, "items": items }), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 查重规则保存
pub async fn dup_rule_save(state: web::Data<AppState>, req: HttpRequest, item: web::Json<DupRuleSaveBody>) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    let payload = customer_pool_dup_rule_service::DupRuleSaveRequest {
        id: item.id,
        name: item.name.clone(),
        fields: item.fields.clone(),
        strength: item.strength,
        valid_days: item.valid_days,
        match_ratio: item.match_ratio,
        enabled: item.enabled,
    };
    match customer_pool_dup_rule_service::save(db, &payload, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 查重规则删除
pub async fn dup_rule_delete(state: web::Data<AppState>, req: HttpRequest, item: web::Json<DupRuleDeleteBody>) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_dup_rule_service::batch_delete(db, &item.ids, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 查重检查（提示级，前端新建客户弹窗展示命中）
pub async fn dup_check(state: web::Data<AppState>, query: web::Query<DupCheckQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    match customer_pool_dup_rule_service::check(
        db,
        query.name.as_deref(),
        query.mobile.as_deref(),
        query.phone.as_deref(),
        query.wechat.as_deref(),
        query.email.as_deref(),
    )
    .await
    {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 管理端：自动分配规则 ====================

/// 规则分页
pub async fn assign_rule_page(
    state: web::Data<AppState>,
    query: web::Query<crate::modules::crm::model::customer_pool::AssignRuleListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match customer_pool_assign_service::rule_page(db, &query.0).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 规则保存（整体覆盖条目）
pub async fn assign_rule_save(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<crate::modules::crm::model::customer_pool::AssignRuleSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_assign_service::rule_save(db, &item.0, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 规则删除
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct AssignRuleDeleteBody {
    #[serde(default)]
    pub ids: Vec<i64>,
}

pub async fn assign_rule_delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<AssignRuleDeleteBody>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_assign_service::rule_delete(db, &item.ids, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 考核统计 ====================

/// 个人公海成绩
pub async fn stats_mine(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match customer_pool_stats_service::mine(db, user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 管理员按池透视
pub async fn stats_admin(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match customer_pool_stats_service::admin_view(db).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}


// ==================== 路由注册（挂载于 /customer-pool scope 内） ====================

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/workbench")
            .route(
                "/pools",
                web::get()
                    .to(workbench_pools)
                    .wrap(require_permission("crm:customer:list")),
            )
            .route(
                "/customers",
                web::get()
                    .to(workbench_customers)
                    .wrap(require_permission("crm:customer:list")),
            )
            .route(
                "/claim",
                web::post()
                    .to(workbench_claim)
                    .wrap(require_permission("crm:customer:claim")),
            )
            .route(
                "/apply",
                web::post()
                    .to(workbench_apply)
                    .wrap(require_permission("crm:customer:apply")),
            )
            .route(
                "/release",
                web::post()
                    .to(workbench_release)
                    .wrap(require_permission("crm:customer:return-pool")),
            )
            .route(
                "/retain-apply",
                web::post()
                    .to(workbench_retain_apply)
                    .wrap(require_permission("crm:customer:retain-apply")),
            )
            .route(
                "/trace",
                web::get()
                    .to(workbench_trace)
                    .wrap(require_permission("crm:customer:trace")),
            ),
    )
    .route(
        "/page",
        web::get()
            .to(pool_page)
            .wrap(require_permission("crm:customer-pool:list")),
    )
    .route(
        "/detail",
        web::get()
            .to(pool_detail)
            .wrap(require_permission("crm:customer-pool:list")),
    )
    .route(
        "/create",
        web::post()
            .to(pool_create)
            .wrap(require_permission("crm:customer-pool:save")),
    )
    .route(
        "/update",
        web::post()
            .to(pool_update)
            .wrap(require_permission("crm:customer-pool:update")),
    )
    .route(
        "/delete",
        web::post()
            .to(pool_delete)
            .wrap(require_permission("crm:customer-pool:delete")),
    )
    .route(
        "/status",
        web::post()
            .to(pool_status)
            .wrap(require_permission("crm:customer-pool:update")),
    )
    .route(
        "/member/list",
        web::get()
            .to(member_list)
            .wrap(require_permission("crm:customer-pool:member")),
    )
    .route(
        "/member/add",
        web::post()
            .to(member_add)
            .wrap(require_permission("crm:customer-pool:member")),
    )
    .route(
        "/member/remove",
        web::post()
            .to(member_remove)
            .wrap(require_permission("crm:customer-pool:member")),
    )
    .route(
        "/config",
        web::get()
            .to(config_get)
            .wrap(require_permission("crm:customer-pool:config")),
    )
    .route(
        "/config/save",
        web::post()
            .to(config_save)
            .wrap(require_permission("crm:customer-pool:config")),
    )
    .route(
        "/apply/page",
        web::get()
            .to(apply_page)
            .wrap(require_permission("crm:customer-pool:apply-audit")),
    )
    .route(
        "/apply/audit",
        web::post()
            .to(apply_audit)
            .wrap(require_permission("crm:customer-pool:apply-audit")),
    )
    .route(
        "/retain/page",
        web::get()
            .to(retain_page)
            .wrap(require_permission("crm:customer-pool:retain-audit")),
    )
    .route(
        "/retain/audit",
        web::post()
            .to(retain_audit)
            .wrap(require_permission("crm:customer-pool:retain-audit")),
    )
    .route(
        "/assign",
        web::post()
            .to(pool_assign)
            .wrap(require_permission("crm:customer-pool:assign")),
    )
    .route(
        "/freeze/page",
        web::get()
            .to(freeze_page)
            .wrap(require_permission("crm:customer-pool:freeze")),
    )
    .route(
        "/freeze/restore",
        web::post()
            .to(freeze_restore)
            .wrap(require_permission("crm:customer-pool:freeze")),
    )
    .route(
        "/dup-rule/page",
        web::get()
            .to(dup_rule_page)
            .wrap(require_permission("crm:customer-pool:dup-rule")),
    )
    .route(
        "/dup-rule/save",
        web::post()
            .to(dup_rule_save)
            .wrap(require_permission("crm:customer-pool:dup-rule")),
    )
    .route(
        "/dup-rule/delete",
        web::post()
            .to(dup_rule_delete)
            .wrap(require_permission("crm:customer-pool:dup-rule")),
    )
    .route(
        "/dup-rule/check",
        web::get()
            .to(dup_check)
            .wrap(require_permission("crm:customer:save")),
    )
    .route(
        "/assign-rule/page",
        web::get()
            .to(assign_rule_page)
            .wrap(require_permission("crm:customer-pool:assign-rule")),
    )
    .route(
        "/assign-rule/save",
        web::post()
            .to(assign_rule_save)
            .wrap(require_permission("crm:customer-pool:assign-rule")),
    )
    .route(
        "/assign-rule/delete",
        web::post()
            .to(assign_rule_delete)
            .wrap(require_permission("crm:customer-pool:assign-rule")),
    )
    .route(
        "/stats",
        web::get()
            .to(stats_admin)
            .wrap(require_permission("crm:customer-pool:stats")),
    )
    .route(
        "/stats/mine",
        web::get()
            .to(stats_mine)
            .wrap(require_permission("crm:customer:list")),
    );
}
