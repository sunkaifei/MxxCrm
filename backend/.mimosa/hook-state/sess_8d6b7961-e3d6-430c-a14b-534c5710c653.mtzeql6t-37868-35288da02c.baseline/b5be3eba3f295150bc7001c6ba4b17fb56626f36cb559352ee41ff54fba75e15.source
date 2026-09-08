//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 低库存采购建议控制器
//!

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::inventory::service::inventory_suggestion_service;
use actix_web::{web, HttpRequest, HttpResponse};

/// 查询低库存采购建议清单（仅返回建议，不创建采购申请单）
pub async fn suggestion_list(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match inventory_suggestion_service::get_suggestions(&db).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 基于低库存建议自动生成采购申请单
pub async fn suggestion_generate(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let operator_id = get_current_user_id(&req);

    match inventory_suggestion_service::generate_requisition(&db, operator_id).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/suggestion")
            .route(
                "/list",
                web::get()
                    .to(suggestion_list)
                    .wrap(require_permission("product:suggestion:list")),
            )
            .route(
                "/generate",
                web::post()
                    .to(suggestion_generate)
                    .wrap(require_permission("product:suggestion:create")),
            ),
    );
}
