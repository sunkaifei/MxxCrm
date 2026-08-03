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
use crate::core::web::permission_guard::require_permission;
use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::crm::model::contract::{ContractApprovalDetailVO, ContractApprovalRequest, ContractDetailVO, ContractListQuery, ContractListVO, ContractSaveDTO, ContractSaveRequest, ContractUpdateRequest};
use crate::modules::crm::model::contract_commission_member::ContractCommissionMemberSaveDTO;
use crate::modules::crm::service::contract_commission_service;
use crate::modules::crm::service::contract_service;
use crate::modules::crm::controller::admin::contract_payment_plan_controller;
use crate::modules::finance::service::commission_calc_service;
use crate::modules::system::service::edit_log_service;
use crate::modules::system::entity::admin::Entity as Admin;
use sea_orm::EntityTrait;
use serde::Deserialize;
use serde_json::json;

const CONTRACT_FIELD_LABELS: &[(&str, &str)] = &[
    ("title", "合同标题"),
    ("contractNo", "合同编号"),
    ("contractType", "合同类型"),
    ("customerName", "客户名称"),
    ("amount", "合同金额"),
    ("currency", "币种"),
    ("taxAmount", "税额"),
    ("totalAmount", "含税总额"),
    ("startDate", "开始日期"),
    ("endDate", "结束日期"),
    ("signDate", "签订日期"),
    ("paymentTerms", "付款条款"),
    ("deliveryTerms", "交货条款"),
    ("paymentMethodType", "收款方式"),
    ("assignedTo", "负责人"),
    ("ourSignerName", "我方签署人"),
    ("theirSignerName", "对方签署人"),
    ("theirSignerPhone", "对方签署电话"),
    ("remark", "备注"),
];

pub async fn contract_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ContractSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data: ContractSaveDTO = form_data.0.into();

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();

    let result = contract_service::insert(&db, &form_data, user_id).await;

    if let Ok(contract_id) = result {
        let new_data = if let Ok(new) = contract_service::find_by_id(&db, contract_id).await {
            serde_json::to_value(&new).unwrap_or_default()
        } else {
            json!({})
        };

        let editor_name = Admin::find_by_id(user_id)
            .one(db)
            .await
            .ok()
            .flatten()
            .map(|admin| admin.nick_name.or(admin.user_name).unwrap_or_default());

        let contract_no = new_data.get("contractNo")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let title = new_data.get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let _ = edit_log_service::log_update(
            db,
            3,
            contract_id,
            contract_no,
            title,
            user_id,
            editor_name,
            &json!({}),
            &new_data,
            CONTRACT_FIELD_LABELS,
        ).await;
    }

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn contract_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ContractUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data: ContractSaveDTO = form_data.0.into();

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "合同ID不能为空", "local")));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();
    let contract_id = form_data.id.unwrap();

    let old_data = if let Ok(old) = contract_service::find_by_id(&db, contract_id).await {
        serde_json::to_value(&old).unwrap_or_default()
    } else {
        json!({})
    };

    let result = contract_service::update(&db, &form_data, user_id).await;

    if result.is_ok() {
        let new_data = if let Ok(new) = contract_service::find_by_id(&db, contract_id).await {
            serde_json::to_value(&new).unwrap_or_default()
        } else {
            json!({})
        };

        let editor_name = Admin::find_by_id(user_id)
            .one(db)
            .await
            .ok()
            .flatten()
            .map(|admin| admin.nick_name.or(admin.user_name).unwrap_or_default());

        let contract_no = old_data.get("contractNo")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let title = old_data.get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let _ = edit_log_service::log_update(
            db,
            3,
            contract_id,
            contract_no,
            title,
            user_id,
            editor_name,
            &old_data,
            &new_data,
            CONTRACT_FIELD_LABELS,
        ).await;
    }

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn bath_delete_contract(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到删除的合同ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = contract_service::batch_delete_by_ids(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

pub async fn contract_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "合同ID不能为空", "local"));
    }

    match contract_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn contract_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<ContractListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();

    match contract_service::list(&db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn contract_submit(state: web::Data<AppState>, req: HttpRequest, item: web::Json<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "合同ID不能为空", "local"));
    }

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    match contract_service::submit_contract(&db, item.id.unwrap(), jwt_token.id.unwrap_or_default(), &jwt_token.username.unwrap_or_default()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn contract_approve(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ContractApprovalRequest>) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    match contract_service::approve_contract(&db, &form_data, jwt_token.id.unwrap_or_default(), &jwt_token.username.unwrap_or_default()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn contract_reject(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<ContractApprovalRequest>) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    match contract_service::reject_contract(&db, &form_data, jwt_token.id.unwrap_or_default(), &jwt_token.username.unwrap_or_default()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn contract_approval_detail(state: web::Data<AppState>, path: web::Path<i64>) -> HttpResponse {
    let db = &state.db;
    let contract_id = path.into_inner();

    match contract_service::get_approval_detail(&db, contract_id).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveCommissionMembersReq {
    contract_id: i64,
    members: Vec<ContractCommissionMemberSaveDTO>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetCommissionRuleReq {
    contract_id: i64,
    rule_id: Option<i64>,
}

pub async fn get_contract_commission_members(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "合同ID不能为空", "local"));
    }

    match contract_commission_service::get_commission_config(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn save_contract_commission_members(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<SaveCommissionMembersReq>) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();

    match contract_commission_service::save_contract_members(&db, form_data.contract_id, &form_data.members, jwt_token.id.unwrap_or_default()).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("保存成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn set_contract_commission_rule(state: web::Data<AppState>, form_data: web::Json<SetCommissionRuleReq>) -> HttpResponse {
    let db = &state.db;
    let form_data = form_data.0;

    match contract_commission_service::set_commission_rule(&db, form_data.contract_id, form_data.rule_id).await {
        Ok(_) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("设置成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn preview_contract_commission(state: web::Data<AppState>, item: web::Json<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "合同ID不能为空", "local"));
    }

    match commission_calc_service::preview_contract_commission(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册合同模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(contract_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/contract")
            // POST /contract/save - 新建合同
            .route(
                "/save",
                web::post()
                    .to(contract_insert)
                    .wrap(require_permission("crm:contract:save")),
            )
            // PUT /contract/update - 修改合同
            .route(
                "/update",
                web::put()
                    .to(contract_update)
                    .wrap(require_permission("crm:contract:update")),
            )
            // DELETE /contract/bath_delete - 批量删除合同
            .route(
                "/bath_delete",
                web::delete()
                    .to(bath_delete_contract)
                    .wrap(require_permission("crm:contract:delete")),
            )
            // GET /contract/info - 合同详情
            .route(
                "/info",
                web::get()
                    .to(contract_info)
                    .wrap(require_permission("crm:contract:info")),
            )
            // GET /contract/list - 合同列表
            .route(
                "/list",
                web::get()
                    .to(contract_list)
                    .wrap(require_permission("crm:contract:list")),
            )
            // POST /contract/submit - 提交合同审批
            .route(
                "/submit",
                web::post()
                    .to(contract_submit)
                    .wrap(require_permission("crm:contract:submit")),
            )
            // POST /contract/approve - 审批通过
            .route(
                "/approve",
                web::post()
                    .to(contract_approve)
                    .wrap(require_permission("crm:contract:approve")),
            )
            // POST /contract/reject - 审批驳回
            .route(
                "/reject",
                web::post()
                    .to(contract_reject)
                    .wrap(require_permission("crm:contract:reject")),
            )
            // GET /contract/approval-detail/{contract_id} - 审批详情
            .route(
                "/approval-detail/{contract_id}",
                web::get()
                    .to(contract_approval_detail)
                    .wrap(require_permission("crm:contract:list")),
            )
            // GET /contract/commission-members - 获取合同提成成员
            .route(
                "/commission-members",
                web::get()
                    .to(get_contract_commission_members)
                    .wrap(require_permission("crm:contract:list")),
            )
            // POST /contract/commission-members/save - 保存合同提成成员
            .route(
                "/commission-members/save",
                web::post()
                    .to(save_contract_commission_members)
                    .wrap(require_permission("crm:contract:edit")),
            )
            // POST /contract/commission-rule/set - 设置合同提成规则
            .route(
                "/commission-rule/set",
                web::post()
                    .to(set_contract_commission_rule)
                    .wrap(require_permission("crm:contract:edit")),
            )
            // POST /contract/commission/preview - 预览合同提成
            .route(
                "/commission/preview",
                web::post()
                    .to(preview_contract_commission)
                    .wrap(require_permission("crm:contract:list")),
            )
            // 合同回款计划（注册在 /contract scope 内，避免被 /contract scope 吞掉）
            .configure(contract_payment_plan_controller::register),
    );
}