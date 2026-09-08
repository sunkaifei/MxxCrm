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
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::model::quotation::{QuotationListQuery, QuotationSaveRequest, QuotationUpdateRequest};
use crate::modules::sale::service::quotation_service;
use crate::modules::system::service::edit_log_service;
use crate::modules::system::entity::admin::Entity as Admin;
use sea_orm::EntityTrait;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::json;

const QUOTATION_FIELD_LABELS: &[(&str, &str)] = &[
    ("title", "报价单标题"),
    ("customerName", "客户名称"),
    ("contactName", "联系人"),
    ("opportunityTitle", "关联商机"),
    ("currency", "币种"),
    ("validUntil", "有效期至"),
    ("quotationDate", "报价日期"),
    ("paymentTerms", "付款方式"),
    ("deliveryTerms", "交货方式"),
    ("deliveryDate", "交货日期"),
    ("portOfLoading", "装运港"),
    ("portOfDestination", "目的港"),
    ("bankInfo", "银行信息"),
    ("remark", "备注"),
    ("ownerUserId", "负责人"),
    ("totalAmount", "商品总额"),
    ("discountAmount", "折扣金额"),
    ("taxAmount", "税额"),
    ("grandTotal", "合计金额"),
    ("items", "商品明细"),
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotationApprovalRequest {
    pub remark: Option<String>,
}

pub async fn quotation_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<QuotationSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let user_id = get_current_user_id(&req);
    let user_id_str = user_id.to_string();
    let result = quotation_service::insert(db, &form_data, user_id_str).await;

    if let Ok(quotation_id) = result {
        let new_data = quotation_service::find_by_id(db, quotation_id).await
            .ok()
            .map(|m| serde_json::to_value(&m).unwrap_or_default())
            .unwrap_or_default();

        let editor_name = Admin::find_by_id(user_id)
            .one(db)
            .await
            .ok()
            .flatten()
            .and_then(|admin| admin.nick_name.or(admin.user_name));

        let business_no = new_data.get("quotationNo")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let business_title = new_data.get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let old_data = json!({});

        let _ = edit_log_service::log_update(
            db,
            1,
            quotation_id,
            business_no,
            business_title,
            user_id,
            editor_name,
            &old_data,
            &new_data,
            QUOTATION_FIELD_LABELS,
        ).await;
    }

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn quotation_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<QuotationUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "报价单ID不能为空", "local")));
    }
    let user_id = get_current_user_id(&req);
    let user_id_str = user_id.to_string();

    let quotation_id = form_data.id.unwrap_or_default();

    let old_data = quotation_service::find_by_id(db, quotation_id).await
        .ok()
        .map(|m| serde_json::to_value(&m).unwrap_or_default())
        .unwrap_or_default();

    let result = quotation_service::update(db, &form_data, user_id_str).await;

    if result.is_ok() {
        let new_data = quotation_service::find_by_id(db, quotation_id).await
            .ok()
            .map(|m| serde_json::to_value(&m).unwrap_or_default())
            .unwrap_or_default();

        let editor_name = Admin::find_by_id(user_id)
            .one(db)
            .await
            .ok()
            .flatten()
            .and_then(|admin| admin.nick_name.or(admin.user_name));

        let business_no = old_data.get("quotationNo")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let business_title = old_data.get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let _ = edit_log_service::log_update(
            db,
            1,
            quotation_id,
            business_no,
            business_title,
            user_id,
            editor_name,
            &old_data,
            &new_data,
            QUOTATION_FIELD_LABELS,
        ).await;
    }

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn bath_delete_quotation(state: web::Data<AppState>, form_data: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = form_data.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids: Vec<i64> = ids_vec.into_iter().filter_map(|id| id.and_then(|s| s.parse().ok())).collect();
        let result = quotation_service::batch_delete_by_ids(db, &ids).await;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn quotation_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "报价单ID不能为空", "local"));
    }
    match quotation_service::find_by_id(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn quotation_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<QuotationListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let current_user_id = get_current_user_id(&req);
    match quotation_service::list(db, &query, current_user_id).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn quotation_submit_approval(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<InfoId>,
    form_data: web::Json<QuotationApprovalRequest>,
) -> HttpResponse {
    let db = &state.db;
    let id = path.id.unwrap_or_default();
    if id == 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "报价单ID不能为空", "local"));
    }
    let (operator_id, operator_name) = get_current_user(&req);
    match quotation_service::submit_approval(db, id, operator_id, &operator_name, form_data.remark.clone()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn quotation_approve(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<InfoId>,
    form_data: web::Json<QuotationApprovalRequest>,
) -> HttpResponse {
    let db = &state.db;
    let id = path.id.unwrap_or_default();
    if id == 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "报价单ID不能为空", "local"));
    }
    let (operator_id, operator_name) = get_current_user(&req);
    match quotation_service::approve(db, id, operator_id, &operator_name, form_data.remark.clone()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn quotation_reject(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<InfoId>,
    form_data: web::Json<QuotationApprovalRequest>,
) -> HttpResponse {
    let db = &state.db;
    let id = path.id.unwrap_or_default();
    if id == 0 {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "报价单ID不能为空", "local"));
    }
    let (operator_id, operator_name) = get_current_user(&req);
    match quotation_service::reject(db, id, operator_id, &operator_name, form_data.remark.clone()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn quotation_convert_order(state: web::Data<AppState>, req: HttpRequest, path: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = quotation_service::convert_to_order(db, path.id.unwrap_or_default(), get_current_user_id(&req).to_string()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

// ==================== 路由注册（单点维护）====================

/// 注册报价单模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(quotation_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/quotation")
            // POST /sale/quotation/save - 新建报价单
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/save",
                web::post()
                    .to(quotation_insert)
                    .wrap(require_permission("sale:quotation:save")),
            )
            // PUT /sale/quotation/update - 修改报价单
            .route(
                "/update",
                web::put()
                    .to(quotation_update)
                    .wrap(require_permission("sale:quotation:update")),
            )
            // POST /sale/quotation/batch-delete - 批量删除报价单
            .route(
                "/batch-delete",
                web::post()
                    .to(bath_delete_quotation)
                    .wrap(require_permission("sale:quotation:delete")),
            )
            // GET /sale/quotation/info - 报价单详情
            .route(
                "/info",
                web::get()
                    .to(quotation_info)
                    .wrap(require_permission("sale:quotation:list")),
            )
            // GET /sale/quotation/list - 报价单列表
            .route(
                "/list",
                web::get()
                    .to(quotation_list)
                    .wrap(require_permission("sale:quotation:list")),
            )
            // POST /sale/quotation/{id}/submit-approval - 提交审批
            .route(
                "/{id}/submit-approval",
                web::post()
                    .to(quotation_submit_approval)
                    .wrap(require_permission("sale:quotation:update")),
            )
            // POST /sale/quotation/{id}/approve - 审批通过
            .route(
                "/{id}/approve",
                web::post()
                    .to(quotation_approve)
                    .wrap(require_permission("sale:quotation:audit")),
            )
            // POST /sale/quotation/{id}/reject - 驳回
            .route(
                "/{id}/reject",
                web::post()
                    .to(quotation_reject)
                    .wrap(require_permission("sale:quotation:audit")),
            )
            // POST /sale/quotation/{id}/convert-order - 转换为订单
            .route(
                "/{id}/convert-order",
                web::post()
                    .to(quotation_convert_order)
                    .wrap(require_permission("sale:quotation:update")),
            ),
    );
}
