//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 电子签约控制器
//!
//! ## 路由表
//!
//! | 方法   | 路径                                    | 权限码              | handler       | 说明                  |
//! |--------|----------------------------------------|---------------------|---------------|-----------------------|
//! | POST   | /crm/electronic-signature/create       | crm:contract:save   | create        | 创建签约               |
//! | GET    | /crm/electronic-signature/info         | crm:contract:list   | info          | 签约详情               |
//! | GET    | /crm/electronic-signature/by-contract  | crm:contract:list   | by_contract   | 按合同查询签约         |
//! | POST   | /crm/electronic-signature/cancel       | crm:contract:save   | cancel        | 撤销签约               |
//! | POST   | /crm/electronic-signature/callback     | （无权限）           | callback      | 第三方签约回调         |
//!

use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::entity::common::InfoId;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::crm::service::electronic_signature_service;

/// 创建签约请求参数
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSignatureRequest {
    pub contract_id: Option<i64>,
    pub platform: Option<i32>,
    pub signer_name: Option<String>,
    pub signer_phone: Option<String>,
    pub signer_email: Option<String>,
}

/// 签约回调参数
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignCallbackRequest {
    pub sign_no: Option<String>,
    pub status: Option<i32>,
    pub signed_pdf_url: Option<String>,
}

/// 按合同查询参数
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractIdQuery {
    pub contract_id: Option<i64>,
}

/// 创建签约
pub async fn create(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<CreateSignatureRequest>,
) -> HttpResponse {
    let db = &state.db;
    let user_id = get_current_user_id(&req);

    let contract_id = match form_data.contract_id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "合同ID不能为空", "local")),
    };
    let platform = form_data.platform.unwrap_or(1);
    let signer_name = form_data.signer_name.clone().unwrap_or_default();
    let signer_phone = form_data.signer_phone.clone().unwrap_or_default();
    let signer_email = form_data.signer_email.clone().unwrap_or_default();
    if signer_name.is_empty() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "签署人姓名不能为空", "local"));
    }

    match electronic_signature_service::create_signature(
        db, contract_id, platform, signer_name, signer_phone, signer_email, user_id
    ).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 签约详情
pub async fn info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "签约ID不能为空", "local"));
    }
    match electronic_signature_service::get_signature_info(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 按合同查询签约
pub async fn by_contract(state: web::Data<AppState>, query: web::Query<ContractIdQuery>) -> HttpResponse {
    let db = &state.db;
    let contract_id = match query.contract_id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "合同ID不能为空", "local")),
    };
    match electronic_signature_service::get_by_contract(db, contract_id).await {
        Ok(list) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(list, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 撤销签约
pub async fn cancel(state: web::Data<AppState>, form_data: web::Json<serde_json::Value>) -> HttpResponse {
    let db = &state.db;
    let id = form_data.get("id").and_then(|v| v.as_i64());
    if id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "签约ID不能为空", "local"));
    }
    match electronic_signature_service::cancel_signature(db, id.unwrap()).await {
        Ok(rows) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(rows, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 第三方签约回调（无需权限校验，由第三方平台直接调用）
pub async fn callback(
    state: web::Data<AppState>,
    form_data: web::Json<SignCallbackRequest>,
) -> HttpResponse {
    let db = &state.db;
    let sign_no = match &form_data.sign_no {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "签约编号不能为空", "local")),
    };
    let status = form_data.status.unwrap_or(0);
    if status <= 0 {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "回调状态不能为空", "local"));
    }
    match electronic_signature_service::handle_sign_callback(
        db, sign_no, status, form_data.signed_pdf_url.clone()
    ).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 注册电子签约模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/crm/electronic-signature")
            .route("/create", web::post().to(create).wrap(require_permission("crm:contract:save")))
            .route("/info", web::get().to(info).wrap(require_permission("crm:contract:list")))
            .route("/by-contract", web::get().to(by_contract).wrap(require_permission("crm:contract:list")))
            .route("/cancel", web::post().to(cancel).wrap(require_permission("crm:contract:save")))
            // 回调接口无权限校验（第三方平台直接调用，建议在网关层做签名校验）
            .route("/callback", web::post().to(callback)),
    );
}
