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
use crate::core::web::base_controller::{get_current_user, get_current_user_id};
use crate::core::web::permission_guard::require_permission;
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::EntityTrait;

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::crm::model::customer::{CustomerListQuery, CustomerPoolReleaseRequest, CustomerSaveRequest, CustomerUpdateRequest};
use crate::modules::crm::model::customer_financial::{CustomerFinancialSaveDTO, CustomerFinancialModel};
use crate::modules::crm::service::customer_service;
use crate::modules::crm::service::contact_service;
use crate::modules::crm::service::assign_history_service;
use crate::modules::crm::service::customer_edit_log_service;
use crate::modules::crm::service::customer_transfer_service;
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use super::customer_edit_log_controller;

pub async fn customer_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<CustomerSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    // 类型校验由 service 层处理（企业必填公司名，个人必填姓名）
    let result = customer_service::insert(&db, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn customer_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<CustomerUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "客户ID不能为空", "local")));
    }

    // 类型校验由 service 层处理
    let result = customer_service::update(&db, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn bath_delete_customer(state: web::Data<AppState>, req: HttpRequest, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;
    let user_id = get_current_user_id(&req);

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到删除的客户ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    // 删除前快照（审计 before）
    let mut before: Vec<(i64, Option<String>, Option<i64>)> = Vec::new();
    for id in &filtered_ids {
        if let Ok(c) = customer_service::find_by_id(&db, *id).await {
            before.push((
                *id,
                c.company_name.or(c.person_name).or(c.nickname),
                c.assigned_to,
            ));
        }
    }

    let result = customer_service::batch_delete_by_ids(&db, &filtered_ids, user_id).await;
    if result.is_ok() {
        // 审计埋点：删除客户（D01-6）
        for (id, name, assigned) in &before {
            crate::modules::system::service::audit_service::record(
                db,
                &req,
                "customer",
                "delete",
                "customer",
                *id,
                format!("删除客户 {}", name.clone().unwrap_or_default()),
                crate::modules::system::service::audit_service::snap(vec![
                    ("assigned_to", serde_json::json!(assigned)),
                ]),
                None,
            ).await;
        }
    }
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

pub async fn customer_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "客户ID不能为空", "local"));
    }

    match customer_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn customer_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<CustomerListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    let current_user_id = get_current_user_id(&req);

    match customer_service::list(&db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 获取客户下的联系人列表
pub async fn customer_contacts(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "客户ID不能为空", "local"));
    }

    match contact_service::find_by_customer(&db, item.id.unwrap()).await {
        Ok((current, history)) => {
            use serde_json::json;
            let data = json!({
                "current": current,
                "history": history
            });
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))
        },
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 公海客户列表
pub async fn customer_pool_list(state: web::Data<AppState>, query: web::Query<CustomerListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    match customer_service::pool_list(&db, &query).await {
        Ok(page_data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 领取公海客户
pub async fn customer_claim(state: web::Data<AppState>, req: HttpRequest, item: web::Query<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "客户ID不能为空", "local")));
    }

    let user_id = get_current_user_id(&req);

    match customer_service::claim(&db, item.id.unwrap(), user_id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 退回公海
pub async fn customer_add_to_pool(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<CustomerPoolReleaseRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "客户ID不能为空", "local")));
    }

    let user_id = get_current_user_id(&req);
    let customer_id = form_data.id.unwrap();

    let result = customer_service::add_to_pool(
        db,
        customer_id,
        user_id,
        form_data.reason_type,
        form_data.reason.clone(),
    ).await;
    if result.is_ok() {
        // 审计埋点：退回公海（记录退回原因，G6）
        crate::modules::system::service::audit_service::record(
            db,
            &req,
            "customer",
            "release",
            "customer",
            customer_id,
            format!("退回公海，原因类型：{:?}", form_data.reason_type),
            crate::modules::system::service::audit_service::snap(vec![
                ("reason_type", serde_json::json!(form_data.reason_type)),
                ("reason", serde_json::json!(form_data.reason)),
            ]),
            None,
        ).await;
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 获取客户分配历史（负责人时间轴）
pub async fn customer_assign_history(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "客户ID不能为空", "local"));
    }

    match assign_history_service::list_by_customer(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 检查客户名称是否已存在（按 customerType 区分字段：1=企业按 companyName，2=个人按 personName）
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckNameQuery {
    pub customer_type: i32,
    pub name: String,
    pub exclude_id: Option<i64>,
}

pub async fn customer_check_name(state: web::Data<AppState>, query: web::Query<CheckNameQuery>) -> HttpResponse {
    let db = &state.db;
    let q = query.into_inner();
    match customer_service::check_customer_name(db, q.customer_type, q.name.trim(), q.exclude_id).await {
        Ok(exists) => {
            use serde_json::json;
            let data = json!({ "exists": exists });
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 查询客户财务信息
pub async fn customer_financial_info(
    state: web::Data<AppState>,
    customer_id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let customer_id = customer_id.into_inner();

    match CustomerFinancialModel::find_by_customer_id(db, customer_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 更新客户财务信息（存在则更新，不存在则新增）
pub async fn customer_financial_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<CustomerFinancialSaveDTO>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let dto = form_data.into_inner();
    let user_id = get_current_user_id(&req);

    // 获取操作人名称
    let editor_name = Admin::find_by_id(user_id)
        .one(db)
        .await
        .ok()
        .flatten()
        .and_then(|a| a.nick_name.or(a.user_name));

    // 查询旧数据（用于日志对比）
    let old_data = CustomerFinancialModel::find_by_customer_id(db, dto.customer_id).await
        .ok()
        .flatten()
        .map(|m| serde_json::to_value(m).unwrap_or_default())
        .unwrap_or_else(|| serde_json::json!({}));

    // 执行更新或新增
    let result = match CustomerFinancialModel::find_by_customer_id(db, dto.customer_id).await {
        Ok(Some(_)) => {
            CustomerFinancialModel::update_by_customer_id(db, dto.customer_id, Some(user_id), &dto).await
                .map(|v| (v, "update"))
        }
        Ok(None) => {
            CustomerFinancialModel::insert(db, &dto).await
                .map(|v| (v, "insert"))
        }
        Err(e) => Err(e),
    };

    match result {
        Ok((v, _op)) => {
            // 查询新数据用于日志对比
            let new_data = CustomerFinancialModel::find_by_customer_id(db, dto.customer_id).await
                .ok()
                .flatten()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .unwrap_or_else(|| serde_json::json!({}));

            // 记录编辑日志（财务信息类型 log_type=1）
            let _ = customer_edit_log_service::log_update(
                db, dto.customer_id, user_id, editor_name, &old_data, &new_data, Some(1),
            ).await;

            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local")))
        }
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 客户转移 ====================

/// 预览转移影响范围
pub async fn customer_transfer_preview(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<crate::modules::crm::service::customer_transfer_service::TransferPreviewRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match customer_transfer_service::preview_transfer(db, &form_data.0).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 执行客户转移
pub async fn customer_transfer(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<crate::modules::crm::service::customer_transfer_service::TransferRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (operator_id, operator_name) = get_current_user(&req);
    let transfer_customer_ids = form_data.customer_ids.clone();
    let transfer_to_user = form_data.to_user_id;

    match customer_transfer_service::transfer_customer(
        db, &form_data.0, operator_id, Some(operator_name),
    )
    .await
    {
        Ok(data) => {
            // 审计埋点：转移客户（D01-7，before/after 负责人）
            crate::modules::system::service::audit_service::record(
                db,
                &req,
                "customer",
                "transfer",
                "customer_batch",
                0,
                format!("转移 {} 个客户给用户 #{}", transfer_customer_ids.len(), transfer_to_user.unwrap_or(0)),
                crate::modules::system::service::audit_service::snap(vec![
                    ("customer_ids", serde_json::json!(transfer_customer_ids)),
                ]),
                crate::modules::system::service::audit_service::snap(vec![
                    ("to_user_id", serde_json::json!(transfer_to_user)),
                ]),
            ).await;
            Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::success(data, "local")))
        }
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册客户模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(customer_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/customer")
            // POST /customer/save - 新建客户
            .route(
                "/save",
                web::post()
                    .to(customer_insert)
                    .wrap(require_permission("crm:customer:save")),
            )
            // PUT /customer/update - 修改客户
            .route(
                "/update",
                web::put()
                    .to(customer_update)
                    .wrap(require_permission("crm:customer:update")),
            )
            // DELETE /customer/bath_delete - 批量删除客户
            .route(
                "/bath_delete",
                web::delete()
                    .to(bath_delete_customer)
                    .wrap(require_permission("crm:customer:delete")),
            )
            // GET /customer/info - 客户详情
            .route(
                "/info",
                web::get()
                    .to(customer_info)
                    .wrap(require_permission("crm:customer:view")),
            )
            // GET /customer/list - 客户列表
            .route(
                "/list",
                web::get()
                    .to(customer_list)
                    .wrap(require_permission("crm:customer:list")),
            )
            // GET /customer/contacts - 客户下的联系人列表
            .route(
                "/contacts",
                web::get()
                    .to(customer_contacts)
                    .wrap(require_permission("crm:customer:view")),
            )
            // PUT /customer/claim - 领取公海客户
            .route(
                "/claim",
                web::put()
                    .to(customer_claim)
                    .wrap(require_permission("crm:customer:claim")),
            )
            // PUT /customer/add-to-pool - 退回公海
            .route(
                "/add-to-pool",
                web::put()
                    .to(customer_add_to_pool)
                    .wrap(require_permission("crm:customer:return-pool")),
            )
            // GET /customer/assign-history - 客户分配历史
            .route(
                "/assign-history",
                web::get()
                    .to(customer_assign_history)
                    .wrap(require_permission("crm:customer:view")),
            )
            // GET /customer/check-name - 检查客户名称是否已存在（按 customerType 区分字段）
            .route(
                "/check-name",
                web::get()
                    .to(customer_check_name)
                    .wrap(require_permission("crm:customer:list")),
            )
            // GET /customer/financial/{customer_id} - 查询客户财务信息
            .route(
                "/financial/{customer_id}",
                web::get()
                    .to(customer_financial_info)
                    .wrap(require_permission("crm:customer:view")),
            )
            // PUT /customer/financial/update - 更新客户财务信息
            .route(
                "/financial/update",
                web::put()
                    .to(customer_financial_update)
                    .wrap(require_permission("crm:customer:update")),
            )
            // POST /customer/transfer/preview - 预览转移影响范围
            .route(
                "/transfer/preview",
                web::post()
                    .to(customer_transfer_preview)
                    .wrap(require_permission("crm:customer:transfer")),
            )
            // POST /customer/transfer - 执行客户转移
            .route(
                "/transfer",
                web::post()
                    .to(customer_transfer)
                    .wrap(require_permission("crm:customer:transfer")),
            )
            // 客户修改日志（注册在 /customer scope 内，避免被 scope 捕获导致 404）
            .configure(customer_edit_log_controller::register),
    );
    cfg.service(
        web::scope("/customer-pool")
            // GET /customer-pool/list - 公海客户列表
            .route(
                "/list",
                web::get()
                    .to(customer_pool_list)
                    .wrap(require_permission("crm:customer:list")),
            ),
    );
}