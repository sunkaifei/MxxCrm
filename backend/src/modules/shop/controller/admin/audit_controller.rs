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
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use actix_web::{web, HttpRequest, HttpResponse};

/// Audit supplier application
pub async fn audit_apply(
    state: web::Data<AppState>,
    _req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call audit_service::audit_supplier(db, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

/// Audit SPU
pub async fn audit_spu(
    state: web::Data<AppState>,
    _req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    // TODO: call audit_service::audit_spu(db, body.into_inner()).await
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::fail(200, "success", "local")))
}

// ==================== 路由注册（单点维护）====================

/// 注册审核模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(audit_controller::register)` 注册。
///
/// 注意：本模块包含两个不相关的路径前缀（/supplier 和 /spu），
/// 因此在 register 中使用两个独立的 scope。
pub fn register(cfg: &mut web::ServiceConfig) {
    // 供应商审核
    cfg.service(
        web::scope("/supplier")
            // PUT /supplier/audit - 审核供应商申请
            .route(
                "/audit",
                web::put()
                    .to(audit_apply)
                    .wrap(require_permission("system:supplier:audit")),
            ),
    );
    // SPU 审核
    cfg.service(
        web::scope("/spu")
            // PUT /spu/audit - 审核 SPU
            .route(
                "/audit",
                web::put()
                    .to(audit_spu)
                    .wrap(require_permission("system:spu:audit")),
            ),
    );
}
