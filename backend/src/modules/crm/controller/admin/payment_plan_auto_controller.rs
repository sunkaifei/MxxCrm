//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 回款计划自动生成控制器
//!
//! ## 路由表
//!
//! | 方法   | 路径                                  | 权限码            | handler  | 说明                       |
//! |--------|--------------------------------------|-------------------|----------|----------------------------|
//! | POST   | /crm/payment-plan-auto/generate      | crm:contract:save | generate | 根据合同自动生成回款计划     |
//! | GET    | /crm/payment-plan-auto/terms-config  | crm:contract:list | config   | 付款条款配置说明（前端辅助） |
//!

use actix_web::{web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::crm::service::payment_plan_auto_service;

/// 自动生成请求参数
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateRequest {
    pub contract_id: Option<i64>,
}

/// 根据合同自动生成回款计划
pub async fn generate(
    state: web::Data<AppState>,
    form_data: web::Json<GenerateRequest>,
) -> HttpResponse {
    let db = &state.db;
    let contract_id = match form_data.contract_id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "合同ID不能为空", "local")),
    };
    match payment_plan_auto_service::generate_plans_for_contract(db, contract_id).await {
        Ok(count) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(serde_json::json!({ "generatedCount": count }), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 付款条款配置说明（供前端构造 payment_terms JSON 参考）
pub async fn config() -> HttpResponse {
    let data = payment_plan_auto_service::get_payment_terms_config();
    HttpResponse::Ok().content_type(MPACK)
        .body(MetaResp::success(data, "local"))
}

/// 注册回款计划自动生成模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/crm/payment-plan-auto")
            .route("/generate", web::post().to(generate).wrap(require_permission("crm:contract:save")))
            .route("/terms-config", web::get().to(config).wrap(require_permission("crm:contract:list"))),
    );
}
